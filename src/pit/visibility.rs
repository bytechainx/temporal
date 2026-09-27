//! 单个事实版本的历史可见性判定。

use crate::error::PitError;
use crate::evidence::TimeDecision;
use crate::model::TimeField;
use crate::pit::context::{AsOfContext, KnowledgeBasis};
use crate::pit::selection::validate_version_limits;
use crate::revision::{FactVersionRef, SourceReleaseRequirement};

/// 可见性判定的稳定原因类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionReason {
    /// 来源发布晚于截止点。
    ReleaseAfterCutoff,
    /// 系统可读时间晚于截止点。
    SystemAvailableAfterCutoff,
    /// 可读事件不在历史切片中。
    OutsideSystemCut,
    /// 来源时间缺乏证明。
    SourceTimeUnproven,
    /// 系统时间缺乏证明。
    SystemTimeUnproven,
    /// 时间精度不足。
    PrecisionInsufficient,
    /// 接收链缺乏证明。
    ReceiptUnproven,
    /// 必需来源释放条件缺乏证明。
    ReleaseConditionUnproven,
}

impl DecisionReason {
    /// 返回与规格一致的机器原因码。
    pub const fn code(self) -> &'static str {
        match self {
            Self::ReleaseAfterCutoff => "RELEASE_AFTER_CUTOFF",
            Self::SystemAvailableAfterCutoff => "SYSTEM_AVAILABLE_AFTER_CUTOFF",
            Self::OutsideSystemCut => "OUTSIDE_SYSTEM_CUT",
            Self::SourceTimeUnproven => "SOURCE_TIME_UNPROVEN",
            Self::SystemTimeUnproven => "SYSTEM_TIME_UNPROVEN",
            Self::PrecisionInsufficient => "PRECISION_INSUFFICIENT",
            Self::ReceiptUnproven => "RECEIPT_UNPROVEN",
            Self::ReleaseConditionUnproven => "RELEASE_CONDITION_UNPROVEN",
        }
    }
}

/// 单版本判定；原因不含候选身份或载荷。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisibilityDecision {
    /// 所有必要门禁均得到证明。
    Visible,
    /// 已证明该版本在本次查询不可见。
    NotVisible(DecisionReason),
    /// 尚不能证明可见或排除。
    Indeterminate(DecisionReason),
}

/// 只评估当前版本，不推断其是否为最新修订。
pub fn evaluate_visibility(
    version: FactVersionRef<'_>,
    context: AsOfContext<'_>,
) -> Result<VisibilityDecision, PitError> {
    let limits = context.policy().limits();
    validate_version_limits(
        version,
        limits.max_identity_bytes(),
        limits.max_evidence_per_version(),
    )?;
    if context
        .system_cut()
        .is_some_and(|cut| cut.partitions().len() > limits.max_partitions())
    {
        return Err(PitError::LimitExceeded);
    }
    if version.identity().authority_scope() != context.authority_scope()
        || version.dataset_snapshot() != context.dataset_snapshot()
        || version.profile() != context.policy().profile()
    {
        return Err(PitError::ContextMismatch);
    }
    let cutoff = context.known_at();
    let release = match version.source_release() {
        SourceReleaseRequirement::Required => {
            if let Some(evidence) = version.source_evidence() {
                if let Some(scope) = context.source_scope() {
                    if evidence.source_scope() != scope {
                        return Err(PitError::EvidenceMismatch);
                    }
                } else {
                    return Err(PitError::ContextMismatch);
                }
                let bounds = evidence.bounds();
                if let TimeField::Known(published) = version.source_temporal().publication_time()
                    && (bounds
                        .earliest_possible()
                        .is_some_and(|lower| *published < lower)
                        || bounds
                            .definitely_available_by()
                            .is_some_and(|upper| *published > upper))
                {
                    return Err(PitError::EvidenceMismatch);
                }
                match bounds.at(cutoff, context.policy().evidence_acceptance()) {
                    TimeDecision::ProvenByCutoff => VisibilityDecision::Visible,
                    TimeDecision::NotYetVisible => {
                        VisibilityDecision::NotVisible(DecisionReason::ReleaseAfterCutoff)
                    }
                    TimeDecision::Indeterminate => {
                        VisibilityDecision::Indeterminate(DecisionReason::SourceTimeUnproven)
                    }
                }
            } else {
                VisibilityDecision::Indeterminate(DecisionReason::ReleaseConditionUnproven)
            }
        }
        SourceReleaseRequirement::NotApplicable { .. } => {
            if context.knowledge_basis() == KnowledgeBasis::SourcePublishedAsOf {
                VisibilityDecision::Indeterminate(DecisionReason::ReleaseConditionUnproven)
            } else {
                VisibilityDecision::Visible
            }
        }
    };
    if let TimeField::Known(published) = version.source_temporal().publication_time()
        && *published > cutoff
    {
        return Ok(VisibilityDecision::NotVisible(
            DecisionReason::ReleaseAfterCutoff,
        ));
    }
    if let TimeField::Known(revised) = version.source_temporal().revision_time()
        && *revised > cutoff
    {
        return Ok(VisibilityDecision::NotVisible(
            DecisionReason::ReleaseAfterCutoff,
        ));
    }
    if matches!(release, VisibilityDecision::NotVisible(_)) {
        return Ok(release);
    }
    if context.knowledge_basis() == KnowledgeBasis::SourcePublishedAsOf {
        return Ok(release);
    }
    let boundary = context
        .consumer_boundary()
        .ok_or(PitError::ContextMismatch)?;
    let cut = context.system_cut().ok_or(PitError::UnprovenSystemCut)?;
    let mut saw_in_cut = false;
    let mut saw_proven = false;
    let mut saw_future = false;
    let mut saw_unknown = false;
    let mut saw_unmatched_receipt = false;
    for &event in version.system_evidence() {
        if event.consumer_boundary() != boundary || event.log_epoch() != cut.boundary_epoch() {
            continue;
        }
        if !cut.contains(event, boundary, cutoff)? {
            continue;
        }
        saw_in_cut = true;
        let Some(receipt) = version.receipts().iter().find(|item| {
            item.receiver_boundary() == event.receiver_boundary()
                && item.receiver_epoch() == event.receiver_epoch()
                && item.receipt_id() == event.receipt_id()
        }) else {
            saw_unmatched_receipt = true;
            continue;
        };
        if receipt.version() != event.version() {
            return Err(PitError::EvidenceMismatch);
        }
        if version.receipts().iter().any(|item| {
            item.receiver_boundary() == receipt.receiver_boundary()
                && item.receiver_epoch() == receipt.receiver_epoch()
                && item.receipt_id() == receipt.receipt_id()
                && item != receipt
        }) {
            return Err(PitError::EvidenceMismatch);
        }
        if receipt
            .received_bounds()
            .at(cutoff, context.policy().evidence_acceptance())
            != TimeDecision::ProvenByCutoff
        {
            saw_unmatched_receipt = true;
            continue;
        }
        match event
            .bounds()
            .at(cutoff, context.policy().evidence_acceptance())
        {
            TimeDecision::ProvenByCutoff => {
                saw_proven = true;
            }
            TimeDecision::NotYetVisible => saw_future = true,
            TimeDecision::Indeterminate => saw_unknown = true,
        }
    }
    if saw_proven {
        return Ok(release);
    }
    if saw_unknown {
        return Ok(VisibilityDecision::Indeterminate(
            DecisionReason::SystemTimeUnproven,
        ));
    }
    if saw_unmatched_receipt {
        return Ok(VisibilityDecision::Indeterminate(
            DecisionReason::ReceiptUnproven,
        ));
    }
    if saw_future {
        return Ok(VisibilityDecision::NotVisible(
            DecisionReason::SystemAvailableAfterCutoff,
        ));
    }
    if !saw_in_cut && !version.system_evidence().is_empty() {
        return Ok(VisibilityDecision::NotVisible(
            DecisionReason::OutsideSystemCut,
        ));
    }
    if matches!(release, VisibilityDecision::Indeterminate(_)) {
        return Ok(release);
    }
    Ok(VisibilityDecision::Indeterminate(
        DecisionReason::SystemTimeUnproven,
    ))
}
