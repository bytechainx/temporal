//! AUD-001 最小反例（repro）。依据 `docs/audit/AUDIT-VALIDATION.md` 的反例 V1。
//!
//! 场景：版本 V1 只带 Consumer A 的系统可读证据；查询目标是 Consumer B，
//! 而 B 没有任何对应证据。当前实现（`src/pit/visibility.rs:153-159` 跳过边界
//! 不匹配的事件、`:217-221` 据此返回 `NotVisible(OutsideSystemCut)`）把
//! 「A 有证据、B 没有证据」推断成「V1 对 B 确定不可见」。
//!
//! 期望语义（草案 XH-TEMPORAL-REMEDIATION-002 WP-01，规格 §13.1）：缺少目标
//! 消费者的覆盖证明时，判定必须是 `Indeterminate`，不得给出确定的 `NotVisible`。
//!
//! 行为约定：
//! - `cross_consumer_evidence_is_indeterminate` 在未修复代码上必然失败；
//!   WP-01 修复后必然通过。它只断言 `Indeterminate`，不锁定原因码，
//!   原因码由 R1 裁决决定。
//! - 其余三个测试在修复前后都必须通过，用于固定修复边界：他消费者证据既不得
//!   证明目标不可见，也不得证明目标可见；已证明前沿之后的事件仍然 `NotVisible`。
//!
//! 本文件只使用现有公开 API，不改动 `src/`。
#![allow(clippy::unwrap_used)]

use temporal::{
    AsOfContext, AsOfContextParts, CutPartitionRef, DatasetSnapshotRef, DecisionReason, DigestRef,
    EvidenceAcceptance, EvidenceProofRef, FactKeyRef, FactVersionIdRef, FactVersionParts,
    FactVersionRef, KnowledgeBasis, ObservationTime, OutputScope, PayloadRef, QueryPolicy,
    ReceiptEvidenceRef, ResourceLimits, RevisionIdRef, RevisionOperation, RevisionOrderPolicy,
    RevisionOrderRef, SourceReleaseRequirement, SourceVisibilityEvidenceRef, SystemKnowledgeCutRef,
    SystemVisibilityEvidenceRef, TemporalDraft, TemporalProfile, TimeBounds, TimeField,
    TimeQuality, UnixTimeNs, VisibilityDecision, evaluate_visibility,
};

const CONSUMER_A: &str = "consumer1";
const CONSUMER_B: &str = "consumer2";

fn time(value: i64) -> UnixTimeNs {
    UnixTimeNs::from_unix_nanos(value)
}

fn limits() -> ResourceLimits {
    ResourceLimits::try_new(128, 16, 8, 128, 16, 32, 32, 16, 65536).unwrap()
}

fn snapshot() -> DatasetSnapshotRef<'static> {
    DatasetSnapshotRef::try_new("s1", "manifest1", "schema1", &limits()).unwrap()
}

/// V1：来源发布早于截止点 20，自身没有系统可读证据。
fn version(key: &'static [u8], revision: &'static [u8]) -> FactVersionRef<'static> {
    let limits = limits();
    let key = FactKeyRef::try_new(key, &limits).unwrap();
    let revision = RevisionIdRef::try_new(revision, &limits).unwrap();
    let digest = DigestRef::try_new("sha256-v1", revision.as_bytes(), &limits).unwrap();
    let identity = FactVersionIdRef::try_new("authority1", key, revision, digest, &limits).unwrap();
    let draft = TemporalDraft {
        event_time: TimeField::Unknown,
        observation_time: TimeField::Known(ObservationTime::At(time(1000))),
        publication_time: TimeField::Known(time(10)),
        received_time: TimeField::Known(time(100)),
        revision_time: TimeField::Unknown,
        effective_time: TimeField::Unknown,
    };
    let temporal = Box::leak(Box::new(
        draft
            .validate(TemporalProfile::PublishedObservation)
            .unwrap(),
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
    FactVersionRef::try_new(
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
    .unwrap()
}

/// 给 V1 附加一条绑定指定消费者的 receipt 与可读事件（分区 part1）。
fn with_consumer_evidence(
    base: FactVersionRef<'static>,
    boundary: &'static str,
    cursor: u64,
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
        Some(time(15)),
        Some(time(15)),
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
        boundary,
        "log1",
        "event1",
        "receiver1",
        "rx1",
        "receipt1",
        "part1",
        cursor,
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

/// 目标消费者的 SystemAsKnown 上下文：cutoff 20，分区 part1 前沿 `frontier`。
fn query_context(boundary: &'static str, frontier: u64) -> AsOfContext<'static> {
    let limits = limits();
    let partitions = Box::leak(Box::new([CutPartitionRef::try_new(
        "part1", "log1", frontier, &limits,
    )
    .unwrap()]));
    let cut = SystemKnowledgeCutRef::try_new(
        boundary,
        "log1",
        "cut1",
        "cut-manifest1",
        time(20),
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
        known_at: time(20),
        knowledge_basis: KnowledgeBasis::SystemAsKnown,
        authority_scope: "authority1",
        source_scope: Some("source1"),
        consumer_boundary: Some(boundary),
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

/// 对照组：证据绑定的消费者就是查询目标，事件在 Cut 内 → `Visible`。
/// 它证明夹具本身有效，反例失败的原因只能是边界归属。
#[test]
fn own_boundary_evidence_in_cut_is_visible() {
    let v1 = with_consumer_evidence(version(b"aud001-a", b"v1"), CONSUMER_A, 1);
    assert!(matches!(
        evaluate_visibility(v1, query_context(CONSUMER_A, 1)).unwrap(),
        VisibilityDecision::Visible
    ));
}

/// AUD-001 反例：V1 只有 Consumer A 的证据，查询目标是 Consumer B。
/// 期望 `Indeterminate`；未修复代码返回 `NotVisible(OutsideSystemCut)`，必然失败。
#[test]
fn cross_consumer_evidence_is_indeterminate() {
    let v1 = with_consumer_evidence(version(b"aud001-b", b"v1"), CONSUMER_A, 1);
    let decision = evaluate_visibility(v1, query_context(CONSUMER_B, 1)).unwrap();
    assert!(
        matches!(decision, VisibilityDecision::Indeterminate(_)),
        "AUD-001 repro: 目标消费者缺少覆盖证明时必须 Indeterminate，实际为 {decision:?}"
    );
}

/// 修复边界：他消费者证据同样不得证明目标消费者可见。修复前后都必须通过。
#[test]
fn cross_consumer_evidence_never_grants_visibility() {
    let v1 = with_consumer_evidence(version(b"aud001-c", b"v1"), CONSUMER_A, 1);
    let decision = evaluate_visibility(v1, query_context(CONSUMER_B, 1)).unwrap();
    assert!(
        !matches!(decision, VisibilityDecision::Visible),
        "AUD-001 修复不得把 Consumer A 的证据当作 Consumer B 的可见性证明，实际为 {decision:?}"
    );
}

/// 修复边界：事件属于目标消费者边界但 cursor 越过已证明的前沿 → 仍然 `NotVisible`。
/// 这是 `OUTSIDE_SYSTEM_CUT` 的合法用途（规格 §22），修复不得一并放宽。
#[test]
fn event_after_proven_frontier_stays_not_visible() {
    let v1 = with_consumer_evidence(version(b"aud001-d", b"v1"), CONSUMER_A, 2);
    assert!(matches!(
        evaluate_visibility(v1, query_context(CONSUMER_A, 1)).unwrap(),
        VisibilityDecision::NotVisible(DecisionReason::OutsideSystemCut)
    ));
}
