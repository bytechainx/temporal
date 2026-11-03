//! P1-A / 22.2：数值基础与算术的 10 个场景。
#![allow(clippy::unwrap_used)]

use std::time::Duration;

use temporal::{TimeError, UnixTimeNs};

#[test]
fn tp_t011_epoch_and_arbitrary_nanos() {
    for value in [0, 1, -1, i64::MIN, i64::MAX] {
        let time = UnixTimeNs::from_unix_nanos(value);
        assert_eq!(time.as_unix_nanos(), value);
    }
}

#[test]
fn tp_t012_signed_unit_construction() {
    assert_eq!(
        UnixTimeNs::try_from_unix_seconds(2)
            .unwrap()
            .as_unix_nanos(),
        2_000_000_000
    );
    assert_eq!(
        UnixTimeNs::try_from_unix_seconds(-2)
            .unwrap()
            .as_unix_nanos(),
        -2_000_000_000
    );
    assert_eq!(
        UnixTimeNs::try_from_unix_millis(3).unwrap().as_unix_nanos(),
        3_000_000
    );
    assert_eq!(
        UnixTimeNs::try_from_unix_millis(-3)
            .unwrap()
            .as_unix_nanos(),
        -3_000_000
    );
    assert_eq!(
        UnixTimeNs::try_from_unix_micros(4).unwrap().as_unix_nanos(),
        4_000
    );
    assert_eq!(
        UnixTimeNs::try_from_unix_micros(-4)
            .unwrap()
            .as_unix_nanos(),
        -4_000
    );
}

#[test]
fn tp_t013_unit_multiplication_overflow() {
    let max_seconds = i64::MAX / 1_000_000_000;
    let min_seconds = i64::MIN / 1_000_000_000;
    assert!(UnixTimeNs::try_from_unix_seconds(max_seconds).is_ok());
    assert_eq!(
        UnixTimeNs::try_from_unix_seconds(max_seconds + 1),
        Err(TimeError::Overflow)
    );
    assert!(UnixTimeNs::try_from_unix_seconds(min_seconds).is_ok());
    assert_eq!(
        UnixTimeNs::try_from_unix_seconds(min_seconds - 1),
        Err(TimeError::Overflow)
    );
    let max_millis = i64::MAX / 1_000_000;
    let min_millis = i64::MIN / 1_000_000;
    assert!(UnixTimeNs::try_from_unix_millis(max_millis).is_ok());
    assert_eq!(
        UnixTimeNs::try_from_unix_millis(max_millis + 1),
        Err(TimeError::Overflow)
    );
    assert!(UnixTimeNs::try_from_unix_millis(min_millis).is_ok());
    assert_eq!(
        UnixTimeNs::try_from_unix_millis(min_millis - 1),
        Err(TimeError::Overflow)
    );
}

#[test]
fn tp_t014_add_and_sub_are_reversible() {
    let base = UnixTimeNs::from_unix_nanos(1_234_567_890);
    let delta = Duration::from_nanos(987_654_321);
    let moved = base.checked_add(delta).unwrap();
    assert_eq!(moved.checked_sub(delta), Ok(base));
}

#[test]
fn tp_t015_extreme_add_and_sub_overflow() {
    assert_eq!(
        UnixTimeNs::MAX.checked_add(Duration::from_nanos(1)),
        Err(TimeError::Overflow)
    );
    assert_eq!(
        UnixTimeNs::MIN.checked_sub(Duration::from_nanos(1)),
        Err(TimeError::Overflow)
    );
}

#[test]
fn tp_t016_full_i64_span() {
    assert_eq!(
        UnixTimeNs::MAX.duration_since(UnixTimeNs::MIN),
        Ok(Duration::from_nanos(u64::MAX))
    );
}

#[test]
fn tp_t017_reverse_and_equal_difference() {
    let earlier = UnixTimeNs::from_unix_nanos(1);
    let later = UnixTimeNs::from_unix_nanos(2);
    assert_eq!(earlier.duration_since(later), Err(TimeError::InvalidOrder));
    assert_eq!(later.duration_since(later), Ok(Duration::ZERO));
}

#[test]
fn tp_t018_duration_max_is_structured() {
    assert_eq!(
        UnixTimeNs::MAX.checked_add(Duration::MAX),
        Err(TimeError::Overflow)
    );
    assert_eq!(
        UnixTimeNs::MIN.checked_sub(Duration::MAX),
        Err(TimeError::Overflow)
    );
    assert_eq!(
        UnixTimeNs::UNIX_EPOCH.checked_add(Duration::MAX),
        Err(TimeError::Overflow)
    );
}

#[test]
fn tp_t019_negative_subsecond_normalization() {
    assert_eq!(
        UnixTimeNs::from_unix_nanos(-1).to_seconds_nanos(),
        (-1, 999_999_999)
    );
    assert_eq!(
        UnixTimeNs::from_unix_nanos(-1_000_000_001).to_seconds_nanos(),
        (-2, 999_999_999)
    );
    for value in [-1, -1_000_000_001, -999_999_999, 0, 1] {
        let (seconds, nanos) = UnixTimeNs::from_unix_nanos(value).to_seconds_nanos();
        assert_eq!(
            UnixTimeNs::try_from_seconds_nanos(seconds, nanos),
            Ok(UnixTimeNs::from_unix_nanos(value))
        );
    }
}

#[test]
fn tp_t020_subsecond_and_range_validation() {
    assert_eq!(
        UnixTimeNs::try_from_seconds_nanos(0, 999_999_999),
        Ok(UnixTimeNs::from_unix_nanos(999_999_999))
    );
    assert_eq!(
        UnixTimeNs::try_from_seconds_nanos(0, 1_000_000_000),
        Err(TimeError::InvalidSubsecond)
    );
    assert_eq!(
        UnixTimeNs::try_from_seconds_nanos(i64::MAX, 999_999_999),
        Err(TimeError::Overflow)
    );
    assert_eq!(
        UnixTimeNs::try_from_seconds_nanos(i64::MIN, 0),
        Err(TimeError::Overflow)
    );
}
