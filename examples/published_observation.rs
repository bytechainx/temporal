//! 民用期间与实际发布时间分开保存；未知发布时刻不补零。

use temporal::{
    CivilDate, ObservationGranularity, ObservationPeriod, ObservationTime, TemporalDraft,
    TemporalProfile, TimeField, UnixTimeNs,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let period = ObservationPeriod::try_new(
        CivilDate::try_new(2026, 8, 1)?,
        CivilDate::try_new(2026, 9, 1)?,
        ObservationGranularity::Month,
        "gregorian-v1",
    )?;
    let draft = TemporalDraft {
        event_time: TimeField::NotApplicable,
        observation_time: TimeField::Known(ObservationTime::Period(period)),
        publication_time: TimeField::Unknown,
        received_time: TimeField::Known(UnixTimeNs::from_unix_nanos(1_000)),
        revision_time: TimeField::Unknown,
        effective_time: TimeField::NotApplicable,
    };
    let observation = draft.validate(TemporalProfile::PublishedObservation)?;
    assert!(matches!(observation.publication_time(), TimeField::Unknown));
    Ok(())
}
