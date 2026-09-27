#![allow(clippy::unwrap_used)]

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use temporal::{PitError, TemporalError, TimeError, UnixTimeNs};

#[test]
fn 显式单位与全域纳秒() {
    for raw in [0, 1, -1, i64::MIN, i64::MAX] {
        assert_eq!(UnixTimeNs::from_unix_nanos(raw).as_unix_nanos(), raw);
    }
    assert_eq!(UnixTimeNs::UNIX_EPOCH.as_unix_nanos(), 0);
    assert_eq!(UnixTimeNs::MIN.as_unix_nanos(), i64::MIN);
    assert_eq!(UnixTimeNs::MAX.as_unix_nanos(), i64::MAX);
    assert_eq!(
        UnixTimeNs::try_from_unix_seconds(-2)
            .unwrap()
            .as_unix_nanos(),
        -2_000_000_000
    );
    assert_eq!(
        UnixTimeNs::try_from_unix_millis(-2)
            .unwrap()
            .as_unix_nanos(),
        -2_000_000
    );
    assert_eq!(
        UnixTimeNs::try_from_unix_micros(-2)
            .unwrap()
            .as_unix_nanos(),
        -2_000
    );
    for scale in [
        UnixTimeNs::try_from_unix_seconds,
        UnixTimeNs::try_from_unix_millis,
        UnixTimeNs::try_from_unix_micros,
    ] {
        assert_eq!(scale(i64::MAX), Err(TimeError::Overflow));
        assert_eq!(scale(i64::MIN), Err(TimeError::Overflow));
    }
}

#[test]
fn 算术拒绝溢出并保留全跨度() {
    let one = Duration::from_nanos(1);
    assert_eq!(UnixTimeNs::MAX.checked_add(one), Err(TimeError::Overflow));
    assert_eq!(UnixTimeNs::MIN.checked_sub(one), Err(TimeError::Overflow));
    assert_eq!(
        UnixTimeNs::UNIX_EPOCH.checked_add(Duration::MAX),
        Err(TimeError::Overflow)
    );
    assert_eq!(
        UnixTimeNs::UNIX_EPOCH.checked_sub(Duration::MAX),
        Err(TimeError::Overflow)
    );
    assert_eq!(
        UnixTimeNs::MAX.duration_since(UnixTimeNs::MIN),
        Ok(Duration::from_nanos(u64::MAX))
    );
    assert_eq!(
        UnixTimeNs::MIN.duration_since(UnixTimeNs::MAX),
        Err(TimeError::InvalidOrder)
    );
    assert_eq!(
        UnixTimeNs::MIN.duration_since(UnixTimeNs::MIN),
        Ok(Duration::ZERO)
    );
    let start = UnixTimeNs::from_unix_nanos(-10);
    assert_eq!(start.checked_add(one).unwrap().checked_sub(one), Ok(start));
}

#[test]
fn 负子秒使用欧几里得表示() {
    for raw in [-1, -1_000_000_001, i64::MIN, i64::MAX] {
        let value = UnixTimeNs::from_unix_nanos(raw);
        let (seconds, nanos) = value.to_seconds_nanos();
        assert!(nanos < 1_000_000_000);
        assert_eq!(
            UnixTimeNs::try_from_seconds_nanos(seconds, nanos),
            Ok(value)
        );
    }
    assert_eq!(
        UnixTimeNs::from_unix_nanos(-1).to_seconds_nanos(),
        (-1, 999_999_999)
    );
    assert_eq!(
        UnixTimeNs::try_from_seconds_nanos(0, 1_000_000_000),
        Err(TimeError::InvalidSubsecond)
    );
    assert_eq!(
        UnixTimeNs::try_from_seconds_nanos(i64::MAX, 0),
        Err(TimeError::Overflow)
    );
}

#[test]
fn 系统时间转换严格无损() {
    for raw in [0, 1, -1, -1_000_000_001, i64::MAX, i64::MIN] {
        let value = UnixTimeNs::from_unix_nanos(raw);
        match value.try_into_system_time() {
            Ok(system) => assert_eq!(UnixTimeNs::try_from_system_time(system), Ok(value)),
            Err(error) => assert!(matches!(
                error,
                TimeError::SystemTimeOutOfRange | TimeError::SystemTimePrecisionLoss
            )),
        }
    }
    assert_eq!(
        UnixTimeNs::try_from_system_time(UNIX_EPOCH),
        Ok(UnixTimeNs::UNIX_EPOCH)
    );
    if let Some(before_epoch) = UNIX_EPOCH.checked_sub(Duration::from_nanos(1)) {
        assert_eq!(
            UnixTimeNs::try_from_system_time(before_epoch),
            Ok(UnixTimeNs::from_unix_nanos(-1))
        );
    }
    if let Some(too_late) = UNIX_EPOCH.checked_add(Duration::from_secs(i64::MAX as u64)) {
        assert_eq!(
            UnixTimeNs::try_from_system_time(too_late),
            Err(TimeError::SystemTimeOutOfRange)
        );
    }
    let _: fn(SystemTime) -> Result<UnixTimeNs, TimeError> = UnixTimeNs::try_from_system_time;
}

#[test]
fn 时间错误码固定() {
    assert_eq!(TimeError::Overflow.code(), "TIME_OVERFLOW");
    assert_eq!(TimeError::InvalidOrder.code(), "TIME_INVALID_ORDER");
    assert_eq!(TimeError::InvalidSubsecond.code(), "TIME_INVALID_SUBSECOND");
    assert_eq!(TimeError::SystemTimeOutOfRange.code(), "TIME_SYSTEM_RANGE");
    assert_eq!(
        TimeError::SystemTimePrecisionLoss.code(),
        "TIME_SYSTEM_PRECISION_LOSS"
    );
    assert_eq!(
        TimeError::SourceUnavailable.code(),
        "TIME_SOURCE_UNAVAILABLE"
    );
}

#[test]
fn 时间模型与查询错误码固定() {
    for (error, code) in [
        (TemporalError::MissingRequiredRole, "TEMP_ROLE_REQUIRED"),
        (TemporalError::RoleNotApplicable, "TEMP_ROLE_NOT_APPLICABLE"),
        (TemporalError::InvalidDate, "TEMP_DATE_INVALID"),
        (TemporalError::InvalidPeriod, "TEMP_PERIOD_INVALID"),
        (TemporalError::InvalidInterval, "TEMP_INTERVAL_INVALID"),
        (TemporalError::ProfileMismatch, "TEMP_PROFILE_MISMATCH"),
        (TemporalError::MetadataMismatch, "TEMP_METADATA_MISMATCH"),
    ] {
        assert_eq!(error.code(), code);
    }
    for (error, code) in [
        (PitError::ContextMismatch, "PIT_CONTEXT_MISMATCH"),
        (PitError::EvidenceMismatch, "PIT_EVIDENCE_MISMATCH"),
        (PitError::ConflictingRevision, "PIT_REVISION_CONFLICT"),
        (
            PitError::AmbiguousRevisionOrder,
            "PIT_REVISION_ORDER_AMBIGUOUS",
        ),
        (
            PitError::IncomparableOrderScope,
            "PIT_ORDER_SCOPE_INCOMPARABLE",
        ),
        (PitError::InvalidRevisionChain, "PIT_REVISION_CHAIN_INVALID"),
        (PitError::IndeterminateSelection, "PIT_INDETERMINATE"),
        (PitError::UnprovenSystemCut, "PIT_SYSTEM_CUT_UNPROVEN"),
        (PitError::IncompleteCandidateSet, "PIT_INCOMPLETE"),
        (PitError::SnapshotUnavailable, "PIT_SNAPSHOT_UNAVAILABLE"),
        (
            PitError::ConflictingEffectiveState,
            "PIT_EFFECTIVE_STATE_CONFLICT",
        ),
        (PitError::LimitExceeded, "PIT_LIMIT"),
        (PitError::ResourceUnavailable, "PIT_RESOURCE_UNAVAILABLE"),
    ] {
        assert_eq!(error.code(), code);
    }
}
