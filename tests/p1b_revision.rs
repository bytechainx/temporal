//! P1-B / 22.6：事实身份与不可变修订的 10 个场景。
#![allow(clippy::unwrap_used)]

use temporal::{
    AsOfContext, AsOfContextParts, CandidateCoverageRef, CoverageState, DatasetSnapshotRef,
    DigestRef, EvidenceAcceptance, EvidenceProofRef, FactKeyRef, FactOutcome, FactVersionIdRef,
    FactVersionParts, FactVersionRef, KnowledgeBasis, ObservationTime, OutputScope, PayloadRef,
    PitError, QueryInputRef, QueryPolicy, ReceiptEvidenceRef, ResourceLimits, RevisionIdRef,
    RevisionOperation, RevisionOrderPolicy, RevisionOrderRef, SourceReleaseRequirement,
    SourceVisibilityEvidenceRef, TemporalDraft, TemporalProfile, TimeBounds, TimeField,
    TimeQuality, UnixTimeNs, select_fact_as_of, select_facts_as_of,
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

fn context(at: i64, scope: OutputScope) -> AsOfContext<'static> {
    let policy = QueryPolicy::try_new(
        TemporalProfile::PublishedObservation,
        EvidenceAcceptance::ConservativeBounds,
        RevisionOrderPolicy::AuthoritativeSequence,
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

fn make(
    key: &'static [u8],
    revision: &'static [u8],
    digest_bytes: &'static [u8],
    authority: &'static str,
    ordinal: u64,
    release: i64,
    receipts: &'static [ReceiptEvidenceRef<'static>],
) -> FactVersionRef<'static> {
    let limits = limits();
    let key = FactKeyRef::try_new(key, &limits).unwrap();
    let revision = RevisionIdRef::try_new(revision, &limits).unwrap();
    let digest = DigestRef::try_new("sha256-v1", digest_bytes, &limits).unwrap();
    let identity = FactVersionIdRef::try_new(authority, key, revision, digest, &limits).unwrap();
    let draft = TemporalDraft {
        event_time: TimeField::Unknown,
        observation_time: TimeField::Known(ObservationTime::At(time(1_000))),
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
    let bounds = TimeBounds::try_new(
        Some(time(release)),
        Some(time(release)),
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
            revision_order: RevisionOrderRef::sequence("order1", ordinal, &limits).unwrap(),
            source_evidence: Some(evidence),
            receipts,
            system_evidence: &[],
            effective_interval: None,
        },
        &limits,
    )
    .unwrap()
}

fn reproof(base: FactVersionRef<'static>, proof_id: &'static str) -> FactVersionRef<'static> {
    let limits = limits();
    let bounds = base.source_evidence().unwrap().bounds();
    let proof = EvidenceProofRef::try_new(
        proof_id,
        "schema1",
        "extractor1",
        "raw1",
        "accepted1",
        &limits,
    )
    .unwrap();
    let evidence =
        SourceVisibilityEvidenceRef::try_new(base.identity(), "source1", bounds, proof, &limits)
            .unwrap();
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
            source_evidence: Some(evidence),
            receipts: base.receipts(),
            system_evidence: base.system_evidence(),
            effective_interval: base.effective_interval(),
        },
        &limits,
    )
    .unwrap()
}

fn with_order(
    base: FactVersionRef<'static>,
    order: RevisionOrderRef<'static>,
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
            source_evidence: base.source_evidence(),
            receipts: base.receipts(),
            system_evidence: base.system_evidence(),
            effective_interval: base.effective_interval(),
        },
        &limits(),
    )
    .unwrap()
}

fn receipt(
    version: FactVersionIdRef<'static>,
    boundary: &'static str,
    receipt_id: &'static str,
) -> ReceiptEvidenceRef<'static> {
    let limits = limits();
    let bounds = TimeBounds::try_new(
        Some(time(100)),
        Some(time(100)),
        TimeQuality::Exact,
        "quality1",
        &limits,
    )
    .unwrap();
    let proof = EvidenceProofRef::try_new(
        receipt_id,
        "schema1",
        "extractor1",
        "raw1",
        "accepted1",
        &limits,
    )
    .unwrap();
    ReceiptEvidenceRef::try_new(
        version, boundary, "epoch1", receipt_id, bounds, proof, &limits,
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

fn selected_revision<'a>(outcome: FactOutcome<'a>) -> Option<&'a [u8]> {
    match outcome {
        FactOutcome::Selected { version, .. } | FactOutcome::Withdrawn { version, .. } => {
            Some(version.revision_id().as_bytes())
        }
        _ => None,
    }
}

#[test]
fn tp_t051_same_version_same_content_is_idempotent() {
    let base = make(b"fact-dup", b"v1", b"digest-a", "authority1", 1, 10, &[]);
    let first_receipt = receipt(base.identity(), "boundary-a", "receipt-1");
    let second_receipt = receipt(base.identity(), "boundary-b", "receipt-2");
    let receipts: &'static [ReceiptEvidenceRef<'static>] =
        Box::leak(vec![first_receipt, second_receipt].into_boxed_slice());
    let version = FactVersionRef::try_new(
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
            system_evidence: base.system_evidence(),
            effective_interval: base.effective_interval(),
        },
        &limits(),
    )
    .unwrap();
    let key = [version.identity().fact_key()];
    let candidates = [version, version];
    let outcome = select_fact_as_of(
        &candidates,
        coverage(&key, 20, CoverageState::Complete),
        context(20, OutputScope::SingleFact),
    )
    .unwrap();
    assert_eq!(selected_revision(outcome), Some(b"v1".as_slice()));
    let all = select_facts_as_of(
        QueryInputRef {
            candidates: &candidates,
            coverage: coverage(&key, 20, CoverageState::Complete),
        },
        context(20, OutputScope::AllFacts),
    )
    .unwrap();
    assert_eq!(all.outcomes.len(), 1);
}

#[test]
fn tp_t052_same_version_different_digest_conflicts() {
    let first = make(
        b"fact-conflict",
        b"v1",
        b"digest-a",
        "authority1",
        1,
        10,
        &[],
    );
    let second = reproof(
        make(
            b"fact-conflict",
            b"v1",
            b"digest-b",
            "authority1",
            1,
            10,
            &[],
        ),
        "proof2",
    );
    let key = [first.identity().fact_key()];
    for candidates in [[first, second], [second, first]] {
        let got = select_fact_as_of(
            &candidates,
            coverage(&key, 20, CoverageState::Complete),
            context(20, OutputScope::SingleFact),
        );
        assert!(matches!(got, Err(PitError::ConflictingRevision)));
    }
}

#[test]
fn tp_t053_same_content_different_receipts_is_not_a_new_revision() {
    let base = make(
        b"fact-receipt",
        b"v1",
        b"digest-a",
        "authority1",
        1,
        10,
        &[],
    );
    let east = receipt(base.identity(), "boundary-east", "receipt-east");
    let west = receipt(base.identity(), "boundary-west", "receipt-west");
    let east_receipts: &'static [ReceiptEvidenceRef<'static>] =
        Box::leak(vec![east].into_boxed_slice());
    let west_receipts: &'static [ReceiptEvidenceRef<'static>] =
        Box::leak(vec![west].into_boxed_slice());
    let with = |receipts: &'static [ReceiptEvidenceRef<'static>]| {
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
                system_evidence: base.system_evidence(),
                effective_interval: base.effective_interval(),
            },
            &limits(),
        )
        .unwrap()
    };
    let first = with(east_receipts);
    let second = with(west_receipts);
    let key = [first.identity().fact_key()];
    let candidates = [first, second];
    let all = select_facts_as_of(
        QueryInputRef {
            candidates: &candidates,
            coverage: coverage(&key, 20, CoverageState::Complete),
        },
        context(20, OutputScope::AllFacts),
    )
    .unwrap();
    assert_eq!(all.outcomes.len(), 1);
    assert_eq!(selected_revision(all.outcomes[0]), Some(b"v1".as_slice()));
}

#[test]
fn tp_t054_same_ids_across_facts_and_authorities_are_not_merged() {
    let left = make(b"fact-a", b"v1", b"digest-a", "authority1", 1, 10, &[]);
    let right = make(b"fact-b", b"v1", b"digest-a", "authority1", 1, 10, &[]);
    let keys = [left.identity().fact_key(), right.identity().fact_key()];
    let candidates = [left, right];
    let all = select_facts_as_of(
        QueryInputRef {
            candidates: &candidates,
            coverage: coverage(&keys, 20, CoverageState::Complete),
        },
        context(20, OutputScope::AllFacts),
    )
    .unwrap();
    assert_eq!(all.outcomes.len(), 2);
    for outcome in all.outcomes {
        assert_eq!(selected_revision(outcome), Some(b"v1".as_slice()));
    }
    let foreign = make(b"fact-a", b"v1", b"digest-a", "authority2", 1, 10, &[]);
    let key = [foreign.identity().fact_key()];
    assert!(matches!(
        select_fact_as_of(
            &[foreign],
            coverage(&key, 20, CoverageState::Complete),
            context(20, OutputScope::SingleFact),
        ),
        Err(PitError::ContextMismatch)
    ));
}

#[test]
fn tp_t055_same_instant_events_remain_distinct_facts() {
    let first = make(b"event-a", b"v1", b"digest-a", "authority1", 1, 10, &[]);
    let second = make(b"event-b", b"v1", b"digest-a", "authority1", 1, 10, &[]);
    let keys = [first.identity().fact_key(), second.identity().fact_key()];
    let candidates = [first, second];
    let all = select_facts_as_of(
        QueryInputRef {
            candidates: &candidates,
            coverage: coverage(&keys, 20, CoverageState::Complete),
        },
        context(20, OutputScope::AllFacts),
    )
    .unwrap();
    assert_eq!(all.outcomes.len(), 2);
    assert_ne!(
        first.identity().fact_key().as_bytes(),
        second.identity().fact_key().as_bytes()
    );
}

#[test]
fn tp_t056_late_arriving_older_revision_does_not_win() {
    let older = make(b"fact-late", b"v1", b"digest-a", "authority1", 1, 30, &[]);
    let newer = make(b"fact-late", b"v2", b"digest-b", "authority1", 2, 10, &[]);
    let key = [older.identity().fact_key()];
    for candidates in [[older, newer], [newer, older]] {
        let outcome = select_fact_as_of(
            &candidates,
            coverage(&key, 40, CoverageState::Complete),
            context(40, OutputScope::SingleFact),
        )
        .unwrap();
        assert_eq!(selected_revision(outcome), Some(b"v2".as_slice()));
    }
}

#[test]
fn tp_t057_equal_ordinals_are_ambiguous() {
    let first = make(b"fact-order", b"v1", b"digest-a", "authority1", 1, 10, &[]);
    let second = make(b"fact-order", b"v2", b"digest-b", "authority1", 1, 11, &[]);
    let key = [first.identity().fact_key()];
    for candidates in [[first, second], [second, first]] {
        assert!(matches!(
            select_fact_as_of(
                &candidates,
                coverage(&key, 20, CoverageState::Complete),
                context(20, OutputScope::SingleFact),
            ),
            Err(PitError::AmbiguousRevisionOrder)
        ));
    }
}

#[test]
fn tp_t058_distinct_order_scopes_are_incomparable() {
    let first = make(b"fact-scope", b"v1", b"digest-a", "authority1", 2, 10, &[]);
    let second = make(b"fact-scope", b"v2", b"digest-b", "authority1", 1, 11, &[]);
    let second = with_order(
        second,
        RevisionOrderRef::sequence("order2", 1, &limits()).unwrap(),
    );
    let key = [first.identity().fact_key()];
    for candidates in [[first, second], [second, first]] {
        assert!(matches!(
            select_fact_as_of(
                &candidates,
                coverage(&key, 20, CoverageState::Complete),
                context(20, OutputScope::SingleFact),
            ),
            Err(PitError::IncomparableOrderScope)
        ));
    }
}

#[test]
fn tp_t059_linear_chain_accepts_single_predecessor() {
    let v1 = make(b"fact-chain", b"v1", b"digest-a", "authority1", 1, 10, &[]);
    let v2 = make(b"fact-chain", b"v2", b"digest-b", "authority1", 2, 20, &[]);
    let digest = DigestRef::try_new("sha256-v1", b"chain", &limits()).unwrap();
    let root = with_order(
        v1,
        RevisionOrderRef::LinearSupersedes {
            predecessor: None,
            chain_digest: digest,
        },
    );
    let child = with_order(
        v2,
        RevisionOrderRef::LinearSupersedes {
            predecessor: Some(v1.identity().revision_id()),
            chain_digest: digest,
        },
    );
    let key = [v1.identity().fact_key()];
    let candidates = [child, root];
    let outcome = select_fact_as_of(
        &candidates,
        coverage(&key, 40, CoverageState::Complete),
        chain_context(40),
    )
    .unwrap();
    assert_eq!(selected_revision(outcome), Some(b"v2".as_slice()));
    let self_reference = FactVersionRef::try_new(
        FactVersionParts {
            dataset_snapshot: v2.dataset_snapshot(),
            profile: v2.profile(),
            source_release: v2.source_release(),
            identity: v2.identity(),
            input_provenance_ref: v2.input_provenance_ref(),
            operation: v2.operation(),
            payload_ref: v2.payload_ref(),
            source_temporal: v2.source_temporal(),
            revision_order: RevisionOrderRef::LinearSupersedes {
                predecessor: Some(v2.identity().revision_id()),
                chain_digest: digest,
            },
            source_evidence: v2.source_evidence(),
            receipts: v2.receipts(),
            system_evidence: v2.system_evidence(),
            effective_interval: v2.effective_interval(),
        },
        &limits(),
    );
    assert!(matches!(
        self_reference,
        Err(PitError::InvalidRevisionChain)
    ));
}

#[test]
fn tp_t060_branched_revisions_fail_closed() {
    let v1 = make(b"fact-branch", b"v1", b"digest-a", "authority1", 1, 10, &[]);
    let v2 = make(b"fact-branch", b"v2", b"digest-b", "authority1", 2, 20, &[]);
    let v3 = make(b"fact-branch", b"v3", b"digest-c", "authority1", 3, 30, &[]);
    let digest = DigestRef::try_new("sha256-v1", b"branch", &limits()).unwrap();
    let root = with_order(
        v1,
        RevisionOrderRef::LinearSupersedes {
            predecessor: None,
            chain_digest: digest,
        },
    );
    let left = with_order(
        v2,
        RevisionOrderRef::LinearSupersedes {
            predecessor: Some(v1.identity().revision_id()),
            chain_digest: digest,
        },
    );
    let right = with_order(
        v3,
        RevisionOrderRef::LinearSupersedes {
            predecessor: Some(v1.identity().revision_id()),
            chain_digest: digest,
        },
    );
    let key = [v1.identity().fact_key()];
    for candidates in [[root, left, right], [right, root, left]] {
        assert!(matches!(
            select_fact_as_of(
                &candidates,
                coverage(&key, 40, CoverageState::Complete),
                chain_context(40),
            ),
            Err(PitError::InvalidRevisionChain)
        ));
    }
}
