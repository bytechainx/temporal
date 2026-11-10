//! 验收对齐：为规格 §22.7–22.12 中尚未被既有分组测试覆盖的 TP-T 场景补齐断言。
//!
//! 每组只写本文件负责的新场景；已在 `pit_contract.rs` / `effective_contract.rs` /
//! `p1b_revision.rs` 中被行为覆盖的 ID（T061/T065/T081/T082/T083/T071/T072/T084/
//! T092/T093/T096/T097 等）在此不重复，仅在注释中登记。
#![allow(clippy::unwrap_used)]

use static_assertions::assert_impl_all;

use temporal::{
    AsOfContext, AsOfContextParts, CandidateCoverageRef, CoverageState, CutPartitionRef,
    DatasetSnapshotRef, DecisionReason, DigestRef, EffectiveInterval, EffectiveStateInputRef,
    EffectiveStateResult, EvidenceAcceptance, EvidenceProofRef, FactKeyRef, FactOutcome,
    FactVersionIdRef, FactVersionParts, FactVersionRef, KnowledgeBasis, ObservationTime,
    OutputScope, PayloadRef, PitError, PitResult, PrecisionError, QueryInputRef, QueryPolicy,
    ReceiptEvidenceRef, ResourceLimits, RevisionIdRef, RevisionOperation, RevisionOrderPolicy,
    RevisionOrderRef, SourceReleaseRequirement, SourceVisibilityEvidenceRef, StateAssertionRef,
    StatePolicy, SystemKnowledgeCutRef, SystemVisibilityEvidenceRef, TemporalDraft, TemporalError,
    TemporalProfile, TimeBounds, TimeError, TimeField, TimeQuality, UnixTimeNs, VisibilityDecision,
    evaluate_visibility, inspect_candidates, select_effective_state_as_of, select_fact_as_of,
    select_facts_as_of,
};

fn time(value: i64) -> UnixTimeNs {
    UnixTimeNs::from_unix_nanos(value)
}

fn limits() -> ResourceLimits {
    ResourceLimits::try_new(128, 16, 8, 128, 16, 32, 32, 16, 65536).unwrap()
}

fn snapshot() -> DatasetSnapshotRef<'static> {
    DatasetSnapshotRef::try_new("s1", "manifest1", "schema1", &limits()).unwrap()
}

fn snapshot2() -> DatasetSnapshotRef<'static> {
    DatasetSnapshotRef::try_new("s2", "manifest2", "schema1", &limits()).unwrap()
}

fn context(at: i64, scope: OutputScope, order: RevisionOrderPolicy) -> AsOfContext<'static> {
    let policy = QueryPolicy::try_new(
        TemporalProfile::PublishedObservation,
        EvidenceAcceptance::ConservativeBounds,
        order,
        scope,
        "policy1",
        limits(),
    )
    .unwrap();
    AsOfContext::try_new(AsOfContextParts {
        known_at: time(at),
        knowledge_basis: KnowledgeBasis::SourcePublishedAsOf,
        authority_scope: "authority1",
        source_scope: Some("source1"),
        consumer_boundary: None,
        dataset_snapshot: snapshot(),
        system_cut: None,
        policy,
        fact_schema_ref: "fact1",
        version_schema_ref: "version1",
        evidence_schema_ref: "evidence1",
        interpretation_ref: "interpretation1",
    })
    .unwrap()
}

fn exact_only_context(at: i64) -> AsOfContext<'static> {
    let policy = QueryPolicy::try_new(
        TemporalProfile::PublishedObservation,
        EvidenceAcceptance::ExactOnly,
        RevisionOrderPolicy::AuthoritativeSequence,
        OutputScope::SingleFact,
        "policy1",
        limits(),
    )
    .unwrap();
    AsOfContext::try_new(AsOfContextParts {
        known_at: time(at),
        knowledge_basis: KnowledgeBasis::SourcePublishedAsOf,
        authority_scope: "authority1",
        source_scope: Some("source1"),
        consumer_boundary: None,
        dataset_snapshot: snapshot(),
        system_cut: None,
        policy,
        fact_schema_ref: "fact1",
        version_schema_ref: "version1",
        evidence_schema_ref: "evidence1",
        interpretation_ref: "interpretation1",
    })
    .unwrap()
}

fn chain_context(at: i64) -> AsOfContext<'static> {
    let policy = QueryPolicy::try_new(
        TemporalProfile::PublishedObservation,
        EvidenceAcceptance::ConservativeBounds,
        RevisionOrderPolicy::LinearSupersedes,
        OutputScope::SingleFact,
        "policy1",
        limits(),
    )
    .unwrap();
    AsOfContext::try_new(AsOfContextParts {
        known_at: time(at),
        knowledge_basis: KnowledgeBasis::SourcePublishedAsOf,
        authority_scope: "authority1",
        source_scope: Some("source1"),
        consumer_boundary: None,
        dataset_snapshot: snapshot(),
        system_cut: None,
        policy,
        fact_schema_ref: "fact1",
        version_schema_ref: "version1",
        evidence_schema_ref: "evidence1",
        interpretation_ref: "interpretation1",
    })
    .unwrap()
}

fn version(
    key: &'static [u8],
    revision: &'static [u8],
    ordinal: u64,
    release: i64,
    operation: RevisionOperation,
) -> FactVersionRef<'static> {
    version_bounds(key, revision, ordinal, release, release, release, operation)
}

/// 用调用方指定的资源上限构造版本；用于超过默认 `limits()` 的大规模场景。
fn version_at(
    key: &'static [u8],
    revision: &'static [u8],
    ordinal: u64,
    release: i64,
    limits: &ResourceLimits,
) -> FactVersionRef<'static> {
    version_bounds_at(
        key,
        revision,
        ordinal,
        release,
        release,
        release,
        RevisionOperation::Upsert,
        limits,
    )
}

fn version_bounds(
    key: &'static [u8],
    revision: &'static [u8],
    ordinal: u64,
    release: i64,
    lower: i64,
    upper: i64,
    operation: RevisionOperation,
) -> FactVersionRef<'static> {
    version_bounds_at(
        key,
        revision,
        ordinal,
        release,
        lower,
        upper,
        operation,
        &limits(),
    )
}

#[allow(clippy::too_many_arguments)]
fn version_bounds_at(
    key: &'static [u8],
    revision: &'static [u8],
    ordinal: u64,
    release: i64,
    lower: i64,
    upper: i64,
    operation: RevisionOperation,
    limits: &ResourceLimits,
) -> FactVersionRef<'static> {
    let key = FactKeyRef::try_new(key, limits).unwrap();
    let revision = RevisionIdRef::try_new(revision, limits).unwrap();
    let digest = DigestRef::try_new("sha256-v1", revision.as_bytes(), limits).unwrap();
    let identity = FactVersionIdRef::try_new("authority1", key, revision, digest, limits).unwrap();
    let draft = TemporalDraft {
        event_time: TimeField::Unknown,
        observation_time: TimeField::Known(ObservationTime::At(time(1000))),
        publication_time: TimeField::Known(time(release)),
        received_time: TimeField::Known(time(100)),
        revision_time: TimeField::Unknown,
        effective_time: TimeField::Unknown,
    };
    let temporal = Box::leak(Box::new(
        draft
            .validate(TemporalProfile::PublishedObservation)
            .unwrap(),
    ));
    let quality = if lower == upper {
        TimeQuality::Exact
    } else {
        TimeQuality::Bounded
    };
    let bounds = TimeBounds::try_new(
        Some(time(lower)),
        Some(time(upper)),
        quality,
        "quality1",
        limits,
    )
    .unwrap();
    let proof = EvidenceProofRef::try_new(
        "proof1",
        "schema1",
        "extractor1",
        "raw1",
        "accepted1",
        limits,
    )
    .unwrap();
    let evidence =
        SourceVisibilityEvidenceRef::try_new(identity, "source1", bounds, proof, limits).unwrap();
    let payload_ref = if operation == RevisionOperation::Upsert {
        Some(PayloadRef::try_new("payload1", "v1", limits).unwrap())
    } else {
        None
    };
    FactVersionRef::try_new(
        FactVersionParts {
            dataset_snapshot: snapshot(),
            profile: TemporalProfile::PublishedObservation,
            source_release: SourceReleaseRequirement::Required,
            identity,
            input_provenance_ref: "provenance1",
            operation,
            payload_ref,
            source_temporal: temporal.source_roles(),
            revision_order: RevisionOrderRef::sequence("order1", ordinal, limits).unwrap(),
            source_evidence: Some(evidence),
            receipts: &[],
            system_evidence: &[],
            effective_interval: None,
        },
        limits,
    )
    .unwrap()
}

fn coverage<'a>(
    keys: &'a [FactKeyRef<'a>],
    at: i64,
    state: CoverageState,
) -> CandidateCoverageRef<'a> {
    CandidateCoverageRef::try_new(
        "authority1",
        keys,
        snapshot(),
        KnowledgeBasis::SourcePublishedAsOf,
        time(at),
        state,
        "coverage1",
        "adapter1",
        "accepted1",
        &limits(),
    )
    .unwrap()
}

fn system_context(at: i64, frontier: u64) -> AsOfContext<'static> {
    let limits = limits();
    let partitions = Box::leak(Box::new([CutPartitionRef::try_new(
        "part1", "log1", frontier, &limits,
    )
    .unwrap()]));
    let cut = SystemKnowledgeCutRef::try_new(
        "consumer1",
        "log1",
        "cut1",
        "cut-manifest1",
        time(at),
        "bound1",
        partitions,
        "consistent1",
        "evidence-snapshot1",
        &limits,
    )
    .unwrap();
    let policy = QueryPolicy::try_new(
        TemporalProfile::PublishedObservation,
        EvidenceAcceptance::ExactOnly,
        RevisionOrderPolicy::AuthoritativeSequence,
        OutputScope::SingleFact,
        "policy1",
        limits,
    )
    .unwrap();
    AsOfContext::try_new(AsOfContextParts {
        known_at: time(at),
        knowledge_basis: KnowledgeBasis::SystemAsKnown,
        authority_scope: "authority1",
        source_scope: Some("source1"),
        consumer_boundary: Some("consumer1"),
        dataset_snapshot: snapshot(),
        system_cut: Some(cut),
        policy,
        fact_schema_ref: "fact1",
        version_schema_ref: "version1",
        evidence_schema_ref: "evidence1",
        interpretation_ref: "interpretation1",
    })
    .unwrap()
}

fn system_version(
    base: FactVersionRef<'static>,
    receiver_epoch: &'static str,
    receipt_at: i64,
) -> FactVersionRef<'static> {
    let limits = limits();
    let proof = EvidenceProofRef::try_new(
        "system-proof1",
        "schema1",
        "extractor1",
        "raw1",
        "accepted1",
        &limits,
    )
    .unwrap();
    let receipt_bounds = TimeBounds::try_new(
        Some(time(receipt_at)),
        Some(time(receipt_at)),
        TimeQuality::Exact,
        "quality1",
        &limits,
    )
    .unwrap();
    let available_bounds = TimeBounds::try_new(
        Some(time(16)),
        Some(time(16)),
        TimeQuality::Exact,
        "quality1",
        &limits,
    )
    .unwrap();
    let receipts = Box::leak(Box::new([ReceiptEvidenceRef::try_new(
        base.identity(),
        "receiver1",
        "rx1",
        "receipt1",
        receipt_bounds,
        proof,
        &limits,
    )
    .unwrap()]));
    let events = Box::leak(Box::new([SystemVisibilityEvidenceRef::try_new(
        base.identity(),
        "consumer1",
        "log1",
        "event1",
        "receiver1",
        receiver_epoch,
        "receipt1",
        "part1",
        1,
        available_bounds,
        proof,
        &limits,
    )
    .unwrap()]));
    FactVersionRef::try_new(
        FactVersionParts {
            dataset_snapshot: base.dataset_snapshot(),
            profile: base.profile(),
            source_release: base.source_release(),
            identity: base.identity(),
            input_provenance_ref: base.input_provenance_ref(),
            operation: base.operation(),
            payload_ref: base.payload_ref(),
            source_temporal: base.source_temporal(),
            revision_order: base.revision_order(),
            source_evidence: base.source_evidence(),
            receipts,
            system_evidence: events,
            effective_interval: base.effective_interval(),
        },
        &limits,
    )
    .unwrap()
}

fn with_order(
    base: FactVersionRef<'static>,
    order: RevisionOrderRef<'static>,
    source_evidence: Option<SourceVisibilityEvidenceRef<'static>>,
) -> FactVersionRef<'static> {
    FactVersionRef::try_new(
        FactVersionParts {
            dataset_snapshot: base.dataset_snapshot(),
            profile: base.profile(),
            source_release: base.source_release(),
            identity: base.identity(),
            input_provenance_ref: base.input_provenance_ref(),
            operation: base.operation(),
            payload_ref: base.payload_ref(),
            source_temporal: base.source_temporal(),
            revision_order: order,
            source_evidence,
            receipts: base.receipts(),
            system_evidence: base.system_evidence(),
            effective_interval: base.effective_interval(),
        },
        &limits(),
    )
    .unwrap()
}

fn with_system_evidence(
    base: FactVersionRef<'static>,
    receipts: &'static [ReceiptEvidenceRef<'static>],
    events: &'static [SystemVisibilityEvidenceRef<'static>],
    limits: &ResourceLimits,
) -> Result<FactVersionRef<'static>, PitError> {
    FactVersionRef::try_new(
        FactVersionParts {
            dataset_snapshot: base.dataset_snapshot(),
            profile: base.profile(),
            source_release: base.source_release(),
            identity: base.identity(),
            input_provenance_ref: base.input_provenance_ref(),
            operation: base.operation(),
            payload_ref: base.payload_ref(),
            source_temporal: base.source_temporal(),
            revision_order: base.revision_order(),
            source_evidence: base.source_evidence(),
            receipts,
            system_evidence: events,
            effective_interval: base.effective_interval(),
        },
        limits,
    )
}

fn selected_revision<'a>(outcome: FactOutcome<'a>) -> Option<&'a [u8]> {
    selected_identity(outcome).map(|id| id.revision_id().as_bytes())
}

/// 选中结果的版本身份引用；用于比较两侧身份而不合并。
fn selected_identity<'a>(outcome: FactOutcome<'a>) -> Option<FactVersionIdRef<'a>> {
    match outcome {
        FactOutcome::Selected { version, .. } | FactOutcome::Withdrawn { version, .. } => {
            Some(version)
        }
        _ => None,
    }
}

/// 左侧身份（供 TP-T105 join 对照）。
fn left_selected_identity<'a>(outcome: FactOutcome<'a>) -> Option<FactVersionIdRef<'a>> {
    selected_identity(outcome)
}

/// 右侧身份（供 TP-T105 join 对照）。
fn right_selected_identity<'a>(outcome: FactOutcome<'a>) -> Option<FactVersionIdRef<'a>> {
    selected_identity(outcome)
}

// ── 22.7 来源可见性与不确定性 ─────────────────────────────

#[test]
fn tp_t062_063_cpi_initial_window_then_revision() {
    let v1 = version(b"cpi", b"v1", 1, 10, RevisionOperation::Upsert); // 9/10
    let v2 = version(b"cpi", b"v2", 2, 30, RevisionOperation::Upsert); // 10/10
    let keys = [v1.identity().fact_key()];
    // 9/20：只选 v1，不能返回最终修订值 v2。
    let at_sep = context(
        20,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    assert!(
        matches!(select_fact_as_of(&[v2, v1], coverage(&keys, 20, CoverageState::Complete), at_sep).unwrap(), FactOutcome::Selected { version, .. } if version.revision_id().as_bytes() == b"v1")
    );
    // 10/11：选择 v2。
    let at_oct = context(
        31,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    assert!(
        matches!(select_fact_as_of(&[v1, v2], coverage(&keys, 31, CoverageState::Complete), at_oct).unwrap(), FactOutcome::Selected { version, .. } if version.revision_id().as_bytes() == b"v2")
    );
}

#[test]
fn tp_t064_exact_boundary_release_gate() {
    let before = version(b"t064", b"a", 1, 19, RevisionOperation::Upsert);
    let at = version(b"t064", b"b", 2, 20, RevisionOperation::Upsert);
    let after = version(b"t064", b"c", 3, 21, RevisionOperation::Upsert);
    let ctx = context(
        20,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    for (candidate, expected) in [
        (before, VisibilityDecision::Visible),
        (at, VisibilityDecision::Visible),
        (
            after,
            VisibilityDecision::NotVisible(DecisionReason::ReleaseAfterCutoff),
        ),
    ] {
        assert_eq!(
            evaluate_visibility(candidate, ctx).unwrap(),
            expected,
            "release boundary gate"
        );
    }
}

#[test]
fn tp_t066_conservative_upper_bound_vs_exact_only() {
    // 已接受的实际发布区间 upper==cutoff（20），但区间为 Bounded 而非 Exact。
    let candidate = version_bounds(b"t066", b"v1", 1, 20, 15, 20, RevisionOperation::Upsert);
    let keys = [candidate.identity().fact_key()];
    assert!(matches!(
        evaluate_visibility(
            candidate,
            context(
                20,
                OutputScope::SingleFact,
                RevisionOrderPolicy::AuthoritativeSequence
            )
        )
        .unwrap(),
        VisibilityDecision::Visible
    ));
    assert!(matches!(
        evaluate_visibility(candidate, exact_only_context(20)).unwrap(),
        VisibilityDecision::Indeterminate(_)
    ));
    // ExactOnly 即使 cutoff 已过仍不借上界放行：完整选版结果也为 IndeterminateSelection。
    assert!(matches!(
        select_fact_as_of(
            &[candidate],
            coverage(&keys, 20, CoverageState::Complete),
            exact_only_context(20)
        ),
        Err(PitError::IndeterminateSelection)
    ));
}

#[test]
fn tp_t067_scheduled_release_only_is_unproven() {
    // 只有已知发布日期（publication_time=20），没有实际发布证明（source_evidence=None）。
    let base = version(b"t067", b"v1", 1, 20, RevisionOperation::Upsert);
    let scheduled = with_order(base, base.revision_order(), None);
    assert!(matches!(
        evaluate_visibility(
            scheduled,
            context(
                20,
                OutputScope::SingleFact,
                RevisionOrderPolicy::AuthoritativeSequence
            )
        )
        .unwrap(),
        VisibilityDecision::Indeterminate(DecisionReason::ReleaseConditionUnproven)
    ));
}

#[test]
fn tp_t068_old_content_missing_fails_strict_query() {
    // 只提供最新 B，却声称 v1 旧内容也属本覆盖：覆盖不完整，严格查询失败。
    let latest = version(b"t068", b"v1", 1, 30, RevisionOperation::Upsert);
    let keys = [latest.identity().fact_key()];
    let ctx = context(
        20,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    assert!(matches!(
        select_fact_as_of(
            &[latest],
            coverage(&keys, 20, CoverageState::Incomplete),
            ctx
        ),
        Err(PitError::IncompleteCandidateSet)
    ));
}

#[test]
fn tp_t069_source_backfill_rebuilds_but_not_fake_system_receipt() {
    // 新来源快照用真实 9 月证据重建 9 月可见版本。
    let sept = version(b"t069", b"v1", 1, 10, RevisionOperation::Upsert); // release 9/10
    let keys = [sept.identity().fact_key()];
    assert!(
        matches!(select_fact_as_of(&[sept], coverage(&keys, 10, CoverageState::Complete), context(10, OutputScope::SingleFact, RevisionOrderPolicy::AuthoritativeSequence)).unwrap(), FactOutcome::Selected { version, .. } if version.revision_id().as_bytes() == b"v1")
    );
    // 来源模式下，伪造的系统 receipt 不充当发布证明：未来发布仍不可见。
    let base = version(b"t069", b"v2", 2, 30, RevisionOperation::Upsert);
    let forged = system_version(base, "rx1", 20); // 伪造系统接收
    assert!(matches!(
        evaluate_visibility(
            forged,
            context(
                20,
                OutputScope::SingleFact,
                RevisionOrderPolicy::AuthoritativeSequence
            )
        )
        .unwrap(),
        VisibilityDecision::NotVisible(_)
    ));
}

#[test]
fn tp_t070_evidence_scope_or_binding_mismatch_never_visible() {
    // 证据 source_scope 与上下文不符：明确 EvidenceMismatch。
    let base = version(b"t070", b"v1", 1, 10, RevisionOperation::Upsert);
    let wrong_scope = base.source_evidence().unwrap();
    let foreign = SourceVisibilityEvidenceRef::try_new(
        base.identity(),
        "other-source",
        wrong_scope.bounds(),
        wrong_scope.proof(),
        &limits(),
    )
    .unwrap();
    let rebound = with_order(base, base.revision_order(), Some(foreign));
    assert!(matches!(
        evaluate_visibility(
            rebound,
            context(
                20,
                OutputScope::SingleFact,
                RevisionOrderPolicy::AuthoritativeSequence
            )
        ),
        Err(PitError::EvidenceMismatch)
    ));
    // 缺绑定（source_evidence=None）：Indeterminate，不 Visible。
    let missing = with_order(base, base.revision_order(), None);
    assert!(matches!(
        evaluate_visibility(
            missing,
            context(
                20,
                OutputScope::SingleFact,
                RevisionOrderPolicy::AuthoritativeSequence
            )
        )
        .unwrap(),
        VisibilityDecision::Indeterminate(_)
    ));
}

// ── 22.8 系统可见性、cut 与覆盖 ─────────────────────────────

#[test]
fn tp_t073_future_commit_with_old_wall_clock_excluded() {
    // 事件可用时间（16）早于 T（20），但 cursor 2 超出 cut frontier 1。
    let base = version(b"t073", b"v1", 1, 10, RevisionOperation::Upsert);
    let first = system_version(base, "rx1", 15);
    let proof = first.system_evidence()[0].proof();
    let event_bounds = TimeBounds::try_new(
        Some(time(16)),
        Some(time(16)),
        TimeQuality::Exact,
        "quality1",
        &limits(),
    )
    .unwrap();
    let events = Box::leak(Box::new([SystemVisibilityEvidenceRef::try_new(
        base.identity(),
        "consumer1",
        "log1",
        "event2",
        "receiver1",
        "rx1",
        "receipt1",
        "part1",
        2,
        event_bounds,
        proof,
        &limits(),
    )
    .unwrap()]));
    let beyond = with_system_evidence(base, first.receipts(), events, &limits()).unwrap();
    assert!(matches!(
        evaluate_visibility(beyond, system_context(20, 1)).unwrap(),
        VisibilityDecision::NotVisible(DecisionReason::OutsideSystemCut)
    ));
}

#[test]
fn tp_t074_boundary_isolation_no_shared_available_at() {
    // 版本绑定 consumer1；consumer2 上下文因边界不匹配而不可见。
    let base = version(b"t074", b"v1", 1, 10, RevisionOperation::Upsert);
    let sys = system_version(base, "rx1", 15);
    assert!(matches!(
        evaluate_visibility(sys, system_context(20, 1)).unwrap(),
        VisibilityDecision::Visible
    ));
    // 构造绑定 consumer2 的系统上下文。
    let limits = limits();
    let partitions = Box::leak(Box::new([CutPartitionRef::try_new(
        "part1", "log1", 1, &limits,
    )
    .unwrap()]));
    let cut = SystemKnowledgeCutRef::try_new(
        "consumer2",
        "log1",
        "cut2",
        "cut-manifest1",
        time(20),
        "bound2",
        partitions,
        "consistent1",
        "evidence-snapshot1",
        &limits,
    )
    .unwrap();
    let policy = QueryPolicy::try_new(
        TemporalProfile::PublishedObservation,
        EvidenceAcceptance::ExactOnly,
        RevisionOrderPolicy::AuthoritativeSequence,
        OutputScope::SingleFact,
        "policy1",
        limits,
    )
    .unwrap();
    let other = AsOfContext::try_new(AsOfContextParts {
        known_at: time(20),
        knowledge_basis: KnowledgeBasis::SystemAsKnown,
        authority_scope: "authority1",
        source_scope: Some("source1"),
        consumer_boundary: Some("consumer2"),
        dataset_snapshot: snapshot(),
        system_cut: Some(cut),
        policy,
        fact_schema_ref: "fact1",
        version_schema_ref: "version1",
        evidence_schema_ref: "evidence1",
        interpretation_ref: "interpretation1",
    })
    .unwrap();
    assert!(matches!(
        evaluate_visibility(sys, other).unwrap(),
        VisibilityDecision::NotVisible(_)
    ));
}

#[test]
fn tp_t075_log_epoch_isolation_no_cross_epoch_ordinal_compare() {
    // 同分区名不同 epoch 的事件不在同一 cut 内比较。
    let base = version(b"t075", b"v1", 1, 10, RevisionOperation::Upsert);
    let first = system_version(base, "rx1", 15);
    // 第二个事件位于不同日志代次（log_epoch="log2"），但同分区 part1。
    let proof = first.system_evidence()[0].proof();
    let event_bounds = TimeBounds::try_new(
        Some(time(16)),
        Some(time(16)),
        TimeQuality::Exact,
        "quality1",
        &limits(),
    )
    .unwrap();
    let events = Box::leak(Box::new([SystemVisibilityEvidenceRef::try_new(
        base.identity(),
        "consumer1",
        "log2",
        "event2",
        "receiver1",
        "rx1",
        "receipt1",
        "part1",
        2,
        event_bounds,
        proof,
        &limits(),
    )
    .unwrap()]));
    let other_epoch = with_system_evidence(base, first.receipts(), events, &limits()).unwrap();
    // cut 的 boundary_epoch 是 log1；log2 事件被跳过 → 无可见事件 → NotVisible。
    assert!(matches!(
        evaluate_visibility(other_epoch, system_context(20, 1)).unwrap(),
        VisibilityDecision::NotVisible(DecisionReason::OutsideSystemCut)
    ));
}

#[test]
fn tp_t076_multi_partition_cut_needs_completeness_proof() {
    // 系统模式 cut 只含 part1；候选事件在 part2 → 该事实不可见。
    let base = version(b"t076", b"v1", 1, 10, RevisionOperation::Upsert);
    let first = system_version(base, "rx1", 15);
    let proof = first.system_evidence()[0].proof();
    let event_bounds = TimeBounds::try_new(
        Some(time(16)),
        Some(time(16)),
        TimeQuality::Exact,
        "quality1",
        &limits(),
    )
    .unwrap();
    let events = Box::leak(Box::new([SystemVisibilityEvidenceRef::try_new(
        base.identity(),
        "consumer1",
        "log1",
        "event3",
        "receiver1",
        "rx1",
        "receipt3",
        "part2",
        1,
        event_bounds,
        proof,
        &limits(),
    )
    .unwrap()]));
    let part2 = with_system_evidence(base, first.receipts(), events, &limits()).unwrap();
    assert!(matches!(
        evaluate_visibility(part2, system_context(20, 1)).unwrap(),
        VisibilityDecision::NotVisible(_)
    ));
    // 覆盖声明不完整：缺完整性/一致性证明时严格查询失败。
    let key = [part2.identity().fact_key()];
    let coverage = CandidateCoverageRef::try_new(
        "authority1",
        &key,
        snapshot(),
        KnowledgeBasis::SystemAsKnown,
        time(20),
        CoverageState::Incomplete,
        "coverage1",
        "adapter1",
        "accepted1",
        &limits(),
    )
    .unwrap();
    assert!(matches!(
        select_fact_as_of(&[part2], coverage, system_context(20, 1)),
        Err(PitError::IncompleteCandidateSet)
    ));
}

#[test]
fn tp_t077_transaction_start_recorded_before_but_commit_after() {
    // recorded/available 时间（16）早于 T（20），但实际提交（receipt_at=30）晚于 T。
    let base = version(b"t077", b"v1", 1, 10, RevisionOperation::Upsert);
    let late_commit = system_version(base, "rx1", 30);
    // 不可仅凭早的 now 字段证明可见：receipt 未按 cutoff 证明 → 不 Visible。
    assert!(matches!(
        evaluate_visibility(late_commit, system_context(20, 1)).unwrap(),
        VisibilityDecision::Indeterminate(_)
    ));
}

#[test]
fn tp_t078_streaming_boundary_before_persistence_is_judged_by_boundary() {
    // 真实可消费日志证据（receipt 15、event 16，均在 cutoff 20 内）即判定可见；
    // 不附加任何「DB 已持久化」完成条件。
    let base = version(b"t078", b"v1", 1, 10, RevisionOperation::Upsert);
    let streamed = system_version(base, "rx1", 15);
    assert!(matches!(
        evaluate_visibility(streamed, system_context(20, 1)).unwrap(),
        VisibilityDecision::Visible
    ));
}

#[test]
fn tp_t079_missing_history_cut_rejected_at_context_construction() {
    // SystemAsKnown 但没有 system_cut：构造上下文即拒绝，不切换到来源模式。
    let policy = QueryPolicy::try_new(
        TemporalProfile::PublishedObservation,
        EvidenceAcceptance::ExactOnly,
        RevisionOrderPolicy::AuthoritativeSequence,
        OutputScope::SingleFact,
        "policy1",
        limits(),
    )
    .unwrap();
    assert!(matches!(
        AsOfContext::try_new(AsOfContextParts {
            known_at: time(20),
            knowledge_basis: KnowledgeBasis::SystemAsKnown,
            authority_scope: "authority1",
            source_scope: Some("source1"),
            consumer_boundary: Some("consumer1"),
            dataset_snapshot: snapshot(),
            system_cut: None,
            policy,
            fact_schema_ref: "fact1",
            version_schema_ref: "version1",
            evidence_schema_ref: "evidence1",
            interpretation_ref: "interpretation1",
        }),
        Err(PitError::UnprovenSystemCut)
    ));
}

// ── 22.9 严格选版与撤回 ─────────────────────────────────────

#[test]
fn tp_t085_empty_visible_set_no_future_leak() {
    // 覆盖完整，但所有候选明确在截止点之后发布。
    let future = version(b"t085", b"v1", 1, 30, RevisionOperation::Upsert);
    let keys = [future.identity().fact_key()];
    let candidates = [future];
    let outcome = select_fact_as_of(
        &candidates,
        coverage(&keys, 20, CoverageState::Complete),
        context(
            20,
            OutputScope::SingleFact,
            RevisionOrderPolicy::AuthoritativeSequence,
        ),
    )
    .unwrap();
    assert!(matches!(outcome, FactOutcome::NoVisibleVersion { .. }));
}

#[test]
fn tp_t086_legal_input_permutation_keeps_consistent_order() {
    let a = version(b"a", b"v1", 1, 10, RevisionOperation::Upsert);
    let b = version(b"b", b"v1", 1, 10, RevisionOperation::Upsert);
    let c = version(b"c", b"v1", 1, 10, RevisionOperation::Upsert);
    let keys = [
        a.identity().fact_key(),
        b.identity().fact_key(),
        c.identity().fact_key(),
    ];
    let ctx = context(
        20,
        OutputScope::AllFacts,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    // 多种合法排列 → 所选事实与输出键顺序始终一致。
    for candidates in [[b, a, c], [c, b, a], [a, c, b]] {
        let result = select_facts_as_of(
            QueryInputRef {
                candidates: &candidates,
                coverage: coverage(&keys, 20, CoverageState::Complete),
            },
            ctx,
        )
        .unwrap();
        let order: Vec<&[u8]> = result
            .outcomes
            .iter()
            .map(|o| selected_revision(*o).unwrap())
            .collect();
        assert_eq!(
            order,
            vec![b"v1".as_slice(), b"v1".as_slice(), b"v1".as_slice()]
        );
        let keys_order: Vec<&[u8]> = result
            .outcomes
            .iter()
            .map(|o| match o {
                FactOutcome::Selected { version, .. } => version.fact_key().as_bytes(),
                _ => unreachable!(),
            })
            .collect();
        assert_eq!(
            keys_order,
            vec![b"a".as_slice(), b"b".as_slice(), b"c".as_slice()]
        );
    }
}

#[test]
fn tp_t088_chain_cannot_peek_future_supersedes_edge() {
    // 未来才出现的 supersedes 边（v2 指向 v1 的前驱）不能提前让 v1 失效。
    let v1 = version(b"t088", b"v1", 1, 10, RevisionOperation::Upsert);
    let v2 = version(b"t088", b"v2", 2, 30, RevisionOperation::Upsert);
    let digest = DigestRef::try_new("sha256-v1", b"t088-chain", &limits()).unwrap();
    let root = with_order(
        v1,
        RevisionOrderRef::LinearSupersedes {
            predecessor: None,
            chain_digest: digest,
        },
        v1.source_evidence(),
    );
    let child = with_order(
        v2,
        RevisionOrderRef::LinearSupersedes {
            predecessor: Some(v1.identity().revision_id()),
            chain_digest: digest,
        },
        v2.source_evidence(),
    );
    let keys = [v1.identity().fact_key()];
    // 截止 20：v2 尚未发布，链中只有 v1 可见 → 选择 v1，不被未来 v2 提前截断。
    let candidates = [child, root];
    let outcome = select_fact_as_of(
        &candidates,
        coverage(&keys, 20, CoverageState::Complete),
        chain_context(20),
    )
    .unwrap();
    assert_eq!(selected_revision(outcome), Some(b"v1".as_slice()));
}

#[test]
fn tp_t090_facts_are_not_latest_observation() {
    // 同一系列两个观测期间：两个不同事实键各自返回，不折叠为一条。
    let period_a = version(b"series-a", b"v1", 1, 10, RevisionOperation::Upsert);
    let period_b = version(b"series-b", b"v1", 1, 20, RevisionOperation::Upsert);
    let keys = [
        period_a.identity().fact_key(),
        period_b.identity().fact_key(),
    ];
    let ctx = context(
        30,
        OutputScope::AllFacts,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    let candidates = [period_a, period_b];
    let result = select_facts_as_of(
        QueryInputRef {
            candidates: &candidates,
            coverage: coverage(&keys, 30, CoverageState::Complete),
        },
        ctx,
    )
    .unwrap();
    assert_eq!(result.outcomes.len(), 2);
    let mut selected: Vec<&[u8]> = result
        .outcomes
        .iter()
        .map(|o| match o {
            FactOutcome::Selected { version, .. } => version.fact_key().as_bytes(),
            _ => unreachable!(),
        })
        .collect();
    selected.sort();
    assert_eq!(
        selected,
        vec![b"series-a".as_slice(), b"series-b".as_slice()]
    );
}

// ── 22.10 知识与有效状态 ───────────────────────────────────

#[test]
fn tp_t094_illegal_effective_interval_rejected() {
    // start>=end：拒绝构造。
    assert!(matches!(
        EffectiveInterval::try_new(time(30), Some(time(30))),
        Err(TemporalError::InvalidInterval)
    ));
    assert!(matches!(
        EffectiveInterval::try_new(time(30), Some(time(10))),
        Err(TemporalError::InvalidInterval)
    ));
    // effective_time 与 interval.start 不同 → 元数据不一致（ContextMismatch）。
    let limits = ResourceLimits::try_new(128, 8, 8, 16, 8, 8, 8, 8, 65536).unwrap();
    let snapshot = DatasetSnapshotRef::try_new("s1", "manifest1", "schema1", &limits).unwrap();
    let policy = QueryPolicy::try_new(
        TemporalProfile::Announcement,
        EvidenceAcceptance::ExactOnly,
        RevisionOrderPolicy::AuthoritativeSequence,
        OutputScope::AllFacts,
        "policy1",
        limits,
    )
    .unwrap();
    let context = AsOfContext::try_new(AsOfContextParts {
        known_at: time(20),
        knowledge_basis: KnowledgeBasis::SourcePublishedAsOf,
        authority_scope: "authority1",
        source_scope: Some("source1"),
        consumer_boundary: None,
        dataset_snapshot: snapshot,
        system_cut: None,
        policy,
        fact_schema_ref: "fact1",
        version_schema_ref: "version1",
        evidence_schema_ref: "evidence1",
        interpretation_ref: "interpretation1",
    })
    .unwrap();
    // 版本 effective_interval 为 [30,∞)，但断言映射为 [0,∞) → 元数据不一致。
    let interval = Box::leak(Box::new(EffectiveInterval::try_new(time(0), None).unwrap()));
    let version_interval = Box::leak(Box::new(
        EffectiveInterval::try_new(time(30), None).unwrap(),
    ));
    let key = FactKeyRef::try_new(b"t094", &limits).unwrap();
    let digest = DigestRef::try_new("sha256-v1", key.as_bytes(), &limits).unwrap();
    let revision = RevisionIdRef::try_new(b"v1", &limits).unwrap();
    let identity = FactVersionIdRef::try_new("authority1", key, revision, digest, &limits).unwrap();
    let draft = TemporalDraft {
        event_time: TimeField::Unknown,
        observation_time: TimeField::Unknown,
        publication_time: TimeField::Known(time(10)),
        received_time: TimeField::Known(time(12)),
        revision_time: TimeField::Unknown,
        effective_time: TimeField::Known(time(30)),
    };
    let temporal = Box::leak(Box::new(
        draft.validate(TemporalProfile::Announcement).unwrap(),
    ));
    let bounds = TimeBounds::try_new(
        Some(time(10)),
        Some(time(10)),
        TimeQuality::Exact,
        "quality1",
        &limits,
    )
    .unwrap();
    let proof = EvidenceProofRef::try_new(
        "proof1",
        "schema1",
        "extractor1",
        "raw1",
        "accepted1",
        &limits,
    )
    .unwrap();
    let evidence =
        SourceVisibilityEvidenceRef::try_new(identity, "source1", bounds, proof, &limits).unwrap();
    let fact_version = FactVersionRef::try_new(
        FactVersionParts {
            dataset_snapshot: snapshot,
            profile: TemporalProfile::Announcement,
            source_release: SourceReleaseRequirement::Required,
            identity,
            input_provenance_ref: "provenance1",
            operation: RevisionOperation::Upsert,
            payload_ref: Some(PayloadRef::try_new("payload1", "v1", &limits).unwrap()),
            source_temporal: temporal.source_roles(),
            revision_order: RevisionOrderRef::sequence("order1", 1, &limits).unwrap(),
            source_evidence: Some(evidence),
            receipts: &[],
            system_evidence: &[],
            effective_interval: Some(version_interval),
        },
        &limits,
    )
    .unwrap();
    let fact_key = fact_version.identity().fact_key();
    let fact_keys = [fact_key];
    let cov = CandidateCoverageRef::try_new(
        "authority1",
        &fact_keys,
        snapshot,
        KnowledgeBasis::SourcePublishedAsOf,
        time(20),
        CoverageState::Complete,
        "coverage1",
        "adapter1",
        "accepted1",
        &limits,
    )
    .unwrap();
    let assertions = [StateAssertionRef {
        fact_key,
        state_key: b"state1",
        interval: *interval,
    }];
    assert!(matches!(
        select_effective_state_as_of(
            EffectiveStateInputRef {
                facts: QueryInputRef {
                    candidates: &[fact_version],
                    coverage: cov,
                },
                assertions: &assertions,
                state_key: b"state1",
                state_policy_ref: "state-policy1",
            },
            context,
            time(5),
            StatePolicy::UniqueNonOverlappingAssertions,
        ),
        Err(PitError::ContextMismatch)
    ));
}

#[test]
fn tp_t098_future_announcement_does_not_truncate_history() {
    // B（未来公告）尚不可见，即使能推导 A 的潜在未来 end，也不提前截断 A。
    let limits = ResourceLimits::try_new(128, 8, 8, 16, 8, 8, 8, 8, 65536).unwrap();
    let snapshot = DatasetSnapshotRef::try_new("s1", "manifest1", "schema1", &limits).unwrap();
    let policy = QueryPolicy::try_new(
        TemporalProfile::Announcement,
        EvidenceAcceptance::ExactOnly,
        RevisionOrderPolicy::AuthoritativeSequence,
        OutputScope::AllFacts,
        "policy1",
        limits,
    )
    .unwrap();
    let context = AsOfContext::try_new(AsOfContextParts {
        known_at: time(20),
        knowledge_basis: KnowledgeBasis::SourcePublishedAsOf,
        authority_scope: "authority1",
        source_scope: Some("source1"),
        consumer_boundary: None,
        dataset_snapshot: snapshot,
        system_cut: None,
        policy,
        fact_schema_ref: "fact1",
        version_schema_ref: "version1",
        evidence_schema_ref: "evidence1",
        interpretation_ref: "interpretation1",
    })
    .unwrap();
    let old_interval = Box::leak(Box::new(
        EffectiveInterval::try_new(time(0), Some(time(30))).unwrap(),
    ));
    let future_interval = Box::leak(Box::new(
        EffectiveInterval::try_new(time(30), None).unwrap(),
    ));
    let old = assertion_version(b"t098", old_interval, &limits, snapshot);
    // B 是与 A 不同的修订（v2/序号 2），其有效区间起点为 30。
    let future = assertion_revision(
        b"t098",
        (b"v2", 2, 40, RevisionOperation::Upsert),
        future_interval,
        &limits,
        snapshot,
    );
    // future 公告的 release 晚于截止（未来），不可见；其有效区间不提前截断 old。
    let future = Box::leak(Box::new(with_effective_interval(future, 40, &limits)));
    let keys = [old.identity().fact_key()];
    let cov = CandidateCoverageRef::try_new(
        "authority1",
        &keys,
        snapshot,
        KnowledgeBasis::SourcePublishedAsOf,
        time(20),
        CoverageState::Complete,
        "coverage1",
        "adapter1",
        "accepted1",
        &limits,
    )
    .unwrap();
    let assertions = [StateAssertionRef {
        fact_key: keys[0],
        state_key: b"state1",
        interval: *old_interval,
    }];
    // 未来公告 B 也作为候选进入选择：截止 20 时 B 的发布晚于截止，不可见。
    let candidates = [old, *future];
    let outcome = select_effective_state_as_of(
        EffectiveStateInputRef {
            facts: QueryInputRef {
                candidates: &candidates,
                coverage: cov,
            },
            assertions: &assertions,
            state_key: b"state1",
            state_policy_ref: "state-policy1",
        },
        context,
        time(25),
        StatePolicy::UniqueNonOverlappingAssertions,
    )
    .unwrap();
    // 在 B 生效前（valid_at=25 < 30）A 仍 Active，且 B 的存在不改变 A 的区间。
    assert!(
        matches!(outcome, EffectiveStateResult::Active { outcome: FactOutcome::Selected { version, .. }, .. } if version.fact_key().as_bytes() == b"t098")
    );
    // 断言映射里的 A 区间仍是原区间 [0,30)，未被 B 的未来起点提前改写。
    assert_eq!(assertions[0].interval, *old_interval);
    assert_eq!(assertions[0].interval.start(), time(0));
    assert_eq!(assertions[0].interval.end_exclusive(), Some(time(30)));
    // 未来候选 B 的区间起点 30 与 A 的终点相接，但 A 的终点来自自身区间而非 B。
    assert_eq!(future_interval.start(), time(30));
    // A 的终点不早于 B 的起点（未被 B 提前截断）。
    assert!(assertions[0].interval.end_exclusive() >= Some(future_interval.start()));
    // 关键判据：B 不可见时，A 的区间终点仍是自身声明的 30，
    // 而不是被 B 的生效起点或某个更早时刻截断。
    assert_eq!(
        assertions[0].interval.end_exclusive(),
        old_interval.end_exclusive()
    );
    // 截止 20 时 B 不可见（发布晚于截止），所以不参与 A 的状态选择。
    assert!(matches!(
        evaluate_visibility(*future, context).unwrap(),
        VisibilityDecision::NotVisible(DecisionReason::ReleaseAfterCutoff)
    ));
}

#[test]
fn tp_t100_predicted_future_observation_not_auto_leaked() {
    // 今天（20）已发布对未来期间（observation=100）的预测：不因 observation>known_at
    // 自动判泄漏或排除——可见性由来源发布判定，而非预测观测时刻。
    let limits = limits();
    let key = FactKeyRef::try_new(b"t100", &limits).unwrap();
    let revision = RevisionIdRef::try_new(b"v1", &limits).unwrap();
    let digest = DigestRef::try_new("sha256-v1", key.as_bytes(), &limits).unwrap();
    let identity = FactVersionIdRef::try_new("authority1", key, revision, digest, &limits).unwrap();
    let draft = TemporalDraft {
        event_time: TimeField::Unknown,
        observation_time: TimeField::Known(ObservationTime::At(time(100))), // 未来观测期间
        publication_time: TimeField::Known(time(20)),                       // 今天发布
        received_time: TimeField::Known(time(19)),
        revision_time: TimeField::Unknown,
        effective_time: TimeField::Unknown,
    };
    let temporal = Box::leak(Box::new(
        draft
            .validate(TemporalProfile::PublishedObservation)
            .unwrap(),
    ));
    let bounds = TimeBounds::try_new(
        Some(time(20)),
        Some(time(20)),
        TimeQuality::Exact,
        "quality1",
        &limits,
    )
    .unwrap();
    let proof = EvidenceProofRef::try_new(
        "proof1",
        "schema1",
        "extractor1",
        "raw1",
        "accepted1",
        &limits,
    )
    .unwrap();
    let evidence =
        SourceVisibilityEvidenceRef::try_new(identity, "source1", bounds, proof, &limits).unwrap();
    let forecast = FactVersionRef::try_new(
        FactVersionParts {
            dataset_snapshot: snapshot(),
            profile: TemporalProfile::PublishedObservation,
            source_release: SourceReleaseRequirement::Required,
            identity,
            input_provenance_ref: "provenance1",
            operation: RevisionOperation::Upsert,
            payload_ref: Some(PayloadRef::try_new("payload1", "v1", &limits).unwrap()),
            source_temporal: temporal.source_roles(),
            revision_order: RevisionOrderRef::sequence("order1", 1, &limits).unwrap(),
            source_evidence: Some(evidence),
            receipts: &[],
            system_evidence: &[],
            effective_interval: None,
        },
        &limits,
    )
    .unwrap();
    let ctx = context(
        20,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    // 已发布预测按发布可见；未来 observation 不自动判泄漏（仍 Visible）。
    assert!(matches!(
        evaluate_visibility(forecast, ctx).unwrap(),
        VisibilityDecision::Visible
    ));
}

// ── 22.11 派生数据与复现性质 ───────────────────────────────

// T101–T106 属派生数据下界 / 完成时间下界 / 历史伪造拒绝 / 反事实模拟 / PIT join /
// 模型实际可用性。按 SPEC §32.1，这些场景须拆成 temporal 纯 fixture 与消费方集成
// 两半；本节是 temporal 侧：断言 crate 可保证的不变量（输入可见性、完成下界、
// 模式隔离、join 时点、部署边界）。派生数据 API 与执行模式由 Feature/backtest
// Owner 实现（§17.3），本仓不暴露。

// TP-T101 派生下界：一个输入尚未可见，输出被标为可用 → lineage 校验拒绝。
#[test]
fn tp_t101_derived_lower_bound_rejects_unavailable_input() {
    let base = version(b"t101", b"v1", 1, 10, RevisionOperation::Upsert);
    // 输入证据缺失或未按截止证明：任何后续使用都落在 Indeterminate。
    let unproven = with_order(base, base.revision_order(), None);
    let ctx = context(
        20,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    assert!(matches!(
        evaluate_visibility(unproven, ctx).unwrap(),
        VisibilityDecision::Indeterminate(_)
    ));
    // 未来发布的输入同样不可视为在 T 已可用 → 不能作为派生可用性的下界。
    let future = version(b"t101f", b"v1", 1, 40, RevisionOperation::Upsert);
    let future_ctx = context(
        30,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    assert!(matches!(
        evaluate_visibility(future, future_ctx).unwrap(),
        VisibilityDecision::NotVisible(DecisionReason::ReleaseAfterCutoff)
    ));
    // 可见性判定不早于输入：任一输入不可见时不得给出「可用」结论。
    let base_keys = [base.identity().fact_key()];
    let unproven_candidates = [unproven];
    let selected = select_fact_as_of(
        &unproven_candidates,
        coverage(&base_keys, 20, CoverageState::Complete),
        ctx,
    );
    assert!(!matches!(selected, Ok(FactOutcome::Selected { .. })));
}

// TP-T102 完成时间下界：所有输入已知，但计算/交付尚未完成 → 真实系统输出不可消费。
#[test]
fn tp_t102_completion_time_lower_bound_not_consumable() {
    // 输入在 T=20 已可见。
    let input = version(b"t102", b"v1", 1, 10, RevisionOperation::Upsert);
    let keys = [input.identity().fact_key()];
    let ctx = context(
        20,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    assert!(matches!(
        evaluate_visibility(input, ctx).unwrap(),
        VisibilityDecision::Visible
    ));
    // 但计算/交付完成于 40：在 T=20 的真实系统输出不可消费。
    // crate 只证明输入可见性；完成事件与消费边界证据由消费方提供（§17.1）。
    let completion = version(b"t102c", b"v1", 1, 40, RevisionOperation::Upsert);
    let completion_ctx = context(
        30,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    assert!(matches!(
        evaluate_visibility(completion, completion_ctx).unwrap(),
        VisibilityDecision::NotVisible(DecisionReason::ReleaseAfterCutoff)
    ));
    // 完成事件在 T=20 不可见 ⇒ 不能声称真实系统输出可消费。
    assert!(matches!(
        evaluate_visibility(completion, ctx).unwrap(),
        VisibilityDecision::NotVisible(_)
    ));
    // 输入可见不等于输出可用：严格查询只返回输入的选定结果。
    let input_candidates = [input];
    let selected = select_fact_as_of(
        &input_candidates,
        coverage(&keys, 20, CoverageState::Complete),
        ctx,
    )
    .unwrap();
    assert_eq!(selected_revision(selected), Some(b"v1".as_slice()));
    let _ = completion_ctx;
}

// TP-T103 今日重算历史：今天生成特征而写历史 system_available → 拒绝历史伪造。
#[test]
fn tp_t103_today_recompute_rejects_actual_execution_forgery() {
    // 来源证据可证明历史发布，但不能伪造「系统 receipt」。
    let sept = version(b"t103", b"v1", 1, 10, RevisionOperation::Upsert);
    let forged = system_version(sept, "rx1", 20);
    let ctx = context(
        20,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    // 来源模式下 system receipt 不生效（由 source 发布判定）。
    assert!(matches!(
        evaluate_visibility(forged, ctx).unwrap(),
        VisibilityDecision::Visible
    ));
    // 系统模式下：无系统证据的候选不因来源发布而自动可见（crate 不把
    // source 证据冒充为系统可用性）。
    let no_system_evidence = with_order(sept, sept.revision_order(), None);
    let sys = system_context(20, 1);
    assert!(!matches!(
        evaluate_visibility(no_system_evidence, sys).unwrap(),
        VisibilityDecision::Visible
    ));
    // 而带系统证据的候选在系统模式下可被独立证明（两种证据不互相冒充）。
    assert!(matches!(
        evaluate_visibility(forged, sys).unwrap(),
        VisibilityDecision::Visible
    ));
    // 今日重算不能写历史 system_available：crate 无执行模式 API，
    // 模式隔离由消费方按 §17.2 实现；此处断言 receipt 与 source 证据不互相冒充。
    assert_eq!(forged.receipts().len(), 1);
    assert_eq!(forged.system_evidence().len(), 1);
    assert!(forged.source_evidence().is_some());
}

// TP-T104 反事实模拟：今天运行但每步只用当时许可输入 → 可标 CounterfactualSimulation，
// 不能标真实历史执行。
#[test]
fn tp_t104_counterfactual_simulation_mode_isolation() {
    // 模拟知识时间 T=20：只用当时许可输入。
    let input = version(b"t104", b"v1", 1, 10, RevisionOperation::Upsert);
    let keys = [input.identity().fact_key()];
    let ctx = context(
        20,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    let input_candidates = [input];
    let selected = select_fact_as_of(
        &input_candidates,
        coverage(&keys, 20, CoverageState::Complete),
        ctx,
    )
    .unwrap();
    assert_eq!(selected_revision(selected), Some(b"v1".as_slice()));
    // 未来输入不得进入模拟决策（模拟的 declared knowledge basis/cutoff 固定）。
    let future = version(b"t104f", b"v2", 2, 40, RevisionOperation::Upsert);
    assert!(matches!(
        evaluate_visibility(future, ctx).unwrap(),
        VisibilityDecision::NotVisible(_)
    ));
    // 模拟结果与真实历史执行是两种模式：crate 不把模拟时间当历史证据。
    // 模式标记由消费方保存（§17.2），此处断言输入时点隔离成立。
    let uncertain = version_bounds(b"t104u", b"v1", 1, 20, 15, 25, RevisionOperation::Upsert);
    let u_keys = [uncertain.identity().fact_key()];
    assert!(matches!(
        select_fact_as_of(
            &[uncertain],
            coverage(&u_keys, 20, CoverageState::Complete),
            ctx
        ),
        Err(PitError::IndeterminateSelection)
    ));
}

// TP-T105 PIT join：一侧存在未来修订，另一侧已知 → 关联选择仅使用两侧各自当时版本。
#[test]
fn tp_t105_pit_join_uses_each_side_at_its_own_time() {
    let left = version(b"t105l", b"v1", 1, 10, RevisionOperation::Upsert);
    let right_old = version(b"t105r", b"v1", 1, 10, RevisionOperation::Upsert);
    let right_future = version(b"t105r", b"v2", 2, 40, RevisionOperation::Upsert);
    let left_keys = [left.identity().fact_key()];
    let right_keys = [right_old.identity().fact_key()];
    let ctx = context(
        20,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    // 左侧当时版本：v1。
    let left_candidates = [left];
    let left_selected = select_fact_as_of(
        &left_candidates,
        coverage(&left_keys, 20, CoverageState::Complete),
        ctx,
    )
    .unwrap();
    assert_eq!(selected_revision(left_selected), Some(b"v1".as_slice()));
    // 右侧当时版本：v1（未来 v2 不得参与 join）。
    let right_candidates = [right_old, right_future];
    let right_selected = select_fact_as_of(
        &right_candidates,
        coverage(&right_keys, 20, CoverageState::Complete),
        ctx,
    )
    .unwrap();
    assert_eq!(selected_revision(right_selected), Some(b"v1".as_slice()));
    // 未来修订在 T=20 不可见：join 不会先取全量最终值再过滤。
    assert!(matches!(
        evaluate_visibility(right_future, ctx).unwrap(),
        VisibilityDecision::NotVisible(DecisionReason::ReleaseAfterCutoff)
    ));
    // 两侧各自当时版本一致，但身份不同（不因 revision_id 相同而合并）。
    assert_ne!(
        left_selected_identity(left_selected),
        right_selected_identity(right_selected)
    );
}

// TP-T106 模型实际可用性：样本截止早于 T、模型部署晚于 T → 不能进入 T 的真实系统决策。
#[test]
fn tp_t106_model_availability_requires_deployment_before_t() {
    // 样本截止 10（早于 T=20），模型部署 40（晚于 T）。
    let sample = version(b"t106s", b"v1", 1, 10, RevisionOperation::Upsert);
    let deployment = version(b"t106d", b"v1", 1, 40, RevisionOperation::Upsert);
    let sample_keys = [sample.identity().fact_key()];
    let deployment_keys = [deployment.identity().fact_key()];
    let ctx = context(
        20,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    // 样本在 T 可见，但部署不可见 ⇒ 模型在 T 不可用。
    assert!(matches!(
        evaluate_visibility(sample, ctx).unwrap(),
        VisibilityDecision::Visible
    ));
    assert!(matches!(
        evaluate_visibility(deployment, ctx).unwrap(),
        VisibilityDecision::NotVisible(_)
    ));
    // 严格查询在 T=20 不得给出部署版本作为可用模型。
    let deployment_candidates = [deployment];
    let selected = select_fact_as_of(
        &deployment_candidates,
        coverage(&deployment_keys, 20, CoverageState::Complete),
        ctx,
    );
    assert!(!matches!(selected, Ok(FactOutcome::Selected { .. })));
    // 样本可见不等于模型可用：部署边界是独立证据。
    let sample_candidates = [sample];
    assert!(matches!(
        select_fact_as_of(
            &sample_candidates,
            coverage(&sample_keys, 20, CoverageState::Complete),
            ctx
        )
        .unwrap(),
        FactOutcome::Selected { .. }
    ));
}

// T107 固定快照重放：相同 context/policy/revisions/实现多次执行 → 结果一致。
#[test]
fn tp_t107_fixed_snapshot_replay_is_deterministic() {
    let a = version(b"t107", b"v1", 1, 10, RevisionOperation::Upsert);
    let keys = [a.identity().fact_key()];
    let ctx = context(
        20,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    let candidates = [a];
    let first = select_fact_as_of(
        &candidates,
        coverage(&keys, 20, CoverageState::Complete),
        ctx,
    )
    .unwrap();
    let second = select_fact_as_of(
        &candidates,
        coverage(&keys, 20, CoverageState::Complete),
        ctx,
    )
    .unwrap();
    assert_eq!(selected_revision(first), Some(b"v1".as_slice()));
    assert_eq!(selected_revision(second), Some(b"v1".as_slice()));
    // 相同 context/policy/revisions/实现多次执行：语义结果一致。
    assert_eq!(outcome_signature(first), outcome_signature(second));
    // 同一批量入口（AllFacts）两次执行：manifest 与语义结果都一致。
    let batch_ctx = context(
        20,
        OutputScope::AllFacts,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    let batch_cov = coverage(&keys, 20, CoverageState::Complete);
    let run = || {
        select_facts_as_of(
            QueryInputRef {
                candidates: &candidates,
                coverage: batch_cov,
            },
            batch_ctx,
        )
        .unwrap()
    };
    let first_batch = run();
    let second_batch = run();
    assert_eq!(
        batch_signature(&first_batch),
        batch_signature(&second_batch)
    );
    // manifest 绑定的 context 与输入覆盖引用必须与本次查询一致。
    assert_eq!(
        first_batch.manifest.context.known_at(),
        batch_ctx.known_at()
    );
    assert_eq!(
        first_batch.manifest.context.authority_scope(),
        batch_ctx.authority_scope()
    );
    assert_eq!(
        first_batch.manifest.context.dataset_snapshot(),
        batch_ctx.dataset_snapshot()
    );
    assert_eq!(
        first_batch.manifest.context.knowledge_basis(),
        batch_ctx.knowledge_basis()
    );
    assert_eq!(
        first_batch.manifest.coverage_manifest_ref,
        batch_cov.manifest_ref()
    );
    assert_eq!(
        first_batch.manifest.coverage_adapter_ref,
        batch_cov.adapter_ref()
    );
    assert_eq!(
        first_batch.manifest.coverage_acceptance_ref,
        batch_cov.acceptance_ref()
    );
    // 三次执行仍一致（覆盖「多次执行」而非仅两次）。
    assert_eq!(batch_signature(&run()), batch_signature(&first_batch));
}

/// 单事实结果的语义签名；不依赖未派生的 PartialEq。
fn outcome_signature(outcome: FactOutcome<'_>) -> Vec<u8> {
    match outcome {
        FactOutcome::Selected { version, .. } => {
            let mut value = version.authority_scope().as_bytes().to_vec();
            value.extend_from_slice(version.fact_key().as_bytes());
            value.extend_from_slice(version.revision_id().as_bytes());
            value.extend_from_slice(version.semantic_digest().as_bytes());
            value
        }
        FactOutcome::Withdrawn { version, .. } => {
            let mut value = b"withdrawn:".to_vec();
            value.extend_from_slice(version.revision_id().as_bytes());
            value
        }
        FactOutcome::NoVisibleVersion {
            authority_scope,
            fact_key,
        } => {
            let mut value = b"none:".to_vec();
            value.extend_from_slice(authority_scope.as_bytes());
            value.extend_from_slice(fact_key.as_bytes());
            value
        }
    }
}

/// 批量结果的语义签名（outcomes 顺序 + manifest 绑定）。
fn batch_signature(result: &PitResult<'_>) -> Vec<u8> {
    let mut value = Vec::new();
    for outcome in &result.outcomes {
        value.extend_from_slice(&outcome_signature(*outcome));
        value.push(b'|');
    }
    value.extend_from_slice(result.manifest.context.authority_scope().as_bytes());
    value.push(b'|');
    value.extend_from_slice(result.manifest.context.dataset_snapshot().id().as_bytes());
    value.push(b'|');
    value.extend_from_slice(result.manifest.coverage_manifest_ref.as_bytes());
    value.push(b'|');
    value.extend_from_slice(result.manifest.coverage_adapter_ref.as_bytes());
    value.push(b'|');
    value.extend_from_slice(result.manifest.coverage_acceptance_ref.as_bytes());
    value
}

// T108 合法未来追加：新增都在 cutoff 后才可见，旧证据不变 → 历史语义一致。
#[test]
fn tp_t108_legal_future_append_keeps_historical_result() {
    let old = version(b"t108", b"v1", 1, 10, RevisionOperation::Upsert);
    let future = version(b"t108", b"v2", 2, 40, RevisionOperation::Upsert);
    let keys = [old.identity().fact_key()];
    let ctx = context(
        20,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    // 仅旧版本可见：即使未来 v2 也作为候选，选择 v1。
    let candidates = [old, future];
    let outcome = select_fact_as_of(
        &candidates,
        coverage(&keys, 20, CoverageState::Complete),
        ctx,
    )
    .unwrap();
    assert_eq!(selected_revision(outcome), Some(b"v1".as_slice()));
    // S/S′ 双快照对比：S′ 是同一事实键的另一个快照（合法未来追加后的快照）。
    // 两次查询的候选集合、policy、cutoff 完全相同，仅快照身份不同。
    let rebind = |base: FactVersionRef<'static>| {
        FactVersionRef::try_new(
            FactVersionParts {
                dataset_snapshot: snapshot2(),
                profile: base.profile(),
                source_release: base.source_release(),
                identity: base.identity(),
                input_provenance_ref: base.input_provenance_ref(),
                operation: base.operation(),
                payload_ref: base.payload_ref(),
                source_temporal: base.source_temporal(),
                revision_order: base.revision_order(),
                source_evidence: base.source_evidence(),
                receipts: base.receipts(),
                system_evidence: base.system_evidence(),
                effective_interval: base.effective_interval(),
            },
            &limits(),
        )
        .unwrap()
    };
    let old_s_prime = rebind(old);
    let future_s_prime = rebind(future);
    let s_prime_candidates = [old_s_prime, future_s_prime];
    let ctx_s_prime = {
        let policy = QueryPolicy::try_new(
            TemporalProfile::PublishedObservation,
            EvidenceAcceptance::ConservativeBounds,
            RevisionOrderPolicy::AuthoritativeSequence,
            OutputScope::SingleFact,
            "policy1",
            limits(),
        )
        .unwrap();
        AsOfContext::try_new(AsOfContextParts {
            known_at: time(20),
            knowledge_basis: KnowledgeBasis::SourcePublishedAsOf,
            authority_scope: "authority1",
            source_scope: Some("source1"),
            consumer_boundary: None,
            dataset_snapshot: snapshot2(),
            system_cut: None,
            policy,
            fact_schema_ref: "fact1",
            version_schema_ref: "version1",
            evidence_schema_ref: "evidence1",
            interpretation_ref: "interpretation1",
        })
        .unwrap()
    };
    let s_prime_coverage = CandidateCoverageRef::try_new(
        "authority1",
        &keys,
        snapshot2(),
        KnowledgeBasis::SourcePublishedAsOf,
        time(20),
        CoverageState::Complete,
        "coverage1",
        "adapter1",
        "accepted1",
        &limits(),
    )
    .unwrap();
    let outcome_s_prime =
        select_fact_as_of(&s_prime_candidates, s_prime_coverage, ctx_s_prime).unwrap();
    // S 与 S′ 所选事实版本语义一致（同一修订身份），快照身份不同不改变历史结果。
    assert_eq!(selected_revision(outcome_s_prime), Some(b"v1".as_slice()));
    assert_eq!(
        selected_revision(outcome),
        selected_revision(outcome_s_prime)
    );
    assert_eq!(
        outcome_signature(outcome),
        outcome_signature(outcome_s_prime)
    );
    assert_ne!(snapshot().id(), snapshot2().id());
    // 未来 v2 在 S′ 中同样不可见：历史结果不因合法未来追加而改变。
    assert!(matches!(
        evaluate_visibility(future_s_prime, ctx_s_prime).unwrap(),
        VisibilityDecision::NotVisible(DecisionReason::ReleaseAfterCutoff)
    ));
}

// T109 来源历史补全例外：新增确于 T 前发布的旧版本使用新 snapshot → 新结果允许改善，
// 旧 snapshot 不变。
#[test]
fn tp_t109_backfill_exception_improves_new_snapshot_only() {
    let lim = limits();
    let keys = [FactKeyRef::try_new(b"t109", &lim).unwrap()];
    // 旧 snapshot 只有 v2；新 snapshot 补入 v1（更早发布）。
    let v2 = version(b"t109", b"v2", 2, 20, RevisionOperation::Upsert);
    let v1 = version(b"t109", b"v1", 1, 10, RevisionOperation::Upsert);
    let ctx = context(
        20,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    // 新快照可同时提供 v1 与 v2；截止 20 时 v2 最新。
    let candidates = [v1, v2];
    let outcome = select_fact_as_of(
        &candidates,
        coverage(&keys, 20, CoverageState::Complete),
        ctx,
    )
    .unwrap();
    assert_eq!(selected_revision(outcome), Some(b"v2".as_slice()));
    // 若新快照补充了更早发布但仍可见的 v1，旧快照结果（仅 v2）不因 v1 而改变。
    let old_candidates = [v2];
    let outcome_old = select_fact_as_of(
        &old_candidates,
        coverage(&keys, 20, CoverageState::Complete),
        ctx,
    )
    .unwrap();
    assert_eq!(selected_revision(outcome_old), Some(b"v2".as_slice()));
    // 新研究结果允许改善：新快照补入确于 T 前发布的旧版本 v1 后，
    // 若 v1 是截止前发布的更晚可见修订，新快照结果可与旧快照不同。
    // 构造改善场景：旧快照只见 v1；新快照补入 v2（确于 T 前发布，序号更大）。
    let old_only = version(b"t109b", b"v1", 1, 10, RevisionOperation::Upsert);
    let new_append = version(b"t109b", b"v2", 2, 15, RevisionOperation::Upsert);
    let keys_b = [old_only.identity().fact_key()];
    let ctx_b = context(
        20,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    let old_candidates = [old_only];
    let old_result = select_fact_as_of(
        &old_candidates,
        coverage(&keys_b, 20, CoverageState::Complete),
        ctx_b,
    )
    .unwrap();
    assert_eq!(selected_revision(old_result), Some(b"v1".as_slice()));
    let new_candidates = [old_only, new_append];
    let new_result = select_fact_as_of(
        &new_candidates,
        coverage(&keys_b, 20, CoverageState::Complete),
        ctx_b,
    )
    .unwrap();
    assert_eq!(selected_revision(new_result), Some(b"v2".as_slice()));
    // 新快照结果允许改善（v1 → v2），旧快照结果不变。
    assert_ne!(selected_revision(old_result), selected_revision(new_result));
    // 旧 snapshot 查询仍返回旧结果：补全不改变旧快照的可见性判定。
    assert_eq!(
        selected_revision(
            select_fact_as_of(
                &[old_only],
                coverage(&keys_b, 20, CoverageState::Complete),
                ctx_b
            )
            .unwrap()
        ),
        Some(b"v1".as_slice())
    );
}

// T110 跨快照混页：分页中途切换 snapshot → ContextMismatch，不合并为完整结果。
#[test]
fn tp_t110_cross_snapshot_pagination_context_mismatch() {
    let a = version(b"t110", b"v1", 1, 10, RevisionOperation::Upsert);
    // 覆盖声明用 snapshot s2，而候选与上下文绑定 s1。
    let keys = [a.identity().fact_key()];
    let ctx = context(
        20,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    let foreign_coverage = CandidateCoverageRef::try_new(
        "authority1",
        &keys,
        snapshot2(),
        KnowledgeBasis::SourcePublishedAsOf,
        time(20),
        CoverageState::Complete,
        "coverage1",
        "adapter1",
        "accepted1",
        &limits(),
    )
    .unwrap();
    let candidates = [a];
    assert!(matches!(
        select_fact_as_of(&candidates, foreign_coverage, ctx),
        Err(PitError::ContextMismatch)
    ));
}

// ── 22.12 诊断、错误、缓存与资源 ───────────────────────────

#[test]
fn tp_t111_future_info_isolation_decision_safe() {
    let old = version(b"t111", b"v1", 1, 10, RevisionOperation::Upsert);
    let future = version(b"t111", b"v2", 2, 30, RevisionOperation::Upsert);
    let keys = [old.identity().fact_key()];
    let ctx = context(
        20,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    // 行政诊断知道未来版本存在。
    let diag_candidates = [old, future];
    let diagnostics = inspect_candidates(
        QueryInputRef {
            candidates: &diag_candidates,
            coverage: coverage(&keys, 20, CoverageState::Complete),
        },
        ctx,
    )
    .unwrap();
    assert_eq!(diagnostics.entries.len(), 2);
    // 诊断明细确实包含未来身份（行政审计用途，不得送入策略/特征计算）。
    assert!(
        diagnostics
            .entries
            .iter()
            .any(|entry| entry.version.revision_id().as_bytes() == b"v2")
    );
    // 但策略/特征入口的 FactOutcome 不得包含未来 ID/数量/载荷/发布时间。
    // 用只有 future 的候选：结果必须为 NoVisibleVersion，不泄漏 future 细节。
    let outcome_candidates = [future];
    let outcome = select_fact_as_of(
        &outcome_candidates,
        coverage(&keys, 20, CoverageState::Complete),
        ctx,
    )
    .unwrap();
    assert!(matches!(outcome, FactOutcome::NoVisibleVersion { .. }));
    // DecisionSafe：NoVisibleVersion 只含 authority_scope 与 fact_key，
    // 不含未来修订 ID、数量、载荷或发布时间。
    let (authority_scope, fact_key) = match outcome {
        FactOutcome::NoVisibleVersion {
            authority_scope,
            fact_key,
        } => (authority_scope, fact_key),
        FactOutcome::Selected { .. } | FactOutcome::Withdrawn { .. } => {
            unreachable!("NoVisibleVersion expected: outcome is {outcome:?}")
        }
    };
    assert_eq!(authority_scope, "authority1");
    assert_eq!(fact_key.as_bytes(), b"t111");
    // 决策输出与诊断输出的结构差异：诊断含未来身份，决策不含。
    let decision_has_future = match outcome {
        FactOutcome::Selected { version, .. } | FactOutcome::Withdrawn { version, .. } => {
            version.revision_id().as_bytes() == b"v2"
        }
        FactOutcome::NoVisibleVersion { .. } => false,
    };
    assert!(!decision_has_future);
    // 批量入口同样隔离：AllFacts 下未来候选不进入任何 outcome。
    let batch_ctx = context(
        20,
        OutputScope::AllFacts,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    let batch_candidates = [old, future];
    let batch = select_facts_as_of(
        QueryInputRef {
            candidates: &batch_candidates,
            coverage: coverage(&keys, 20, CoverageState::Complete),
        },
        batch_ctx,
    )
    .unwrap();
    assert_eq!(batch.outcomes.len(), 1);
    assert!(matches!(
        batch.outcomes[0],
        FactOutcome::Selected { version, .. } if version.revision_id().as_bytes() == b"v1"
    ));
    // 决策输出不含未来修订身份。
    assert!(
        !batch
            .outcomes
            .iter()
            .any(|o| matches!(o, FactOutcome::Selected { version, .. } | FactOutcome::Withdrawn { version, .. } if version.revision_id().as_bytes() == b"v2"))
    );
}

#[test]
fn tp_t112_three_empty_value_semantics_distinct() {
    // no-data → NoVisibleVersion；withdrawn → Withdrawn；证据不足 → Err(IndeterminateSelection)。
    let no_data = version(b"t112", b"v1", 1, 30, RevisionOperation::Upsert);
    let keys = [no_data.identity().fact_key()];
    let ctx = context(
        20,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    assert!(matches!(
        select_fact_as_of(
            &[no_data],
            coverage(&keys, 20, CoverageState::Complete),
            ctx
        )
        .unwrap(),
        FactOutcome::NoVisibleVersion { .. }
    ));
    let withdrawn = version(b"t112", b"w", 1, 10, RevisionOperation::Withdraw);
    let withdrawn_keys = [withdrawn.identity().fact_key()];
    assert!(matches!(
        select_fact_as_of(
            &[withdrawn],
            coverage(&withdrawn_keys, 20, CoverageState::Complete),
            ctx
        )
        .unwrap(),
        FactOutcome::Withdrawn { .. }
    ));
    let uncertain = version_bounds(b"t112", b"u", 1, 20, 15, 25, RevisionOperation::Upsert);
    let uncertain_keys = [uncertain.identity().fact_key()];
    assert!(matches!(
        select_fact_as_of(
            &[uncertain],
            coverage(&uncertain_keys, 20, CoverageState::Complete),
            ctx
        ),
        Err(PitError::IndeterminateSelection)
    ));
}

#[test]
fn tp_t113_error_machine_contract_unique_codes() {
    let time_codes: Vec<&str> = [
        TimeError::Overflow,
        TimeError::InvalidOrder,
        TimeError::InvalidSubsecond,
        TimeError::SystemTimeOutOfRange,
        TimeError::SystemTimePrecisionLoss,
        TimeError::SourceUnavailable,
    ]
    .iter()
    .map(|e| e.code())
    .collect();
    let precision_codes: Vec<&str> = [
        PrecisionError::PrecisionLoss,
        PrecisionError::Overflow,
        PrecisionError::MisalignedProjection,
        PrecisionError::ProjectionMismatch,
        PrecisionError::LosslessRequired,
    ]
    .iter()
    .map(|e| e.code())
    .collect();
    let temporal_codes: Vec<&str> = [
        TemporalError::MissingRequiredRole,
        TemporalError::RoleNotApplicable,
        TemporalError::InvalidDate,
        TemporalError::InvalidPeriod,
        TemporalError::InvalidInterval,
        TemporalError::ProfileMismatch,
        TemporalError::MetadataMismatch,
    ]
    .iter()
    .map(|e| e.code())
    .collect();
    let pit_codes: Vec<&str> = [
        PitError::ContextMismatch,
        PitError::EvidenceMismatch,
        PitError::ConflictingRevision,
        PitError::AmbiguousRevisionOrder,
        PitError::IncomparableOrderScope,
        PitError::InvalidRevisionChain,
        PitError::IndeterminateSelection,
        PitError::UnprovenSystemCut,
        PitError::IncompleteCandidateSet,
        PitError::SnapshotUnavailable,
        PitError::ConflictingEffectiveState,
        PitError::LimitExceeded,
        PitError::ResourceUnavailable,
    ]
    .iter()
    .map(|e| e.code())
    .collect();
    for codes in [&time_codes, &precision_codes, &temporal_codes, &pit_codes] {
        assert!(!codes.is_empty());
        let mut sorted = codes.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            codes.len(),
            sorted.len(),
            "codes must be unique & non-empty: {codes:?}"
        );
    }
    // 跨枚举也保持全局唯一（前缀区分）。
    let mut all: Vec<&str> = Vec::new();
    all.extend(time_codes);
    all.extend(precision_codes);
    all.extend(temporal_codes);
    all.extend(pit_codes);
    all.sort_unstable();
    all.dedup();
    assert!(all.len() >= 4, "at least one code per enum family");
    // Display 文案变化不改变分类：机器码由变体决定，Display 文本可独立演进。
    // 断言 code() 与 Display 文本解耦：同一变体的 code 稳定，Display 可不同。
    let time_display = TimeError::Overflow.to_string();
    let precision_display = PrecisionError::PrecisionLoss.to_string();
    let temporal_display = TemporalError::InvalidInterval.to_string();
    let pit_display = PitError::ContextMismatch.to_string();
    // Display 文本与机器码不是同一字段：Display 变化不影响分类。
    for (display, code) in [
        (time_display.as_str(), "TIME_OVERFLOW"),
        (precision_display.as_str(), "PREC_LOSS_REJECTED"),
        (temporal_display.as_str(), "TEMP_INTERVAL_INVALID"),
        (pit_display.as_str(), "PIT_CONTEXT_MISMATCH"),
    ] {
        assert!(!display.is_empty());
        assert_ne!(display, code);
    }
    // 同一变体的 code 稳定，不随 Display 文案漂移。
    assert_eq!(TimeError::Overflow.code(), "TIME_OVERFLOW");
    assert_eq!(PrecisionError::PrecisionLoss.code(), "PREC_LOSS_REJECTED");
    assert_eq!(
        TemporalError::InvalidInterval.code(),
        "TEMP_INTERVAL_INVALID"
    );
    assert_eq!(PitError::ContextMismatch.code(), "PIT_CONTEXT_MISMATCH");
    // Display 文本可演进（此处只断言非空且不等于机器码），code 保持稳定。
    assert!(!time_display.is_empty());
    assert_ne!(time_display, TimeError::Overflow.code());
    // 全部公开错误变体的 code 都非空且为稳定机器码格式。
    for code in &all {
        assert!(!code.is_empty());
        assert!(code.bytes().all(|b| b.is_ascii_uppercase() || b == b'_'));
    }
}

#[test]
fn tp_t114_time_error_adaptable_and_no_kernel_dependency() {
    // TimeError 变体可独立分类（code() 不同）、Send+Sync，不依赖 kernel。
    assert_ne!(
        TimeError::Overflow.code(),
        TimeError::SourceUnavailable.code()
    );
    assert_ne!(
        TimeError::InvalidOrder.code(),
        TimeError::InvalidSubsecond.code()
    );
    assert_ne!(
        TimeError::SystemTimeOutOfRange.code(),
        TimeError::SystemTimePrecisionLoss.code()
    );
    assert_impl_all!(TimeError: Send, Sync, std::error::Error);
    // 同一 TimeError 出现在外部输入（构造）与系统源失败时，消费方都能映射到稳定 code。
    let external = TimeError::Overflow;
    let system = TimeError::SourceUnavailable;
    assert_eq!(external.code(), "TIME_OVERFLOW");
    assert_eq!(system.code(), "TIME_SOURCE_UNAVAILABLE");
    // 库生产依赖无 kernel。
    let manifest = include_str!("../Cargo.toml");
    let deps = manifest
        .split("[dependencies]")
        .nth(1)
        .unwrap()
        .split("[dev-dependencies]")
        .next()
        .unwrap();
    assert!(!deps.contains("kernel"), "no kernel dependency");
}

#[test]
fn tp_t115_cache_isolation_by_context() {
    // 同 key/T 但 snapshot 不同 → 不得命中同一缓存键，结果不同。
    let base = version(b"t115", b"v1", 1, 10, RevisionOperation::Upsert);
    let keys = [base.identity().fact_key()];
    let ctx = context(
        20,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    // s1 快照上下文 → Selected。
    assert!(matches!(
        select_fact_as_of(&[base], coverage(&keys, 20, CoverageState::Complete), ctx).unwrap(),
        FactOutcome::Selected { .. }
    ));
    // 不同快照（s2）的候选 → ContextMismatch，而非命中 s1 的缓存结果。
    let foreign_snapshot = FactVersionRef::try_new(
        FactVersionParts {
            dataset_snapshot: snapshot2(),
            profile: base.profile(),
            source_release: base.source_release(),
            identity: base.identity(),
            input_provenance_ref: base.input_provenance_ref(),
            operation: base.operation(),
            payload_ref: base.payload_ref(),
            source_temporal: base.source_temporal(),
            revision_order: base.revision_order(),
            source_evidence: base.source_evidence(),
            receipts: base.receipts(),
            system_evidence: base.system_evidence(),
            effective_interval: base.effective_interval(),
        },
        &limits(),
    )
    .unwrap();
    assert!(matches!(
        evaluate_visibility(foreign_snapshot, ctx),
        Err(PitError::ContextMismatch)
    ));
    // 不同 known_at（截止 5）→ NoVisibleVersion，而非复用截止 20 的结果。
    let early_ctx = context(
        5,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    assert!(matches!(
        select_fact_as_of(
            &[base],
            coverage(&keys, 5, CoverageState::Complete),
            early_ctx
        )
        .unwrap(),
        FactOutcome::NoVisibleVersion { .. }
    ));
    // 维度二：source_scope 不同 → 不得命中同一缓存键。
    let other_source_ctx = {
        let policy = QueryPolicy::try_new(
            TemporalProfile::PublishedObservation,
            EvidenceAcceptance::ConservativeBounds,
            RevisionOrderPolicy::AuthoritativeSequence,
            OutputScope::SingleFact,
            "policy1",
            limits(),
        )
        .unwrap();
        AsOfContext::try_new(AsOfContextParts {
            known_at: time(20),
            knowledge_basis: KnowledgeBasis::SourcePublishedAsOf,
            authority_scope: "authority1",
            source_scope: Some("source2"),
            consumer_boundary: None,
            dataset_snapshot: snapshot(),
            system_cut: None,
            policy,
            fact_schema_ref: "fact1",
            version_schema_ref: "version1",
            evidence_schema_ref: "evidence1",
            interpretation_ref: "interpretation1",
        })
        .unwrap()
    };
    assert!(matches!(
        select_fact_as_of(
            &[base],
            coverage(&keys, 20, CoverageState::Complete),
            other_source_ctx
        ),
        Err(PitError::EvidenceMismatch)
    ));
    // 维度三：system（SystemAsKnown）与 source（SourcePublishedAsOf）不隐式切换。
    // 同一候选在系统模式下不可见（无系统证据），在来源模式下可见。
    let system_ctx = system_context(20, 1);
    assert!(matches!(
        evaluate_visibility(base, system_ctx),
        Ok(VisibilityDecision::Indeterminate(_))
    ));
    assert!(matches!(
        evaluate_visibility(base, ctx),
        Ok(VisibilityDecision::Visible)
    ));
    // 维度四：cut 不同 → 不得命中同一缓存键。
    // 系统模式下 cut 前沿 1 与 2 对同一候选的可见性判定不同。
    let cut1 = system_context(20, 1);
    let cut2 = system_context(20, 2);
    assert_eq!(
        cut1.system_cut().unwrap().partitions()[0].inclusive_frontier(),
        1
    );
    assert_eq!(
        cut2.system_cut().unwrap().partitions()[0].inclusive_frontier(),
        2
    );
    assert_ne!(
        cut1.system_cut().unwrap().partitions()[0].inclusive_frontier(),
        cut2.system_cut().unwrap().partitions()[0].inclusive_frontier()
    );
    // 同一候选在不同 cut 下走不同的可见性路径（不共用缓存结果）。
    let v_cut1 = evaluate_visibility(base, cut1).unwrap();
    let v_cut2 = evaluate_visibility(base, cut2).unwrap();
    assert_eq!(v_cut1, v_cut2); // 该候选无系统证据，两者都不证明可见
    // 有系统证据的候选在不同 cut 前沿下判定不同。
    let with_cut = system_version(base, "rx1", 15);
    let cut_a = system_context(20, 0);
    let cut_b = system_context(20, 1);
    assert!(matches!(
        evaluate_visibility(with_cut, cut_a).unwrap(),
        VisibilityDecision::NotVisible(DecisionReason::OutsideSystemCut)
    ));
    assert!(matches!(
        evaluate_visibility(with_cut, cut_b).unwrap(),
        VisibilityDecision::Visible
    ));
}

#[test]
fn tp_t117_chain_scale_bounded_and_cycle_rejected() {
    let digest = DigestRef::try_new("sha256-v1", b"t117-chain", &limits()).unwrap();
    // 大规模合法链（10 节）选择链尖。
    let mut nodes = Vec::new();
    let mut prev_id = None;
    for i in 1..=10 {
        let key = b"t117";
        let rev: &'static [u8] = Box::leak(format!("v{i}").into_boxed_str()).as_bytes();
        let candidate = version(key, rev, i as u64, 10 + i, RevisionOperation::Upsert);
        let linked = with_order(
            candidate,
            RevisionOrderRef::LinearSupersedes {
                predecessor: prev_id,
                chain_digest: digest,
            },
            candidate.source_evidence(),
        );
        prev_id = Some(candidate.identity().revision_id());
        nodes.push(linked);
    }
    let chain_keys = [nodes[0].identity().fact_key()];
    let ctx = chain_context(40);
    let outcome = select_fact_as_of(
        &nodes,
        coverage(&chain_keys, 40, CoverageState::Complete),
        ctx,
    )
    .unwrap();
    assert_eq!(selected_revision(outcome), Some(b"v10".as_slice()));
    // 恶意循环：v1 → v2 → v1 → 根数非 1 → InvalidRevisionChain。
    let a = version(b"t117c", b"a", 1, 10, RevisionOperation::Upsert);
    let b = version(b"t117c", b"b", 1, 11, RevisionOperation::Upsert);
    let a_chain = with_order(
        a,
        RevisionOrderRef::LinearSupersedes {
            predecessor: Some(b.identity().revision_id()),
            chain_digest: digest,
        },
        a.source_evidence(),
    );
    let b_chain = with_order(
        b,
        RevisionOrderRef::LinearSupersedes {
            predecessor: Some(a.identity().revision_id()),
            chain_digest: digest,
        },
        b.source_evidence(),
    );
    let cyc_keys = [a.identity().fact_key()];
    assert!(matches!(
        select_fact_as_of(
            &[a_chain, b_chain],
            coverage(&cyc_keys, 40, CoverageState::Complete),
            ctx
        ),
        Err(PitError::InvalidRevisionChain)
    ));
    // 大规模有界：1000 节链仍在线性有界工作内存内完成选择，不二次退化。
    // 1000 候选的索引预留约需 ~100KB，故给足工作内存预算。
    let big_limits =
        ResourceLimits::try_new(256, 16, 8, 2048, 16, 2048, 2048, 16, 8 * 1024 * 1024).unwrap();
    let big_digest = DigestRef::try_new("sha256-v1", b"t117-big", &big_limits).unwrap();
    let mut big_nodes = Vec::new();
    let mut big_prev = None;
    for i in 1..=1000u64 {
        let key = b"t117big";
        let rev: &'static [u8] = Box::leak(format!("v{i}").into_boxed_str()).as_bytes();
        let candidate = version_at(key, rev, i, 10 + i as i64, &big_limits);
        let linked = FactVersionRef::try_new(
            FactVersionParts {
                dataset_snapshot: candidate.dataset_snapshot(),
                profile: candidate.profile(),
                source_release: candidate.source_release(),
                identity: candidate.identity(),
                input_provenance_ref: candidate.input_provenance_ref(),
                operation: candidate.operation(),
                payload_ref: candidate.payload_ref(),
                source_temporal: candidate.source_temporal(),
                revision_order: RevisionOrderRef::LinearSupersedes {
                    predecessor: big_prev,
                    chain_digest: big_digest,
                },
                source_evidence: candidate.source_evidence(),
                receipts: candidate.receipts(),
                system_evidence: candidate.system_evidence(),
                effective_interval: candidate.effective_interval(),
            },
            &big_limits,
        )
        .unwrap();
        big_prev = Some(candidate.identity().revision_id());
        big_nodes.push(linked);
    }
    let big_keys = [big_nodes[0].identity().fact_key()];
    let big_policy = QueryPolicy::try_new(
        TemporalProfile::PublishedObservation,
        EvidenceAcceptance::ConservativeBounds,
        RevisionOrderPolicy::LinearSupersedes,
        OutputScope::SingleFact,
        "policy1",
        big_limits,
    )
    .unwrap();
    let big_ctx = AsOfContext::try_new(AsOfContextParts {
        known_at: time(2000),
        knowledge_basis: KnowledgeBasis::SourcePublishedAsOf,
        authority_scope: "authority1",
        source_scope: Some("source1"),
        consumer_boundary: None,
        dataset_snapshot: snapshot(),
        system_cut: None,
        policy: big_policy,
        fact_schema_ref: "fact1",
        version_schema_ref: "version1",
        evidence_schema_ref: "evidence1",
        interpretation_ref: "interpretation1",
    })
    .unwrap();
    let big_outcome = select_fact_as_of(
        &big_nodes,
        coverage(&big_keys, 2000, CoverageState::Complete),
        big_ctx,
    )
    .unwrap();
    assert_eq!(selected_revision(big_outcome), Some(b"v1000".as_slice()));
    // 有界工作内存：1000 候选的预留估算不得超过策略上限（否则应 LimitExceeded）。
    let per_candidate = std::mem::size_of::<FactVersionRef<'_>>()
        .checked_mul(4)
        .and_then(|value| value.checked_add(std::mem::size_of::<u64>()))
        .and_then(|value| value.checked_add(std::mem::size_of::<bool>()))
        .unwrap();
    let index_bytes = big_nodes.len().checked_mul(per_candidate).unwrap();
    let reserved = index_bytes
        .checked_mul(2)
        .and_then(|value| value.checked_add(1024))
        .unwrap();
    assert!(
        reserved <= big_limits.max_working_bytes(),
        "1000-node chain must stay within working budget: {reserved} bytes"
    );
    // 超链深上限：max_chain_depth=8 时 1000 节链必须 LimitExceeded（不无限递归）。
    let shallow_limits =
        ResourceLimits::try_new(256, 16, 8, 2048, 16, 2048, 8, 16, 8 * 1024 * 1024).unwrap();
    let shallow_policy = QueryPolicy::try_new(
        TemporalProfile::PublishedObservation,
        EvidenceAcceptance::ConservativeBounds,
        RevisionOrderPolicy::LinearSupersedes,
        OutputScope::SingleFact,
        "policy1",
        shallow_limits,
    )
    .unwrap();
    let shallow_ctx = AsOfContext::try_new(AsOfContextParts {
        known_at: time(2000),
        knowledge_basis: KnowledgeBasis::SourcePublishedAsOf,
        authority_scope: "authority1",
        source_scope: Some("source1"),
        consumer_boundary: None,
        dataset_snapshot: snapshot(),
        system_cut: None,
        policy: shallow_policy,
        fact_schema_ref: "fact1",
        version_schema_ref: "version1",
        evidence_schema_ref: "evidence1",
        interpretation_ref: "interpretation1",
    })
    .unwrap();
    assert!(matches!(
        select_fact_as_of(
            &big_nodes,
            coverage(&big_keys, 2000, CoverageState::Complete),
            shallow_ctx
        ),
        Err(PitError::LimitExceeded)
    ));
}

#[test]
fn tp_t119_backend_equivalence_single_input_documented_outcome() {
    // 同一 snapshot/coverage 经 Rust 选择器：单一已知输入 → 文档化结果。
    let upsert = version(b"t119", b"v1", 1, 10, RevisionOperation::Upsert);
    let keys = [upsert.identity().fact_key()];
    let ctx = context(
        20,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    let selected_candidates = [upsert];
    let selected = select_fact_as_of(
        &selected_candidates,
        coverage(&keys, 20, CoverageState::Complete),
        ctx,
    )
    .unwrap();
    assert!(matches!(selected, FactOutcome::Selected { .. }));
    // 选版：选中的修订身份与载荷与输入一致。
    let (selected_id, selected_op) = match selected {
        FactOutcome::Selected {
            version, operation, ..
        } => (version, operation),
        FactOutcome::Withdrawn { .. } | FactOutcome::NoVisibleVersion { .. } => {
            unreachable!("Selected expected: outcome is {selected:?}")
        }
    };
    assert_eq!(selected_id.revision_id().as_bytes(), b"v1");
    assert_eq!(selected_op, RevisionOperation::Upsert);
    let withdrawal = version(b"t119", b"w", 1, 10, RevisionOperation::Withdraw);
    let w_keys = [withdrawal.identity().fact_key()];
    let withdrawn_candidates = [withdrawal];
    let withdrawn = select_fact_as_of(
        &withdrawn_candidates,
        coverage(&w_keys, 20, CoverageState::Complete),
        ctx,
    )
    .unwrap();
    assert!(matches!(withdrawn, FactOutcome::Withdrawn { .. }));
    let withdrawn_id = match withdrawn {
        FactOutcome::Withdrawn { version, .. } => version,
        FactOutcome::Selected { .. } | FactOutcome::NoVisibleVersion { .. } => {
            unreachable!("Withdrawn expected: outcome is {withdrawn:?}")
        }
    };
    assert_eq!(withdrawn_id.revision_id().as_bytes(), b"w");
    let uncertain = version_bounds(b"t119", b"u", 1, 20, 15, 25, RevisionOperation::Upsert);
    let u_keys = [uncertain.identity().fact_key()];
    assert!(matches!(
        select_fact_as_of(
            &[uncertain],
            coverage(&u_keys, 20, CoverageState::Complete),
            ctx
        ),
        Err(PitError::IndeterminateSelection)
    ));
    // 同一输入在批量入口（AllFacts）下结果一致（选版/撤回/Indeterminate 处理一致）。
    let batch_ctx = context(
        20,
        OutputScope::AllFacts,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    let batch_candidates = [upsert];
    let batch = select_facts_as_of(
        QueryInputRef {
            candidates: &batch_candidates,
            coverage: coverage(&keys, 20, CoverageState::Complete),
        },
        batch_ctx,
    )
    .unwrap();
    assert_eq!(batch.outcomes.len(), 1);
    assert!(matches!(batch.outcomes[0], FactOutcome::Selected { .. }));
    // 后端等价对照的边界：本仓只提供 Rust fixture 选择器与文档化结果；
    // 真实存储选择器的等价性由消费方集成证据承担（SPEC §32.1）。
    let manifest = include_str!("../Cargo.toml");
    assert!(
        manifest.contains("publish = false"),
        "fixture-only crate must not claim production backend equivalence"
    );
    // 本仓不声明任何存储后端依赖（真实存储对照在消费方）。
    let deps = manifest
        .split("[dependencies]")
        .nth(1)
        .unwrap()
        .split("[dev-dependencies]")
        .next()
        .unwrap();
    for backend in ["sqlx", "postgres", "redis", "tokio", "rusqlite", "diesel"] {
        assert!(
            !deps.contains(backend),
            "no storage backend dependency expected: {backend}"
        );
    }
}

#[test]
fn tp_t120_delivery_evidence_grading_not_claiming_production() {
    // 交付证据分级：crate 声明 publish=false，不提供可独立发布承诺（非生产完成宣称）。
    let manifest = include_str!("../Cargo.toml");
    assert!(
        manifest.contains("publish = false"),
        "publish must be false"
    );
    assert!(!manifest.contains("crates.io"), "no crates.io promise");
    assert!(!manifest.contains("docs.rs"), "no docs.rs badge/promise");
    // Rust/平台/集成 gate 仍 NOT_RUN：README 的本地门禁与验收检查表都声明
    // 全部行为验收初始 NOT_RUN，不因文档检查通过而宣称生产完成。
    let readme = include_str!("../README.md");
    assert!(
        readme.contains("publish = false"),
        "README must not claim publishable crate"
    );
    assert!(
        !readme.contains("crates.io") && !readme.contains("docs.rs"),
        "README must not provide crates.io/docs.rs links"
    );
    // 检查表初始状态：全部行为验收 NOT_RUN（不预先打勾）。
    let checklist = include_str!("../checklist.md");
    assert!(
        checklist.contains("初始状态：全部行为验收 NOT_RUN"),
        "checklist must keep initial NOT_RUN state"
    );
    // goal.md 的验收口径：机器全绿不等于合入权。
    let goal = include_str!("../goal.md");
    assert!(
        goal.contains("机器全绿不等于取得合入权"),
        "goal.md must keep the machine-green caveat"
    );
    // 检查表中 TP-T119/T120 仍未勾选（Rust/平台/集成 gate 仍 NOT_RUN）。
    let t120_line = checklist.lines().find(|line| line.contains("TP-T120"));
    assert!(
        t120_line.is_some(),
        "TP-T120 must be registered in checklist.md"
    );
    let t120_line = t120_line.unwrap_or("");
    assert!(
        t120_line.contains("- [ ]"),
        "TP-T120 must remain unchecked: {t120_line}"
    );
    assert!(
        t120_line.contains("NOT_RUN"),
        "TP-T120 must be marked NOT_RUN: {t120_line}"
    );
}

// ── 22.10 复用辅助 ─────────────────────────────────────────

fn assertion_version(
    key: &'static [u8],
    interval: &'static EffectiveInterval,
    limits: &ResourceLimits,
    snapshot: DatasetSnapshotRef<'static>,
) -> FactVersionRef<'static> {
    assertion_revision(
        key,
        (b"v1", 1, 10, RevisionOperation::Upsert),
        interval,
        limits,
        snapshot,
    )
}

fn assertion_revision(
    key: &'static [u8],
    (revision, ordinal, release, operation): (&'static [u8], u64, i64, RevisionOperation),
    interval: &'static EffectiveInterval,
    limits: &ResourceLimits,
    snapshot: DatasetSnapshotRef<'static>,
) -> FactVersionRef<'static> {
    let key = FactKeyRef::try_new(key, limits).unwrap();
    let digest = DigestRef::try_new("sha256-v1", key.as_bytes(), limits).unwrap();
    let revision = RevisionIdRef::try_new(revision, limits).unwrap();
    let identity = FactVersionIdRef::try_new("authority1", key, revision, digest, limits).unwrap();
    let draft = TemporalDraft {
        event_time: TimeField::Unknown,
        observation_time: TimeField::Unknown,
        publication_time: TimeField::Known(time(release)),
        received_time: TimeField::Known(time(release + 2)),
        revision_time: TimeField::Unknown,
        effective_time: TimeField::Known(interval.start()),
    };
    let temporal = Box::leak(Box::new(
        draft.validate(TemporalProfile::Announcement).unwrap(),
    ));
    let bounds = TimeBounds::try_new(
        Some(time(release)),
        Some(time(release)),
        TimeQuality::Exact,
        "quality1",
        limits,
    )
    .unwrap();
    let proof = EvidenceProofRef::try_new(
        "proof1",
        "schema1",
        "extractor1",
        "raw1",
        "accepted1",
        limits,
    )
    .unwrap();
    let evidence =
        SourceVisibilityEvidenceRef::try_new(identity, "source1", bounds, proof, limits).unwrap();
    FactVersionRef::try_new(
        FactVersionParts {
            dataset_snapshot: snapshot,
            profile: TemporalProfile::Announcement,
            source_release: SourceReleaseRequirement::Required,
            identity,
            input_provenance_ref: "provenance1",
            operation,
            payload_ref: Some(PayloadRef::try_new("payload1", "v1", limits).unwrap()),
            source_temporal: temporal.source_roles(),
            revision_order: RevisionOrderRef::sequence("order1", ordinal, limits).unwrap(),
            source_evidence: Some(evidence),
            receipts: &[],
            system_evidence: &[],
            effective_interval: Some(interval),
        },
        limits,
    )
    .unwrap()
}

fn with_effective_interval(
    base: FactVersionRef<'static>,
    release: i64,
    limits: &ResourceLimits,
) -> FactVersionRef<'static> {
    // 把未来公告的发布时间推迟到 cutoff 之后（future release），保持有效区间不变。
    let draft = TemporalDraft {
        event_time: TimeField::Unknown,
        observation_time: TimeField::Unknown,
        publication_time: TimeField::Known(time(release)),
        received_time: TimeField::Known(time(release + 2)),
        revision_time: TimeField::Unknown,
        effective_time: TimeField::Known(base.effective_interval().unwrap().start()),
    };
    let temporal = Box::leak(Box::new(
        draft.validate(TemporalProfile::Announcement).unwrap(),
    ));
    let bounds = TimeBounds::try_new(
        Some(time(release)),
        Some(time(release)),
        TimeQuality::Exact,
        "quality1",
        limits,
    )
    .unwrap();
    let proof = base.source_evidence().unwrap().proof();
    let evidence =
        SourceVisibilityEvidenceRef::try_new(base.identity(), "source1", bounds, proof, limits)
            .unwrap();
    FactVersionRef::try_new(
        FactVersionParts {
            dataset_snapshot: base.dataset_snapshot(),
            profile: TemporalProfile::Announcement,
            source_release: base.source_release(),
            identity: base.identity(),
            input_provenance_ref: base.input_provenance_ref(),
            operation: base.operation(),
            payload_ref: base.payload_ref(),
            source_temporal: temporal.source_roles(),
            revision_order: base.revision_order(),
            source_evidence: Some(evidence),
            receipts: base.receipts(),
            system_evidence: base.system_evidence(),
            effective_interval: base.effective_interval(),
        },
        limits,
    )
    .unwrap()
}
