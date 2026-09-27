//! 固定输入的 PIT 选版采样；正式回归阈值须在受控机器冻结。

use std::error::Error;
use std::hint::black_box;
use std::time::Instant;

use temporal::{
    AsOfContext, AsOfContextParts, CandidateCoverageRef, CoverageState, DatasetSnapshotRef,
    DigestRef, EvidenceAcceptance, EvidenceProofRef, FactKeyRef, FactVersionIdRef,
    FactVersionParts, FactVersionRef, KnowledgeBasis, ObservationTime, OutputScope, PayloadRef,
    QueryPolicy, ResourceLimits, RevisionIdRef, RevisionOperation, RevisionOrderPolicy,
    RevisionOrderRef, SourceReleaseRequirement, SourceVisibilityEvidenceRef, TemporalDraft,
    TemporalProfile, TimeBounds, TimeField, TimeQuality, UnixTimeNs, select_fact_as_of,
};

fn sample(revisions: usize) -> Result<u128, Box<dyn Error>> {
    let limits = ResourceLimits::try_new(
        128,
        1,
        1,
        revisions,
        1,
        revisions,
        revisions,
        1,
        revisions * 32_768,
    )?;
    let snapshot = DatasetSnapshotRef::try_new("snapshot", "manifest", "schema", &limits)?;
    let at = UnixTimeNs::from_unix_nanos(1);
    let policy = QueryPolicy::try_new(
        TemporalProfile::PublishedObservation,
        EvidenceAcceptance::ExactOnly,
        RevisionOrderPolicy::AuthoritativeSequence,
        OutputScope::SingleFact,
        "policy",
        limits,
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
    let key = FactKeyRef::try_new(b"fact", &limits)?;
    let keys = [key];
    let coverage = CandidateCoverageRef::try_new(
        "authority",
        &keys,
        snapshot,
        KnowledgeBasis::SourcePublishedAsOf,
        at,
        CoverageState::Complete,
        "coverage",
        "adapter",
        "accepted",
        &limits,
    )?;
    let temporal = TemporalDraft {
        event_time: TimeField::Unknown,
        observation_time: TimeField::Known(ObservationTime::At(at)),
        publication_time: TimeField::Known(UnixTimeNs::from_unix_nanos(0)),
        received_time: TimeField::Unknown,
        revision_time: TimeField::Unknown,
        effective_time: TimeField::Unknown,
    }
    .validate(TemporalProfile::PublishedObservation)?;
    let bounds = TimeBounds::try_new(
        Some(UnixTimeNs::from_unix_nanos(0)),
        Some(UnixTimeNs::from_unix_nanos(0)),
        TimeQuality::Exact,
        "quality",
        &limits,
    )?;
    let proof =
        EvidenceProofRef::try_new("proof", "schema", "extractor", "raw", "accepted", &limits)?;
    let payload = PayloadRef::try_new("payload", "v1", &limits)?;
    let revision_ids: Vec<String> = (0..revisions).map(|ordinal| ordinal.to_string()).collect();
    let mut candidates = Vec::new();
    candidates.try_reserve(revisions)?;
    for (ordinal, revision_id) in revision_ids.iter().enumerate() {
        let revision_bytes = revision_id.as_bytes();
        let revision = RevisionIdRef::try_new(revision_bytes, &limits)?;
        let digest = DigestRef::try_new("sha256-v1", revision_bytes, &limits)?;
        let identity = FactVersionIdRef::try_new("authority", key, revision, digest, &limits)?;
        let evidence =
            SourceVisibilityEvidenceRef::try_new(identity, "source", bounds, proof, &limits)?;
        candidates.push(FactVersionRef::try_new(
            FactVersionParts {
                dataset_snapshot: snapshot,
                profile: TemporalProfile::PublishedObservation,
                source_release: SourceReleaseRequirement::Required,
                identity,
                input_provenance_ref: "provenance",
                operation: RevisionOperation::Upsert,
                payload_ref: Some(payload),
                source_temporal: temporal.source_roles(),
                revision_order: RevisionOrderRef::sequence("order", ordinal as u64, &limits)?,
                source_evidence: Some(evidence),
                receipts: &[],
                system_evidence: &[],
                effective_interval: None,
            },
            &limits,
        )?);
    }
    let start = Instant::now();
    black_box(select_fact_as_of(
        black_box(&candidates),
        coverage,
        context,
    )?);
    Ok(start.elapsed().as_nanos())
}

fn main() -> Result<(), Box<dyn Error>> {
    for revisions in [1, 10, 100, 1_000, 100_000] {
        let mut samples = [0; 5];
        for elapsed in &mut samples {
            *elapsed = sample(revisions)?;
        }
        samples.sort_unstable();
        println!("修订数={revisions} 中位耗时={}ns", samples[2]);
    }
    Ok(())
}
