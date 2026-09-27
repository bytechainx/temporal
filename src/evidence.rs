//! 可见性证据引用与历史知识切片的结构检查。

use crate::error::PitError;
use crate::pit::context::{EvidenceAcceptance, ResourceLimits};
use crate::revision::FactVersionIdRef;
use crate::unix_time::UnixTimeNs;

fn label<'a>(value: &'a str, limits: &ResourceLimits) -> Result<&'a str, PitError> {
    if value.is_empty() {
        return Err(PitError::EvidenceMismatch);
    }
    if value.len() > limits.max_identity_bytes() {
        return Err(PitError::LimitExceeded);
    }
    Ok(value)
}

/// 时间区间的质量说明；`Exact` 要求上下界同一时刻。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeQuality {
    /// 已证明为同一精确时刻。
    Exact,
    /// 单侧或双侧保守时间界限。
    Bounded,
}

/// 截止点相对可用时刻的三态判断。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeDecision {
    /// 已证明不晚于截止点可用。
    ProvenByCutoff,
    /// 已证明晚于截止点才可能可用。
    NotYetVisible,
    /// 时间证据不足或跨越截止点。
    Indeterminate,
}

/// 来源证明的最早与最晚可用时间；两个边界可以独立缺失。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeBounds<'a> {
    earliest_possible: Option<UnixTimeNs>,
    definitely_available_by: Option<UnixTimeNs>,
    quality: TimeQuality,
    quality_ref: &'a str,
}

impl<'a> TimeBounds<'a> {
    /// 检查区间顺序与质量声明，不证明原始证据真实。
    pub fn try_new(
        lower: Option<UnixTimeNs>,
        upper: Option<UnixTimeNs>,
        quality: TimeQuality,
        quality_ref: &'a str,
        limits: &ResourceLimits,
    ) -> Result<Self, PitError> {
        label(quality_ref, limits)?;
        if lower.zip(upper).is_some_and(|(start, end)| start > end) {
            return Err(PitError::EvidenceMismatch);
        }
        if quality == TimeQuality::Exact && (lower.is_none() || lower != upper) {
            return Err(PitError::EvidenceMismatch);
        }
        Ok(Self {
            earliest_possible: lower,
            definitely_available_by: upper,
            quality,
            quality_ref,
        })
    }

    /// 最早可能已可用的时刻。
    pub const fn earliest_possible(self) -> Option<UnixTimeNs> {
        self.earliest_possible
    }
    /// 可保守确认已可用的时刻。
    pub const fn definitely_available_by(self) -> Option<UnixTimeNs> {
        self.definitely_available_by
    }
    /// 证据精度分类。
    pub const fn quality(self) -> TimeQuality {
        self.quality
    }
    /// 精度、误差与提取合同引用。
    pub const fn quality_ref(self) -> &'a str {
        self.quality_ref
    }
    /// 只判断时间维度；证据作用域和快照由调用方另行检查。
    pub fn at(self, cutoff: UnixTimeNs, acceptance: EvidenceAcceptance) -> TimeDecision {
        if self.earliest_possible.is_some_and(|lower| lower > cutoff) {
            return TimeDecision::NotYetVisible;
        }
        if self
            .definitely_available_by
            .is_some_and(|upper| upper <= cutoff)
            && (acceptance == EvidenceAcceptance::ConservativeBounds
                || self.quality == TimeQuality::Exact)
        {
            return TimeDecision::ProvenByCutoff;
        }
        TimeDecision::Indeterminate
    }
}

/// 外部受信证据入口的验收记录引用；本库仅验证结构与绑定。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvidenceProofRef<'a> {
    evidence_id: &'a str,
    schema_version: &'a str,
    extractor_version: &'a str,
    raw_evidence_ref: &'a str,
    acceptance_ref: &'a str,
}

impl<'a> EvidenceProofRef<'a> {
    /// 登记受信入口验收结果的稳定引用；任意字符串不自动构成可信证明。
    pub fn try_new(
        evidence_id: &'a str,
        schema_version: &'a str,
        extractor_version: &'a str,
        raw_evidence_ref: &'a str,
        acceptance_ref: &'a str,
        limits: &ResourceLimits,
    ) -> Result<Self, PitError> {
        Ok(Self {
            evidence_id: label(evidence_id, limits)?,
            schema_version: label(schema_version, limits)?,
            extractor_version: label(extractor_version, limits)?,
            raw_evidence_ref: label(raw_evidence_ref, limits)?,
            acceptance_ref: label(acceptance_ref, limits)?,
        })
    }

    /// 证据唯一身份。
    pub const fn evidence_id(self) -> &'a str {
        self.evidence_id
    }
    /// 证据 schema 版本。
    pub const fn schema_version(self) -> &'a str {
        self.schema_version
    }
    /// 证据提取器版本。
    pub const fn extractor_version(self) -> &'a str {
        self.extractor_version
    }
    /// 不可变原始证据引用。
    pub const fn raw_evidence_ref(self) -> &'a str {
        self.raw_evidence_ref
    }
    /// 外部受信入口的验收记录引用。
    pub const fn acceptance_ref(self) -> &'a str {
        self.acceptance_ref
    }
}

/// 来源实际发布且面向特定受众的证据。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceVisibilityEvidenceRef<'a> {
    version: FactVersionIdRef<'a>,
    source_scope: &'a str,
    bounds: TimeBounds<'a>,
    proof: EvidenceProofRef<'a>,
}

impl<'a> SourceVisibilityEvidenceRef<'a> {
    /// 将来源作用域、版本内容及发布界限绑定在一条证据上。
    pub fn try_new(
        version: FactVersionIdRef<'a>,
        source_scope: &'a str,
        bounds: TimeBounds<'a>,
        proof: EvidenceProofRef<'a>,
        limits: &ResourceLimits,
    ) -> Result<Self, PitError> {
        Ok(Self {
            version,
            source_scope: label(source_scope, limits)?,
            bounds,
            proof,
        })
    }
    /// 当前规范化版本身份。
    pub const fn version(self) -> FactVersionIdRef<'a> {
        self.version
    }
    /// 来源及受众作用域。
    pub const fn source_scope(self) -> &'a str {
        self.source_scope
    }
    /// 已证明的实际发布界限。
    pub const fn bounds(self) -> TimeBounds<'a> {
        self.bounds
    }
    /// 受信入口验收记录。
    pub const fn proof(self) -> EvidenceProofRef<'a> {
        self.proof
    }
}

/// 实际接收事件，不自动证明消费边界已可读取。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReceiptEvidenceRef<'a> {
    version: FactVersionIdRef<'a>,
    receiver_boundary: &'a str,
    receiver_epoch: &'a str,
    receipt_id: &'a str,
    received_bounds: TimeBounds<'a>,
    proof: EvidenceProofRef<'a>,
}

impl<'a> ReceiptEvidenceRef<'a> {
    /// 绑定接收边界、代次、独立事件身份与版本。
    pub fn try_new(
        version: FactVersionIdRef<'a>,
        receiver_boundary: &'a str,
        receiver_epoch: &'a str,
        receipt_id: &'a str,
        received_bounds: TimeBounds<'a>,
        proof: EvidenceProofRef<'a>,
        limits: &ResourceLimits,
    ) -> Result<Self, PitError> {
        Ok(Self {
            version,
            receiver_boundary: label(receiver_boundary, limits)?,
            receiver_epoch: label(receiver_epoch, limits)?,
            receipt_id: label(receipt_id, limits)?,
            received_bounds,
            proof,
        })
    }
    /// 当前规范化版本身份。
    pub const fn version(self) -> FactVersionIdRef<'a> {
        self.version
    }
    /// 接收边界。
    pub const fn receiver_boundary(self) -> &'a str {
        self.receiver_boundary
    }
    /// 接收代次。
    pub const fn receiver_epoch(self) -> &'a str {
        self.receiver_epoch
    }
    /// 此次独立接收事件身份。
    pub const fn receipt_id(self) -> &'a str {
        self.receipt_id
    }
    /// 接收采样时间界限。
    pub const fn received_bounds(self) -> TimeBounds<'a> {
        self.received_bounds
    }
    /// 受信入口验收记录。
    pub const fn proof(self) -> EvidenceProofRef<'a> {
        self.proof
    }
}

/// 消费边界实际允许读取当前版本的独立事件。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SystemVisibilityEvidenceRef<'a> {
    version: FactVersionIdRef<'a>,
    consumer_boundary: &'a str,
    log_epoch: &'a str,
    event_id: &'a str,
    receiver_boundary: &'a str,
    receiver_epoch: &'a str,
    receipt_id: &'a str,
    partition: &'a str,
    cursor: u64,
    bounds: TimeBounds<'a>,
    proof: EvidenceProofRef<'a>,
}

impl<'a> SystemVisibilityEvidenceRef<'a> {
    /// 绑定消费边界、日志代次、接收引用、游标与可读时间。
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        version: FactVersionIdRef<'a>,
        consumer_boundary: &'a str,
        log_epoch: &'a str,
        event_id: &'a str,
        receiver_boundary: &'a str,
        receiver_epoch: &'a str,
        receipt_id: &'a str,
        partition: &'a str,
        cursor: u64,
        bounds: TimeBounds<'a>,
        proof: EvidenceProofRef<'a>,
        limits: &ResourceLimits,
    ) -> Result<Self, PitError> {
        Ok(Self {
            version,
            consumer_boundary: label(consumer_boundary, limits)?,
            log_epoch: label(log_epoch, limits)?,
            event_id: label(event_id, limits)?,
            receiver_boundary: label(receiver_boundary, limits)?,
            receiver_epoch: label(receiver_epoch, limits)?,
            receipt_id: label(receipt_id, limits)?,
            partition: label(partition, limits)?,
            cursor,
            bounds,
            proof,
        })
    }
    /// 当前规范化版本身份。
    pub const fn version(self) -> FactVersionIdRef<'a> {
        self.version
    }
    /// 目标消费者可读边界。
    pub const fn consumer_boundary(self) -> &'a str {
        self.consumer_boundary
    }
    /// 日志或进程代次。
    pub const fn log_epoch(self) -> &'a str {
        self.log_epoch
    }
    /// 独立可读事件身份。
    pub const fn event_id(self) -> &'a str {
        self.event_id
    }
    /// 被引用接收事件的接收边界。
    pub const fn receiver_boundary(self) -> &'a str {
        self.receiver_boundary
    }
    /// 被引用接收事件的接收代次。
    pub const fn receiver_epoch(self) -> &'a str {
        self.receiver_epoch
    }
    /// 所绑定的接收事件身份。
    pub const fn receipt_id(self) -> &'a str {
        self.receipt_id
    }
    /// 事件所在分区。
    pub const fn partition(self) -> &'a str {
        self.partition
    }
    /// 在代次和分区内的可读事件序号。
    pub const fn cursor(self) -> u64 {
        self.cursor
    }
    /// 消费者可用时间界限。
    pub const fn bounds(self) -> TimeBounds<'a> {
        self.bounds
    }
    /// 受信入口验收记录。
    pub const fn proof(self) -> EvidenceProofRef<'a> {
        self.proof
    }
}

/// 固定来源版本、元数据与证据集合的不可变快照。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DatasetSnapshotRef<'a> {
    id: &'a str,
    manifest_ref: &'a str,
    schema_version: &'a str,
}

impl<'a> DatasetSnapshotRef<'a> {
    /// 检查快照身份及不可变清单引用。
    pub fn try_new(
        id: &'a str,
        manifest_ref: &'a str,
        schema_version: &'a str,
        limits: &ResourceLimits,
    ) -> Result<Self, PitError> {
        Ok(Self {
            id: label(id, limits)?,
            manifest_ref: label(manifest_ref, limits)?,
            schema_version: label(schema_version, limits)?,
        })
    }
    /// 快照身份。
    pub const fn id(self) -> &'a str {
        self.id
    }
    /// 不可变清单引用。
    pub const fn manifest_ref(self) -> &'a str {
        self.manifest_ref
    }
    /// 快照 schema 版本。
    pub const fn schema_version(self) -> &'a str {
        self.schema_version
    }
}

/// 某日志代次和分区的包含式可读前沿。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CutPartitionRef<'a> {
    partition: &'a str,
    log_epoch: &'a str,
    inclusive_frontier: u64,
}

impl<'a> CutPartitionRef<'a> {
    /// 构造具名分区的历史可读前沿。
    pub fn try_new(
        partition: &'a str,
        log_epoch: &'a str,
        inclusive_frontier: u64,
        limits: &ResourceLimits,
    ) -> Result<Self, PitError> {
        Ok(Self {
            partition: label(partition, limits)?,
            log_epoch: label(log_epoch, limits)?,
            inclusive_frontier,
        })
    }
    /// 分区名称。
    pub const fn partition(self) -> &'a str {
        self.partition
    }
    /// 日志代次。
    pub const fn log_epoch(self) -> &'a str {
        self.log_epoch
    }
    /// 可读事件的包含式前沿。
    pub const fn inclusive_frontier(self) -> u64 {
        self.inclusive_frontier
    }
}

/// 历史决策边界固定的可读事件集合证明。
#[derive(Debug, Clone, Copy)]
pub struct SystemKnowledgeCutRef<'a> {
    boundary_id: &'a str,
    boundary_epoch: &'a str,
    cut_id: &'a str,
    manifest_ref: &'a str,
    known_at: UnixTimeNs,
    known_at_binding_ref: &'a str,
    partitions: &'a [CutPartitionRef<'a>],
    consistency_policy_ref: &'a str,
    evidence_snapshot_ref: &'a str,
}

impl<'a> SystemKnowledgeCutRef<'a> {
    /// 检查切片各作用域、证明引用、分区唯一性及资源界限。
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        boundary_id: &'a str,
        boundary_epoch: &'a str,
        cut_id: &'a str,
        manifest_ref: &'a str,
        known_at: UnixTimeNs,
        known_at_binding_ref: &'a str,
        partitions: &'a [CutPartitionRef<'a>],
        consistency_policy_ref: &'a str,
        evidence_snapshot_ref: &'a str,
        limits: &ResourceLimits,
    ) -> Result<Self, PitError> {
        if partitions.len() > limits.max_partitions() {
            return Err(PitError::LimitExceeded);
        }
        let working_bytes = partitions
            .len()
            .checked_mul(std::mem::size_of::<&str>())
            .ok_or(PitError::LimitExceeded)?;
        if working_bytes > limits.max_working_bytes() {
            return Err(PitError::LimitExceeded);
        }
        let mut names = Vec::new();
        names
            .try_reserve_exact(partitions.len())
            .map_err(|_| PitError::ResourceUnavailable)?;
        for item in partitions {
            if item.log_epoch != boundary_epoch {
                return Err(PitError::UnprovenSystemCut);
            }
            names.push(item.partition);
        }
        names.sort_unstable();
        if names.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(PitError::UnprovenSystemCut);
        }
        Ok(Self {
            boundary_id: label(boundary_id, limits)?,
            boundary_epoch: label(boundary_epoch, limits)?,
            cut_id: label(cut_id, limits)?,
            manifest_ref: label(manifest_ref, limits)?,
            known_at,
            known_at_binding_ref: label(known_at_binding_ref, limits)?,
            partitions,
            consistency_policy_ref: label(consistency_policy_ref, limits)?,
            evidence_snapshot_ref: label(evidence_snapshot_ref, limits)?,
        })
    }
    /// 目标消费者边界。
    pub const fn boundary_id(self) -> &'a str {
        self.boundary_id
    }
    /// 边界代次。
    pub const fn boundary_epoch(self) -> &'a str {
        self.boundary_epoch
    }
    /// 切片身份。
    pub const fn cut_id(self) -> &'a str {
        self.cut_id
    }
    /// 不可变切片清单引用。
    pub const fn manifest_ref(self) -> &'a str {
        self.manifest_ref
    }
    /// 由外部证据绑定的查询截止点。
    pub const fn known_at(self) -> UnixTimeNs {
        self.known_at
    }
    /// 截止点绑定证明引用。
    pub const fn known_at_binding_ref(self) -> &'a str {
        self.known_at_binding_ref
    }
    /// 具名分区的可读前沿。
    pub const fn partitions(self) -> &'a [CutPartitionRef<'a>] {
        self.partitions
    }
    /// 一致性策略版本引用。
    pub const fn consistency_policy_ref(self) -> &'a str {
        self.consistency_policy_ref
    }
    /// 此历史切片的证据集合引用。
    pub const fn evidence_snapshot_ref(self) -> &'a str {
        self.evidence_snapshot_ref
    }
    /// 判断事件是否属于同一边界、代次、分区及截止点的历史可读集合。
    pub fn contains(
        self,
        event: SystemVisibilityEvidenceRef<'_>,
        boundary: &str,
        known_at: UnixTimeNs,
    ) -> Result<bool, PitError> {
        if boundary != self.boundary_id
            || known_at != self.known_at
            || event.consumer_boundary != self.boundary_id
            || event.log_epoch != self.boundary_epoch
        {
            return Err(PitError::UnprovenSystemCut);
        }
        Ok(self.partitions.iter().any(|part| {
            part.partition == event.partition
                && part.log_epoch == event.log_epoch
                && event.cursor <= part.inclusive_frontier
        }))
    }
}
