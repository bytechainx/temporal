//! 不拥有业务键与载荷的事实版本身份。

use crate::error::PitError;
use crate::evidence::{
    DatasetSnapshotRef, ReceiptEvidenceRef, SourceVisibilityEvidenceRef,
    SystemVisibilityEvidenceRef,
};
use crate::model::{SourceTemporalRef, TemporalProfile, TimeField};
use crate::observation::EffectiveInterval;
use crate::pit::context::ResourceLimits;

fn checked_bytes(value: &[u8], limit: usize) -> Result<&[u8], PitError> {
    if value.is_empty() {
        return Err(PitError::ContextMismatch);
    }
    if value.len() > limit {
        return Err(PitError::LimitExceeded);
    }
    Ok(value)
}

fn checked_label(value: &str, limit: usize) -> Result<&str, PitError> {
    checked_bytes(value.as_bytes(), limit)?;
    Ok(value)
}

/// 产品已规范化的完整事实键；本库只按原始字节比较。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FactKeyRef<'a>(&'a [u8]);

impl<'a> FactKeyRef<'a> {
    /// 校验非空及显式资源上限，不修改键字节。
    pub fn try_new(value: &'a [u8], limits: &ResourceLimits) -> Result<Self, PitError> {
        Ok(Self(checked_bytes(value, limits.max_identity_bytes())?))
    }

    /// 返回已规范化的原始字节。
    pub const fn as_bytes(self) -> &'a [u8] {
        self.0
    }
}

/// 权威作用域内的修订身份。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RevisionIdRef<'a>(&'a [u8]);

impl<'a> RevisionIdRef<'a> {
    /// 校验非空及显式资源上限，不修改身份字节。
    pub fn try_new(value: &'a [u8], limits: &ResourceLimits) -> Result<Self, PitError> {
        Ok(Self(checked_bytes(value, limits.max_identity_bytes())?))
    }

    /// 返回已规范化的原始字节。
    pub const fn as_bytes(self) -> &'a [u8] {
        self.0
    }
}

/// 带版本的语义摘要引用；摘要内容由外部受信入口核对。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DigestRef<'a> {
    algorithm_version: &'a str,
    bytes: &'a [u8],
}

impl<'a> DigestRef<'a> {
    /// 校验算法版本及摘要的显式大小界限。
    pub fn try_new(
        algorithm_version: &'a str,
        bytes: &'a [u8],
        limits: &ResourceLimits,
    ) -> Result<Self, PitError> {
        Ok(Self {
            algorithm_version: checked_label(algorithm_version, limits.max_identity_bytes())?,
            bytes: checked_bytes(bytes, limits.max_identity_bytes())?,
        })
    }

    /// 摘要算法及规范化格式的版本。
    pub const fn algorithm_version(self) -> &'a str {
        self.algorithm_version
    }
    /// 摘要字节。
    pub const fn as_bytes(self) -> &'a [u8] {
        self.bytes
    }
}

/// 不由本库解引用的不可变完整载荷位置。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PayloadRef<'a> {
    identity: &'a str,
    version: &'a str,
}

impl<'a> PayloadRef<'a> {
    /// 校验引用身份与版本均已声明。
    pub fn try_new(
        identity: &'a str,
        version: &'a str,
        limits: &ResourceLimits,
    ) -> Result<Self, PitError> {
        Ok(Self {
            identity: checked_label(identity, limits.max_identity_bytes())?,
            version: checked_label(version, limits.max_identity_bytes())?,
        })
    }

    /// 不可变载荷身份。
    pub const fn identity(self) -> &'a str {
        self.identity
    }
    /// 载荷表示版本。
    pub const fn version(self) -> &'a str {
        self.version
    }
}

/// 当前事实版本的操作。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RevisionOperation {
    /// 当前修订提供完整有效值。
    Upsert,
    /// 当前修订撤回此前有效值。
    Withdraw,
}

/// 来源发布证明的明确适用规则。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceReleaseRequirement<'a> {
    /// 必须提供与当前候选绑定的来源发布证明。
    Required,
    /// 产品按版本化权威合同证明独立发布阶段不适用。
    NotApplicable {
        /// 此产品权威及 profile 的版本化适用性合同。
        authority_profile_contract_ref: &'a str,
    },
}

/// 来源或产品权威提供的修订顺序。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevisionOrderRef<'a> {
    /// 同一个 order scope 内的非负来源序号。
    AuthoritativeSequence {
        /// 权威修订序号的比较作用域。
        order_scope: &'a str,
        /// 该作用域内的非负修订序号。
        ordinal: u64,
    },
    /// 显式初始节点或单一前驱；摘要固定权威链关系。
    LinearSupersedes {
        /// 单一直接前驱；初始节点为 None。
        predecessor: Option<RevisionIdRef<'a>>,
        /// 固定完整权威链关系的摘要。
        chain_digest: DigestRef<'a>,
    },
}

impl<'a> RevisionOrderRef<'a> {
    /// 构造有界且非空的顺序作用域。
    pub fn sequence(
        order_scope: &'a str,
        ordinal: u64,
        limits: &ResourceLimits,
    ) -> Result<Self, PitError> {
        Ok(Self::AuthoritativeSequence {
            order_scope: checked_label(order_scope, limits.max_identity_bytes())?,
            ordinal,
        })
    }
}

/// 包含当前语义摘要的完整版本身份。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FactVersionIdRef<'a> {
    authority_scope: &'a str,
    fact_key: FactKeyRef<'a>,
    revision_id: RevisionIdRef<'a>,
    semantic_digest: DigestRef<'a>,
}

impl<'a> FactVersionIdRef<'a> {
    /// 校验当前版本域的权威作用域。
    pub fn try_new(
        authority_scope: &'a str,
        fact_key: FactKeyRef<'a>,
        revision_id: RevisionIdRef<'a>,
        semantic_digest: DigestRef<'a>,
        limits: &ResourceLimits,
    ) -> Result<Self, PitError> {
        Ok(Self {
            authority_scope: checked_label(authority_scope, limits.max_identity_bytes())?,
            fact_key,
            revision_id,
            semantic_digest,
        })
    }

    /// 当前版本权威作用域。
    pub const fn authority_scope(self) -> &'a str {
        self.authority_scope
    }
    /// 完整事实键。
    pub const fn fact_key(self) -> FactKeyRef<'a> {
        self.fact_key
    }
    /// 修订身份。
    pub const fn revision_id(self) -> RevisionIdRef<'a> {
        self.revision_id
    }
    /// 当前规范化版本的语义摘要。
    pub const fn semantic_digest(self) -> DigestRef<'a> {
        self.semantic_digest
    }
}

/// 构造事实版本所需的不可变借用。
#[derive(Debug, Clone, Copy)]
pub struct FactVersionParts<'a> {
    /// 当前版本所在的不可变输入快照。
    pub dataset_snapshot: DatasetSnapshotRef<'a>,
    /// 已验证时间资料的 profile。
    pub profile: TemporalProfile,
    /// 当前候选的来源发布证明适用规则。
    pub source_release: SourceReleaseRequirement<'a>,
    /// 当前事实版本的完整身份与摘要。
    pub identity: FactVersionIdRef<'a>,
    /// 当前版本与原始输入、转换清单的不可变绑定。
    pub input_provenance_ref: &'a str,
    /// 完整值或撤回操作。
    pub operation: RevisionOperation,
    /// 完整值的不可变载荷引用。
    pub payload_ref: Option<PayloadRef<'a>>,
    /// 五个不含接收时刻的固有时间角色。
    pub source_temporal: SourceTemporalRef<'a>,
    /// 版本权威给出的修订顺序。
    pub revision_order: RevisionOrderRef<'a>,
    /// 当前版本的来源发布证据。
    pub source_evidence: Option<SourceVisibilityEvidenceRef<'a>>,
    /// 独立接收事件证据。
    pub receipts: &'a [ReceiptEvidenceRef<'a>],
    /// 独立消费可用事件证据。
    pub system_evidence: &'a [SystemVisibilityEvidenceRef<'a>],
    /// 与有效时间角色一致的可选业务区间。
    pub effective_interval: Option<&'a EffectiveInterval>,
}

/// 已进行结构校验的事实版本借用；证据真实性仍由外部证明。
#[derive(Debug, Clone, Copy)]
pub struct FactVersionRef<'a>(FactVersionParts<'a>);

impl<'a> FactVersionRef<'a> {
    /// 检查身份、载荷、顺序、证据引用及资源界限。
    pub fn try_new(parts: FactVersionParts<'a>, limits: &ResourceLimits) -> Result<Self, PitError> {
        checked_label(parts.input_provenance_ref, limits.max_identity_bytes())?;
        let roles = parts.source_temporal;
        let profile_matches = match parts.profile {
            TemporalProfile::EventRecord => matches!(roles.event_time(), TimeField::Known(_)),
            TemporalProfile::PublishedObservation => {
                matches!(roles.observation_time(), TimeField::Known(_))
                    && !matches!(roles.publication_time(), TimeField::NotApplicable)
            }
            TemporalProfile::Announcement => {
                matches!(roles.effective_time(), TimeField::Known(_))
                    && !matches!(roles.publication_time(), TimeField::NotApplicable)
            }
            TemporalProfile::DerivedRecord => {
                matches!(roles.event_time(), TimeField::Known(_))
                    || matches!(roles.observation_time(), TimeField::Known(_))
            }
            TemporalProfile::SampledObservation => true,
        };
        if !profile_matches {
            return Err(PitError::ContextMismatch);
        }
        if let Some(interval) = parts.effective_interval
            && !matches!(roles.effective_time(), TimeField::Known(time) if *time == interval.start())
        {
            return Err(PitError::ContextMismatch);
        }
        if let SourceReleaseRequirement::NotApplicable {
            authority_profile_contract_ref,
        } = parts.source_release
        {
            checked_label(authority_profile_contract_ref, limits.max_identity_bytes())?;
            if !matches!(
                parts.profile,
                TemporalProfile::EventRecord | TemporalProfile::SampledObservation
            ) || parts.source_evidence.is_some()
            {
                return Err(PitError::EvidenceMismatch);
            }
        }
        match (parts.operation, parts.payload_ref) {
            (RevisionOperation::Upsert, Some(_)) | (RevisionOperation::Withdraw, None) => {}
            _ => return Err(PitError::ContextMismatch),
        }
        if parts.receipts.len() > limits.max_evidence_per_version()
            || parts.system_evidence.len() > limits.max_evidence_per_version()
        {
            return Err(PitError::LimitExceeded);
        }
        if let RevisionOrderRef::LinearSupersedes {
            predecessor: Some(previous),
            ..
        } = parts.revision_order
            && previous == parts.identity.revision_id()
        {
            return Err(PitError::InvalidRevisionChain);
        }
        if let RevisionOrderRef::AuthoritativeSequence { order_scope, .. } = parts.revision_order {
            checked_label(order_scope, limits.max_identity_bytes())?;
        }
        if let Some(evidence) = parts.source_evidence
            && evidence.version() != parts.identity
        {
            return Err(PitError::EvidenceMismatch);
        }
        for receipt in parts.receipts {
            if receipt.version() != parts.identity {
                return Err(PitError::EvidenceMismatch);
            }
        }
        for (index, receipt) in parts.receipts.iter().enumerate() {
            if parts.receipts[..index].iter().any(|other| {
                other.receiver_boundary() == receipt.receiver_boundary()
                    && other.receiver_epoch() == receipt.receiver_epoch()
                    && other.receipt_id() == receipt.receipt_id()
                    && other != receipt
            }) {
                return Err(PitError::EvidenceMismatch);
            }
        }
        for evidence in parts.system_evidence {
            if evidence.version() != parts.identity {
                return Err(PitError::EvidenceMismatch);
            }
        }
        for (index, event) in parts.system_evidence.iter().enumerate() {
            if parts.system_evidence[..index].iter().any(|other| {
                other.consumer_boundary() == event.consumer_boundary()
                    && other.log_epoch() == event.log_epoch()
                    && ((other.event_id() == event.event_id())
                        || (other.partition() == event.partition()
                            && other.cursor() == event.cursor()))
                    && other != event
            }) {
                return Err(PitError::EvidenceMismatch);
            }
        }
        Ok(Self(parts))
    }

    /// 不可变版本身份。
    pub const fn identity(self) -> FactVersionIdRef<'a> {
        self.0.identity
    }
    /// 当前候选所属的不可变快照。
    pub const fn dataset_snapshot(self) -> DatasetSnapshotRef<'a> {
        self.0.dataset_snapshot
    }
    /// 当前候选的时间资料 profile。
    pub const fn profile(self) -> TemporalProfile {
        self.0.profile
    }
    /// 独立来源发布证明的适用规则。
    pub const fn source_release(self) -> SourceReleaseRequirement<'a> {
        self.0.source_release
    }
    /// 来源与转换溯源引用。
    pub const fn input_provenance_ref(self) -> &'a str {
        self.0.input_provenance_ref
    }
    /// 操作类型。
    pub const fn operation(self) -> RevisionOperation {
        self.0.operation
    }
    /// 完整载荷引用。
    pub const fn payload_ref(self) -> Option<PayloadRef<'a>> {
        self.0.payload_ref
    }
    /// 五个固有时间角色。
    pub const fn source_temporal(self) -> SourceTemporalRef<'a> {
        self.0.source_temporal
    }
    /// 修订顺序关系。
    pub const fn revision_order(self) -> RevisionOrderRef<'a> {
        self.0.revision_order
    }
    /// 来源可见性证据。
    pub const fn source_evidence(self) -> Option<SourceVisibilityEvidenceRef<'a>> {
        self.0.source_evidence
    }
    /// 接收事件证据集合。
    pub const fn receipts(self) -> &'a [ReceiptEvidenceRef<'a>] {
        self.0.receipts
    }
    /// 消费可用性证据集合。
    pub const fn system_evidence(self) -> &'a [SystemVisibilityEvidenceRef<'a>] {
        self.0.system_evidence
    }
    /// 可选业务生效区间。
    pub const fn effective_interval(self) -> Option<&'a EffectiveInterval> {
        self.0.effective_interval
    }
}
