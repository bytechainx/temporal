#![allow(clippy::unwrap_used)]

use temporal::{
    ObservationTime, TemporalDraft, TemporalError, TemporalProfile, TimeField, UnixTimeNs,
};

fn time(nanos: i64) -> UnixTimeNs {
    UnixTimeNs::from_unix_nanos(nanos)
}

fn draft() -> TemporalDraft {
    TemporalDraft {
        event_time: TimeField::Unknown,
        observation_time: TimeField::Unknown,
        publication_time: TimeField::Unknown,
        received_time: TimeField::Known(time(10)),
        revision_time: TimeField::Unknown,
        effective_time: TimeField::Unknown,
    }
}

#[test]
fn tp_t035_event_record_requires_known_event_and_receipt() {
    let mut event = draft();
    assert!(matches!(
        event.clone().validate(TemporalProfile::EventRecord),
        Err(TemporalError::MissingRequiredRole)
    ));
    event.event_time = TimeField::Known(time(20));
    event.received_time = TimeField::Unknown;
    assert!(matches!(
        event.clone().validate(TemporalProfile::EventRecord),
        Err(TemporalError::MissingRequiredRole)
    ));
    event.received_time = TimeField::Known(time(10));
    event.publication_time = TimeField::NotApplicable;
    assert!(event.validate(TemporalProfile::EventRecord).is_ok());
}

#[test]
fn tp_t036_published_observation_allows_unknown_publication() {
    let mut observation = draft();
    observation.observation_time = TimeField::Known(ObservationTime::At(time(0)));
    let validated = observation
        .clone()
        .validate(TemporalProfile::PublishedObservation)
        .unwrap();
    assert_eq!(validated.publication_time(), &TimeField::Unknown);
    assert_eq!(
        validated.source_roles().observation_time(),
        validated.observation_time()
    );
    observation.publication_time = TimeField::NotApplicable;
    assert!(matches!(
        observation.validate(TemporalProfile::PublishedObservation),
        Err(TemporalError::RoleNotApplicable)
    ));
}

#[test]
fn tp_t037_future_effective_time_does_not_fail_structure() {
    let mut announcement = draft();
    announcement.publication_time = TimeField::Known(time(5));
    announcement.effective_time = TimeField::Known(time(100));
    let validated = announcement
        .validate(TemporalProfile::Announcement)
        .unwrap();
    assert_eq!(validated.effective_time(), &TimeField::Known(time(100)));
}

#[test]
fn tp_t038_derived_record_needs_semantic_anchor() {
    let mut derived = draft();
    assert!(matches!(
        derived.clone().validate(TemporalProfile::DerivedRecord),
        Err(TemporalError::MissingRequiredRole)
    ));
    derived.observation_time = TimeField::Known(ObservationTime::At(time(0)));
    assert!(derived.validate(TemporalProfile::DerivedRecord).is_ok());
}

#[test]
fn tp_t039_three_states_never_collapse_into_epoch() {
    let mut sampled = draft();
    sampled.event_time = TimeField::NotApplicable;
    sampled.publication_time = TimeField::Unknown;
    sampled.revision_time = TimeField::Known(time(0));
    let validated = sampled
        .validate(TemporalProfile::SampledObservation)
        .unwrap();
    assert_eq!(validated.event_time(), &TimeField::NotApplicable);
    assert_eq!(validated.publication_time(), &TimeField::Unknown);
    assert_eq!(validated.revision_time(), &TimeField::Known(time(0)));
    assert_eq!(validated.received_time(), time(10));
}

#[test]
fn tp_t040_profiles_do_not_impose_universal_timestamp_order() {
    let mut event = draft();
    event.event_time = TimeField::Known(time(1000));
    event.received_time = TimeField::Known(time(-1000));
    assert!(event.validate(TemporalProfile::EventRecord).is_ok());
    let mut forecast = draft();
    forecast.observation_time = TimeField::Known(ObservationTime::At(time(1000)));
    assert!(
        forecast
            .validate(TemporalProfile::PublishedObservation)
            .is_ok()
    );
}
