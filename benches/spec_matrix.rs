//! spec.md §21.3 覆盖矩阵采样；正式回归阈值须在受控机器冻结。
//!
//! 输出每行一个 JSON 对象：cell / n / median_ns / p95_ns / allocs_per_op / peak_rss_kib。

use std::alloc::{GlobalAlloc, Layout, System};
use std::error::Error;
use std::hint::black_box;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant, UNIX_EPOCH};

use temporal::{
    AsOfContext, AsOfContextParts, CandidateCoverageRef, CoverageState, DatasetSnapshotRef,
    DigestRef, EvidenceAcceptance, EvidenceProofRef, FactKeyRef, FactVersionIdRef,
    FactVersionParts, FactVersionRef, KnowledgeBasis, ObservationTime, OutputScope, PayloadRef,
    PrecisionLossPolicy, QueryPolicy, ResourceLimits, RevisionIdRef, RevisionOperation,
    RevisionOrderPolicy, RevisionOrderRef, SourceReleaseRequirement, SourceVisibilityEvidenceRef,
    TemporalDraft, TemporalProfile, TimeBounds, TimeField, TimePrecision, TimeQuality, UnixTimeNs,
    evaluate_visibility, quantize_unix_ns, select_fact_as_of,
};

static COUNTING: AtomicBool = AtomicBool::new(false);
static ALLOCS: AtomicU64 = AtomicU64::new(0);

struct CountingAlloc;

// SAFETY: 转发到 System 分配器；计数只在测量窗口内累加。
unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if COUNTING.load(Ordering::Relaxed) {
            ALLOCS.fetch_add(1, Ordering::Relaxed);
        }
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if COUNTING.load(Ordering::Relaxed) {
            ALLOCS.fetch_add(1, Ordering::Relaxed);
        }
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAlloc = CountingAlloc;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Variant {
    Ordered,
    Reversed,
    Duplicates,
    Withdrawn,
    Incomplete,
}

struct Fixture {
    candidates: Vec<FactVersionRef<'static>>,
    coverage: CandidateCoverageRef<'static>,
    context: AsOfContext<'static>,
}

fn build(revisions: usize, variant: Variant) -> Result<Fixture, Box<dyn Error>> {
    let capacity = if variant == Variant::Duplicates {
        revisions + 1
    } else {
        revisions
    };
    // 基准进程短生命周期；泄漏 limits 以获得 'static 借用，避免自引用夹具。
    let limits: &'static ResourceLimits = Box::leak(Box::new(ResourceLimits::try_new(
        128,
        1,
        1,
        capacity,
        1,
        capacity,
        capacity,
        1,
        capacity * 32_768,
    )?));
    let snapshot = DatasetSnapshotRef::try_new("snapshot", "manifest", "schema", limits)?;
    let at = UnixTimeNs::from_unix_nanos(1);
    let policy = QueryPolicy::try_new(
        TemporalProfile::PublishedObservation,
        EvidenceAcceptance::ExactOnly,
        RevisionOrderPolicy::AuthoritativeSequence,
        OutputScope::SingleFact,
        "policy",
        *limits,
    )?;
    let context = AsOfContext::try_new(AsOfContextParts {
        known_at: at,
        knowledge_basis: KnowledgeBasis::SourcePublishedAsOf,
        authority_scope: "authority",
        source_scope: Some("source"),
        consumer_boundary: None,
        dataset_snapshot: snapshot,
        system_cut: None,
        policy,
        fact_schema_ref: "fact-schema",
        version_schema_ref: "version-schema",
        evidence_schema_ref: "evidence-schema",
        interpretation_ref: "interpretation",
    })?;
    let key = FactKeyRef::try_new(b"fact", limits)?;
    let keys: &'static [FactKeyRef<'static>] = Box::leak(vec![key].into_boxed_slice());
    let coverage_state = if variant == Variant::Incomplete {
        CoverageState::Incomplete
    } else {
        CoverageState::Complete
    };
    let coverage = CandidateCoverageRef::try_new(
        "authority",
        keys,
        snapshot,
        KnowledgeBasis::SourcePublishedAsOf,
        at,
        coverage_state,
        "coverage",
        "adapter",
        "accepted",
        limits,
    )?;
    let temporal: &'static temporal::Temporal = Box::leak(Box::new(
        TemporalDraft {
            event_time: TimeField::Unknown,
            observation_time: TimeField::Known(ObservationTime::At(at)),
            publication_time: TimeField::Known(UnixTimeNs::from_unix_nanos(0)),
            received_time: TimeField::Known(at),
            revision_time: TimeField::Unknown,
            effective_time: TimeField::Unknown,
        }
        .validate(TemporalProfile::PublishedObservation)?,
    ));
    let bounds = TimeBounds::try_new(
        Some(UnixTimeNs::from_unix_nanos(0)),
        Some(UnixTimeNs::from_unix_nanos(0)),
        TimeQuality::Exact,
        "quality",
        limits,
    )?;
    let proof =
        EvidenceProofRef::try_new("proof", "schema", "extractor", "raw", "accepted", limits)?;
    let payload = PayloadRef::try_new("payload", "v1", limits)?;
    let mut candidates = Vec::new();
    candidates.try_reserve(capacity)?;
    for ordinal in 0..revisions {
        let revision_id: &'static str = Box::leak(ordinal.to_string().into_boxed_str());
        let revision_bytes = revision_id.as_bytes();
        let revision = RevisionIdRef::try_new(revision_bytes, limits)?;
        let digest = DigestRef::try_new("sha256-v1", revision_bytes, limits)?;
        let identity = FactVersionIdRef::try_new("authority", key, revision, digest, limits)?;
        let evidence =
            SourceVisibilityEvidenceRef::try_new(identity, "source", bounds, proof, limits)?;
        let withdraw = variant == Variant::Withdrawn && ordinal + 1 == revisions;
        candidates.push(FactVersionRef::try_new(
            FactVersionParts {
                dataset_snapshot: snapshot,
                profile: TemporalProfile::PublishedObservation,
                source_release: SourceReleaseRequirement::Required,
                identity,
                input_provenance_ref: "provenance",
                operation: if withdraw {
                    RevisionOperation::Withdraw
                } else {
                    RevisionOperation::Upsert
                },
                payload_ref: if withdraw { None } else { Some(payload) },
                source_temporal: temporal.source_roles(),
                revision_order: RevisionOrderRef::sequence("order", ordinal as u64, limits)?,
                source_evidence: Some(evidence),
                receipts: &[],
                system_evidence: &[],
                effective_interval: None,
            },
            limits,
        )?);
    }
    if variant == Variant::Reversed {
        candidates.reverse();
    }
    if variant == Variant::Duplicates {
        candidates.push(candidates[0]);
    }
    Ok(Fixture {
        candidates,
        coverage,
        context,
    })
}

fn peak_rss_kib() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|text| {
            text.lines().find_map(|line| {
                let rest = line.strip_prefix("VmHWM:")?;
                rest.split_whitespace().next()?.parse().ok()
            })
        })
        .unwrap_or(0)
}

fn stats(mut values: Vec<u128>) -> (u128, u128) {
    values.sort_unstable();
    let n = values.len();
    let median = values[n / 2];
    let index = ((n as f64 * 0.95).ceil() as usize)
        .saturating_sub(1)
        .min(n - 1);
    (median, values[index])
}

fn emit(cell: &str, n: usize, values: Vec<u128>, allocs_per_op: u128, peak: u64) {
    let (median, p95) = stats(values);
    println!(
        "{{\"cell\":\"{cell}\",\"n\":{n},\"median_ns\":{median},\"p95_ns\":{p95},\"allocs_per_op\":{allocs_per_op},\"peak_rss_kib\":{peak}}}"
    );
}

fn measure<F: FnMut()>(cell: &str, mut op: F, iterations: usize, samples: usize) {
    let mut values = Vec::with_capacity(samples);
    let mut allocs = 0_u64;
    for _ in 0..samples {
        ALLOCS.store(0, Ordering::Relaxed);
        COUNTING.store(true, Ordering::Relaxed);
        let started = Instant::now();
        for _ in 0..iterations {
            op();
        }
        let elapsed = started.elapsed().as_nanos();
        COUNTING.store(false, Ordering::Relaxed);
        allocs += ALLOCS.load(Ordering::Relaxed);
        values.push(elapsed / iterations as u128);
    }
    let per_op = allocs / (samples as u64 * iterations as u64);
    emit(cell, iterations, values, u128::from(per_op), peak_rss_kib());
}

fn select_cell(revisions: usize, variant: Variant, samples: usize) -> Result<(), Box<dyn Error>> {
    let fixture = build(revisions, variant)?;
    let mut values = Vec::with_capacity(samples);
    let mut allocs_total = 0_u64;
    for _ in 0..samples {
        ALLOCS.store(0, Ordering::Relaxed);
        COUNTING.store(true, Ordering::Relaxed);
        let started = Instant::now();
        let outcome = select_fact_as_of(
            black_box(&fixture.candidates),
            fixture.coverage,
            fixture.context,
        );
        let elapsed = started.elapsed().as_nanos();
        COUNTING.store(false, Ordering::Relaxed);
        allocs_total += ALLOCS.load(Ordering::Relaxed);
        if variant == Variant::Ordered {
            black_box(outcome?);
        } else {
            black_box(&outcome);
        }
        values.push(elapsed);
    }
    let name = format!("select_{variant:?}_{revisions}").to_lowercase();
    emit(
        &name,
        revisions,
        values,
        u128::from(allocs_total / samples as u64),
        peak_rss_kib(),
    );
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let at = UnixTimeNs::from_unix_nanos(1_234_567_890);
    let system_time = UNIX_EPOCH + Duration::from_nanos(1_234_567_890);

    measure(
        "unit_construct",
        || {
            black_box(UnixTimeNs::from_unix_nanos(black_box(-1_234_567_890)));
        },
        100_000,
        15,
    );
    measure(
        "checked_add",
        || {
            let _ = black_box(at.checked_add(black_box(Duration::from_nanos(1))));
        },
        100_000,
        15,
    );
    measure(
        "checked_sub",
        || {
            let _ = black_box(at.checked_sub(black_box(Duration::from_nanos(1))));
        },
        100_000,
        15,
    );
    measure(
        "system_time_from",
        || {
            let _ = black_box(UnixTimeNs::try_from_system_time(black_box(system_time)));
        },
        100_000,
        15,
    );
    measure(
        "system_time_to",
        || {
            let _ = black_box(at.try_into_system_time());
        },
        100_000,
        15,
    );
    for (cell, policy) in [
        ("quantize_reject", PrecisionLossPolicy::Reject),
        ("quantize_truncate", PrecisionLossPolicy::ExplicitTruncate),
        ("quantize_floor", PrecisionLossPolicy::ExplicitFloor),
    ] {
        measure(
            cell,
            || {
                let _ = black_box(quantize_unix_ns(
                    black_box(-1_234_567_891),
                    TimePrecision::Microseconds,
                    policy,
                ));
            },
            100_000,
            15,
        );
    }
    let draft = TemporalDraft {
        event_time: TimeField::Unknown,
        observation_time: TimeField::Known(ObservationTime::At(at)),
        publication_time: TimeField::Known(UnixTimeNs::from_unix_nanos(0)),
        received_time: TimeField::Known(at),
        revision_time: TimeField::Unknown,
        effective_time: TimeField::Unknown,
    };
    measure(
        "six_role_validate",
        || {
            let _ = black_box(
                draft
                    .clone()
                    .validate(black_box(TemporalProfile::PublishedObservation)),
            );
        },
        100_000,
        15,
    );
    let single = build(1, Variant::Ordered)?;
    measure(
        "single_visibility",
        || {
            let _ = black_box(evaluate_visibility(
                black_box(single.candidates[0]),
                single.context,
            ));
        },
        100_000,
        15,
    );

    select_cell(1, Variant::Ordered, 30)?;
    select_cell(10, Variant::Ordered, 30)?;
    select_cell(100, Variant::Ordered, 30)?;
    select_cell(1_000, Variant::Ordered, 10)?;
    select_cell(100_000, Variant::Ordered, 5)?;
    select_cell(1_000, Variant::Reversed, 10)?;
    select_cell(1_000, Variant::Duplicates, 10)?;
    select_cell(1_000, Variant::Withdrawn, 10)?;
    select_cell(1_000, Variant::Incomplete, 10)?;
    Ok(())
}
