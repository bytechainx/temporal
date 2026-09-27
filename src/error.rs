use thiserror::Error;

/// 绝对时间数值与系统时钟转换错误。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum TimeError {
    /// 时间计算超出纳秒整数范围。
    #[error("时间计算超出纳秒整数范围")]
    Overflow,
    /// 结束时刻早于起始时刻。
    #[error("结束时刻早于起始时刻")]
    InvalidOrder,
    /// 纳秒子秒字段超出范围。
    #[error("纳秒子秒字段超出范围")]
    InvalidSubsecond,
    /// 系统时间超出可表示范围。
    #[error("系统时间超出可表示范围")]
    SystemTimeOutOfRange,
    /// 系统时间无法无损保留纳秒精度。
    #[error("系统时间无法无损保留纳秒精度")]
    SystemTimePrecisionLoss,
    /// 时间源不可用。
    #[error("时间源不可用")]
    SourceUnavailable,
}

impl TimeError {
    /// 返回稳定的机器错误码。
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Overflow => "TIME_OVERFLOW",
            Self::InvalidOrder => "TIME_INVALID_ORDER",
            Self::InvalidSubsecond => "TIME_INVALID_SUBSECOND",
            Self::SystemTimeOutOfRange => "TIME_SYSTEM_RANGE",
            Self::SystemTimePrecisionLoss => "TIME_SYSTEM_PRECISION_LOSS",
            Self::SourceUnavailable => "TIME_SOURCE_UNAVAILABLE",
        }
    }
}

/// 精度换算与存储投影错误。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PrecisionError {
    /// 精度降低会丢失时间信息。
    #[error("精度降低会丢失时间信息")]
    PrecisionLoss,
    /// 精度换算超出纳秒整数范围。
    #[error("精度换算超出纳秒整数范围")]
    Overflow,
    /// 读回投影未按声明精度对齐。
    #[error("读回投影未按声明精度对齐")]
    MisalignedProjection,
    /// 读回投影与权威值的预期投影不一致。
    #[error("读回投影与权威值的预期投影不一致")]
    ProjectionMismatch,
    /// 存储能力未声明可无损保存纳秒时间。
    #[error("存储能力未声明可无损保存纳秒时间")]
    LosslessRequired,
}

impl PrecisionError {
    /// 返回稳定的机器错误码。
    pub const fn code(&self) -> &'static str {
        match self {
            Self::PrecisionLoss => "PREC_LOSS_REJECTED",
            Self::Overflow => "PREC_OVERFLOW",
            Self::MisalignedProjection => "PREC_MISALIGNED",
            Self::ProjectionMismatch => "PREC_PROJECTION_MISMATCH",
            Self::LosslessRequired => "PREC_LOSSLESS_REQUIRED",
        }
    }
}

/// 时间角色、日历与有效区间校验错误。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum TemporalError {
    /// 缺少必需时间角色。
    #[error("缺少必需时间角色")]
    MissingRequiredRole,
    /// 时间角色不适用于该资料。
    #[error("时间角色不适用于该资料")]
    RoleNotApplicable,
    /// 日期无效。
    #[error("日期无效")]
    InvalidDate,
    /// 期间无效。
    #[error("期间无效")]
    InvalidPeriod,
    /// 时间区间无效。
    #[error("时间区间无效")]
    InvalidInterval,
    /// 时间资料与配置不匹配。
    #[error("时间资料与配置不匹配")]
    ProfileMismatch,
    /// 时间元数据不一致。
    #[error("时间元数据不一致")]
    MetadataMismatch,
}

impl TemporalError {
    /// 返回稳定的机器错误码。
    pub const fn code(&self) -> &'static str {
        match self {
            Self::MissingRequiredRole => "TEMP_ROLE_REQUIRED",
            Self::RoleNotApplicable => "TEMP_ROLE_NOT_APPLICABLE",
            Self::InvalidDate => "TEMP_DATE_INVALID",
            Self::InvalidPeriod => "TEMP_PERIOD_INVALID",
            Self::InvalidInterval => "TEMP_INTERVAL_INVALID",
            Self::ProfileMismatch => "TEMP_PROFILE_MISMATCH",
            Self::MetadataMismatch => "TEMP_METADATA_MISMATCH",
        }
    }
}

/// 时间点查询的上下文、证据与选版错误。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PitError {
    /// 查询上下文不匹配。
    #[error("查询上下文不匹配")]
    ContextMismatch,
    /// 时间证据不匹配。
    #[error("时间证据不匹配")]
    EvidenceMismatch,
    /// 同一修订的内容冲突。
    #[error("同一修订的内容冲突")]
    ConflictingRevision,
    /// 修订顺序存在歧义。
    #[error("修订顺序存在歧义")]
    AmbiguousRevisionOrder,
    /// 修订顺序作用域不可比较。
    #[error("修订顺序作用域不可比较")]
    IncomparableOrderScope,
    /// 修订链无效。
    #[error("修订链无效")]
    InvalidRevisionChain,
    /// 无法确定唯一可见修订。
    #[error("无法确定唯一可见修订")]
    IndeterminateSelection,
    /// 系统历史切片缺乏证明。
    #[error("系统历史切片缺乏证明")]
    UnprovenSystemCut,
    /// 候选集合不完整。
    #[error("候选集合不完整")]
    IncompleteCandidateSet,
    /// 指定快照不可用。
    #[error("指定快照不可用")]
    SnapshotUnavailable,
    /// 有效状态断言冲突。
    #[error("有效状态断言冲突")]
    ConflictingEffectiveState,
    /// 查询超过资源上限。
    #[error("查询超过资源上限")]
    LimitExceeded,
    /// 查询所需资源不可用。
    #[error("查询所需资源不可用")]
    ResourceUnavailable,
}

impl PitError {
    /// 返回稳定的机器错误码。
    pub const fn code(&self) -> &'static str {
        match self {
            Self::ContextMismatch => "PIT_CONTEXT_MISMATCH",
            Self::EvidenceMismatch => "PIT_EVIDENCE_MISMATCH",
            Self::ConflictingRevision => "PIT_REVISION_CONFLICT",
            Self::AmbiguousRevisionOrder => "PIT_REVISION_ORDER_AMBIGUOUS",
            Self::IncomparableOrderScope => "PIT_ORDER_SCOPE_INCOMPARABLE",
            Self::InvalidRevisionChain => "PIT_REVISION_CHAIN_INVALID",
            Self::IndeterminateSelection => "PIT_INDETERMINATE",
            Self::UnprovenSystemCut => "PIT_SYSTEM_CUT_UNPROVEN",
            Self::IncompleteCandidateSet => "PIT_INCOMPLETE",
            Self::SnapshotUnavailable => "PIT_SNAPSHOT_UNAVAILABLE",
            Self::ConflictingEffectiveState => "PIT_EFFECTIVE_STATE_CONFLICT",
            Self::LimitExceeded => "PIT_LIMIT",
            Self::ResourceUnavailable => "PIT_RESOURCE_UNAVAILABLE",
        }
    }
}
