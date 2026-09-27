//! 基础时间在全域输入上的确定性性质。

use proptest::prelude::*;
use temporal::{PrecisionLossPolicy, TimePrecision, UnixTimeNs, quantize_unix_ns};

proptest! {
    #[test]
    fn 整秒与子秒规范化总能往返(value in any::<i64>()) {
        let time = UnixTimeNs::from_unix_nanos(value);
        let (seconds, nanos) = time.to_seconds_nanos();
        prop_assert!(nanos < 1_000_000_000);
        prop_assert_eq!(UnixTimeNs::try_from_seconds_nanos(seconds, nanos), Ok(time));
    }

    #[test]
    fn 任意有序时刻差值与加法互逆(a in any::<i64>(), b in any::<i64>()) {
        let earlier = UnixTimeNs::from_unix_nanos(a.min(b));
        let later = UnixTimeNs::from_unix_nanos(a.max(b));
        let duration = later.duration_since(earlier);
        prop_assert!(duration.is_ok());
        if let Ok(duration) = duration {
            prop_assert_eq!(earlier.checked_add(duration), Ok(later));
        }
    }

    #[test]
    fn 显式向下取整不晚于原时刻(value in any::<i64>()) {
        if let Ok(projected) = quantize_unix_ns(value, TimePrecision::Microseconds, PrecisionLossPolicy::ExplicitFloor) {
            prop_assert!(projected <= value);
            prop_assert_eq!(projected % 1_000, 0);
        }
    }
}
