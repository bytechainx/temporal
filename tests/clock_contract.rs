#![allow(clippy::unwrap_used)]

use std::sync::Mutex;
use std::time::{Duration, Instant};

use temporal::{
    MonotonicClock, RuntimeClock, SystemMonotonicClock, SystemWallClock, TimeError, UnixTimeNs,
    WallClock,
};

struct ManualWallClock(Mutex<Result<UnixTimeNs, TimeError>>);

impl ManualWallClock {
    fn new(value: UnixTimeNs) -> Self {
        Self(Mutex::new(Ok(value)))
    }

    fn set(&self, value: UnixTimeNs) {
        *self.0.lock().unwrap() = Ok(value);
    }

    fn fail(&self) {
        *self.0.lock().unwrap() = Err(TimeError::SourceUnavailable);
    }
}

impl WallClock for ManualWallClock {
    fn now(&self) -> Result<UnixTimeNs, TimeError> {
        *self.0.lock().unwrap()
    }
}

struct ManualMonotonicClock {
    anchor: Instant,
    elapsed: Mutex<Duration>,
}

impl ManualMonotonicClock {
    fn new() -> Self {
        Self {
            anchor: Instant::now(),
            elapsed: Mutex::new(Duration::ZERO),
        }
    }

    fn advance(&self, duration: Duration) -> Result<(), TimeError> {
        let mut elapsed = self.elapsed.lock().unwrap();
        let next = elapsed.checked_add(duration).ok_or(TimeError::Overflow)?;
        self.anchor.checked_add(next).ok_or(TimeError::Overflow)?;
        *elapsed = next;
        Ok(())
    }
}

impl MonotonicClock for ManualMonotonicClock {
    fn now(&self) -> Instant {
        self.anchor
            .checked_add(*self.elapsed.lock().unwrap())
            .unwrap()
    }
}

struct CombinedClock {
    wall: ManualWallClock,
    monotonic: ManualMonotonicClock,
}

impl WallClock for CombinedClock {
    fn now(&self) -> Result<UnixTimeNs, TimeError> {
        WallClock::now(&self.wall)
    }
}

impl MonotonicClock for CombinedClock {
    fn now(&self) -> Instant {
        MonotonicClock::now(&self.monotonic)
    }
}

#[test]
fn 墙钟允许回退且失败不伪造时间() {
    let wall = ManualWallClock::new(UnixTimeNs::from_unix_nanos(100));
    assert_eq!(wall.now().unwrap().as_unix_nanos(), 100);
    wall.set(UnixTimeNs::from_unix_nanos(90));
    assert_eq!(wall.now().unwrap().as_unix_nanos(), 90);
    wall.set(UnixTimeNs::from_unix_nanos(110));
    assert_eq!(wall.now().unwrap().as_unix_nanos(), 110);
    wall.fail();
    assert_eq!(wall.now(), Err(TimeError::SourceUnavailable));
}

#[test]
fn 单调测试推进拒绝超范围() {
    let clock = ManualMonotonicClock::new();
    let start = clock.now();
    clock.advance(Duration::from_nanos(10)).unwrap();
    assert_eq!(clock.now().duration_since(start), Duration::from_nanos(10));
    assert_eq!(clock.advance(Duration::MAX), Err(TimeError::Overflow));
}

#[test]
fn 组合接口要求两种能力() {
    fn require_runtime_clock<T: RuntimeClock + ?Sized>(clock: &T) {
        let _ = WallClock::now(clock);
        let _ = MonotonicClock::now(clock);
    }
    let combined = CombinedClock {
        wall: ManualWallClock::new(UnixTimeNs::UNIX_EPOCH),
        monotonic: ManualMonotonicClock::new(),
    };
    require_runtime_clock(&combined);
    assert_eq!(WallClock::now(&combined), Ok(UnixTimeNs::UNIX_EPOCH));
    let _: Instant = MonotonicClock::now(&combined);
}

#[test]
fn 系统接口仅作可表示冒烟检查() {
    let wall = SystemWallClock::new();
    let monotonic = SystemMonotonicClock::new();
    let _ = WallClock::now(&wall).unwrap();
    let _: Instant = MonotonicClock::now(&monotonic);
}
