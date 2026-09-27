//! 显式查询合同、资源界限及候选覆盖声明。

use crate::error::PitError;
use crate::evidence::{DatasetSnapshotRef, SystemKnowledgeCutRef};
use crate::model::TemporalProfile;
use crate::revision::FactKeyRef;
use crate::unix_time::UnixTimeNs;

fn label(value: &str, max: usize) -> Result<&str, PitError> {
    if value.is_empty() {
        return Err(PitError::ContextMismatch);
    }
    if value.len() > max {
        return Err(PitError::LimitExceeded);
    }
    Ok(value)
}

/// 单次严格查询的显式资源上限。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceLimits {
    max_identity_bytes: usize,
    max_evidence_per_version: usize,
    max_partitions: usize,
    max_candidates: usize,
    max_facts: usize,
    max_versions_per_fact: usize,
    max_chain_depth: usize,
    max_diagnostics: usize,
    max_working_bytes: usize,
}

impl ResourceLimits {
    /// 所有界限必须为正，调用方需在策略中固定其值。
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        max_identity_bytes: usize,
        max_evidence_per_version: usize,
        max_partitions: usize,
        max_candidates: usize,
        max_facts: usize,
        max_versions_per_fact: usize,
        max_chain_depth: usize,
        max_diagnostics: usize,
        max_working_bytes: usize,
    ) -> Result<Self, PitError> {
        if [
            max_identity_bytes,
            max_evidence_per_version,
            max_partitions,
            max_candidates,
            max_facts,
            max_versions_per_fact,
            max_chain_depth,
            max_diagnostics,
            max_working_bytes,
        ]
        .contains(&0)
        {
            return Err(PitError::LimitExceeded);
        }
        Ok(Self {
            max_identity_bytes,
            max_evidence_per_version,
            max_partitions,
            max_candidates,
            max_facts,
            max_versions_per_fact,
            max_chain_depth,
            max_diagnostics,
            max_working_bytes,
        })
    }
    /// 单个身份或引用的最大字节数。
    pub const fn max_identity_bytes(self) -> usize {
        self.max_identity_bytes
    }
    /// 单版本每类证据的最大条数。
    pub const fn max_evidence_per_version(self) -> usize {
        self.max_evidence_per_version
    }
    /// 历史切片最多包含的分区数。
    pub const fn max_partitions(self) -> usize {
        self.max_partitions
    }
    /// 单次查询最多评估的版本数。
    pub const fn max_candidates(self) -> usize {
        self.max_candidates
    }
    /// 单次查询最多评估的完整事实键数。
    pub const fn max_facts(self) -> usize {
        self.max_facts
    }
    /// 单个完整事实最多评估的版本数。
    pub const fn max_versions_per_fact(self) -> usize {
        self.max_versions_per_fact
    }
    /// 单一修订链允许检查的最大深度。
    pub const fn max_chain_depth(self) -> usize {
        self.max_chain_depth
    }
    /// 管理诊断最多包含的条目数。
    pub const fn max_diagnostics(self) -> usize {
        self.max_diagnostics
    }
    /// 单次查询显式允许的工作内存字节数。
    pub const fn max_working_bytes(self) -> usize {
        self.max_working_bytes
    }
}

/// 来源历史知识或真实消费边界的历史知识。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnowledgeBasis {
    /// 来源面向声明受众的历史发布知识。
    SourcePublishedAsOf,
    /// 指定消费边界历史实际可读的知识。
    SystemAsKnown,
}

/// 已证明可用时间的接受方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceAcceptance {
    /// 只接受上下界同一时刻的证明。
    ExactOnly,
    /// 接受真实区间的保守上界。
    ConservativeBounds,
}

/// 查询范围：单事实或完整事实集合。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputScope {
    /// 单个完整事实键。
    SingleFact,
    /// 一个权威作用域内的完整事实集合。
    AllFacts,
}

/// 本版允许的修订权威顺序模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevisionOrderPolicy {
    /// 来源认可的序号顺序。
    AuthoritativeSequence,
    /// 无分叉的单一前驱链。
    LinearSupersedes,
}

/// 已冻结的单次查询策略；严格错误行为不可降级。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueryPolicy<'a> {
    profile: TemporalProfile,
    evidence_acceptance: EvidenceAcceptance,
    revision_order: RevisionOrderPolicy,
    output_scope: OutputScope,
    policy_ref: &'a str,
    limits: ResourceLimits,
}

impl<'a> QueryPolicy<'a> {
    /// 固定 profile、证据模式、顺序模式、输出范围和全部资源上限。
    pub fn try_new(
        profile: TemporalProfile,
        evidence_acceptance: EvidenceAcceptance,
        revision_order: RevisionOrderPolicy,
        output_scope: OutputScope,
        policy_ref: &'a str,
        limits: ResourceLimits,
    ) -> Result<Self, PitError> {
        Ok(Self {
            profile,
            evidence_acceptance,
            revision_order,
            output_scope,
            policy_ref: label(policy_ref, limits.max_identity_bytes())?,
            limits,
        })
    }
    /// 时间资料校验 profile。
    pub const fn profile(self) -> TemporalProfile {
        self.profile
    }
    /// 时间证据接受模式。
    pub const fn evidence_acceptance(self) -> EvidenceAcceptance {
        self.evidence_acceptance
    }
    /// 版本权威顺序模式。
    pub const fn revision_order(self) -> RevisionOrderPolicy {
        self.revision_order
    }
    /// 查询结果范围。
    pub const fn output_scope(self) -> OutputScope {
        self.output_scope
    }
    /// 完整策略内容及版本的稳定引用。
    pub const fn policy_ref(self) -> &'a str {
        self.policy_ref
    }
    /// 资源上限。
    pub const fn limits(self) -> ResourceLimits {
        self.limits
    }
}

/// 构造历史知识查询的全部必要输入。
#[derive(Debug, Clone, Copy)]
pub struct AsOfContextParts<'a> {
    /// 知识截止点。
    pub known_at: UnixTimeNs,
    /// 来源或系统历史知识模式。
    pub knowledge_basis: KnowledgeBasis,
    /// 唯一版本权威作用域。
    pub authority_scope: &'a str,
    /// 来源视图必需的发布受众范围。
    pub source_scope: Option<&'a str>,
    /// 系统视图必需的消费边界。
    pub consumer_boundary: Option<&'a str>,
    /// 不可变输入快照。
    pub dataset_snapshot: DatasetSnapshotRef<'a>,
    /// 系统视图必需的历史知识切片。
    pub system_cut: Option<SystemKnowledgeCutRef<'a>>,
    /// 已冻结的查询策略。
    pub policy: QueryPolicy<'a>,
    /// 完整事实键 schema 版本。
    pub fact_schema_ref: &'a str,
    /// 事实版本身份 schema 版本。
    pub version_schema_ref: &'a str,
    /// 证据 schema 版本。
    pub evidence_schema_ref: &'a str,
    /// 转换、日历与提取解释合同引用。
    pub interpretation_ref: &'a str,
}

/// 不允许隐式切换知识模式或改取最新数据的查询上下文。
#[derive(Debug, Clone, Copy)]
pub struct AsOfContext<'a>(AsOfContextParts<'a>);

impl<'a> AsOfContext<'a> {
    /// 校验模式所需边界、快照、cut 和全部合同引用。
    pub fn try_new(parts: AsOfContextParts<'a>) -> Result<Self, PitError> {
        let max = parts.policy.limits.max_identity_bytes();
        label(parts.authority_scope, max)?;
        label(parts.fact_schema_ref, max)?;
        label(parts.version_schema_ref, max)?;
        label(parts.evidence_schema_ref, max)?;
        label(parts.interpretation_ref, max)?;
        for value in [
            parts.dataset_snapshot.id(),
            parts.dataset_snapshot.manifest_ref(),
            parts.dataset_snapshot.schema_version(),
        ] {
            label(value, max)?;
        }
        match parts.knowledge_basis {
            KnowledgeBasis::SourcePublishedAsOf => {
                label(parts.source_scope.ok_or(PitError::ContextMismatch)?, max)?;
                if parts.consumer_boundary.is_some() || parts.system_cut.is_some() {
                    return Err(PitError::ContextMismatch);
                }
            }
            KnowledgeBasis::SystemAsKnown => {
                let boundary = label(
                    parts.consumer_boundary.ok_or(PitError::ContextMismatch)?,
                    max,
                )?;
                if matches!(
                    parts.policy.profile(),
                    TemporalProfile::PublishedObservation
                        | TemporalProfile::Announcement
                        | TemporalProfile::DerivedRecord
                ) && parts.source_scope.is_none()
                {
                    return Err(PitError::ContextMismatch);
                }
                if let Some(scope) = parts.source_scope {
                    label(scope, max)?;
                }
                let cut = parts.system_cut.ok_or(PitError::UnprovenSystemCut)?;
                if cut.boundary_id() != boundary || cut.known_at() != parts.known_at {
                    return Err(PitError::UnprovenSystemCut);
                }
                if cut.partitions().len() > parts.policy.limits.max_partitions() {
                    return Err(PitError::LimitExceeded);
                }
                for value in [
                    cut.boundary_epoch(),
                    cut.cut_id(),
                    cut.manifest_ref(),
                    cut.known_at_binding_ref(),
                    cut.consistency_policy_ref(),
                    cut.evidence_snapshot_ref(),
                ] {
                    label(value, max)?;
                }
                for partition in cut.partitions() {
                    label(partition.partition(), max)?;
                    label(partition.log_epoch(), max)?;
                }
            }
        }
        Ok(Self(parts))
    }
    /// 知识截止点；包含已证明的相同时刻。
    pub const fn known_at(self) -> UnixTimeNs {
        self.0.known_at
    }
    /// 来源或系统历史知识模式。
    pub const fn knowledge_basis(self) -> KnowledgeBasis {
        self.0.knowledge_basis
    }
    /// 唯一版本权威作用域。
    pub const fn authority_scope(self) -> &'a str {
        self.0.authority_scope
    }
    /// 来源及发布受众作用域。
    pub const fn source_scope(self) -> Option<&'a str> {
        self.0.source_scope
    }
    /// 系统目标消费边界。
    pub const fn consumer_boundary(self) -> Option<&'a str> {
        self.0.consumer_boundary
    }
    /// 固定输入快照。
    pub const fn dataset_snapshot(self) -> DatasetSnapshotRef<'a> {
        self.0.dataset_snapshot
    }
    /// 历史消费知识切片。
    pub const fn system_cut(self) -> Option<SystemKnowledgeCutRef<'a>> {
        self.0.system_cut
    }
    /// 不可变查询策略。
    pub const fn policy(self) -> QueryPolicy<'a> {
        self.0.policy
    }
    /// 产品完整键 schema。
    pub const fn fact_schema_ref(self) -> &'a str {
        self.0.fact_schema_ref
    }
    /// 版本身份 schema。
    pub const fn version_schema_ref(self) -> &'a str {
        self.0.version_schema_ref
    }
    /// 证据 schema。
    pub const fn evidence_schema_ref(self) -> &'a str {
        self.0.evidence_schema_ref
    }
    /// 转换、时区及提取解释合同。
    pub const fn interpretation_ref(self) -> &'a str {
        self.0.interpretation_ref
    }
}

/// 输入候选是否已经由受信适配器完整枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoverageState {
    /// 受信适配器声明并留存证明的完整覆盖。
    Complete,
    /// 尚不能证明候选已完整枚举。
    Incomplete,
}

/// 外部受信枚举器提供的覆盖清单与验收引用。
#[derive(Debug, Clone, Copy)]
pub struct CandidateCoverageRef<'a> {
    authority_scope: &'a str,
    fact_keys: &'a [FactKeyRef<'a>],
    dataset_snapshot: DatasetSnapshotRef<'a>,
    knowledge_basis: KnowledgeBasis,
    known_at: UnixTimeNs,
    state: CoverageState,
    manifest_ref: &'a str,
    adapter_ref: &'a str,
    acceptance_ref: &'a str,
}

impl<'a> CandidateCoverageRef<'a> {
    /// 校验覆盖作用域、键唯一性和外部验收记录引用。
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        authority_scope: &'a str,
        fact_keys: &'a [FactKeyRef<'a>],
        dataset_snapshot: DatasetSnapshotRef<'a>,
        knowledge_basis: KnowledgeBasis,
        known_at: UnixTimeNs,
        state: CoverageState,
        manifest_ref: &'a str,
        adapter_ref: &'a str,
        acceptance_ref: &'a str,
        limits: &ResourceLimits,
    ) -> Result<Self, PitError> {
        if fact_keys.len() > limits.max_facts() {
            return Err(PitError::LimitExceeded);
        }
        let working_bytes = fact_keys
            .len()
            .checked_mul(std::mem::size_of::<&[u8]>())
            .ok_or(PitError::LimitExceeded)?;
        if working_bytes > limits.max_working_bytes() {
            return Err(PitError::LimitExceeded);
        }
        let mut sorted = Vec::new();
        sorted
            .try_reserve_exact(fact_keys.len())
            .map_err(|_| PitError::ResourceUnavailable)?;
        sorted.extend(fact_keys.iter().map(|key| key.as_bytes()));
        sorted.sort_unstable();
        if sorted.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(PitError::IncompleteCandidateSet);
        }
        Ok(Self {
            authority_scope: label(authority_scope, limits.max_identity_bytes())?,
            fact_keys,
            dataset_snapshot,
            knowledge_basis,
            known_at,
            state,
            manifest_ref: label(manifest_ref, limits.max_identity_bytes())?,
            adapter_ref: label(adapter_ref, limits.max_identity_bytes())?,
            acceptance_ref: label(acceptance_ref, limits.max_identity_bytes())?,
        })
    }
    /// 本次枚举的权威作用域。
    pub const fn authority_scope(self) -> &'a str {
        self.authority_scope
    }
    /// 已声明完整的事实键集合。
    pub const fn fact_keys(self) -> &'a [FactKeyRef<'a>] {
        self.fact_keys
    }
    /// 覆盖所依赖的快照。
    pub const fn dataset_snapshot(self) -> DatasetSnapshotRef<'a> {
        self.dataset_snapshot
    }
    /// 覆盖所依赖的知识模式。
    pub const fn knowledge_basis(self) -> KnowledgeBasis {
        self.knowledge_basis
    }
    /// 枚举截止点。
    pub const fn known_at(self) -> UnixTimeNs {
        self.known_at
    }
    /// 完整或未完成状态。
    pub const fn state(self) -> CoverageState {
        self.state
    }
    /// 候选枚举证明清单。
    pub const fn manifest_ref(self) -> &'a str {
        self.manifest_ref
    }
    /// 受信适配器身份与版本。
    pub const fn adapter_ref(self) -> &'a str {
        self.adapter_ref
    }
    /// 适配器验收记录引用。
    pub const fn acceptance_ref(self) -> &'a str {
        self.acceptance_ref
    }
    /// 在每次调用时重新验证覆盖与本次查询绑定。
    pub fn validate_for(self, context: AsOfContext<'_>) -> Result<(), PitError> {
        if self.state != CoverageState::Complete {
            return Err(PitError::IncompleteCandidateSet);
        }
        if self.authority_scope != context.authority_scope()
            || self.dataset_snapshot != context.dataset_snapshot()
            || self.knowledge_basis != context.knowledge_basis()
            || self.known_at != context.known_at()
        {
            return Err(PitError::ContextMismatch);
        }
        Ok(())
    }
}
