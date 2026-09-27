use temporal::{
    DatasetSnapshotRef, DigestRef, FactKeyRef, FactVersionIdRef, FactVersionParts, FactVersionRef,
    PayloadRef, PitError, ResourceLimits, RevisionIdRef, RevisionOperation, RevisionOrderRef,
    SourceReleaseRequirement, TemporalDraft, TemporalProfile, TimeField, UnixTimeNs,
};

fn limits() -> Result<ResourceLimits, PitError> {
    ResourceLimits::try_new(64, 8, 4, 20, 10, 10, 10, 10, 4096)
}

#[test]
fn 完整身份与载荷操作受到校验() -> Result<(), Box<dyn std::error::Error>> {
    let limits = limits()?;
    let key = FactKeyRef::try_new(b"same", &limits)?;
    let revision = RevisionIdRef::try_new(b"v1", &limits)?;
    let digest = DigestRef::try_new("sha256-v1", b"digest", &limits)?;
    let source = FactVersionIdRef::try_new("source-a", key, revision, digest, &limits)?;
    let other = FactVersionIdRef::try_new("source-b", key, revision, digest, &limits)?;
    assert_ne!(source, other);

    let temporal = TemporalDraft {
        event_time: TimeField::Known(UnixTimeNs::UNIX_EPOCH),
        observation_time: TimeField::NotApplicable,
        publication_time: TimeField::Unknown,
        received_time: TimeField::Known(UnixTimeNs::UNIX_EPOCH),
        revision_time: TimeField::Unknown,
        effective_time: TimeField::Unknown,
    }
    .validate(TemporalProfile::EventRecord)?;
    let payload = PayloadRef::try_new("blob-a", "v1", &limits)?;
    let snapshot = DatasetSnapshotRef::try_new("snapshot-a", "manifest-a", "schema-v1", &limits)?;
    let parts = FactVersionParts {
        dataset_snapshot: snapshot,
        profile: TemporalProfile::EventRecord,
        source_release: SourceReleaseRequirement::NotApplicable {
            authority_profile_contract_ref: "event-v1",
        },
        identity: source,
        input_provenance_ref: "raw+mapping-v1",
        operation: RevisionOperation::Upsert,
        payload_ref: Some(payload),
        source_temporal: temporal.source_roles(),
        revision_order: RevisionOrderRef::sequence("vendor-order", 1, &limits)?,
        source_evidence: None,
        receipts: &[],
        system_evidence: &[],
        effective_interval: None,
    };
    assert_eq!(FactVersionRef::try_new(parts, &limits)?.identity(), source);
    assert!(matches!(
        FactVersionRef::try_new(
            FactVersionParts {
                revision_order: RevisionOrderRef::AuthoritativeSequence {
                    order_scope: "",
                    ordinal: 1,
                },
                ..parts
            },
            &limits
        ),
        Err(PitError::ContextMismatch)
    ));
    assert!(
        FactVersionRef::try_new(
            FactVersionParts {
                source_release: SourceReleaseRequirement::Required,
                ..parts
            },
            &limits
        )
        .is_ok()
    );
    assert_eq!(
        FactVersionRef::try_new(
            FactVersionParts {
                source_release: SourceReleaseRequirement::NotApplicable {
                    authority_profile_contract_ref: "",
                },
                ..parts
            },
            &limits
        )
        .err()
        .ok_or(PitError::ContextMismatch)?,
        PitError::ContextMismatch
    );
    assert_eq!(
        FactVersionRef::try_new(
            FactVersionParts {
                operation: RevisionOperation::Withdraw,
                ..parts
            },
            &limits
        )
        .err()
        .ok_or(PitError::ContextMismatch)?,
        PitError::ContextMismatch
    );
    assert_eq!(
        FactKeyRef::try_new(b"", &limits),
        Err(PitError::ContextMismatch)
    );
    assert_eq!(
        FactKeyRef::try_new(&[b'x'; 65], &limits),
        Err(PitError::LimitExceeded)
    );
    Ok(())
}
