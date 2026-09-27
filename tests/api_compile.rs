//! 从 crate 根路径检查公开类型身份与刻意缺席的隐式转换。

use static_assertions::{assert_impl_all, assert_not_impl_any};
use temporal::{
    MonotonicClock, PitError, PrecisionError, RuntimeClock, SystemMonotonicClock, SystemWallClock,
    TemporalError, TimeError, UnixTimeNs, WallClock,
};

assert_impl_all!(UnixTimeNs: Copy, Send, Sync);
assert_impl_all!(TimeError: std::error::Error, Send, Sync);
assert_impl_all!(PrecisionError: std::error::Error, Send, Sync);
assert_impl_all!(TemporalError: std::error::Error, Send, Sync);
assert_impl_all!(PitError: std::error::Error, Send, Sync);
assert_not_impl_any!(UnixTimeNs: Default, From<i64>, Into<i64>, serde::Serialize);

#[test]
fn 两类时钟接口均可作动态派发() {
    let wall: &dyn WallClock = &SystemWallClock::new();
    let monotonic: &dyn MonotonicClock = &SystemMonotonicClock::new();
    let _ = wall.now();
    let _ = monotonic.now();
}

#[test]
fn 组合对象可同时实现两类时钟接口() {
    struct Pair(SystemWallClock, SystemMonotonicClock);
    impl WallClock for Pair {
        fn now(&self) -> Result<UnixTimeNs, TimeError> {
            WallClock::now(&self.0)
        }
    }
    impl MonotonicClock for Pair {
        fn now(&self) -> std::time::Instant {
            MonotonicClock::now(&self.1)
        }
    }
    let pair: &dyn RuntimeClock = &Pair(SystemWallClock::new(), SystemMonotonicClock::new());
    let wall: &dyn WallClock = pair;
    let monotonic: &dyn MonotonicClock = pair;
    let _ = wall.now();
    let _ = monotonic.now();
}
