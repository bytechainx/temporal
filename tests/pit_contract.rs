#![allow(clippy::unwrap_used)]

use temporal::{
    AsOfContext, AsOfContextParts, CandidateCoverageRef, CoverageState, CutPartitionRef,
    DatasetSnapshotRef, DigestRef, EvidenceAcceptance, EvidenceProofRef, FactKeyRef, FactOutcome,
    FactVersionIdRef, FactVersionParts, FactVersionRef, KnowledgeBasis, ObservationTime,
    OutputScope, PayloadRef, PitError, QueryInputRef, QueryPolicy, ReceiptEvidenceRef,
    ResourceLimits, RevisionIdRef, RevisionOperation, RevisionOrderPolicy, RevisionOrderRef,
    SourceReleaseRequirement, SourceVisibilityEvidenceRef, SystemKnowledgeCutRef,
    SystemVisibilityEvidenceRef, TemporalDraft, TemporalProfile, TimeBounds, TimeField,
    TimeQuality, UnixTimeNs, VisibilityDecision, evaluate_visibility, inspect_candidates,
    select_fact_as_of, select_facts_as_of,
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

fn version(
    key: &'static [u8],
    revision: &'static [u8],
    ordinal: u64,
    release: i64,
    operation: RevisionOperation,
) -> FactVersionRef<'static> {
    version_bounds(key, revision, ordinal, release, release, release, operation)
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
    let limits = limits();
    let key = FactKeyRef::try_new(key, &limits).unwrap();
    let revision = RevisionIdRef::try_new(revision, &limits).unwrap();
    let digest = DigestRef::try_new("sha256-v1", revision.as_bytes(), &limits).unwrap();
    let identity = FactVersionIdRef::try_new("authority1", key, revision, digest, &limits).unwrap();
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
    let payload_ref = if operation == RevisionOperation::Upsert {
        Some(PayloadRef::try_new("payload1", "v1", &limits).unwrap())
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
            revision_order: RevisionOrderRef::sequence("order1", ordinal, &limits).unwrap(),
            source_evidence: Some(evidence),
            receipts: &[],
            system_evidence: &[],
            effective_interval: None,
        },
        &limits,
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

#[test]
fn tp_t061_065_081_future_and_uncertain_cannot_displace_known_revision() {
    let old = version(b"cpi", b"v1", 1, 10, RevisionOperation::Upsert);
    let future = version(b"cpi", b"v2", 2, 30, RevisionOperation::Upsert);
    let ctx = context(
        20,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    assert!(matches!(
        evaluate_visibility(future, ctx).unwrap(),
        VisibilityDecision::NotVisible(_)
    ));
    let keys = [old.identity().fact_key()];
    let candidates = [future, old];
    let selected = select_fact_as_of(
        &candidates,
        coverage(&keys, 20, CoverageState::Complete),
        ctx,
    )
    .unwrap();
    assert!(
        matches!(selected, FactOutcome::Selected { version, .. } if version.revision_id().as_bytes() == b"v1")
    );
    let early = context(
        1,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    assert!(matches!(
        select_fact_as_of(
            &[future, old],
            coverage(&keys, 1, CoverageState::Complete),
            early
        )
        .unwrap(),
        FactOutcome::NoVisibleVersion { .. }
    ));
    let uncertain = version_bounds(b"cpi", b"v3", 3, 20, 15, 25, RevisionOperation::Upsert);
    assert!(matches!(
        evaluate_visibility(uncertain, ctx).unwrap(),
        VisibilityDecision::Indeterminate(_)
    ));
    assert!(matches!(
        select_fact_as_of(
            &[old, uncertain],
            coverage(&keys, 20, CoverageState::Complete),
            ctx
        ),
        Err(PitError::IndeterminateSelection)
    ));
}

#[test]
fn tp_t082_083_withdraw_and_restore_follow_visible_order() {
    let old = version(b"key", b"v1", 1, 10, RevisionOperation::Upsert);
    let withdrawal = version(b"key", b"v2", 2, 20, RevisionOperation::Withdraw);
    let restored = version(b"key", b"v3", 3, 30, RevisionOperation::Upsert);
    let keys = [old.identity().fact_key()];
    let at25 = context(
        25,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    assert!(matches!(
        select_fact_as_of(
            &[restored, old, withdrawal],
            coverage(&keys, 25, CoverageState::Complete),
            at25
        )
        .unwrap(),
        FactOutcome::Withdrawn { .. }
    ));
    let at35 = context(
        35,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    assert!(
        matches!(select_fact_as_of(&[withdrawal, restored, old], coverage(&keys, 35, CoverageState::Complete), at35).unwrap(), FactOutcome::Selected { version, .. } if version.revision_id().as_bytes() == b"v3")
    );
}

#[test]
fn tp_t080_086_089_incomplete_or_cross_fact_inputs_fail_strictly() {
    let a = version(b"a", b"v1", 1, 10, RevisionOperation::Upsert);
    let b = version(b"b", b"v1", 1, 10, RevisionOperation::Upsert);
    let keys = [a.identity().fact_key(), b.identity().fact_key()];
    let ctx = context(
        20,
        OutputScope::AllFacts,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    let candidates = [b, a];
    let result = select_facts_as_of(
        QueryInputRef {
            candidates: &candidates,
            coverage: coverage(&keys, 20, CoverageState::Complete),
        },
        ctx,
    )
    .unwrap();
    assert_eq!(result.manifest.context.dataset_snapshot(), snapshot());
    assert_eq!(result.manifest.context.known_at(), time(20));
    assert_eq!(result.manifest.context.policy().policy_ref(), "policy1");
    assert_eq!(result.manifest.context.fact_schema_ref(), "fact1");
    assert_eq!(result.manifest.coverage_manifest_ref, "coverage1");
    assert!(
        matches!(result.outcomes[0], FactOutcome::Selected { version, .. } if version.fact_key().as_bytes() == b"a")
    );
    assert!(matches!(
        select_facts_as_of(
            QueryInputRef {
                candidates: &[a, b],
                coverage: coverage(&keys, 20, CoverageState::Incomplete)
            },
            ctx
        ),
        Err(PitError::IncompleteCandidateSet)
    ));
    let only_a = [a.identity().fact_key()];
    assert!(matches!(
        select_facts_as_of(
            QueryInputRef {
                candidates: &[a, b],
                coverage: coverage(&only_a, 20, CoverageState::Complete)
            },
            ctx
        ),
        Err(PitError::ContextMismatch)
    ));
}

#[test]
fn tp_t071_072_system_requires_receipt_scope_time_and_cut() {
    let base = version(b"event", b"v1", 1, 10, RevisionOperation::Upsert);
    let valid = system_version(base, "rx1", 15);
    assert!(matches!(
        evaluate_visibility(valid, system_context(20, 1)).unwrap(),
        VisibilityDecision::Visible
    ));
    assert!(matches!(
        evaluate_visibility(valid, system_context(20, 0)).unwrap(),
        VisibilityDecision::NotVisible(_)
    ));
    let wrong_epoch = system_version(base, "rx2", 15);
    assert!(matches!(
        evaluate_visibility(wrong_epoch, system_context(20, 1)).unwrap(),
        VisibilityDecision::Indeterminate(_)
    ));
    let future_receipt = system_version(base, "rx1", 30);
    assert!(matches!(
        evaluate_visibility(future_receipt, system_context(20, 1)).unwrap(),
        VisibilityDecision::Indeterminate(_)
    ));
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

#[test]
fn 缺来源证据时明确未来声明仍不可见() {
    let base = version(b"future", b"v1", 1, 30, RevisionOperation::Upsert);
    let missing = with_order(base, base.revision_order(), None);
    assert!(matches!(
        evaluate_visibility(
            missing,
            context(
                20,
                OutputScope::SingleFact,
                RevisionOrderPolicy::AuthoritativeSequence
            )
        ),
        Ok(VisibilityDecision::NotVisible(_))
    ));
}

#[test]
fn 可消费路径与未来路径并存时保留来源不确定性() {
    let base = version_bounds(b"mixed", b"v1", 1, 10, 10, 30, RevisionOperation::Upsert);
    let first = system_version(base, "rx1", 15);
    let original = first.system_evidence()[0];
    let future_bounds = TimeBounds::try_new(
        Some(time(30)),
        Some(time(30)),
        TimeQuality::Exact,
        "quality1",
        &limits(),
    )
    .unwrap();
    let second = SystemVisibilityEvidenceRef::try_new(
        base.identity(),
        "consumer1",
        "log1",
        "event2",
        "receiver1",
        "rx1",
        "receipt1",
        "part1",
        2,
        future_bounds,
        original.proof(),
        &limits(),
    )
    .unwrap();
    let events = Box::leak(Box::new([original, second]));
    let mixed = with_system_evidence(base, first.receipts(), events, &limits()).unwrap();
    assert!(matches!(
        evaluate_visibility(mixed, system_context(20, 2)),
        Ok(VisibilityDecision::Indeterminate(_))
    ));
    let conflicting = SystemVisibilityEvidenceRef::try_new(
        base.identity(),
        "consumer1",
        "log1",
        "event1",
        "receiver1",
        "rx1",
        "receipt1",
        "part1",
        1,
        future_bounds,
        original.proof(),
        &limits(),
    )
    .unwrap();
    let bad_events = Box::leak(Box::new([original, conflicting]));
    assert!(matches!(
        with_system_evidence(base, first.receipts(), bad_events, &limits()),
        Err(PitError::EvidenceMismatch)
    ));
}

#[test]
fn 同版不同接收路径的证据选择与输入顺序无关() {
    let base = version(b"repeat", b"v1", 1, 10, RevisionOperation::Upsert);
    let first = system_version(base, "rx1", 15);
    let proof = first.system_evidence()[0].proof();
    let receipt_bounds = TimeBounds::try_new(
        Some(time(16)),
        Some(time(16)),
        TimeQuality::Exact,
        "quality1",
        &limits(),
    )
    .unwrap();
    let event_bounds = TimeBounds::try_new(
        Some(time(17)),
        Some(time(17)),
        TimeQuality::Exact,
        "quality1",
        &limits(),
    )
    .unwrap();
    let receipts = Box::leak(Box::new([ReceiptEvidenceRef::try_new(
        base.identity(),
        "receiver1",
        "rx1",
        "receipt2",
        receipt_bounds,
        proof,
        &limits(),
    )
    .unwrap()]));
    let events = Box::leak(Box::new([SystemVisibilityEvidenceRef::try_new(
        base.identity(),
        "consumer1",
        "log1",
        "event2",
        "receiver1",
        "rx1",
        "receipt2",
        "part1",
        2,
        event_bounds,
        proof,
        &limits(),
    )
    .unwrap()]));
    let second = with_system_evidence(base, receipts, events, &limits()).unwrap();
    let keys = [base.identity().fact_key()];
    let coverage = CandidateCoverageRef::try_new(
        "authority1",
        &keys,
        snapshot(),
        KnowledgeBasis::SystemAsKnown,
        time(20),
        CoverageState::Complete,
        "coverage1",
        "adapter1",
        "accepted1",
        &limits(),
    )
    .unwrap();
    let context = system_context(20, 2);
    for candidates in [[first, second], [second, first]] {
        let result = select_fact_as_of(&candidates, coverage, context).unwrap();
        assert!(matches!(result, FactOutcome::Selected { evidence, .. }
            if evidence.system.is_some_and(|item| item.event_id() == "event1")));
    }
}

#[test]
fn 查询策略重新约束证据数量且行政诊断独立显式请求() {
    let old = version(b"audit", b"v1", 1, 10, RevisionOperation::Upsert);
    let future = version(b"audit", b"v2", 2, 30, RevisionOperation::Upsert);
    let keys = [old.identity().fact_key()];
    let ctx = context(
        20,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    let candidates = [old, future];
    let diagnostics = inspect_candidates(
        QueryInputRef {
            candidates: &candidates,
            coverage: coverage(&keys, 20, CoverageState::Complete),
        },
        ctx,
    )
    .unwrap();
    assert_eq!(diagnostics.entries.len(), 2);
    assert!(matches!(
        diagnostics.entries[1].visibility,
        VisibilityDecision::NotVisible(_)
    ));

    let base = version(b"limited", b"v1", 1, 10, RevisionOperation::Upsert);
    let first = system_version(base, "rx1", 15);
    let events = Box::leak(Box::new([
        first.system_evidence()[0],
        first.system_evidence()[0],
    ]));
    let two = with_system_evidence(base, first.receipts(), events, &limits()).unwrap();
    let narrow = ResourceLimits::try_new(128, 1, 8, 128, 16, 32, 32, 16, 65536).unwrap();
    let original = system_context(20, 1);
    let policy = QueryPolicy::try_new(
        TemporalProfile::PublishedObservation,
        EvidenceAcceptance::ExactOnly,
        RevisionOrderPolicy::AuthoritativeSequence,
        OutputScope::SingleFact,
        "policy1",
        narrow,
    )
    .unwrap();
    let narrow_context = AsOfContext::try_new(AsOfContextParts {
        known_at: original.known_at(),
        knowledge_basis: original.knowledge_basis(),
        authority_scope: original.authority_scope(),
        source_scope: original.source_scope(),
        consumer_boundary: original.consumer_boundary(),
        dataset_snapshot: original.dataset_snapshot(),
        system_cut: original.system_cut(),
        policy,
        fact_schema_ref: original.fact_schema_ref(),
        version_schema_ref: original.version_schema_ref(),
        evidence_schema_ref: original.evidence_schema_ref(),
        interpretation_ref: original.interpretation_ref(),
    })
    .unwrap();
    let limited_keys = [base.identity().fact_key()];
    let limited_coverage = CandidateCoverageRef::try_new(
        "authority1",
        &limited_keys,
        snapshot(),
        KnowledgeBasis::SystemAsKnown,
        time(20),
        CoverageState::Complete,
        "coverage1",
        "adapter1",
        "accepted1",
        &limits(),
    )
    .unwrap();
    assert!(matches!(
        evaluate_visibility(two, narrow_context),
        Err(PitError::LimitExceeded)
    ));
    assert!(matches!(
        select_fact_as_of(&[two], limited_coverage, narrow_context),
        Err(PitError::LimitExceeded)
    ));
}

#[test]
fn 同一来源证据身份的矛盾内容拒绝选版() {
    let base = version(b"source-conflict", b"v1", 1, 10, RevisionOperation::Upsert);
    let old = base.source_evidence().unwrap();
    let conflicting_proof = EvidenceProofRef::try_new(
        old.proof().evidence_id(),
        "schema1",
        "extractor1",
        "different-raw",
        "accepted1",
        &limits(),
    )
    .unwrap();
    let conflicting_source = SourceVisibilityEvidenceRef::try_new(
        base.identity(),
        old.source_scope(),
        old.bounds(),
        conflicting_proof,
        &limits(),
    )
    .unwrap();
    let changed = with_order(base, base.revision_order(), Some(conflicting_source));
    let keys = [base.identity().fact_key()];
    let ctx = context(
        20,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    for candidates in [[base, changed], [changed, base]] {
        assert!(matches!(
            select_fact_as_of(
                &candidates,
                coverage(&keys, 20, CoverageState::Complete),
                ctx
            ),
            Err(PitError::EvidenceMismatch)
        ));
    }
}

#[test]
fn 同一消费事件不能跨修订绑定不同版本() {
    let old = version(b"shared-event", b"v1", 1, 10, RevisionOperation::Upsert);
    let new = version(b"shared-event", b"v2", 2, 11, RevisionOperation::Upsert);
    let old = system_version(old, "rx1", 15);
    let new = system_version(new, "rx1", 15);
    let keys = [old.identity().fact_key()];
    let coverage = CandidateCoverageRef::try_new(
        "authority1",
        &keys,
        snapshot(),
        KnowledgeBasis::SystemAsKnown,
        time(20),
        CoverageState::Complete,
        "coverage1",
        "adapter1",
        "accepted1",
        &limits(),
    )
    .unwrap();
    assert!(matches!(
        select_fact_as_of(&[old, new], coverage, system_context(20, 1)),
        Err(PitError::EvidenceMismatch)
    ));
}

#[test]
fn tp_t059_060_070_missing_proof_and_invalid_chain_fail_closed() {
    let v1 = version(b"chain", b"v1", 1, 10, RevisionOperation::Upsert);
    let v2 = version(b"chain", b"v2", 2, 20, RevisionOperation::Upsert);
    let v3 = version(b"chain", b"v3", 3, 30, RevisionOperation::Upsert);
    let digest = DigestRef::try_new("sha256-v1", b"chain", &limits()).unwrap();
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
    let sibling = with_order(
        v3,
        RevisionOrderRef::LinearSupersedes {
            predecessor: Some(v1.identity().revision_id()),
            chain_digest: digest,
        },
        v3.source_evidence(),
    );
    let key = [v1.identity().fact_key()];
    let ctx = context(
        40,
        OutputScope::SingleFact,
        RevisionOrderPolicy::LinearSupersedes,
    );
    assert!(
        matches!(select_fact_as_of(&[child, root], coverage(&key, 40, CoverageState::Complete), ctx).unwrap(), FactOutcome::Selected { version, .. } if version.revision_id().as_bytes() == b"v2")
    );
    assert!(matches!(
        select_fact_as_of(
            &[root, child, sibling],
            coverage(&key, 40, CoverageState::Complete),
            ctx
        ),
        Err(PitError::InvalidRevisionChain)
    ));
    let unproven = with_order(v2, v2.revision_order(), None);
    assert!(matches!(
        evaluate_visibility(
            unproven,
            context(
                40,
                OutputScope::SingleFact,
                RevisionOrderPolicy::AuthoritativeSequence
            )
        )
        .unwrap(),
        VisibilityDecision::Indeterminate(_)
    ));
}

#[test]
fn tp_t087_same_revision_can_accumulate_independent_accepted_proof() {
    let proven = version(b"dup", b"v1", 1, 10, RevisionOperation::Upsert);
    let missing_proof = with_order(proven, proven.revision_order(), None);
    let key = [proven.identity().fact_key()];
    let ctx = context(
        20,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    let candidates = [missing_proof, proven];
    assert!(matches!(
        select_fact_as_of(
            &candidates,
            coverage(&key, 20, CoverageState::Complete),
            ctx
        )
        .unwrap(),
        FactOutcome::Selected { .. }
    ));
}

#[test]
fn tp_t116_working_budget_rejects_before_allocating_selection_index() {
    let candidate = version(b"limited", b"v1", 1, 10, RevisionOperation::Upsert);
    let key = [candidate.identity().fact_key()];
    for budget in [1, 3217] {
        let constrained = ResourceLimits::try_new(128, 16, 8, 128, 16, 32, 32, 16, budget).unwrap();
        let policy = QueryPolicy::try_new(
            TemporalProfile::PublishedObservation,
            EvidenceAcceptance::ConservativeBounds,
            RevisionOrderPolicy::AuthoritativeSequence,
            OutputScope::SingleFact,
            "policy1",
            constrained,
        )
        .unwrap();
        let ctx = AsOfContext::try_new(AsOfContextParts {
            known_at: time(20),
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
        .unwrap();
        assert!(matches!(
            select_fact_as_of(
                &[candidate],
                coverage(&key, 20, CoverageState::Complete),
                ctx
            ),
            Err(PitError::LimitExceeded)
        ));
    }
}
