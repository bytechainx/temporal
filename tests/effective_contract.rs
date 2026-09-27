#![allow(clippy::unwrap_used)]

use temporal::{
    AsOfContext, AsOfContextParts, CandidateCoverageRef, CoverageState, DatasetSnapshotRef,
    DigestRef, EffectiveInterval, EffectiveStateInputRef, EffectiveStateResult, EvidenceAcceptance,
    EvidenceProofRef, FactKeyRef, FactOutcome, FactVersionIdRef, FactVersionParts, FactVersionRef,
    KnowledgeBasis, OutputScope, PayloadRef, PitError, QueryInputRef, QueryPolicy, ResourceLimits,
    RevisionIdRef, RevisionOperation, RevisionOrderPolicy, RevisionOrderRef,
    SourceReleaseRequirement, SourceVisibilityEvidenceRef, StateAssertionRef, StatePolicy,
    TemporalDraft, TemporalProfile, TimeBounds, TimeField, TimeQuality, UnixTimeNs,
    select_effective_state_as_of,
};

fn time(value: i64) -> UnixTimeNs {
    UnixTimeNs::from_unix_nanos(value)
}

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
            payload_ref: (operation == RevisionOperation::Upsert)
                .then(|| PayloadRef::try_new("payload1", "v1", limits).unwrap()),
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

#[test]
fn tp_t095_empty_complete_scope_has_no_active_state_even_for_future_valid_at() {
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
        known_at: time(10),
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
    let coverage = CandidateCoverageRef::try_new(
        "authority1",
        &[],
        snapshot,
        KnowledgeBasis::SourcePublishedAsOf,
        time(10),
        CoverageState::Complete,
        "coverage1",
        "adapter1",
        "accepted1",
        &limits,
    )
    .unwrap();
    let input = EffectiveStateInputRef {
        facts: QueryInputRef {
            candidates: &[],
            coverage,
        },
        assertions: &[],
        state_key: b"state1",
        state_policy_ref: "state-policy1",
    };
    let outcome = select_effective_state_as_of(
        input,
        context,
        time(100),
        StatePolicy::UniqueNonOverlappingAssertions,
    )
    .unwrap();
    assert!(
        matches!(outcome, EffectiveStateResult::NoActiveState { known_at, valid_at, .. } if known_at == time(10) && valid_at == time(100))
    );
}

#[test]
fn tp_t091_092_093_096_097_announced_future_state_uses_half_open_intervals() {
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
    let old = assertion_version(b"old", old_interval, &limits, snapshot);
    let future = assertion_version(b"future", future_interval, &limits, snapshot);
    let keys = [old.identity().fact_key(), future.identity().fact_key()];
    let candidates = [future, old];
    let coverage = CandidateCoverageRef::try_new(
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
    let assertions = [
        StateAssertionRef {
            fact_key: old.identity().fact_key(),
            state_key: b"state1",
            interval: *old_interval,
        },
        StateAssertionRef {
            fact_key: future.identity().fact_key(),
            state_key: b"state1",
            interval: *future_interval,
        },
    ];
    let input = EffectiveStateInputRef {
        facts: QueryInputRef {
            candidates: &candidates,
            coverage,
        },
        assertions: &assertions,
        state_key: b"state1",
        state_policy_ref: "state-policy1",
    };
    let missing_mapping = EffectiveStateInputRef {
        facts: input.facts,
        assertions: &[],
        state_key: input.state_key,
        state_policy_ref: input.state_policy_ref,
    };
    assert!(matches!(
        select_effective_state_as_of(
            missing_mapping,
            context,
            time(29),
            StatePolicy::UniqueNonOverlappingAssertions
        ),
        Err(PitError::ContextMismatch)
    ));
    let at29 = select_effective_state_as_of(
        input,
        context,
        time(29),
        StatePolicy::UniqueNonOverlappingAssertions,
    )
    .unwrap();
    assert!(
        matches!(at29, EffectiveStateResult::Active { outcome: FactOutcome::Selected { version, .. }, .. } if version.fact_key().as_bytes() == b"old")
    );
    let at30 = select_effective_state_as_of(
        input,
        context,
        time(30),
        StatePolicy::UniqueNonOverlappingAssertions,
    )
    .unwrap();
    assert!(
        matches!(at30, EffectiveStateResult::Active { outcome: FactOutcome::Selected { version, .. }, .. } if version.fact_key().as_bytes() == b"future")
    );
    let overlap_interval = Box::leak(Box::new(
        EffectiveInterval::try_new(time(25), None).unwrap(),
    ));
    let overlap = assertion_version(b"future", overlap_interval, &limits, snapshot);
    let overlap_candidates = [overlap, old];
    let overlapping = [
        assertions[0],
        StateAssertionRef {
            interval: *overlap_interval,
            ..assertions[1]
        },
    ];
    let conflict_input = EffectiveStateInputRef {
        facts: QueryInputRef {
            candidates: &overlap_candidates,
            coverage,
        },
        assertions: &overlapping,
        ..input
    };
    assert!(matches!(
        select_effective_state_as_of(
            conflict_input,
            context,
            time(29),
            StatePolicy::UniqueNonOverlappingAssertions
        ),
        Err(PitError::ConflictingEffectiveState)
    ));
}

#[test]
fn tp_t099_visible_withdrawal_does_not_revive_old_effective_state() {
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
    let context = |known_at| {
        AsOfContext::try_new(AsOfContextParts {
            known_at: time(known_at),
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
        .unwrap()
    };
    let interval = Box::leak(Box::new(EffectiveInterval::try_new(time(0), None).unwrap()));
    let old = assertion_version(b"state-fact", interval, &limits, snapshot);
    let withdrawn = assertion_revision(
        b"state-fact",
        (b"v2", 2, 20, RevisionOperation::Withdraw),
        interval,
        &limits,
        snapshot,
    );
    let keys = [old.identity().fact_key()];
    let assertions = [StateAssertionRef {
        fact_key: keys[0],
        state_key: b"state1",
        interval: *interval,
    }];
    for candidates in [[old, withdrawn], [withdrawn, old]] {
        let evaluate = |known_at| {
            let coverage = CandidateCoverageRef::try_new(
                "authority1",
                &keys,
                snapshot,
                KnowledgeBasis::SourcePublishedAsOf,
                time(known_at),
                CoverageState::Complete,
                "coverage1",
                "adapter1",
                "accepted1",
                &limits,
            )
            .unwrap();
            select_effective_state_as_of(
                EffectiveStateInputRef {
                    facts: QueryInputRef {
                        candidates: &candidates,
                        coverage,
                    },
                    assertions: &assertions,
                    state_key: b"state1",
                    state_policy_ref: "state-policy1",
                },
                context(known_at),
                time(5),
                StatePolicy::UniqueNonOverlappingAssertions,
            )
        };
        assert!(matches!(
            evaluate(15),
            Ok(EffectiveStateResult::Active {
                outcome: FactOutcome::Selected { version, .. },
                ..
            }) if version.revision_id().as_bytes() == b"v1"
        ));
        assert!(matches!(
            evaluate(25),
            Ok(EffectiveStateResult::NoActiveState { .. })
        ));
    }
}
