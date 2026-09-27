#![allow(clippy::unwrap_used)]

use temporal::{
    PrecisionError, PrecisionLossPolicy, TimePrecision, TimeStorageCapabilities, quantize_unix_ns,
    require_lossless_unix_ns, stored_to_unix_ns, unix_ns_to_stored, verify_projection,
};

#[test]
fn 默认拒绝精度损失() {
    assert_eq!(PrecisionLossPolicy::default(), PrecisionLossPolicy::Reject);
    assert_eq!(
        unix_ns_to_stored(
            1,
            TimePrecision::Microseconds,
            PrecisionLossPolicy::default()
        ),
        Err(PrecisionError::PrecisionLoss)
    );
    for value in [-1_000_000, 0, 1_000_000] {
        assert_eq!(
            quantize_unix_ns(
                value,
                TimePrecision::Milliseconds,
                PrecisionLossPolicy::Reject
            ),
            Ok(value)
        );
    }
}

#[test]
fn 负值截断与_floor_不同() {
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
            TimePrecision::Microseconds,
            PrecisionLossPolicy::ExplicitFloor
        ),
        Ok(-1)
    );
    assert_eq!(
        unix_ns_to_stored(
            -1_500_000_001,
            TimePrecision::Milliseconds,
            PrecisionLossPolicy::ExplicitTruncate
        ),
        Ok(-1_500)
    );
    assert_eq!(
        unix_ns_to_stored(
            1_500_000_001,
            TimePrecision::Milliseconds,
            PrecisionLossPolicy::ExplicitTruncate
        ),
        Ok(1_500)
    );
    assert_eq!(
        quantize_unix_ns(
            i64::MIN,
            TimePrecision::Seconds,
            PrecisionLossPolicy::ExplicitFloor
        ),
        Err(PrecisionError::Overflow)
    );
}

#[test]
fn 换算溢出和纳秒恒等() {
    assert_eq!(
        stored_to_unix_ns(i64::MAX, TimePrecision::Seconds),
        Err(PrecisionError::Overflow)
    );
    assert_eq!(
        stored_to_unix_ns(i64::MIN, TimePrecision::Seconds),
        Err(PrecisionError::Overflow)
    );
    for value in [i64::MIN, -1, 0, 1, i64::MAX] {
        for policy in [
            PrecisionLossPolicy::Reject,
            PrecisionLossPolicy::ExplicitTruncate,
            PrecisionLossPolicy::ExplicitFloor,
        ] {
            assert_eq!(
                quantize_unix_ns(value, TimePrecision::Nanoseconds, policy),
                Ok(value)
            );
        }
    }
}

#[test]
fn 投影先检查读回对齐() {
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
}

#[test]
fn 能力函数只检查声明() {
    let mut capabilities = TimeStorageCapabilities {
        stored_precision: TimePrecision::Seconds,
        utc_normalized: false,
        lossless_unix_ns: false,
    };
    assert_eq!(
        require_lossless_unix_ns(capabilities),
        Err(PrecisionError::LosslessRequired)
    );
    capabilities.lossless_unix_ns = true;
    assert_eq!(require_lossless_unix_ns(capabilities), Ok(()));
}

#[test]
fn 精度错误码固定() {
    assert_eq!(PrecisionError::PrecisionLoss.code(), "PREC_LOSS_REJECTED");
    assert_eq!(PrecisionError::Overflow.code(), "PREC_OVERFLOW");
    assert_eq!(
        PrecisionError::MisalignedProjection.code(),
        "PREC_MISALIGNED"
    );
    assert_eq!(
        PrecisionError::ProjectionMismatch.code(),
        "PREC_PROJECTION_MISMATCH"
    );
    assert_eq!(
        PrecisionError::LosslessRequired.code(),
        "PREC_LOSSLESS_REQUIRED"
    );
}
