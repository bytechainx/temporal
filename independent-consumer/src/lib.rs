//! TP-T177 独立外部消费证据。
//!
//! 本 crate 与 `temporal` 无父子 workspace 关系，只经 `path` 依赖检出
//! `temporal`，编译并调用其公开 API。它证明：
//!
//! - 外部 crate 能只检出 `temporal` 并编译通过；
//! - 公开 API 面（根路径 `pub use`）自洽可用；
//! - 私有模块（`clock` / `error` / `pit` 等）不可从外部依赖；
//! - 单一类型身份：`UnixTimeNs` 只有一个来源。
//!
//! 本 crate 不证明外部证据真实性，也不关闭任何 U3 场景；TP-T177 的
//! temporal fixture 半见 `tests/p1e_supplement.rs`。

use temporal::{
    AbsoluteTimeRange, CivilDate, EffectiveInterval, ObservationGranularity, ObservationPeriod,
    ObservationTime, TemporalDraft, TemporalError, TemporalProfile, TimeError, TimeField,
    TimePrecision, UnixTimeNs,
};

/// 公开 API 消费冒烟：构造、换算、区间与校验均可从外部调用。
pub fn consume_public_api() -> Result<u64, Box<dyn std::error::Error>> {
    let epoch = UnixTimeNs::from_unix_nanos(0);
    let one_second = UnixTimeNs::try_from_unix_seconds(1)?;
    let diff = one_second.duration_since(epoch)?;

    let stored = temporal::unix_ns_to_stored(
        1_000_000,
        TimePrecision::Microseconds,
        temporal::PrecisionLossPolicy::Reject,
    )?;

    let window = AbsoluteTimeRange::try_new(epoch, one_second)?;
    let period = ObservationPeriod::try_new(
        CivilDate::try_new(2026, 1, 1)?,
        CivilDate::try_new(2026, 2, 1)?,
        ObservationGranularity::Month,
        "gregorian-v1",
    )?;
    let interval = EffectiveInterval::try_new(epoch, Some(one_second))?;

    let draft = TemporalDraft {
        event_time: TimeField::Unknown,
        observation_time: TimeField::Known(ObservationTime::Window(window)),
        publication_time: TimeField::Unknown,
        received_time: TimeField::Known(epoch),
        revision_time: TimeField::Unknown,
        effective_time: TimeField::Known(epoch),
    };
    let _validated = draft.validate(TemporalProfile::SampledObservation)?;

    let mut nanos = diff.as_nanos() as u64;
    nanos = nanos.wrapping_add(stored as u64);
    nanos = nanos.wrapping_add(period.calendar_ref().len() as u64);
    let _ = interval.contains(epoch);
    Ok(nanos)
}

/// 类型身份：外部只经根路径取得 `UnixTimeNs`，无平行类型。
pub fn single_type_identity() -> UnixTimeNs {
    UnixTimeNs::from_unix_nanos(42)
}

/// 错误可独立分类：外部用自有匹配处理 temporal 错误。
pub fn classify(error: TimeError) -> &'static str {
    match error {
        TimeError::Overflow => "overflow",
        TimeError::InvalidOrder => "invalid-order",
        TimeError::InvalidSubsecond => "invalid-subsecond",
        TimeError::SystemTimeOutOfRange => "system-time-range",
        TimeError::SystemTimePrecisionLoss => "system-time-precision",
        TimeError::SourceUnavailable => "source-unavailable",
        _ => "other",
    }
}

/// 非穷尽枚举要求外部匹配保留通配分支。
pub fn classify_temporal(error: TemporalError) -> &'static str {
    match error {
        TemporalError::MissingRequiredRole => "missing-role",
        TemporalError::RoleNotApplicable => "role-not-applicable",
        TemporalError::InvalidDate => "invalid-date",
        TemporalError::InvalidPeriod => "invalid-period",
        TemporalError::InvalidInterval => "invalid-interval",
        TemporalError::ProfileMismatch => "profile-mismatch",
        TemporalError::MetadataMismatch => "metadata-mismatch",
        _ => "other",
    }
}
