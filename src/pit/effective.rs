//! 先选可见修订、后按业务有效区间求状态。

use crate::error::PitError;
use crate::observation::EffectiveInterval;
use crate::pit::context::AsOfContext;
use crate::pit::context::OutputScope;
use crate::pit::selection::{FactOutcome, QueryInputRef, select_facts_as_of};
use crate::revision::FactKeyRef;
use crate::unix_time::UnixTimeNs;

/// 同一状态键在一个时刻最多有一个存活断言。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatePolicy {
    /// 同一状态键与业务时刻只允许一个存活断言。
    UniqueNonOverlappingAssertions,
}

/// 产品拥有的事实键到状态键、有效区间映射。
#[derive(Debug, Clone, Copy)]
pub struct StateAssertionRef<'a> {
    /// 完整事实键。
    pub fact_key: FactKeyRef<'a>,
    /// 产品定义的状态键。
    pub state_key: &'a [u8],
    /// 当前断言的有效区间。
    pub interval: EffectiveInterval,
}

/// 完整 PIT 输入与状态断言映射。
#[derive(Debug, Clone, Copy)]
pub struct EffectiveStateInputRef<'a> {
    /// 事实候选与完整覆盖。
    pub facts: QueryInputRef<'a>,
    /// 产品声明的状态断言。
    pub assertions: &'a [StateAssertionRef<'a>],
    /// 当前查询的产品状态键。
    pub state_key: &'a [u8],
    /// 状态策略的不可变版本引用。
    pub state_policy_ref: &'a str,
}

/// 已知时刻与业务有效时刻双维度的状态结果。
#[allow(clippy::large_enum_variant)] // 保留无堆分配的决策安全结果；输入规模由策略界限约束。
#[derive(Debug, Clone, Copy)]
pub enum EffectiveStateResult<'a> {
    /// 当前时刻没有已知且有效的状态断言。
    NoActiveState {
        /// 历史知识截止点。
        known_at: UnixTimeNs,
        /// 业务有效时刻。
        valid_at: UnixTimeNs,
        /// 请求的产品状态键。
        state_key: &'a [u8],
        /// 状态策略版本引用。
        state_policy_ref: &'a str,
    },
    /// 唯一已知且有效的状态断言。
    Active {
        /// 历史知识截止点。
        known_at: UnixTimeNs,
        /// 业务有效时刻。
        valid_at: UnixTimeNs,
        /// 请求的产品状态键。
        state_key: &'a [u8],
        /// 状态策略版本引用。
        state_policy_ref: &'a str,
        /// 被选中的存活断言。
        outcome: FactOutcome<'a>,
    },
}

/// 在严格 PIT 选择后计算给定业务时刻的状态。
pub fn select_effective_state_as_of<'a>(
    input: EffectiveStateInputRef<'a>,
    context: AsOfContext<'a>,
    valid_at: UnixTimeNs,
    state_policy: StatePolicy,
) -> Result<EffectiveStateResult<'a>, PitError> {
    let limits = context.policy().limits();
    if context.policy().output_scope() != OutputScope::AllFacts
        || input.state_key.is_empty()
        || input.state_policy_ref.is_empty()
    {
        return Err(PitError::ContextMismatch);
    }
    if input.state_key.len() > limits.max_identity_bytes()
        || input.state_policy_ref.len() > limits.max_identity_bytes()
        || input.assertions.len() > limits.max_facts()
    {
        return Err(PitError::LimitExceeded);
    }
    if state_policy != StatePolicy::UniqueNonOverlappingAssertions {
        return Err(PitError::ContextMismatch);
    }
    for (index, assertion) in input.assertions.iter().enumerate() {
        if assertion.state_key.is_empty() || assertion.state_key.len() > limits.max_identity_bytes()
        {
            return Err(PitError::ContextMismatch);
        }
        if !input
            .facts
            .coverage
            .fact_keys()
            .contains(&assertion.fact_key)
            || input.assertions[..index]
                .iter()
                .any(|item| item.fact_key == assertion.fact_key)
        {
            return Err(PitError::ContextMismatch);
        }
    }
    let facts = select_facts_as_of(input.facts, context)?;
    let mut active = None;
    for outcome in facts.outcomes {
        let FactOutcome::Selected { version, .. } = outcome else {
            continue;
        };
        let assertion = input
            .assertions
            .iter()
            .find(|item| item.fact_key == version.fact_key())
            .ok_or(PitError::ContextMismatch)?;
        if assertion.state_key != input.state_key {
            continue;
        }
        let matching = input
            .facts
            .candidates
            .iter()
            .find(|candidate| candidate.identity() == version)
            .ok_or(PitError::IncompleteCandidateSet)?;
        if matching.effective_interval().copied() != Some(assertion.interval) {
            return Err(PitError::ContextMismatch);
        }
        if assertion.interval.contains(valid_at) {
            if active.is_some() {
                return Err(PitError::ConflictingEffectiveState);
            }
            active = Some(outcome);
        }
    }
    Ok(match active {
        Some(outcome) => EffectiveStateResult::Active {
            known_at: context.known_at(),
            valid_at,
            state_key: input.state_key,
            state_policy_ref: input.state_policy_ref,
            outcome,
        },
        None => EffectiveStateResult::NoActiveState {
            known_at: context.known_at(),
            valid_at,
            state_key: input.state_key,
            state_policy_ref: input.state_policy_ref,
        },
    })
}
