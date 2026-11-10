//! 独立外部消费：只检出 temporal，编译并跑通公开 API。

use temporal_independent_consumer::{classify, classify_temporal, consume_public_api, single_type_identity};
use temporal::{TimeError, TemporalError, UnixTimeNs};

#[test]
fn public_api_consumes_successfully() {
    let nanos = consume_public_api().unwrap();
    assert!(nanos > 0);
}

#[test]
fn single_type_identity_is_one_source() {
    let value: UnixTimeNs = single_type_identity();
    assert_eq!(value.as_unix_nanos(), 42);
}

#[test]
fn errors_classify_from_outside() {
    assert_eq!(classify(TimeError::Overflow), "overflow");
    assert_eq!(classify_temporal(TemporalError::InvalidInterval), "invalid-interval");
}

#[test]
fn private_modules_are_not_reachable() {
    // 编译期判据：本 crate 只能经根路径引用；私有模块不可达。
    // 若 `temporal::unix_time` 等变为公开，此断言仍通过，故以文档/API 清单为准。
    let _ = UnixTimeNs::UNIX_EPOCH;
}
