//! 完整候选覆盖下的严格事实修订选择。

use crate::error::PitError;
use crate::evidence::{
    EvidenceProofRef, ReceiptEvidenceRef, SourceVisibilityEvidenceRef, SystemVisibilityEvidenceRef,
    TimeBounds,
};
use crate::pit::context::{AsOfContext, CandidateCoverageRef};
use crate::pit::context::{OutputScope, RevisionOrderPolicy};
use crate::pit::visibility::{VisibilityDecision, evaluate_visibility};
use crate::revision::{
    FactKeyRef, FactVersionIdRef, FactVersionRef, PayloadRef, RevisionOperation,
};
use crate::revision::{RevisionOrderRef, SourceReleaseRequirement};

/// 选中修订的决策安全证据引用。
#[derive(Debug, Clone, Copy)]
pub struct SelectedEvidenceRef<'a> {
    /// 来源实际发布证据。
    pub source: Option<SourceVisibilityEvidenceRef<'a>>,
    /// 历史切片中的接收证据。
    pub receipt: Option<ReceiptEvidenceRef<'a>>,
    /// 历史切片中的消费可读证据。
    pub system: Option<SystemVisibilityEvidenceRef<'a>>,
}

/// 单事实严格查询的结果。
#[derive(Debug, Clone, Copy)]
pub enum FactOutcome<'a> {
    /// 当前可见且未撤回的完整版本。
    Selected {
        /// 选中的完整版本身份。
        version: FactVersionIdRef<'a>,
        /// 已验证为新增或更新。
        operation: RevisionOperation,
        /// 完整不可变载荷引用。
        payload: PayloadRef<'a>,
        /// 本次证明可见的证据。
        evidence: SelectedEvidenceRef<'a>,
    },
    /// 当前可见的最高权威修订是撤回。
    Withdrawn {
        /// 撤回修订身份。
        version: FactVersionIdRef<'a>,
        /// 本次证明撤回可见的证据。
        evidence: SelectedEvidenceRef<'a>,
    },
    /// 完整覆盖内没有可见版本；不暴露未来候选。
    NoVisibleVersion {
        /// 请求的权威作用域。
        authority_scope: &'a str,
        /// 请求的完整事实键。
        fact_key: FactKeyRef<'a>,
    },
}

/// 一次批量查询的完整候选与覆盖证明。
#[derive(Debug, Clone, Copy)]
pub struct QueryInputRef<'a> {
    /// 当前快照中的候选版本。
    pub candidates: &'a [FactVersionRef<'a>],
    /// 受信适配器的完整覆盖声明。
    pub coverage: CandidateCoverageRef<'a>,
}

/// 按完整事实键的规范字节顺序返回的严格结果。
#[derive(Debug, Clone)]
pub struct PitResult<'a> {
    /// 每个已声明事实的确定性结果。
    pub outcomes: Vec<FactOutcome<'a>>,
    /// 决策安全的复现上下文与输入覆盖证明。
    pub manifest: PitManifestRef<'a>,
}

/// 复现本次历史选择所需的只读清单；所选版本和证据位于结果中。
#[derive(Debug, Clone, Copy)]
pub struct PitManifestRef<'a> {
    /// 包含 snapshot、知识模式、截止点、cut、策略和 schema 的完整上下文。
    pub context: AsOfContext<'a>,
    /// 受信枚举器的覆盖清单引用。
    pub coverage_manifest_ref: &'a str,
    /// 受信适配器身份和版本引用。
    pub coverage_adapter_ref: &'a str,
    /// 覆盖验收记录引用。
    pub coverage_acceptance_ref: &'a str,
}

/// 单一完整事实键的严格 PIT 选版。
pub fn select_fact_as_of<'a>(
    candidates: &'a [FactVersionRef<'a>],
    coverage: CandidateCoverageRef<'a>,
    context: AsOfContext<'a>,
) -> Result<FactOutcome<'a>, PitError> {
    if context.policy().output_scope() != OutputScope::SingleFact || coverage.fact_keys().len() != 1
    {
        return Err(PitError::ContextMismatch);
    }
    let result = select_impl(
        QueryInputRef {
            candidates,
            coverage,
        },
        context,
    )?;
    result
        .outcomes
        .into_iter()
        .next()
        .ok_or(PitError::IncompleteCandidateSet)
}

/// 单一权威作用域内所有声明事实的严格 PIT 选版。
pub fn select_facts_as_of<'a>(
    input: QueryInputRef<'a>,
    context: AsOfContext<'a>,
) -> Result<PitResult<'a>, PitError> {
    if context.policy().output_scope() != OutputScope::AllFacts {
        return Err(PitError::ContextMismatch);
    }
    select_impl(input, context)
}

fn select_impl<'a>(
    input: QueryInputRef<'a>,
    context: AsOfContext<'a>,
) -> Result<PitResult<'a>, PitError> {
    input.coverage.validate_for(context)?;
    let limits = context.policy().limits();
    validate_query_limits(input, context)?;
    if input.candidates.len() > limits.max_candidates()
        || input.coverage.fact_keys().len() > limits.max_facts()
    {
        return Err(PitError::LimitExceeded);
    }
    let per_candidate = std::mem::size_of::<FactVersionRef<'a>>()
        .checked_mul(4)
        .and_then(|value| value.checked_add(std::mem::size_of::<u64>()))
        .and_then(|value| value.checked_add(std::mem::size_of::<bool>()))
        .ok_or(PitError::LimitExceeded)?;
    let per_fact = std::mem::size_of::<FactKeyRef<'a>>()
        .checked_add(std::mem::size_of::<FactOutcome<'a>>())
        .ok_or(PitError::LimitExceeded)?;
    let index_bytes = input
        .candidates
        .len()
        .checked_mul(per_candidate)
        .and_then(|value| {
            value.checked_add(input.coverage.fact_keys().len().checked_mul(per_fact)?)
        })
        .ok_or(PitError::LimitExceeded)?;
    let reserved_bytes = index_bytes
        .checked_mul(2)
        .and_then(|value| value.checked_add(1024))
        .ok_or(PitError::LimitExceeded)?;
    if reserved_bytes > limits.max_working_bytes() {
        return Err(PitError::LimitExceeded);
    }
    validate_evidence_identity(input, context, reserved_bytes)?;
    let mut keys = Vec::new();
    keys.try_reserve_exact(input.coverage.fact_keys().len())
        .map_err(|_| PitError::ResourceUnavailable)?;
    keys.extend_from_slice(input.coverage.fact_keys());
    keys.sort_unstable_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
    let mut ordered = Vec::new();
    ordered
        .try_reserve_exact(input.candidates.len())
        .map_err(|_| PitError::ResourceUnavailable)?;
    ordered.extend_from_slice(input.candidates);
    ordered.sort_unstable_by(|left, right| {
        let a = left.identity();
        let b = right.identity();
        a.fact_key()
            .as_bytes()
            .cmp(b.fact_key().as_bytes())
            .then_with(|| a.revision_id().as_bytes().cmp(b.revision_id().as_bytes()))
            .then_with(|| {
                left.source_evidence()
                    .map(|e| e.proof().evidence_id())
                    .cmp(&right.source_evidence().map(|e| e.proof().evidence_id()))
            })
    });
    let mut visible = Vec::new();
    visible
        .try_reserve_exact(input.candidates.len())
        .map_err(|_| PitError::ResourceUnavailable)?;
    let mut uncertain = Vec::new();
    uncertain
        .try_reserve_exact(input.candidates.len())
        .map_err(|_| PitError::ResourceUnavailable)?;
    let mut prior_key = None;
    let mut versions_for_key = 0;
    let mut prior_version = None;
    for (index, &version) in ordered.iter().enumerate() {
        if version.receipts().len() > limits.max_evidence_per_version()
            || version.system_evidence().len() > limits.max_evidence_per_version()
        {
            return Err(PitError::LimitExceeded);
        }
        let identity = version.identity();
        if index > 0
            && revision_sort_key(ordered[index - 1]) == revision_sort_key(version)
            && prior_version.is_some_and(|previous| !same_revision_content(previous, version))
        {
            return Err(PitError::ConflictingRevision);
        }
        prior_version = Some(version);
        if prior_key != Some(identity.fact_key()) {
            prior_key = Some(identity.fact_key());
            versions_for_key = 0;
        }
        versions_for_key += 1;
        if versions_for_key > limits.max_versions_per_fact() {
            return Err(PitError::LimitExceeded);
        }
        if identity.authority_scope() != context.authority_scope()
            || keys
                .binary_search_by(|item| item.as_bytes().cmp(identity.fact_key().as_bytes()))
                .is_err()
        {
            return Err(PitError::ContextMismatch);
        }
        match evaluate_visibility(version, context)? {
            VisibilityDecision::Visible => visible.push(version),
            VisibilityDecision::NotVisible(_) => {}
            VisibilityDecision::Indeterminate(_) => uncertain.push(version),
        }
    }
    let mut visible_index = 0;
    for unknown in uncertain {
        while visible_index < visible.len()
            && revision_sort_key(visible[visible_index]) < revision_sort_key(unknown)
        {
            visible_index += 1;
        }
        if visible_index >= visible.len()
            || revision_sort_key(visible[visible_index]) != revision_sort_key(unknown)
        {
            return Err(PitError::IndeterminateSelection);
        }
        if !same_revision_content(visible[visible_index], unknown) {
            return Err(PitError::ConflictingRevision);
        }
    }
    let mut outcomes = Vec::new();
    outcomes
        .try_reserve_exact(keys.len())
        .map_err(|_| PitError::ResourceUnavailable)?;
    let mut offset = 0;
    for key in keys {
        let begin = offset;
        while offset < visible.len() && visible[offset].identity().fact_key() == key {
            offset += 1;
        }
        outcomes.push(select_group(&visible[begin..offset], key, context)?);
    }
    Ok(PitResult {
        outcomes,
        manifest: PitManifestRef {
            context,
            coverage_manifest_ref: input.coverage.manifest_ref(),
            coverage_adapter_ref: input.coverage.adapter_ref(),
            coverage_acceptance_ref: input.coverage.acceptance_ref(),
        },
    })
}

pub(super) fn validate_query_limits(
    input: QueryInputRef<'_>,
    context: AsOfContext<'_>,
) -> Result<(), PitError> {
    let limits = context.policy().limits();
    let max = limits.max_identity_bytes();
    let check = |value: &[u8]| -> Result<(), PitError> {
        if value.len() > max {
            Err(PitError::LimitExceeded)
        } else {
            Ok(())
        }
    };
    let snapshot = context.dataset_snapshot();
    for value in [
        snapshot.id(),
        snapshot.manifest_ref(),
        snapshot.schema_version(),
    ] {
        check(value.as_bytes())?;
    }
    for value in [
        input.coverage.authority_scope(),
        input.coverage.manifest_ref(),
        input.coverage.adapter_ref(),
        input.coverage.acceptance_ref(),
    ] {
        check(value.as_bytes())?;
    }
    for key in input.coverage.fact_keys() {
        check(key.as_bytes())?;
    }
    if let Some(cut) = context.system_cut() {
        if cut.partitions().len() > limits.max_partitions() {
            return Err(PitError::LimitExceeded);
        }
        for value in [
            cut.boundary_id(),
            cut.boundary_epoch(),
            cut.cut_id(),
            cut.manifest_ref(),
            cut.known_at_binding_ref(),
            cut.consistency_policy_ref(),
            cut.evidence_snapshot_ref(),
        ] {
            check(value.as_bytes())?;
        }
        for part in cut.partitions() {
            check(part.partition().as_bytes())?;
            check(part.log_epoch().as_bytes())?;
        }
    }
    for &version in input.candidates {
        validate_version_limits(version, max, limits.max_evidence_per_version())?;
    }
    Ok(())
}

pub(super) fn validate_version_limits(
    version: FactVersionRef<'_>,
    max: usize,
    max_evidence: usize,
) -> Result<(), PitError> {
    let check = |value: &[u8]| -> Result<(), PitError> {
        if value.len() > max {
            Err(PitError::LimitExceeded)
        } else {
            Ok(())
        }
    };
    if version.receipts().len() > max_evidence || version.system_evidence().len() > max_evidence {
        return Err(PitError::LimitExceeded);
    }
    let id = version.identity();
    let digest = id.semantic_digest();
    for value in [
        id.authority_scope().as_bytes(),
        id.fact_key().as_bytes(),
        id.revision_id().as_bytes(),
        digest.algorithm_version().as_bytes(),
        digest.as_bytes(),
        version.input_provenance_ref().as_bytes(),
    ] {
        check(value)?;
    }
    if let Some(payload) = version.payload_ref() {
        check(payload.identity().as_bytes())?;
        check(payload.version().as_bytes())?;
    }
    match version.source_release() {
        SourceReleaseRequirement::Required => {}
        SourceReleaseRequirement::NotApplicable {
            authority_profile_contract_ref,
        } => check(authority_profile_contract_ref.as_bytes())?,
    }
    match version.revision_order() {
        RevisionOrderRef::AuthoritativeSequence { order_scope, .. } => {
            check(order_scope.as_bytes())?;
        }
        RevisionOrderRef::LinearSupersedes {
            chain_digest,
            predecessor,
        } => {
            check(chain_digest.algorithm_version().as_bytes())?;
            check(chain_digest.as_bytes())?;
            if let Some(previous) = predecessor {
                check(previous.as_bytes())?;
            }
        }
    }
    if let Some(source) = version.source_evidence() {
        check(source.source_scope().as_bytes())?;
        validate_evidence_refs(source.bounds(), source.proof(), max)?;
    }
    for receipt in version.receipts() {
        for value in [
            receipt.receiver_boundary(),
            receipt.receiver_epoch(),
            receipt.receipt_id(),
        ] {
            check(value.as_bytes())?;
        }
        validate_evidence_refs(receipt.received_bounds(), receipt.proof(), max)?;
    }
    for event in version.system_evidence() {
        for value in [
            event.consumer_boundary(),
            event.log_epoch(),
            event.event_id(),
            event.receiver_boundary(),
            event.receiver_epoch(),
            event.receipt_id(),
            event.partition(),
        ] {
            check(value.as_bytes())?;
        }
        validate_evidence_refs(event.bounds(), event.proof(), max)?;
    }
    Ok(())
}

fn validate_evidence_refs(
    bounds: TimeBounds<'_>,
    proof: EvidenceProofRef<'_>,
    max: usize,
) -> Result<(), PitError> {
    for value in [
        bounds.quality_ref(),
        proof.evidence_id(),
        proof.schema_version(),
        proof.extractor_version(),
        proof.raw_evidence_ref(),
        proof.acceptance_ref(),
    ] {
        if value.len() > max {
            return Err(PitError::LimitExceeded);
        }
    }
    Ok(())
}

fn select_group<'a>(
    versions: &[FactVersionRef<'a>],
    key: FactKeyRef<'a>,
    context: AsOfContext<'a>,
) -> Result<FactOutcome<'a>, PitError> {
    if versions.is_empty() {
        return Ok(FactOutcome::NoVisibleVersion {
            authority_scope: context.authority_scope(),
            fact_key: key,
        });
    }
    let mut distinct: Vec<FactVersionRef<'a>> = Vec::new();
    distinct
        .try_reserve_exact(versions.len())
        .map_err(|_| PitError::ResourceUnavailable)?;
    for &version in versions {
        if let Some(&previous) = distinct.last()
            && previous.identity().revision_id() == version.identity().revision_id()
        {
            if !same_revision_content(previous, version) {
                return Err(PitError::ConflictingRevision);
            }
            continue;
        }
        distinct.push(version);
    }
    let chosen = match context.policy().revision_order() {
        RevisionOrderPolicy::AuthoritativeSequence => select_sequence(&distinct)?,
        RevisionOrderPolicy::LinearSupersedes => {
            select_chain(&distinct, context.policy().limits().max_chain_depth())?
        }
    };
    let mut evidence = selected_evidence(chosen, context)?;
    for &version in versions {
        if version.identity() == chosen.identity() {
            let candidate = selected_evidence(version, context)?;
            if earlier_evidence(candidate, evidence, context) {
                evidence = candidate;
            }
        }
    }
    Ok(match chosen.operation() {
        RevisionOperation::Upsert => FactOutcome::Selected {
            version: chosen.identity(),
            operation: RevisionOperation::Upsert,
            payload: chosen.payload_ref().ok_or(PitError::ContextMismatch)?,
            evidence,
        },
        RevisionOperation::Withdraw => FactOutcome::Withdrawn {
            version: chosen.identity(),
            evidence,
        },
    })
}

fn revision_sort_key(version: FactVersionRef<'_>) -> (&[u8], &[u8]) {
    (
        version.identity().fact_key().as_bytes(),
        version.identity().revision_id().as_bytes(),
    )
}

fn same_revision_content(left: FactVersionRef<'_>, right: FactVersionRef<'_>) -> bool {
    left.identity().semantic_digest() == right.identity().semantic_digest()
        && left.operation() == right.operation()
        && left.payload_ref() == right.payload_ref()
        && left.revision_order() == right.revision_order()
        && left.input_provenance_ref() == right.input_provenance_ref()
        && left.profile() == right.profile()
        && left.source_release() == right.source_release()
        && left.effective_interval() == right.effective_interval()
        && left.source_temporal().event_time() == right.source_temporal().event_time()
        && left.source_temporal().observation_time() == right.source_temporal().observation_time()
        && left.source_temporal().publication_time() == right.source_temporal().publication_time()
        && left.source_temporal().revision_time() == right.source_temporal().revision_time()
        && left.source_temporal().effective_time() == right.source_temporal().effective_time()
}

fn validate_evidence_identity(
    input: QueryInputRef<'_>,
    context: AsOfContext<'_>,
    reserved_bytes: usize,
) -> Result<(), PitError> {
    let system = context.system_cut().is_some();
    let (source_count, receipt_count, event_count) = input
        .candidates
        .iter()
        .try_fold(
            (0_usize, 0_usize, 0_usize),
            |(source, receipt, event), version| {
                Some((
                    source.checked_add(usize::from(version.source_evidence().is_some()))?,
                    receipt.checked_add(if system { version.receipts().len() } else { 0 })?,
                    event.checked_add(if system {
                        version.system_evidence().len()
                    } else {
                        0
                    })?,
                ))
            },
        )
        .ok_or(PitError::LimitExceeded)?;
    let scratch_bytes = source_count
        .checked_mul(std::mem::size_of::<(
            FactVersionIdRef<'_>,
            SourceVisibilityEvidenceRef<'_>,
        )>())
        .and_then(|value| {
            value.checked_add(
                receipt_count.checked_mul(std::mem::size_of::<ReceiptEvidenceRef<'_>>())?,
            )
        })
        .and_then(|value| {
            value.checked_add(
                event_count.checked_mul(std::mem::size_of::<SystemVisibilityEvidenceRef<'_>>())?,
            )
        })
        .ok_or(PitError::LimitExceeded)?;
    if reserved_bytes
        .checked_add(
            scratch_bytes
                .checked_mul(2)
                .ok_or(PitError::LimitExceeded)?,
        )
        .is_none_or(|bytes| bytes > context.policy().limits().max_working_bytes())
    {
        return Err(PitError::LimitExceeded);
    }
    let mut sources = Vec::new();
    sources
        .try_reserve_exact(source_count)
        .map_err(|_| PitError::ResourceUnavailable)?;
    let mut receipts = Vec::new();
    receipts
        .try_reserve_exact(receipt_count)
        .map_err(|_| PitError::ResourceUnavailable)?;
    let mut events = Vec::new();
    events
        .try_reserve_exact(event_count)
        .map_err(|_| PitError::ResourceUnavailable)?;
    for version in input.candidates {
        if let Some(source) = version.source_evidence()
            && source
                .bounds()
                .at(context.known_at(), context.policy().evidence_acceptance())
                != crate::evidence::TimeDecision::NotYetVisible
        {
            sources.push((version.identity(), source));
        }
        if let Some(cut) = context.system_cut() {
            receipts.extend_from_slice(version.receipts());
            for &event in version.system_evidence() {
                if event.consumer_boundary() == cut.boundary_id()
                    && event.log_epoch() == cut.boundary_epoch()
                    && cut.contains(event, cut.boundary_id(), context.known_at())?
                {
                    events.push(event);
                }
            }
        }
    }
    sources.sort_unstable_by(|(a_id, a), (b_id, b)| {
        a_id.fact_key()
            .as_bytes()
            .cmp(b_id.fact_key().as_bytes())
            .then_with(|| {
                a_id.revision_id()
                    .as_bytes()
                    .cmp(b_id.revision_id().as_bytes())
            })
            .then_with(|| a.proof().evidence_id().cmp(b.proof().evidence_id()))
    });
    if sources.windows(2).any(|pair| {
        let (a_id, a) = pair[0];
        let (b_id, b) = pair[1];
        a_id.fact_key() == b_id.fact_key()
            && a_id.revision_id() == b_id.revision_id()
            && a.proof().evidence_id() == b.proof().evidence_id()
            && a != b
    }) {
        return Err(PitError::EvidenceMismatch);
    }
    receipts.sort_unstable_by_key(|item| {
        (
            item.receiver_boundary(),
            item.receiver_epoch(),
            item.receipt_id(),
        )
    });
    if receipts.windows(2).any(|pair| {
        let (a, b) = (pair[0], pair[1]);
        a.receiver_boundary() == b.receiver_boundary()
            && a.receiver_epoch() == b.receiver_epoch()
            && a.receipt_id() == b.receipt_id()
            && a != b
    }) {
        return Err(PitError::EvidenceMismatch);
    }
    events
        .sort_unstable_by_key(|item| (item.consumer_boundary(), item.log_epoch(), item.event_id()));
    if events.windows(2).any(|pair| {
        let (a, b) = (pair[0], pair[1]);
        a.consumer_boundary() == b.consumer_boundary()
            && a.log_epoch() == b.log_epoch()
            && a.event_id() == b.event_id()
            && a != b
    }) {
        return Err(PitError::EvidenceMismatch);
    }
    events.sort_unstable_by_key(|item| {
        (
            item.consumer_boundary(),
            item.log_epoch(),
            item.partition(),
            item.cursor(),
        )
    });
    if events.windows(2).any(|pair| {
        let (a, b) = (pair[0], pair[1]);
        a.consumer_boundary() == b.consumer_boundary()
            && a.log_epoch() == b.log_epoch()
            && a.partition() == b.partition()
            && a.cursor() == b.cursor()
            && a != b
    }) {
        return Err(PitError::EvidenceMismatch);
    }
    Ok(())
}

fn earlier_evidence(
    candidate: SelectedEvidenceRef<'_>,
    current: SelectedEvidenceRef<'_>,
    context: AsOfContext<'_>,
) -> bool {
    if context.system_cut().is_some() {
        let Some(a) = candidate.system else {
            return false;
        };
        let Some(b) = current.system else {
            return true;
        };
        return (
            a.bounds().definitely_available_by(),
            a.partition(),
            a.cursor(),
            a.event_id(),
            a.proof().evidence_id(),
        ) < (
            b.bounds().definitely_available_by(),
            b.partition(),
            b.cursor(),
            b.event_id(),
            b.proof().evidence_id(),
        );
    }
    candidate.source.map(|a| a.proof().evidence_id())
        < current.source.map(|b| b.proof().evidence_id())
}

fn select_sequence<'a>(versions: &[FactVersionRef<'a>]) -> Result<FactVersionRef<'a>, PitError> {
    let mut chosen = versions[0];
    let (scope, mut max) = match chosen.revision_order() {
        RevisionOrderRef::AuthoritativeSequence {
            order_scope,
            ordinal,
        } => (order_scope, ordinal),
        _ => return Err(PitError::ContextMismatch),
    };
    for &version in &versions[1..] {
        let (other_scope, ordinal) = match version.revision_order() {
            RevisionOrderRef::AuthoritativeSequence {
                order_scope,
                ordinal,
            } => (order_scope, ordinal),
            _ => return Err(PitError::ContextMismatch),
        };
        if other_scope != scope {
            return Err(PitError::IncomparableOrderScope);
        }
        if ordinal == max {
            return Err(PitError::AmbiguousRevisionOrder);
        }
        if ordinal > max {
            max = ordinal;
            chosen = version;
        }
    }
    // 最大值以外也可能存在两个相同 ordinal；对此进行完整校验。
    let mut ordinals = Vec::new();
    ordinals
        .try_reserve_exact(versions.len())
        .map_err(|_| PitError::ResourceUnavailable)?;
    for version in versions {
        if let RevisionOrderRef::AuthoritativeSequence { ordinal, .. } = version.revision_order() {
            ordinals.push(ordinal);
        }
    }
    ordinals.sort_unstable();
    if ordinals.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(PitError::AmbiguousRevisionOrder);
    }
    Ok(chosen)
}

fn select_chain<'a>(
    versions: &[FactVersionRef<'a>],
    max_depth: usize,
) -> Result<FactVersionRef<'a>, PitError> {
    let mut has_successor = Vec::new();
    has_successor
        .try_reserve_exact(versions.len())
        .map_err(|_| PitError::ResourceUnavailable)?;
    has_successor.resize(versions.len(), false);
    let mut root_count = 0;
    for &version in versions {
        let predecessor = match version.revision_order() {
            RevisionOrderRef::LinearSupersedes { predecessor, .. } => predecessor,
            _ => return Err(PitError::ContextMismatch),
        };
        if let Some(previous) = predecessor {
            let index = versions
                .binary_search_by(|item| {
                    item.identity()
                        .revision_id()
                        .as_bytes()
                        .cmp(previous.as_bytes())
                })
                .map_err(|_| PitError::InvalidRevisionChain)?;
            if has_successor[index] {
                return Err(PitError::InvalidRevisionChain);
            }
            has_successor[index] = true;
        } else {
            root_count += 1;
        }
    }
    if root_count != 1 {
        return Err(PitError::InvalidRevisionChain);
    }
    let mut tip = None;
    for (index, used) in has_successor.iter().enumerate() {
        if !used && tip.replace(index).is_some() {
            return Err(PitError::InvalidRevisionChain);
        }
    }
    let tip = tip.ok_or(PitError::InvalidRevisionChain)?;
    let mut cursor = tip;
    let mut depth = 0;
    loop {
        depth += 1;
        if depth > max_depth {
            return Err(PitError::LimitExceeded);
        }
        match versions[cursor].revision_order() {
            RevisionOrderRef::LinearSupersedes {
                predecessor: Some(id),
                ..
            } => {
                cursor = versions
                    .binary_search_by(|item| {
                        item.identity().revision_id().as_bytes().cmp(id.as_bytes())
                    })
                    .map_err(|_| PitError::InvalidRevisionChain)?;
            }
            RevisionOrderRef::LinearSupersedes {
                predecessor: None, ..
            } => break,
            _ => return Err(PitError::ContextMismatch),
        }
    }
    if depth != versions.len() {
        return Err(PitError::InvalidRevisionChain);
    }
    Ok(versions[tip])
}

fn selected_evidence<'a>(
    version: FactVersionRef<'a>,
    context: AsOfContext<'_>,
) -> Result<SelectedEvidenceRef<'a>, PitError> {
    let mut result = SelectedEvidenceRef {
        source: version.source_evidence(),
        receipt: None,
        system: None,
    };
    if let Some(cut) = context.system_cut() {
        for &event in version.system_evidence() {
            if event.consumer_boundary()
                != context
                    .consumer_boundary()
                    .ok_or(PitError::ContextMismatch)?
            {
                continue;
            }
            if event.log_epoch() != cut.boundary_epoch() {
                continue;
            }
            if !cut.contains(event, event.consumer_boundary(), context.known_at())? {
                continue;
            }
            if event
                .bounds()
                .at(context.known_at(), context.policy().evidence_acceptance())
                != crate::evidence::TimeDecision::ProvenByCutoff
            {
                continue;
            }
            let Some(&receipt) = version.receipts().iter().find(|receipt| {
                receipt.receiver_boundary() == event.receiver_boundary()
                    && receipt.receiver_epoch() == event.receiver_epoch()
                    && receipt.receipt_id() == event.receipt_id()
            }) else {
                continue;
            };
            if receipt
                .received_bounds()
                .at(context.known_at(), context.policy().evidence_acceptance())
                != crate::evidence::TimeDecision::ProvenByCutoff
            {
                continue;
            }
            let better = result.system.is_none_or(|previous| {
                (
                    event.bounds().definitely_available_by(),
                    event.partition(),
                    event.cursor(),
                    event.event_id(),
                    event.proof().evidence_id(),
                ) < (
                    previous.bounds().definitely_available_by(),
                    previous.partition(),
                    previous.cursor(),
                    previous.event_id(),
                    previous.proof().evidence_id(),
                )
            });
            if better {
                result.receipt = Some(receipt);
                result.system = Some(event);
            }
        }
        if result.system.is_none() {
            return Err(PitError::EvidenceMismatch);
        }
    }
    Ok(result)
}
