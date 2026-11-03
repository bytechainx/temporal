//! P1-B / 22.5：精度、量化与投影的 10 个场景。
#![allow(clippy::unwrap_used)]

use temporal::{
    PrecisionError, PrecisionLossPolicy, TimePrecision, TimeStorageCapabilities, quantize_unix_ns,
    require_lossless_unix_ns, stored_to_unix_ns, unix_ns_to_stored, verify_projection,
};

const POLICIES: [PrecisionLossPolicy; 3] = [
    PrecisionLossPolicy::Reject,
    PrecisionLossPolicy::ExplicitTruncate,
    PrecisionLossPolicy::ExplicitFloor,
];

#[test]
fn tp_t041_default_policy_rejects_subunit_loss() {
    assert_eq!(PrecisionLossPolicy::default(), PrecisionLossPolicy::Reject);
    for (value, precision) in [
        (1_i64, TimePrecision::Microseconds),
        (999, TimePrecision::Microseconds),
        (1, TimePrecision::Milliseconds),
        (999_999, TimePrecision::Milliseconds),
    ] {
        assert_eq!(
            unix_ns_to_stored(value, precision, PrecisionLossPolicy::default()),
            Err(PrecisionError::PrecisionLoss)
        );
        assert_eq!(
            quantize_unix_ns(value, precision, PrecisionLossPolicy::default()),
            Err(PrecisionError::PrecisionLoss)
        );
    }
}

#[test]
fn tp_t042_truncate_is_symmetric_around_zero() {
    assert_eq!(
        unix_ns_to_stored(
            1_500_000_001,
            TimePrecision::Milliseconds,
            PrecisionLossPolicy::ExplicitTruncate
        ),
        Ok(1_500)
    );
    assert_eq!(
        unix_ns_to_stored(
            -1_500_000_001,
            TimePrecision::Milliseconds,
            PrecisionLossPolicy::ExplicitTruncate
        ),
        Ok(-1_500)
    );
    for value in [1_999_999_i64, -1_999_999] {
        assert_eq!(
            unix_ns_to_stored(
                value,
                TimePrecision::Milliseconds,
                PrecisionLossPolicy::ExplicitTruncate
            ),
            Ok(value / 1_000_000)
        );
    }
}

#[test]
fn tp_t043_floor_keeps_negative_subunit() {
    assert_eq!(
        unix_ns_to_stored(
            -1,
            TimePrecision::Microseconds,
            PrecisionLossPolicy::ExplicitFloor
        ),
        Ok(-1)
    );
    assert_eq!(
        unix_ns_to_stored(
            -1,
            TimePrecision::Microseconds,
            PrecisionLossPolicy::ExplicitTruncate
        ),
        Ok(0)
    );
    assert_eq!(
        unix_ns_to_stored(
            -1,
            TimePrecision::Milliseconds,
            PrecisionLossPolicy::ExplicitFloor
        ),
        Ok(-1)
    );
}

#[test]
fn tp_t044_lossless_units_round_trip_under_reject() {
    for (value, precision) in [
        (0_i64, TimePrecision::Milliseconds),
        (1_000_000, TimePrecision::Milliseconds),
        (-1_000_000, TimePrecision::Milliseconds),
        (1_000, TimePrecision::Microseconds),
        (-1_000, TimePrecision::Microseconds),
        (1_000_000_000, TimePrecision::Seconds),
        (-1_000_000_000, TimePrecision::Seconds),
    ] {
        let stored = unix_ns_to_stored(value, precision, PrecisionLossPolicy::Reject).unwrap();
        assert_eq!(stored_to_unix_ns(stored, precision), Ok(value));
        assert_eq!(
            quantize_unix_ns(value, precision, PrecisionLossPolicy::Reject),
            Ok(value)
        );
    }
}

#[test]
fn tp_t045_stored_multiplication_does_not_wrap() {
    for precision in [
        TimePrecision::Seconds,
        TimePrecision::Milliseconds,
        TimePrecision::Microseconds,
    ] {
        assert_eq!(
            stored_to_unix_ns(i64::MAX, precision),
            Err(PrecisionError::Overflow)
        );
        assert_eq!(
            stored_to_unix_ns(i64::MIN, precision),
            Err(PrecisionError::Overflow)
        );
        assert_eq!(
            stored_to_unix_ns(i64::MAX / precision.nanos_per_unit(), precision),
            Ok((i64::MAX / precision.nanos_per_unit()) * precision.nanos_per_unit())
        );
    }
}

#[test]
fn tp_t046_floor_restore_underflow_is_explicit() {
    assert_eq!(
        quantize_unix_ns(
            i64::MIN,
            TimePrecision::Seconds,
            PrecisionLossPolicy::ExplicitFloor
        ),
        Err(PrecisionError::Overflow)
    );
    assert_eq!(
        quantize_unix_ns(
            i64::MIN + 1,
            TimePrecision::Seconds,
            PrecisionLossPolicy::ExplicitFloor
        ),
        Err(PrecisionError::Overflow)
    );
    assert_eq!(
        quantize_unix_ns(
            i64::MIN,
            TimePrecision::Seconds,
            PrecisionLossPolicy::ExplicitTruncate
        ),
        Ok(-9_223_372_036_000_000_000)
    );
}

#[test]
fn tp_t047_nanosecond_identity_holds_for_all_policies() {
    for value in [i64::MIN, -1, 0, 1, 1_234_567_890, i64::MAX] {
        for policy in POLICIES {
            assert_eq!(
                unix_ns_to_stored(value, TimePrecision::Nanoseconds, policy),
                Ok(value)
            );
            assert_eq!(
                quantize_unix_ns(value, TimePrecision::Nanoseconds, policy),
                Ok(value)
            );
            assert_eq!(
                stored_to_unix_ns(value, TimePrecision::Nanoseconds),
                Ok(value)
            );
        }
    }
}

#[test]
fn tp_t048_alignment_is_checked_before_projection() {
    assert_eq!(
        verify_projection(
            1,
            999,
            TimePrecision::Microseconds,
            PrecisionLossPolicy::ExplicitTruncate
        ),
        Err(PrecisionError::MisalignedProjection)
    );
    assert_eq!(
        verify_projection(
            1_500_000_001,
            1_500_000_001,
            TimePrecision::Milliseconds,
            PrecisionLossPolicy::ExplicitTruncate
        ),
        Err(PrecisionError::MisalignedProjection)
    );
}

#[test]
fn tp_t049_aligned_but_wrong_projection_mismatches() {
    assert_eq!(
        verify_projection(
            1,
            1_000,
            TimePrecision::Microseconds,
            PrecisionLossPolicy::ExplicitTruncate
        ),
        Err(PrecisionError::ProjectionMismatch)
    );
    assert_eq!(
        verify_projection(
            -1,
            -1_000,
            TimePrecision::Microseconds,
            PrecisionLossPolicy::ExplicitFloor
        ),
        Ok(())
    );
    assert_eq!(
        verify_projection(
            -1,
            -1_000,
            TimePrecision::Microseconds,
            PrecisionLossPolicy::ExplicitTruncate
        ),
        Err(PrecisionError::ProjectionMismatch)
    );
}

#[test]
fn tp_t050_capability_check_only_reads_declaration() {
    let declared = TimeStorageCapabilities {
        stored_precision: TimePrecision::Seconds,
        utc_normalized: false,
        lossless_unix_ns: true,
    };
    let undeclared = TimeStorageCapabilities {
        lossless_unix_ns: false,
        ..declared
    };
    assert_eq!(require_lossless_unix_ns(declared), Ok(()));
    assert_eq!(
        require_lossless_unix_ns(undeclared),
        Err(PrecisionError::LosslessRequired)
    );
    let no_utc_proof = TimeStorageCapabilities {
        stored_precision: TimePrecision::Milliseconds,
        utc_normalized: false,
        lossless_unix_ns: true,
    };
    assert_eq!(require_lossless_unix_ns(no_utc_proof), Ok(()));
}
