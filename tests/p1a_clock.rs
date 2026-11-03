//! P1-A / 22.3：SystemTime 与时钟的 10 个场景。
#![allow(clippy::unwrap_used)]

use std::sync::Mutex;
use std::time::{Duration, Instant, UNIX_EPOCH};

use temporal::{MonotonicClock, RuntimeClock, TimeError, UnixTimeNs, WallClock};

const PIT: &[&str] = &[
    include_str!("../src/pit.rs"),
    include_str!("../src/pit/context.rs"),
    include_str!("../src/pit/selection.rs"),
    include_str!("../src/pit/visibility.rs"),
    include_str!("../src/pit/effective.rs"),
    include_str!("../src/pit/diagnostics.rs"),
];
const CLOCK_SRC: &str = include_str!("../src/clock.rs");

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

    fn advance(&self, delta: i128) -> Result<(), TimeError> {
        let mut elapsed = self.elapsed.lock().unwrap();
        let next = elapsed.as_nanos() as i128 + delta;
        if next < 0 {
            return Err(TimeError::InvalidOrder);
        }
        let nanos = u64::try_from(next).map_err(|_| TimeError::Overflow)?;
        let duration = Duration::from_nanos(nanos);
        self.anchor
            .checked_add(duration)
            .ok_or(TimeError::Overflow)?;
        *elapsed = duration;
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

struct WallOnly;

impl WallClock for WallOnly {
    fn now(&self) -> Result<UnixTimeNs, TimeError> {
        Ok(UnixTimeNs::UNIX_EPOCH)
    }
}

#[test]
fn tp_t021_system_time_signed_epoch() {
    for offset in [-1_i64, 1, -1_000_000_000, 1_000_000_000] {
        let system = if offset >= 0 {
            UNIX_EPOCH + Duration::from_nanos(offset.unsigned_abs())
        } else {
            UNIX_EPOCH - Duration::from_nanos(offset.unsigned_abs())
        };
        let time = UnixTimeNs::try_from_system_time(system).unwrap();
        assert_eq!(time.as_unix_nanos(), offset);
        assert_eq!(time.try_into_system_time(), Ok(system));
    }
}

#[test]
fn tp_t022_extreme_negative_system_time() {
    let result = UnixTimeNs::MIN.try_into_system_time();
    match result {
        Ok(system) => {
            assert_eq!(
                UnixTimeNs::try_from_system_time(system),
                Ok(UnixTimeNs::MIN)
            );
        }
        Err(error) => assert!(matches!(
            error,
            TimeError::SystemTimeOutOfRange | TimeError::SystemTimePrecisionLoss
        )),
    }
}

#[test]
fn tp_t023_platform_precision_is_checked() {
    let time = UnixTimeNs::from_unix_nanos(1_234_567_890);
    match time.try_into_system_time() {
        Ok(system) => assert_eq!(UnixTimeNs::try_from_system_time(system), Ok(time)),
        Err(error) => assert_eq!(error, TimeError::SystemTimePrecisionLoss),
    }
}

#[test]
fn tp_t024_system_time_out_of_range_is_explicit() {
    let result = UnixTimeNs::MAX.try_into_system_time();
    match result {
        Ok(system) => {
            assert_eq!(
                UnixTimeNs::try_from_system_time(system),
                Ok(UnixTimeNs::MAX)
            );
        }
        Err(error) => assert!(matches!(
            error,
            TimeError::SystemTimeOutOfRange | TimeError::SystemTimePrecisionLoss
        )),
    }
}

#[test]
fn tp_t025_wall_clock_can_move_backwards() {
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
fn tp_t026_monotonic_advance_rejects_backwards_and_overflow() {
    let clock = ManualMonotonicClock::new();
    assert_eq!(clock.advance(-1), Err(TimeError::InvalidOrder));
    let start = clock.now();
    clock.advance(10).unwrap();
    assert_eq!(clock.now().duration_since(start), Duration::from_nanos(10));
    assert_eq!(clock.advance(-11), Err(TimeError::InvalidOrder));
    assert_eq!(
        clock.advance(i128::from(u64::MAX)),
        Err(TimeError::Overflow)
    );
}

#[test]
fn tp_t027_combined_clock_is_a_runtime_clock() {
    fn require_runtime_clock<T: RuntimeClock + ?Sized>(clock: &T) {
        let _ = WallClock::now(clock);
        let _ = MonotonicClock::now(clock);
    }
    let clock = CombinedClock {
        wall: ManualWallClock::new(UnixTimeNs::UNIX_EPOCH),
        monotonic: ManualMonotonicClock::new(),
    };
    require_runtime_clock(&clock);
    assert_eq!(WallClock::now(&clock), Ok(UnixTimeNs::UNIX_EPOCH));
}

#[test]
fn tp_t028_wall_only_clock_does_not_claim_runtime() {
    let wall = WallOnly;
    assert_eq!(wall.now(), Ok(UnixTimeNs::UNIX_EPOCH));
    let dynamic: &dyn WallClock = &wall;
    assert_eq!(dynamic.now(), Ok(UnixTimeNs::UNIX_EPOCH));
}

#[test]
fn tp_t029_pit_does_not_read_now() {
    for source in PIT {
        assert!(!source.contains("now("), "PIT source reads now");
        assert!(!source.contains("::now"), "PIT source reads ::now");
        assert!(
            !source.contains("SystemTime::now"),
            "PIT source reads SystemTime::now"
        );
        assert!(
            !source.contains("Instant::now"),
            "PIT source reads Instant::now"
        );
    }
}

#[test]
fn tp_t030_virtual_advance_does_not_impersonate_waiting() {
    for source in PIT {
        assert!(!source.contains("sleep"), "PIT source contains sleep");
        assert!(!source.contains("thread::sleep"));
    }
    assert!(!CLOCK_SRC.contains("sleep"));
}
