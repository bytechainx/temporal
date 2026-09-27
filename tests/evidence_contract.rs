use temporal::{
    CutPartitionRef, DigestRef, EvidenceAcceptance, EvidenceProofRef, FactKeyRef, FactVersionIdRef,
    PitError, ResourceLimits, RevisionIdRef, SystemKnowledgeCutRef, SystemVisibilityEvidenceRef,
    TimeBounds, TimeDecision, TimeQuality, UnixTimeNs,
};

fn limits() -> Result<ResourceLimits, PitError> {
    ResourceLimits::try_new(64, 8, 4, 20, 10, 10, 10, 10, 4096)
}

fn time(value: i64) -> UnixTimeNs {
    UnixTimeNs::from_unix_nanos(value)
}

#[test]
fn 界限不能冒充精确证据() -> Result<(), Box<dyn std::error::Error>> {
    let limits = limits()?;
    let bounded = TimeBounds::try_new(
        Some(time(10)),
        Some(time(20)),
        TimeQuality::Bounded,
        "day-v1",
        &limits,
    )?;
    assert_eq!(
        bounded.at(time(15), EvidenceAcceptance::ConservativeBounds),
        TimeDecision::Indeterminate
    );
    assert_eq!(
        bounded.at(time(20), EvidenceAcceptance::ConservativeBounds),
        TimeDecision::ProvenByCutoff
    );
    assert_eq!(
        bounded.at(time(20), EvidenceAcceptance::ExactOnly),
        TimeDecision::Indeterminate
    );
    assert_eq!(
        bounded.at(time(9), EvidenceAcceptance::ExactOnly),
        TimeDecision::NotYetVisible
    );
    assert_eq!(
        TimeBounds::try_new(
            Some(time(20)),
            Some(time(10)),
            TimeQuality::Bounded,
            "day-v1",
            &limits
        ),
        Err(PitError::EvidenceMismatch)
    );
    assert_eq!(
        TimeBounds::try_new(
            Some(time(10)),
            Some(time(20)),
            TimeQuality::Exact,
            "clock-v1",
            &limits
        ),
        Err(PitError::EvidenceMismatch)
    );
    Ok(())
}

#[test]
fn 历史切片必须同时匹配边界代次分区和截止点() -> Result<(), Box<dyn std::error::Error>> {
    let limits = limits()?;
    let key = FactKeyRef::try_new(b"fact", &limits)?;
    let revision = RevisionIdRef::try_new(b"v1", &limits)?;
    let digest = DigestRef::try_new("sha256-v1", b"digest", &limits)?;
    let version = FactVersionIdRef::try_new("authority", key, revision, digest, &limits)?;
    let proof = EvidenceProofRef::try_new(
        "e1",
        "schema-v1",
        "extract-v1",
        "raw-v1",
        "accepted-v1",
        &limits,
    )?;
    let exact = TimeBounds::try_new(
        Some(time(10)),
        Some(time(10)),
        TimeQuality::Exact,
        "clock-v1",
        &limits,
    )?;
    let event = SystemVisibilityEvidenceRef::try_new(
        version,
        "consumer-a",
        "epoch-a",
        "event-1",
        "receiver-a",
        "receiver-epoch-a",
        "receipt-1",
        "part-1",
        2,
        exact,
        proof,
        &limits,
    )?;
    let partition = CutPartitionRef::try_new("part-1", "epoch-a", 2, &limits)?;
    let partitions = [partition];
    let cut = SystemKnowledgeCutRef::try_new(
        "consumer-a",
        "epoch-a",
        "cut-1",
        "manifest-v1",
        time(10),
        "binding-v1",
        &partitions,
        "consistent-v1",
        "evidence-snapshot-v1",
        &limits,
    )?;
    assert!(cut.contains(event, "consumer-a", time(10))?);
    assert_eq!(
        cut.contains(event, "consumer-a", time(11)),
        Err(PitError::UnprovenSystemCut)
    );
    assert_eq!(
        cut.contains(event, "consumer-b", time(10)),
        Err(PitError::UnprovenSystemCut)
    );
    let before = CutPartitionRef::try_new("part-1", "epoch-a", 1, &limits)?;
    let old_partitions = [before];
    let old_cut = SystemKnowledgeCutRef::try_new(
        "consumer-a",
        "epoch-a",
        "cut-0",
        "manifest-v0",
        time(10),
        "binding-v1",
        &old_partitions,
        "consistent-v1",
        "evidence-snapshot-v1",
        &limits,
    )?;
    assert!(!old_cut.contains(event, "consumer-a", time(10))?);
    Ok(())
}
