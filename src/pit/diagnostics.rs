//! 仅供显式行政审计读取的候选可见性明细。

use crate::error::PitError;
use crate::pit::context::AsOfContext;
use crate::pit::selection::{QueryInputRef, validate_query_limits};
use crate::pit::visibility::{VisibilityDecision, evaluate_visibility};
use crate::revision::FactVersionIdRef;

/// 单个候选的行政诊断；可能包含历史截止点之后才存在的身份。
#[derive(Debug, Clone, Copy)]
pub struct AdministrativeDiagnosticRef<'a> {
    /// 候选完整身份。
    pub version: FactVersionIdRef<'a>,
    /// 当前知识口径下的独立可见性判断。
    pub visibility: VisibilityDecision,
}

/// 独立于 `FactOutcome` 和 `PitResult` 的行政审计结果。
#[derive(Debug, Clone)]
pub struct AdministrativeDiagnostics<'a> {
    /// 固定的审计查询上下文。
    pub context: AsOfContext<'a>,
    /// 受显式诊断数量界限约束的候选明细。
    pub entries: Vec<AdministrativeDiagnosticRef<'a>>,
}

/// 显式请求候选明细；结果含未来身份，调用方不得送入策略或特征计算。
pub fn inspect_candidates<'a>(
    input: QueryInputRef<'a>,
    context: AsOfContext<'a>,
) -> Result<AdministrativeDiagnostics<'a>, PitError> {
    input.coverage.validate_for(context)?;
    validate_query_limits(input, context)?;
    let limits = context.policy().limits();
    if input.candidates.len() > limits.max_candidates()
        || input.candidates.len() > limits.max_diagnostics()
        || input.coverage.fact_keys().len() > limits.max_facts()
    {
        return Err(PitError::LimitExceeded);
    }
    if input
        .candidates
        .len()
        .checked_mul(std::mem::size_of::<AdministrativeDiagnosticRef<'a>>())
        .is_none_or(|bytes| bytes > limits.max_working_bytes())
    {
        return Err(PitError::LimitExceeded);
    }
    let mut entries = Vec::new();
    entries
        .try_reserve(input.candidates.len())
        .map_err(|_| PitError::ResourceUnavailable)?;
    for &version in input.candidates {
        if version.identity().authority_scope() != context.authority_scope()
            || !input
                .coverage
                .fact_keys()
                .contains(&version.identity().fact_key())
        {
            return Err(PitError::ContextMismatch);
        }
        entries.push(AdministrativeDiagnosticRef {
            version: version.identity(),
            visibility: evaluate_visibility(version, context)?,
        });
    }
    entries.sort_unstable_by(|a, b| {
        a.version
            .fact_key()
            .as_bytes()
            .cmp(b.version.fact_key().as_bytes())
            .then_with(|| {
                a.version
                    .revision_id()
                    .as_bytes()
                    .cmp(b.version.revision_id().as_bytes())
            })
            .then_with(|| decision_key(a.visibility).cmp(&decision_key(b.visibility)))
    });
    Ok(AdministrativeDiagnostics { context, entries })
}

fn decision_key(decision: VisibilityDecision) -> (u8, &'static str) {
    match decision {
        VisibilityDecision::Visible => (0, ""),
        VisibilityDecision::NotVisible(reason) => (1, reason.code()),
        VisibilityDecision::Indeterminate(reason) => (2, reason.code()),
    }
}
