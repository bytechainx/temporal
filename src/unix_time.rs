use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::TimeError;

/// POSIX epoch 起算的有符号纳秒时刻。
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnixTimeNs(i64);

impl UnixTimeNs {
    /// 显式 epoch 时刻。
    pub const UNIX_EPOCH: Self = Self(0);
    /// 可表示的最小时刻。
    pub const MIN: Self = Self(i64::MIN);
    /// 可表示的最大时刻。
    pub const MAX: Self = Self(i64::MAX);

    /// 由 POSIX epoch 纳秒构造。
    pub const fn from_unix_nanos(nanos: i64) -> Self {
        Self(nanos)
    }

    /// 以 POSIX epoch 纳秒导出。
    pub const fn as_unix_nanos(self) -> i64 {
        self.0
    }

    /// 由 POSIX epoch 微秒构造，拒绝溢出。
    pub fn try_from_unix_micros(micros: i64) -> Result<Self, TimeError> {
        micros
            .checked_mul(1_000)
            .map(Self)
            .ok_or(TimeError::Overflow)
    }

    /// 由 POSIX epoch 毫秒构造，拒绝溢出。
    pub fn try_from_unix_millis(millis: i64) -> Result<Self, TimeError> {
        millis
            .checked_mul(1_000_000)
            .map(Self)
            .ok_or(TimeError::Overflow)
    }

    /// 由 POSIX epoch 秒构造，拒绝溢出。
    pub fn try_from_unix_seconds(seconds: i64) -> Result<Self, TimeError> {
        seconds
            .checked_mul(1_000_000_000)
            .map(Self)
            .ok_or(TimeError::Overflow)
    }

    /// 加上非负时长，拒绝溢出。
    pub fn checked_add(self, duration: Duration) -> Result<Self, TimeError> {
        let value = i128::from(self.0)
            .checked_add(duration.as_nanos() as i128)
            .ok_or(TimeError::Overflow)?;
        i64::try_from(value)
            .map(Self)
            .map_err(|_| TimeError::Overflow)
    }

    /// 减去非负时长，拒绝溢出。
    pub fn checked_sub(self, duration: Duration) -> Result<Self, TimeError> {
        let value = i128::from(self.0)
            .checked_sub(duration.as_nanos() as i128)
            .ok_or(TimeError::Overflow)?;
        i64::try_from(value)
            .map(Self)
            .map_err(|_| TimeError::Overflow)
    }

    /// 返回当前时刻相对较早时刻的非负时长。
    pub fn duration_since(self, earlier: Self) -> Result<Duration, TimeError> {
        if self < earlier {
            return Err(TimeError::InvalidOrder);
        }
        let nanos = i128::from(self.0) - i128::from(earlier.0);
        Ok(Duration::from_nanos(nanos as u64))
    }

    /// 返回 Euclidean 标准化的整秒与非负子秒纳秒。
    pub const fn to_seconds_nanos(self) -> (i64, u32) {
        (
            self.0.div_euclid(1_000_000_000),
            self.0.rem_euclid(1_000_000_000) as u32,
        )
    }

    /// 由整秒与非负子秒纳秒构造。
    pub fn try_from_seconds_nanos(seconds: i64, nanos: u32) -> Result<Self, TimeError> {
        if nanos >= 1_000_000_000 {
            return Err(TimeError::InvalidSubsecond);
        }
        let value = i128::from(seconds) * 1_000_000_000 + i128::from(nanos);
        i64::try_from(value)
            .map(Self)
            .map_err(|_| TimeError::Overflow)
    }

    /// 无损读取指定系统时间，允许 epoch 以前的时刻。
    pub fn try_from_system_time(value: SystemTime) -> Result<Self, TimeError> {
        let nanos = match value.duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_nanos() as i128,
            Err(error) => -(error.duration().as_nanos() as i128),
        };
        i64::try_from(nanos)
            .map(Self)
            .map_err(|_| TimeError::SystemTimeOutOfRange)
    }

    /// 转为系统时间，并确认平台无损保留纳秒值。
    pub fn try_into_system_time(self) -> Result<SystemTime, TimeError> {
        let nanos = i128::from(self.0).unsigned_abs() as u64;
        let duration = Duration::from_nanos(nanos);
        let value = if self.0 >= 0 {
            UNIX_EPOCH.checked_add(duration)
        } else {
            UNIX_EPOCH.checked_sub(duration)
        }
        .ok_or(TimeError::SystemTimeOutOfRange)?;
        if Self::try_from_system_time(value)? != self {
            return Err(TimeError::SystemTimePrecisionLoss);
        }
        Ok(value)
    }
}
