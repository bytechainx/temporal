//! 显式构造事件与接收时刻，结构校验不代表历史可见。

use temporal::{TemporalDraft, TemporalProfile, TimeField, UnixTimeNs};

fn main() -> Result<(), temporal::TemporalError> {
    let draft = TemporalDraft {
        event_time: TimeField::Known(UnixTimeNs::from_unix_nanos(1_000)),
        observation_time: TimeField::NotApplicable,
        publication_time: TimeField::NotApplicable,
        received_time: TimeField::Known(UnixTimeNs::from_unix_nanos(1_100)),
        revision_time: TimeField::NotApplicable,
        effective_time: TimeField::NotApplicable,
    };
    let event = draft.validate(TemporalProfile::EventRecord)?;
    assert_eq!(event.received_time().as_unix_nanos(), 1_100);
    Ok(())
}
