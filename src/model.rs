//! 六角色时间模型及显式的结构校验。

use crate::error::TemporalError;
use crate::observation::ObservationTime;
use crate::unix_time::UnixTimeNs;

/// 时间字段的三种明确状态。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TimeField<T> {
    /// 已知的时间值。
    Known(T),
    /// 适用但时间值未知。
    Unknown,
    /// 本版本语义不适用。
    NotApplicable,
}

/// 六角色组合的结构校验规则。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TemporalProfile {
    /// 具有源事件时刻的事件。
    EventRecord,
    /// 具有观测期间或观测点的发布观测。
    PublishedObservation,
    /// 具有生效时刻的公告。
    Announcement,
    /// 具有事件或观测语义锚点的派生记录。
    DerivedRecord,
    /// 缺少可证明源时刻的本地采样观察。
    SampledObservation,
}

/// 待按明确 profile 校验的六角色输入。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemporalDraft {
    /// 事件发生坐标。
    pub event_time: TimeField<UnixTimeNs>,
    /// 观测点、绝对窗口或民用期间。
    pub observation_time: TimeField<ObservationTime>,
    /// 当前修订面向声明受众实际发布的时刻。
    pub publication_time: TimeField<UnixTimeNs>,
    /// 本次投影所绑定接收边界的接收采样。
    pub received_time: TimeField<UnixTimeNs>,
    /// 来源定义的本修订行为时间。
    pub revision_time: TimeField<UnixTimeNs>,
    /// 业务事实开始生效坐标。
    pub effective_time: TimeField<UnixTimeNs>,
}

impl TemporalDraft {
    /// 校验必需角色和 profile 组合，不推断历史可见性。
    pub fn validate(self, profile: TemporalProfile) -> Result<Temporal, TemporalError> {
        let received_time = match self.received_time {
            TimeField::Known(value) => value,
            TimeField::Unknown => return Err(TemporalError::MissingRequiredRole),
            TimeField::NotApplicable => return Err(TemporalError::RoleNotApplicable),
        };
        match profile {
            TemporalProfile::EventRecord => {
                require_known(&self.event_time)?;
            }
            TemporalProfile::PublishedObservation => {
                require_known(&self.observation_time)?;
                require_applicable(&self.publication_time)?;
            }
            TemporalProfile::Announcement => {
                require_known(&self.effective_time)?;
                require_applicable(&self.publication_time)?;
            }
            TemporalProfile::DerivedRecord => {
                if !matches!(self.event_time, TimeField::Known(_))
                    && !matches!(self.observation_time, TimeField::Known(_))
                {
                    return Err(TemporalError::MissingRequiredRole);
                }
            }
            TemporalProfile::SampledObservation => {}
        }
        Ok(Temporal {
            event_time: self.event_time,
            observation_time: self.observation_time,
            publication_time: self.publication_time,
            received_time,
            revision_time: self.revision_time,
            effective_time: self.effective_time,
            profile,
        })
    }
}

fn require_known<T>(field: &TimeField<T>) -> Result<(), TemporalError> {
    match field {
        TimeField::Known(_) => Ok(()),
        TimeField::Unknown => Err(TemporalError::MissingRequiredRole),
        TimeField::NotApplicable => Err(TemporalError::RoleNotApplicable),
    }
}

fn require_applicable<T>(field: &TimeField<T>) -> Result<(), TemporalError> {
    if matches!(field, TimeField::NotApplicable) {
        Err(TemporalError::RoleNotApplicable)
    } else {
        Ok(())
    }
}

/// 已按 profile 校验、只能只读访问的六角色时间视图。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Temporal {
    event_time: TimeField<UnixTimeNs>,
    observation_time: TimeField<ObservationTime>,
    publication_time: TimeField<UnixTimeNs>,
    received_time: UnixTimeNs,
    revision_time: TimeField<UnixTimeNs>,
    effective_time: TimeField<UnixTimeNs>,
    profile: TemporalProfile,
}

impl Temporal {
    /// 校验时采用的 profile。
    pub const fn profile(&self) -> TemporalProfile {
        self.profile
    }

    /// 事件发生坐标及其缺失状态。
    pub const fn event_time(&self) -> &TimeField<UnixTimeNs> {
        &self.event_time
    }

    /// 观测时间及其缺失状态。
    pub const fn observation_time(&self) -> &TimeField<ObservationTime> {
        &self.observation_time
    }

    /// 当前修订发布时间及其缺失状态。
    pub const fn publication_time(&self) -> &TimeField<UnixTimeNs> {
        &self.publication_time
    }

    /// 本次投影绑定的实际接收时刻。
    pub const fn received_time(&self) -> UnixTimeNs {
        self.received_time
    }

    /// 当前修订行为时刻及其缺失状态。
    pub const fn revision_time(&self) -> &TimeField<UnixTimeNs> {
        &self.revision_time
    }

    /// 业务开始生效时刻及其缺失状态。
    pub const fn effective_time(&self) -> &TimeField<UnixTimeNs> {
        &self.effective_time
    }

    /// 投影五个不含接收角色的固有时间字段。
    pub const fn source_roles(&self) -> SourceTemporalRef<'_> {
        SourceTemporalRef {
            event_time: &self.event_time,
            observation_time: &self.observation_time,
            publication_time: &self.publication_time,
            revision_time: &self.revision_time,
            effective_time: &self.effective_time,
        }
    }
}

/// 借用五个固有角色的只读视图；不证明外部来源真实性。
#[derive(Debug, Clone, Copy)]
pub struct SourceTemporalRef<'a> {
    event_time: &'a TimeField<UnixTimeNs>,
    observation_time: &'a TimeField<ObservationTime>,
    publication_time: &'a TimeField<UnixTimeNs>,
    revision_time: &'a TimeField<UnixTimeNs>,
    effective_time: &'a TimeField<UnixTimeNs>,
}

impl<'a> SourceTemporalRef<'a> {
    /// 事件发生坐标及其缺失状态。
    pub const fn event_time(self) -> &'a TimeField<UnixTimeNs> {
        self.event_time
    }

    /// 观测时间及其缺失状态。
    pub const fn observation_time(self) -> &'a TimeField<ObservationTime> {
        self.observation_time
    }

    /// 当前修订发布时间及其缺失状态。
    pub const fn publication_time(self) -> &'a TimeField<UnixTimeNs> {
        self.publication_time
    }

    /// 当前修订行为时刻及其缺失状态。
    pub const fn revision_time(self) -> &'a TimeField<UnixTimeNs> {
        self.revision_time
    }

    /// 业务开始生效时刻及其缺失状态。
    pub const fn effective_time(self) -> &'a TimeField<UnixTimeNs> {
        self.effective_time
    }
}
