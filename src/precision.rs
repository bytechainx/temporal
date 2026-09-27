use crate::PrecisionError;

/// 时间存储单位的分辨率，不表示采样准确度。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimePrecision {
    /// 整秒。
    Seconds,
    /// 整毫秒。
    Milliseconds,
    /// 整微秒。
    Microseconds,
    /// 纳秒。
    Nanoseconds,
}

impl TimePrecision {
    /// 每个存储单位包含的纳秒数。
    pub const fn nanos_per_unit(self) -> i64 {
        match self {
            Self::Seconds => 1_000_000_000,
            Self::Milliseconds => 1_000_000,
            Self::Microseconds => 1_000,
            Self::Nanoseconds => 1,
        }
    }
}

/// 纳秒转为较粗单位时的损失策略。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum PrecisionLossPolicy {
    /// 非整单位时拒绝转换。
    #[default]
    Reject,
    /// 向零截断。
    ExplicitTruncate,
    /// 向负无穷取整。
    ExplicitFloor,
}

/// 存储适配器声明的时间能力；声明本身不是读写证明。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeStorageCapabilities {
    /// 存储字段的声明分辨率。
    pub stored_precision: TimePrecision,
    /// 存储适配器是否声明已标准化为 UTC。
    pub utc_normalized: bool,
    /// 存储适配器是否声明无损保存完整纳秒范围。
    pub lossless_unix_ns: bool,
}

/// 将 POSIX epoch 纳秒换成指定存储单位。
pub fn unix_ns_to_stored(
    unix_ns: i64,
    precision: TimePrecision,
    policy: PrecisionLossPolicy,
) -> Result<i64, PrecisionError> {
    let scale = precision.nanos_per_unit();
    match policy {
        PrecisionLossPolicy::Reject => {
            if unix_ns % scale != 0 {
                return Err(PrecisionError::PrecisionLoss);
            }
            Ok(unix_ns / scale)
        }
        PrecisionLossPolicy::ExplicitTruncate => Ok(unix_ns / scale),
        PrecisionLossPolicy::ExplicitFloor => Ok(unix_ns.div_euclid(scale)),
    }
}

/// 将指定存储单位无损换回 POSIX epoch 纳秒。
pub fn stored_to_unix_ns(stored: i64, precision: TimePrecision) -> Result<i64, PrecisionError> {
    stored
        .checked_mul(precision.nanos_per_unit())
        .ok_or(PrecisionError::Overflow)
}

/// 按明确损失策略投影，再还原到纳秒单位。
pub fn quantize_unix_ns(
    unix_ns: i64,
    precision: TimePrecision,
    policy: PrecisionLossPolicy,
) -> Result<i64, PrecisionError> {
    let stored = unix_ns_to_stored(unix_ns, precision, policy)?;
    stored_to_unix_ns(stored, precision)
}

/// 先检查读回值对齐，再比较权威值的预期投影。
pub fn verify_projection(
    authoritative_ns: i64,
    projected_ns: i64,
    precision: TimePrecision,
    policy: PrecisionLossPolicy,
) -> Result<(), PrecisionError> {
    if projected_ns % precision.nanos_per_unit() != 0 {
        return Err(PrecisionError::MisalignedProjection);
    }
    let expected = quantize_unix_ns(authoritative_ns, precision, policy)?;
    if projected_ns != expected {
        return Err(PrecisionError::ProjectionMismatch);
    }
    Ok(())
}

/// 仅检查适配器是否声明可无损保存纳秒值。
pub fn require_lossless_unix_ns(
    capabilities: TimeStorageCapabilities,
) -> Result<(), PrecisionError> {
    if capabilities.lossless_unix_ns {
        Ok(())
    } else {
        Err(PrecisionError::LosslessRequired)
    }
}
