//! 独立时间值对象与历史可见性规则。
//!
//! 所有历史查询都由调用方显式提供知识截止点、作用域和证据；本库不读取隐式当前时间。
//!
//! 合法的具名单位构造：
//!
//! ```
//! use temporal::UnixTimeNs;
//! let epoch = UnixTimeNs::from_unix_nanos(0);
//! assert_eq!(epoch.as_unix_nanos(), 0);
//! ```
//!
//! 私有字段不能由外部直接构造：
//!
//! ```compile_fail
//! use temporal::UnixTimeNs;
//! let _ = UnixTimeNs(0);
//! ```
//!
//! 无单位整数不能隐式转换为绝对时刻：
//!
//! ```compile_fail
//! use temporal::UnixTimeNs;
//! let _: UnixTimeNs = 0_i64.into();
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(unreachable_pub)]

mod clock;
mod error;
mod evidence;
mod model;
mod observation;
mod pit;
mod precision;
mod revision;
mod unix_time;

pub use clock::{MonotonicClock, RuntimeClock, SystemMonotonicClock, SystemWallClock, WallClock};
pub use error::{PitError, PrecisionError, TemporalError, TimeError};
pub use evidence::{
    CutPartitionRef, DatasetSnapshotRef, EvidenceProofRef, ReceiptEvidenceRef,
    SourceVisibilityEvidenceRef, SystemKnowledgeCutRef, SystemVisibilityEvidenceRef, TimeBounds,
    TimeDecision, TimeQuality,
};
pub use model::{SourceTemporalRef, Temporal, TemporalDraft, TemporalProfile, TimeField};
pub use observation::{
    AbsoluteTimeRange, CivilDate, EffectiveInterval, ObservationGranularity, ObservationPeriod,
    ObservationTime,
};
pub use pit::context::{
    AsOfContext, AsOfContextParts, CandidateCoverageRef, CoverageState, EvidenceAcceptance,
    KnowledgeBasis, OutputScope, QueryPolicy, ResourceLimits, RevisionOrderPolicy,
};
pub use pit::diagnostics::{
    AdministrativeDiagnosticRef, AdministrativeDiagnostics, inspect_candidates,
};
pub use pit::effective::{
    EffectiveStateInputRef, EffectiveStateResult, StateAssertionRef, StatePolicy,
    select_effective_state_as_of,
};
pub use pit::selection::{
    FactOutcome, PitManifestRef, PitResult, QueryInputRef, SelectedEvidenceRef, select_fact_as_of,
    select_facts_as_of,
};
pub use pit::visibility::{DecisionReason, VisibilityDecision, evaluate_visibility};
pub use precision::{
    PrecisionLossPolicy, TimePrecision, TimeStorageCapabilities, quantize_unix_ns,
    require_lossless_unix_ns, stored_to_unix_ns, unix_ns_to_stored, verify_projection,
};
pub use revision::{
    DigestRef, FactKeyRef, FactVersionIdRef, FactVersionParts, FactVersionRef, PayloadRef,
    RevisionIdRef, RevisionOperation, RevisionOrderRef, SourceReleaseRequirement,
};
pub use unix_time::UnixTimeNs;
