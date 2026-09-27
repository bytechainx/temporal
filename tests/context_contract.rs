use temporal::{
    AsOfContext, AsOfContextParts, CandidateCoverageRef, CoverageState, CutPartitionRef,
    DatasetSnapshotRef, EvidenceAcceptance, FactKeyRef, KnowledgeBasis, OutputScope, PitError,
    QueryPolicy, ResourceLimits, RevisionOrderPolicy, SystemKnowledgeCutRef, TemporalProfile,
    UnixTimeNs,
};

#[test]
fn 查询必须显式绑定模式快照和覆盖() -> Result<(), Box<dyn std::error::Error>> {
    let limits = ResourceLimits::try_new(64, 8, 4, 20, 10, 10, 10, 10, 4096)?;
    let policy = QueryPolicy::try_new(
        TemporalProfile::EventRecord,
        EvidenceAcceptance::ExactOnly,
        RevisionOrderPolicy::AuthoritativeSequence,
        OutputScope::SingleFact,
        "policy-v1",
        limits,
    )?;
    let snapshot = DatasetSnapshotRef::try_new("snapshot-a", "manifest-a", "schema-v1", &limits)?;
    let parts = AsOfContextParts {
        known_at: UnixTimeNs::UNIX_EPOCH,
        knowledge_basis: KnowledgeBasis::SourcePublishedAsOf,
        authority_scope: "authority-a",
        source_scope: Some("source-a"),
        consumer_boundary: None,
        dataset_snapshot: snapshot,
        system_cut: None,
        policy,
        fact_schema_ref: "fact-v1",
        version_schema_ref: "revision-v1",
        evidence_schema_ref: "evidence-v1",
        interpretation_ref: "mapping-v1",
    };
    let context = AsOfContext::try_new(parts)?;
    assert_eq!(
        AsOfContext::try_new(AsOfContextParts {
            source_scope: None,
            ..parts
        })
        .err()
        .ok_or(PitError::ContextMismatch)?,
        PitError::ContextMismatch
    );
    assert_eq!(
        AsOfContext::try_new(AsOfContextParts {
            knowledge_basis: KnowledgeBasis::SystemAsKnown,
            source_scope: None,
            consumer_boundary: Some("consumer-a"),
            ..parts
        })
        .err()
        .ok_or(PitError::ContextMismatch)?,
        PitError::UnprovenSystemCut
    );
    let key = FactKeyRef::try_new(b"fact-a", &limits)?;
    let keys = [key];
    let coverage = CandidateCoverageRef::try_new(
        "authority-a",
        &keys,
        snapshot,
        KnowledgeBasis::SourcePublishedAsOf,
        UnixTimeNs::UNIX_EPOCH,
        CoverageState::Complete,
        "coverage-a",
        "adapter-v1",
        "accepted-v1",
        &limits,
    )?;
    coverage.validate_for(context)?;
    let incomplete = CandidateCoverageRef::try_new(
        "authority-a",
        &keys,
        snapshot,
        KnowledgeBasis::SourcePublishedAsOf,
        UnixTimeNs::UNIX_EPOCH,
        CoverageState::Incomplete,
        "coverage-a",
        "adapter-v1",
        "accepted-v1",
        &limits,
    )?;
    assert_eq!(
        incomplete.validate_for(context),
        Err(PitError::IncompleteCandidateSet)
    );

    let partition = CutPartitionRef::try_new("part-a", "epoch-a", 0, &limits)?;
    let partitions = [partition];
    let cut = SystemKnowledgeCutRef::try_new(
        "consumer-a",
        "epoch-a",
        "cut-a",
        "cut-manifest-a",
        UnixTimeNs::UNIX_EPOCH,
        "cut-binding-v1",
        &partitions,
        "consistency-v1",
        "evidence-snapshot-a",
        &limits,
    )?;
    let publication_policy = QueryPolicy::try_new(
        TemporalProfile::PublishedObservation,
        EvidenceAcceptance::ExactOnly,
        RevisionOrderPolicy::AuthoritativeSequence,
        OutputScope::SingleFact,
        "publication-policy-v1",
        limits,
    )?;
    let system_parts = AsOfContextParts {
        knowledge_basis: KnowledgeBasis::SystemAsKnown,
        source_scope: None,
        consumer_boundary: Some("consumer-a"),
        system_cut: Some(cut),
        policy: publication_policy,
        ..parts
    };
    assert_eq!(
        AsOfContext::try_new(system_parts)
            .err()
            .ok_or(PitError::ContextMismatch)?,
        PitError::ContextMismatch
    );
    assert!(
        AsOfContext::try_new(AsOfContextParts {
            source_scope: Some("source-a"),
            ..system_parts
        })
        .is_ok()
    );
    Ok(())
}
