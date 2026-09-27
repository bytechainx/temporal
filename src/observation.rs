//! 民用日期、观测期间与半开时间区间。

use std::time::Duration;

use crate::error::TemporalError;
use crate::unix_time::UnixTimeNs;

/// 公历民用日期；不隐含时区或 UTC 转换。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CivilDate {
    year: u16,
    month: u8,
    day: u8,
}

impl CivilDate {
    /// 构造有效的公历日期，年份限于 1 到 9999。
    pub fn try_new(year: u16, month: u8, day: u8) -> Result<Self, TemporalError> {
        if !(1..=9999).contains(&year) || !(1..=12).contains(&month) {
            return Err(TemporalError::InvalidDate);
        }
        let leap =
            year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
        let last_day = match month {
            2 if leap => 29,
            2 => 28,
            4 | 6 | 9 | 11 => 30,
            _ => 31,
        };
        if day == 0 || day > last_day {
            return Err(TemporalError::InvalidDate);
        }
        Ok(Self { year, month, day })
    }

    /// 年份。
    pub const fn year(self) -> u16 {
        self.year
    }

    /// 月份。
    pub const fn month(self) -> u8 {
        self.month
    }

    /// 月内日期。
    pub const fn day(self) -> u8 {
        self.day
    }
}

/// 绝对时刻上的有限半开范围。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AbsoluteTimeRange {
    start: UnixTimeNs,
    end_exclusive: UnixTimeNs,
}

impl AbsoluteTimeRange {
    /// 构造非空的 `[start, end_exclusive)`。
    pub fn try_new(start: UnixTimeNs, end_exclusive: UnixTimeNs) -> Result<Self, TemporalError> {
        if start >= end_exclusive {
            return Err(TemporalError::InvalidInterval);
        }
        Ok(Self {
            start,
            end_exclusive,
        })
    }

    /// 包含的起点。
    pub const fn start(self) -> UnixTimeNs {
        self.start
    }

    /// 不包含的终点。
    pub const fn end_exclusive(self) -> UnixTimeNs {
        self.end_exclusive
    }

    /// 判断时刻是否位于范围内。
    pub fn contains(self, time: UnixTimeNs) -> bool {
        self.start <= time && time < self.end_exclusive
    }

    /// 返回范围长度；完整 `i64` 坐标跨度也可表示。
    pub fn duration(self) -> Duration {
        let nanos =
            i128::from(self.end_exclusive.as_unix_nanos()) - i128::from(self.start.as_unix_nanos());
        Duration::from_nanos(nanos as u64)
    }

    /// 返回非空交集；仅相邻时返回 `None`。
    pub fn intersection(self, other: Self) -> Option<Self> {
        let start = self.start.max(other.start);
        let end_exclusive = self.end_exclusive.min(other.end_exclusive);
        (start < end_exclusive).then_some(Self {
            start,
            end_exclusive,
        })
    }
}

/// 民用日期期间的粒度。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ObservationGranularity {
    /// 单个公历日。
    Day,
    /// 公历月。
    Month,
    /// 公历季度。
    Quarter,
    /// 公历年。
    Year,
    /// 由调用方的版本化日历定义解释。
    Custom,
}

/// 带版本化日历引用的民用日期期间。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObservationPeriod {
    start: CivilDate,
    end_exclusive: CivilDate,
    granularity: ObservationGranularity,
    calendar_ref: String,
}

impl ObservationPeriod {
    /// 校验半开期间、粒度边界和非空日历引用。
    pub fn try_new(
        start: CivilDate,
        end_exclusive: CivilDate,
        granularity: ObservationGranularity,
        calendar_ref: impl Into<String>,
    ) -> Result<Self, TemporalError> {
        let calendar_ref = calendar_ref.into();
        if start >= end_exclusive || calendar_ref.is_empty() {
            return Err(TemporalError::InvalidPeriod);
        }
        let expected_end = match granularity {
            ObservationGranularity::Day => next_day(start),
            ObservationGranularity::Month if start.day == 1 => next_month(start),
            ObservationGranularity::Quarter
                if start.day == 1 && matches!(start.month, 1 | 4 | 7 | 10) =>
            {
                next_quarter(start)
            }
            ObservationGranularity::Year if start.month == 1 && start.day == 1 => {
                CivilDate::try_new(start.year + 1, 1, 1).ok()
            }
            ObservationGranularity::Custom => Some(end_exclusive),
            _ => None,
        };
        if expected_end != Some(end_exclusive) {
            return Err(TemporalError::InvalidPeriod);
        }
        Ok(Self {
            start,
            end_exclusive,
            granularity,
            calendar_ref,
        })
    }

    /// 包含的民用日期起点。
    pub const fn start(&self) -> CivilDate {
        self.start
    }

    /// 不包含的民用日期终点。
    pub const fn end_exclusive(&self) -> CivilDate {
        self.end_exclusive
    }

    /// 期间粒度。
    pub const fn granularity(&self) -> ObservationGranularity {
        self.granularity
    }

    /// 调用方拥有的版本化日历引用。
    pub fn calendar_ref(&self) -> &str {
        &self.calendar_ref
    }

    /// 判断民用日期是否位于期间内。
    pub fn contains(&self, date: CivilDate) -> bool {
        self.start <= date && date < self.end_exclusive
    }
}

fn next_day(date: CivilDate) -> Option<CivilDate> {
    CivilDate::try_new(date.year, date.month, date.day + 1)
        .or_else(|_| {
            let month = date.month.checked_add(1).filter(|month| *month <= 12);
            if let Some(month) = month {
                CivilDate::try_new(date.year, month, 1)
            } else {
                CivilDate::try_new(date.year + 1, 1, 1)
            }
        })
        .ok()
}

fn next_month(date: CivilDate) -> Option<CivilDate> {
    let (year, month) = if date.month == 12 {
        (date.year + 1, 1)
    } else {
        (date.year, date.month + 1)
    };
    CivilDate::try_new(year, month, 1).ok()
}

fn next_quarter(date: CivilDate) -> Option<CivilDate> {
    let (year, month) = if date.month == 10 {
        (date.year + 1, 1)
    } else {
        (date.year, date.month + 3)
    };
    CivilDate::try_new(year, month, 1).ok()
}

/// 观测点、绝对窗口或民用日期期间。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ObservationTime {
    /// 绝对观测点。
    At(UnixTimeNs),
    /// 绝对时刻半开窗口。
    Window(AbsoluteTimeRange),
    /// 民用日期统计期间。
    Period(ObservationPeriod),
}

/// 绝对时刻上的生效区间；`None` 明确表示无界终点。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EffectiveInterval {
    start: UnixTimeNs,
    end_exclusive: Option<UnixTimeNs>,
}

impl EffectiveInterval {
    /// 构造半开生效区间。
    pub fn try_new(
        start: UnixTimeNs,
        end_exclusive: Option<UnixTimeNs>,
    ) -> Result<Self, TemporalError> {
        if end_exclusive.is_some_and(|end| end <= start) {
            return Err(TemporalError::InvalidInterval);
        }
        Ok(Self {
            start,
            end_exclusive,
        })
    }

    /// 包含的生效起点。
    pub const fn start(self) -> UnixTimeNs {
        self.start
    }

    /// 不包含的终点；`None` 表示明确无界。
    pub const fn end_exclusive(self) -> Option<UnixTimeNs> {
        self.end_exclusive
    }

    /// 判断时刻是否位于生效区间内。
    pub fn contains(self, time: UnixTimeNs) -> bool {
        self.start <= time && self.end_exclusive.is_none_or(|end| time < end)
    }
}
