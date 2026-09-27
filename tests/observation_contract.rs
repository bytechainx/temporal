#![allow(clippy::unwrap_used)]

use std::time::Duration;

use temporal::{
    AbsoluteTimeRange, CivilDate, EffectiveInterval, ObservationGranularity, ObservationPeriod,
    TemporalError, UnixTimeNs,
};

fn date(year: u16, month: u8, day: u8) -> CivilDate {
    CivilDate::try_new(year, month, day).unwrap()
}

fn time(nanos: i64) -> UnixTimeNs {
    UnixTimeNs::from_unix_nanos(nanos)
}

#[test]
fn tp_t031_gregorian_date_validation() {
    assert_eq!(date(2000, 2, 29).day(), 29);
    assert_eq!(date(2024, 2, 29).year(), 2024);
    for invalid in [
        (0, 1, 1),
        (10000, 1, 1),
        (1900, 2, 29),
        (2023, 2, 29),
        (2024, 0, 1),
        (2024, 13, 1),
        (2024, 4, 31),
        (2024, 1, 0),
    ] {
        assert!(matches!(
            CivilDate::try_new(invalid.0, invalid.1, invalid.2),
            Err(TemporalError::InvalidDate)
        ));
    }
}

#[test]
fn tp_t032_half_open_period_and_absolute_range() {
    let start = date(2026, 2, 1);
    let end = date(2026, 3, 1);
    assert!(matches!(
        ObservationPeriod::try_new(start, start, ObservationGranularity::Month, "gregorian-v1"),
        Err(TemporalError::InvalidPeriod)
    ));
    assert!(matches!(
        ObservationPeriod::try_new(end, start, ObservationGranularity::Month, "gregorian-v1"),
        Err(TemporalError::InvalidPeriod)
    ));
    let period =
        ObservationPeriod::try_new(start, end, ObservationGranularity::Month, "gregorian-v1")
            .unwrap();
    assert!(period.contains(date(2026, 2, 28)));
    assert!(!period.contains(end));
    let left = AbsoluteTimeRange::try_new(time(-5), time(0)).unwrap();
    let right = AbsoluteTimeRange::try_new(time(0), time(7)).unwrap();
    assert_eq!(left.intersection(right), None);
    assert!(left.contains(time(-5)));
    assert!(!left.contains(time(0)));
    assert!(matches!(
        AbsoluteTimeRange::try_new(time(1), time(1)),
        Err(TemporalError::InvalidInterval)
    ));
    assert!(matches!(
        AbsoluteTimeRange::try_new(time(1), time(0)),
        Err(TemporalError::InvalidInterval)
    ));
    let full = AbsoluteTimeRange::try_new(time(i64::MIN), time(i64::MAX)).unwrap();
    assert_eq!(full.duration(), Duration::from_nanos(u64::MAX));
    assert_eq!(
        left.intersection(AbsoluteTimeRange::try_new(time(-2), time(3)).unwrap()),
        AbsoluteTimeRange::try_new(time(-2), time(0)).ok()
    );
}

#[test]
fn tp_t033_calendar_boundaries_are_not_fixed_seconds() {
    let february = ObservationPeriod::try_new(
        date(2024, 2, 1),
        date(2024, 3, 1),
        ObservationGranularity::Month,
        "gregorian-v1",
    )
    .unwrap();
    assert!(february.contains(date(2024, 2, 29)));
    assert!(matches!(
        ObservationPeriod::try_new(
            date(2024, 2, 1),
            date(2024, 3, 2),
            ObservationGranularity::Month,
            "gregorian-v1"
        ),
        Err(TemporalError::InvalidPeriod)
    ));
    assert!(
        ObservationPeriod::try_new(
            date(2026, 1, 1),
            date(2026, 4, 1),
            ObservationGranularity::Quarter,
            "gregorian-v1"
        )
        .is_ok()
    );
    assert!(matches!(
        ObservationPeriod::try_new(
            date(2026, 2, 1),
            date(2026, 5, 1),
            ObservationGranularity::Quarter,
            "gregorian-v1"
        ),
        Err(TemporalError::InvalidPeriod)
    ));
    assert!(
        ObservationPeriod::try_new(
            date(2026, 2, 1),
            date(2026, 5, 1),
            ObservationGranularity::Custom,
            "fiscal-v1"
        )
        .is_ok()
    );
    assert!(matches!(
        ObservationPeriod::try_new(
            date(9999, 12, 31),
            date(9999, 12, 31),
            ObservationGranularity::Day,
            "gregorian-v1"
        ),
        Err(TemporalError::InvalidPeriod)
    ));
}

#[test]
fn tp_t034_calendar_reference_is_part_of_period_identity() {
    let first = ObservationPeriod::try_new(
        date(2026, 1, 1),
        date(2026, 2, 1),
        ObservationGranularity::Month,
        "calendar-v1",
    )
    .unwrap();
    let second = ObservationPeriod::try_new(
        date(2026, 1, 1),
        date(2026, 2, 1),
        ObservationGranularity::Month,
        "calendar-v2",
    )
    .unwrap();
    assert_ne!(first, second);
    assert!(matches!(
        ObservationPeriod::try_new(
            date(2026, 1, 1),
            date(2026, 2, 1),
            ObservationGranularity::Month,
            ""
        ),
        Err(TemporalError::InvalidPeriod)
    ));
}

#[test]
fn effective_interval_has_explicit_unbounded_end() {
    let finite = EffectiveInterval::try_new(time(-1), Some(time(1))).unwrap();
    assert!(finite.contains(time(-1)));
    assert!(!finite.contains(time(1)));
    let unbounded = EffectiveInterval::try_new(time(0), None).unwrap();
    assert!(unbounded.contains(time(i64::MAX)));
    assert!(matches!(
        EffectiveInterval::try_new(time(0), Some(time(0))),
        Err(TemporalError::InvalidInterval)
    ));
}
