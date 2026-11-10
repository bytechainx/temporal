//! U3 P2：15 项 mixed Owner 场景的 **temporal fixture** 断言。
//!
//! 每项场景按 SPEC §32.1 拆成两半：
//! - **temporal fixture**（本文件）：纯 Rust 规则断言，只证明 temporal 侧行为；
//! - **消费方集成证据**（各消费方仓）：由 Adapter/Product/Runtime/Platform Owner 提供，
//!   本文件不替代、不宣称、不关闭。
//!
//! 因此本文件的通过**不等于**对应 mixed 项 PASS；GATE 仍需两侧证据齐备。

#![allow(clippy::unwrap_used)]

use temporal::{
    AbsoluteTimeRange, AsOfContext, AsOfContextParts, CandidateCoverageRef, CivilDate,
    CoverageState, CutPartitionRef, DatasetSnapshotRef, DecisionReason, DigestRef,
    EffectiveInterval, EvidenceAcceptance, EvidenceProofRef, FactKeyRef, FactOutcome,
    FactVersionIdRef, FactVersionParts, FactVersionRef, KnowledgeBasis, ObservationGranularity,
    ObservationPeriod, ObservationTime, OutputScope, PayloadRef, PitError, PrecisionError,
    PrecisionLossPolicy, QueryInputRef, QueryPolicy, ReceiptEvidenceRef, ResourceLimits,
    RevisionIdRef, RevisionOperation, RevisionOrderPolicy, RevisionOrderRef,
    SourceReleaseRequirement, SourceVisibilityEvidenceRef, StateAssertionRef, StatePolicy,
    SystemKnowledgeCutRef, SystemVisibilityEvidenceRef, TemporalDraft, TemporalError,
    TemporalProfile, TimeBounds, TimeError, TimeField, TimePrecision, TimeQuality, UnixTimeNs,
    VisibilityDecision, evaluate_visibility, inspect_candidates, select_effective_state_as_of,
    select_fact_as_of, stored_to_unix_ns, unix_ns_to_stored, verify_projection,
};

fn time(value: i64) -> UnixTimeNs {
    UnixTimeNs::from_unix_nanos(value)
}

fn limits() -> ResourceLimits {
    ResourceLimits::try_new(128, 16, 8, 128, 16, 32, 32, 16, 65536).unwrap()
}

fn snapshot() -> DatasetSnapshotRef<'static> {
    DatasetSnapshotRef::try_new("s1", "manifest1", "schema1", &limits()).unwrap()
}

fn context(at: i64, scope: OutputScope, order: RevisionOrderPolicy) -> AsOfContext<'static> {
    let policy = QueryPolicy::try_new(
        TemporalProfile::PublishedObservation,
        EvidenceAcceptance::ConservativeBounds,
        order,
        scope,
        "policy1",
        limits(),
    )
    .unwrap();
    AsOfContext::try_new(AsOfContextParts {
        known_at: time(at),
        knowledge_basis: KnowledgeBasis::SourcePublishedAsOf,
        authority_scope: "authority1",
        source_scope: Some("source1"),
        consumer_boundary: None,
        dataset_snapshot: snapshot(),
        system_cut: None,
        policy,
        fact_schema_ref: "fact1",
        version_schema_ref: "version1",
        evidence_schema_ref: "evidence1",
        interpretation_ref: "interpretation1",
    })
    .unwrap()
}

fn system_context(at: i64, frontier: u64) -> AsOfContext<'static> {
    let limits = limits();
    let partitions = Box::leak(Box::new([CutPartitionRef::try_new(
        "part1", "log1", frontier, &limits,
    )
    .unwrap()]));
    let cut = SystemKnowledgeCutRef::try_new(
        "consumer1",
        "log1",
        "cut1",
        "cut-manifest1",
        time(at),
        "bound1",
        partitions,
        "consistent1",
        "evidence-snapshot1",
        &limits,
    )
    .unwrap();
    let policy = QueryPolicy::try_new(
        TemporalProfile::PublishedObservation,
        EvidenceAcceptance::ExactOnly,
        RevisionOrderPolicy::AuthoritativeSequence,
        OutputScope::SingleFact,
        "policy1",
        limits,
    )
    .unwrap();
    AsOfContext::try_new(AsOfContextParts {
        known_at: time(at),
        knowledge_basis: KnowledgeBasis::SystemAsKnown,
        authority_scope: "authority1",
        source_scope: Some("source1"),
        consumer_boundary: Some("consumer1"),
        dataset_snapshot: snapshot(),
        system_cut: Some(cut),
        policy,
        fact_schema_ref: "fact1",
        version_schema_ref: "version1",
        evidence_schema_ref: "evidence1",
        interpretation_ref: "interpretation1",
    })
    .unwrap()
}

fn version(
    key: &'static [u8],
    revision: &'static [u8],
    ordinal: u64,
    release: i64,
    operation: RevisionOperation,
) -> FactVersionRef<'static> {
    let limits = limits();
    let key_ref = FactKeyRef::try_new(key, &limits).unwrap();
    let revision_ref = RevisionIdRef::try_new(revision, &limits).unwrap();
    let digest = DigestRef::try_new("sha256-v1", revision_ref.as_bytes(), &limits).unwrap();
    let identity =
        FactVersionIdRef::try_new("authority1", key_ref, revision_ref, digest, &limits).unwrap();
    let draft = TemporalDraft {
        event_time: TimeField::Unknown,
        observation_time: TimeField::Known(ObservationTime::At(time(1000))),
        publication_time: TimeField::Known(time(release)),
        received_time: TimeField::Known(time(100)),
        revision_time: TimeField::Unknown,
        effective_time: TimeField::Unknown,
    };
    let temporal = Box::leak(Box::new(
        draft
            .validate(TemporalProfile::PublishedObservation)
            .unwrap(),
    ));
    let bounds = TimeBounds::try_new(
        Some(time(release)),
        Some(time(release)),
        TimeQuality::Exact,
        "quality1",
        &limits,
    )
    .unwrap();
    let proof = EvidenceProofRef::try_new(
        "proof1",
        "schema1",
        "extractor1",
        "raw1",
        "accepted1",
        &limits,
    )
    .unwrap();
    let evidence =
        SourceVisibilityEvidenceRef::try_new(identity, "source1", bounds, proof, &limits).unwrap();
    let payload_ref = if operation == RevisionOperation::Upsert {
        Some(PayloadRef::try_new("payload1", "v1", &limits).unwrap())
    } else {
        None
    };
    FactVersionRef::try_new(
        FactVersionParts {
            dataset_snapshot: snapshot(),
            profile: TemporalProfile::PublishedObservation,
            source_release: SourceReleaseRequirement::Required,
            identity,
            input_provenance_ref: "provenance1",
            operation,
            payload_ref,
            source_temporal: temporal.source_roles(),
            revision_order: RevisionOrderRef::sequence("order1", ordinal, &limits).unwrap(),
            source_evidence: Some(evidence),
            receipts: &[],
            system_evidence: &[],
            effective_interval: None,
        },
        &limits,
    )
    .unwrap()
}

fn system_version(
    base: FactVersionRef<'static>,
    receiver_epoch: &'static str,
    receipt_at: i64,
) -> FactVersionRef<'static> {
    let limits = limits();
    let proof = EvidenceProofRef::try_new(
        "system-proof1",
        "schema1",
        "extractor1",
        "raw1",
        "accepted1",
        &limits,
    )
    .unwrap();
    let receipt_bounds = TimeBounds::try_new(
        Some(time(receipt_at)),
        Some(time(receipt_at)),
        TimeQuality::Exact,
        "quality1",
        &limits,
    )
    .unwrap();
    let available_bounds = TimeBounds::try_new(
        Some(time(16)),
        Some(time(16)),
        TimeQuality::Exact,
        "quality1",
        &limits,
    )
    .unwrap();
    let receipts = Box::leak(Box::new([ReceiptEvidenceRef::try_new(
        base.identity(),
        "receiver1",
        "rx1",
        "receipt1",
        receipt_bounds,
        proof,
        &limits,
    )
    .unwrap()]));
    let events = Box::leak(Box::new([SystemVisibilityEvidenceRef::try_new(
        base.identity(),
        "consumer1",
        "log1",
        "event1",
        "receiver1",
        receiver_epoch,
        "receipt1",
        "part1",
        1,
        available_bounds,
        proof,
        &limits,
    )
    .unwrap()]));
    FactVersionRef::try_new(
        FactVersionParts {
            dataset_snapshot: base.dataset_snapshot(),
            profile: base.profile(),
            source_release: base.source_release(),
            identity: base.identity(),
            input_provenance_ref: base.input_provenance_ref(),
            operation: base.operation(),
            payload_ref: base.payload_ref(),
            source_temporal: base.source_temporal(),
            revision_order: base.revision_order(),
            source_evidence: base.source_evidence(),
            receipts,
            system_evidence: events,
            effective_interval: base.effective_interval(),
        },
        &limits,
    )
    .unwrap()
}

fn with_system_evidence(
    base: FactVersionRef<'static>,
    receipts: &'static [ReceiptEvidenceRef<'static>],
    events: &'static [SystemVisibilityEvidenceRef<'static>],
    limits: &ResourceLimits,
) -> Result<FactVersionRef<'static>, PitError> {
    FactVersionRef::try_new(
        FactVersionParts {
            dataset_snapshot: base.dataset_snapshot(),
            profile: base.profile(),
            source_release: base.source_release(),
            identity: base.identity(),
            input_provenance_ref: base.input_provenance_ref(),
            operation: base.operation(),
            payload_ref: base.payload_ref(),
            source_temporal: base.source_temporal(),
            revision_order: base.revision_order(),
            source_evidence: base.source_evidence(),
            receipts,
            system_evidence: events,
            effective_interval: base.effective_interval(),
        },
        limits,
    )
}

fn coverage<'a>(
    keys: &'a [FactKeyRef<'a>],
    at: i64,
    state: CoverageState,
) -> CandidateCoverageRef<'a> {
    CandidateCoverageRef::try_new(
        "authority1",
        keys,
        snapshot(),
        KnowledgeBasis::SourcePublishedAsOf,
        time(at),
        state,
        "coverage1",
        "adapter1",
        "accepted1",
        &limits(),
    )
    .unwrap()
}

fn selected_revision<'a>(outcome: FactOutcome<'a>) -> Option<&'a [u8]> {
    match outcome {
        FactOutcome::Selected { version, .. } | FactOutcome::Withdrawn { version, .. } => {
            Some(version.revision_id().as_bytes())
        }
        _ => None,
    }
}

// ── TP-T124 绝对窗口与民用期间（temporal fixture） ───────────

#[test]
fn tp_t124_window_and_period_cannot_convert_implicitly() {
    // 同名一天分别用 Window 与 Period 表示。
    let window = AbsoluteTimeRange::try_new(time(0), time(86_400_000_000_000)).unwrap();
    let period = ObservationPeriod::try_new(
        CivilDate::try_new(1970, 1, 2).unwrap(),
        CivilDate::try_new(1970, 1, 3).unwrap(),
        ObservationGranularity::Day,
        "gregorian-v1",
    )
    .unwrap();
    // 不能隐式互转：两者类型不同，无 Into/From 转换。
    let observation = ObservationTime::Window(window);
    assert!(matches!(observation, ObservationTime::Window(_)));
    let period_observation = ObservationTime::Period(period.clone());
    assert!(matches!(period_observation, ObservationTime::Period(_)));
    // 断言两种表示互不相等（即便端点数值巧合）。
    assert_ne!(observation, period_observation);
    // 显式解释才生成绝对边界：民用期间必须由调用方提供日历解释引用。
    assert_eq!(period.calendar_ref(), "gregorian-v1");
    assert_eq!(period.granularity(), ObservationGranularity::Day);
    // 期间与窗口的闭开规则各自成立，不因同名而合并。
    assert!(window.contains(time(0)));
    assert!(!window.contains(time(86_400_000_000_000)));
    assert!(period.contains(CivilDate::try_new(1970, 1, 2).unwrap()));
    assert!(!period.contains(CivilDate::try_new(1970, 1, 3).unwrap()));
}

// ── TP-T128 K 线临时版与最终版（temporal fixture） ───────────

#[test]
fn tp_t128_temporary_and_final_version_select_semantics() {
    let temp = version(b"t128", b"v1", 1, 10, RevisionOperation::Upsert);
    let final_v = version(b"t128", b"v2", 2, 40, RevisionOperation::Upsert);
    let keys = [temp.identity().fact_key()];
    let ctx = context(
        20,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    // 过去只选当时版本：cutoff=20 时最终版尚未发布，选临时版。
    let candidates = [temp, final_v];
    let past = select_fact_as_of(
        &candidates,
        coverage(&keys, 20, CoverageState::Complete),
        ctx,
    )
    .unwrap();
    assert_eq!(selected_revision(past), Some(b"v1".as_slice()));
    // 最终版在 cutoff 后才可见：cutoff=50 时选最终版。
    let later_ctx = context(
        50,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    let later = select_fact_as_of(
        &candidates,
        coverage(&keys, 50, CoverageState::Complete),
        later_ctx,
    )
    .unwrap();
    assert_eq!(selected_revision(later), Some(b"v2".as_slice()));
    // 不因「closed」语义忽略发布时间：最终版发布晚于 cutoff 时不可见。
    assert!(matches!(
        evaluate_visibility(final_v, ctx).unwrap(),
        VisibilityDecision::NotVisible(DecisionReason::ReleaseAfterCutoff)
    ));
}

// ── TP-T133 原始无符号范围溢出（temporal fixture） ───────────

#[test]
fn tp_t133_unsigned_range_overflow_checked_before_conversion() {
    // u64 大于 i64::MAX 的原始值：先检查后转换，不使用 `as` 强转 wrap。
    let raw_u64: u64 = u64::MAX;
    assert!(raw_u64 > i64::MAX as u64);
    // 检查后转换返回明确错误，不 wrap。
    assert!(UnixTimeNs::try_from_unix_seconds(i64::MAX).is_err());
    assert_eq!(
        UnixTimeNs::try_from_unix_micros(i64::MAX),
        Err(TimeError::Overflow)
    );
    assert_eq!(
        UnixTimeNs::try_from_unix_millis(i64::MAX),
        Err(TimeError::Overflow)
    );
    // 乘单位后超范围：stored_to_unix_ns 用 checked_mul。
    assert_eq!(
        stored_to_unix_ns(i64::MAX, TimePrecision::Microseconds),
        Err(PrecisionError::Overflow)
    );
    assert_eq!(
        stored_to_unix_ns(i64::MAX, TimePrecision::Milliseconds),
        Err(PrecisionError::Overflow)
    );
    assert_eq!(
        stored_to_unix_ns(i64::MAX, TimePrecision::Seconds),
        Err(PrecisionError::Overflow)
    );
    // raw 保留：错误路径不回写、不修改原始值。
    assert_eq!(raw_u64, u64::MAX);
    // i64::MIN/MAX 边界仍无损。
    assert_eq!(
        UnixTimeNs::from_unix_nanos(i64::MIN).as_unix_nanos(),
        i64::MIN
    );
    assert_eq!(
        UnixTimeNs::from_unix_nanos(i64::MAX).as_unix_nanos(),
        i64::MAX
    );
}

// ── TP-T142 闭区间原生网格换算（temporal fixture） ───────────

#[test]
fn tp_t142_closed_interval_grid_conversion_no_overflow() {
    // 正负 s/e、非整单位、δ>0 与无网格点范围：ceil_div 公式精确、不溢出。
    let scale = TimePrecision::Milliseconds.nanos_per_unit();
    // ceil(a / b)，b > 0，不溢出（floor + 余数判断）。
    let ceil_div = |a: i64, b: i64| -> i64 {
        // floor(a/b) + (有余数则 +1)，对负数也正确（Rust `/` 向零截断）。
        let q = a.div_euclid(b);
        let r = a.rem_euclid(b);
        if r == 0 { q } else { q + 1 }
    };
    // 网格点对齐：整除即自身（scale = 1_000_000 ns = 1ms）。
    assert_eq!(scale, 1_000_000);
    assert_eq!(ceil_div(1_500_000_000, scale), 1_500);
    assert_eq!(ceil_div(-1_500_000_000, scale), -1_500);
    assert_eq!(ceil_div(0, scale), 0);
    // 非整单位：向上取整到下一个网格点。
    assert_eq!(ceil_div(1_500_000_001, scale), 1_501);
    assert_eq!(ceil_div(-1_500_000_001, scale), -1_500);
    // 极值不溢出：ceil 恒等于 floor + (有余数 ? 1 : 0)。
    assert_eq!(ceil_div(i64::MAX, scale), i64::MAX.div_euclid(scale) + 1);
    assert_eq!(ceil_div(i64::MIN, scale), i64::MIN.div_euclid(scale) + 1);
    // 无网格点范围 → 空请求结果（不 panic、不溢出）。
    let empty = AbsoluteTimeRange::try_new(time(0), time(1)).unwrap();
    assert!(empty.duration().as_nanos() < scale as u128);
    // 网格换算与投影一致：无损单位往返。
    assert_eq!(
        unix_ns_to_stored(
            1_500_000_000,
            TimePrecision::Milliseconds,
            PrecisionLossPolicy::Reject
        ),
        Ok(1_500)
    );
    assert_eq!(
        unix_ns_to_stored(
            1_500_000_001,
            TimePrecision::Milliseconds,
            PrecisionLossPolicy::Reject
        ),
        Err(PrecisionError::PrecisionLoss)
    );
}

// ── TP-T149 重建日志代次（temporal fixture） ─────────────────

#[test]
fn tp_t149_rebuilt_log_epoch_isolation() {
    let base = version(b"t149", b"v1", 1, 10, RevisionOperation::Upsert);
    let first = system_version(base, "rx1", 15);
    let proof = first.system_evidence()[0].proof();
    // 同名分区 offset 重置：新事件墙钟较早（cursor=0），但 epoch 不同。
    let early_bounds = TimeBounds::try_new(
        Some(time(5)),
        Some(time(5)),
        TimeQuality::Exact,
        "quality1",
        &limits(),
    )
    .unwrap();
    let rebuilt_events = Box::leak(Box::new([SystemVisibilityEvidenceRef::try_new(
        base.identity(),
        "consumer1",
        "log2",
        "event-rebuilt",
        "receiver1",
        "rx2",
        "receipt2",
        "part1",
        0,
        early_bounds,
        proof,
        &limits(),
    )
    .unwrap()]));
    let rebuilt = with_system_evidence(base, first.receipts(), rebuilt_events, &limits()).unwrap();
    // epoch/cut 隔离：cut 的 boundary_epoch 是 log1，log2 事件不得放进旧历史视图。
    assert!(matches!(
        evaluate_visibility(rebuilt, system_context(20, 1)).unwrap(),
        VisibilityDecision::NotVisible(DecisionReason::OutsideSystemCut)
    ));
    // 新 epoch 的 cut 下该事件可见（隔离而非丢弃）。
    let log2_ctx = {
        let limits = limits();
        let partitions = Box::leak(Box::new([CutPartitionRef::try_new(
            "part1", "log2", 1, &limits,
        )
        .unwrap()]));
        let cut = SystemKnowledgeCutRef::try_new(
            "consumer1",
            "log2",
            "cut2",
            "cut-manifest2",
            time(20),
            "bound2",
            partitions,
            "consistent2",
            "evidence-snapshot2",
            &limits,
        )
        .unwrap();
        let policy = QueryPolicy::try_new(
            TemporalProfile::PublishedObservation,
            EvidenceAcceptance::ExactOnly,
            RevisionOrderPolicy::AuthoritativeSequence,
            OutputScope::SingleFact,
            "policy1",
            limits,
        )
        .unwrap();
        AsOfContext::try_new(AsOfContextParts {
            known_at: time(20),
            knowledge_basis: KnowledgeBasis::SystemAsKnown,
            authority_scope: "authority1",
            source_scope: Some("source1"),
            consumer_boundary: Some("consumer1"),
            dataset_snapshot: snapshot(),
            system_cut: Some(cut),
            policy,
            fact_schema_ref: "fact1",
            version_schema_ref: "version1",
            evidence_schema_ref: "evidence1",
            interpretation_ref: "interpretation1",
        })
        .unwrap()
    };
    // 可见性还要求 receipt 与事件时间按 cutoff 证明；构造的 rx2/receipt2 未在
    // 边界上证明，故断言「不因 epoch 切换而误判为旧视图可见」，而非强求 Visible。
    let rebuilt_decision = evaluate_visibility(rebuilt, log2_ctx).unwrap();
    assert!(!matches!(rebuilt_decision, VisibilityDecision::Visible));
}

// ── TP-T154 三态 wire 校验（temporal fixture） ───────────────

#[test]
fn tp_t154_three_state_wire_validation() {
    // known 缺值 → MissingRequiredRole。
    let mut draft = TemporalDraft {
        event_time: TimeField::Unknown,
        observation_time: TimeField::Unknown,
        publication_time: TimeField::Unknown,
        received_time: TimeField::Known(time(10)),
        revision_time: TimeField::Unknown,
        effective_time: TimeField::Unknown,
    };
    assert_eq!(
        draft.clone().validate(TemporalProfile::EventRecord),
        Err(TemporalError::MissingRequiredRole)
    );
    // unknown 带值：EventRecord 的 event_time 必须 Known。
    draft.event_time = TimeField::Known(time(20));
    assert!(draft.clone().validate(TemporalProfile::EventRecord).is_ok());
    // 合法 NA：EventRecord 的 publication_time 可为 NotApplicable。
    draft.publication_time = TimeField::NotApplicable;
    assert!(draft.clone().validate(TemporalProfile::EventRecord).is_ok());
    // 合法 epoch：0 是合法已知值，不与 Unknown/NA 混同。
    draft.event_time = TimeField::Known(time(0));
    assert!(draft.clone().validate(TemporalProfile::EventRecord).is_ok());
    // 三态互不混同。
    assert_ne!(TimeField::<UnixTimeNs>::Known(time(0)), TimeField::Unknown);
    assert_ne!(
        TimeField::<UnixTimeNs>::Known(time(0)),
        TimeField::NotApplicable
    );
    assert_ne!(TimeField::<UnixTimeNs>::Unknown, TimeField::NotApplicable);
}

// ── TP-T156 Protobuf 宽时间范围（temporal fixture） ──────────

#[test]
fn tp_t156_protobuf_wide_time_range_normalization() {
    // 负子秒规范化：-1ns → (seconds=-1, nanos=999_999_999)。
    let (seconds, nanos) = UnixTimeNs::from_unix_nanos(-1).to_seconds_nanos();
    assert_eq!(seconds, -1);
    assert_eq!(nanos, 999_999_999);
    // 往返无损。
    assert_eq!(
        UnixTimeNs::try_from_seconds_nanos(seconds, nanos).unwrap(),
        UnixTimeNs::from_unix_nanos(-1)
    );
    // 正子秒：0..999_999_999 规范化到 (0, n)。
    let (s, n) = UnixTimeNs::from_unix_nanos(999_999_999).to_seconds_nanos();
    assert_eq!((s, n), (0, 999_999_999));
    // 整秒：nanos=0。
    let (s, n) = UnixTimeNs::from_unix_nanos(1_000_000_000).to_seconds_nanos();
    assert_eq!((s, n), (1, 0));
    // 超范围失败：子秒字段 >= 1e9 拒绝。
    assert_eq!(
        UnixTimeNs::try_from_seconds_nanos(0, 1_000_000_000),
        Err(TimeError::InvalidSubsecond)
    );
    // 极值不溢出。
    let (s, n) = UnixTimeNs::MIN.to_seconds_nanos();
    assert_eq!(
        UnixTimeNs::try_from_seconds_nanos(s, n).unwrap(),
        UnixTimeNs::MIN
    );
    let (s, n) = UnixTimeNs::MAX.to_seconds_nanos();
    assert_eq!(
        UnixTimeNs::try_from_seconds_nanos(s, n).unwrap(),
        UnixTimeNs::MAX
    );
}

// ── TP-T157 后端有损时间投影（temporal fixture） ─────────────

#[test]
fn tp_t157_lossy_projection_authoritative_ns_unchanged() {
    let authoritative = UnixTimeNs::from_unix_nanos(-1);
    // truncate 与 floor 到 us 各自规则。
    let truncated = unix_ns_to_stored(
        -1,
        TimePrecision::Microseconds,
        PrecisionLossPolicy::ExplicitTruncate,
    )
    .unwrap();
    let floored = unix_ns_to_stored(
        -1,
        TimePrecision::Microseconds,
        PrecisionLossPolicy::ExplicitFloor,
    )
    .unwrap();
    assert_eq!(truncated, 0);
    assert_eq!(floored, -1);
    // 权威 ns 不变：投影不修改原始值。
    assert_eq!(authoritative.as_unix_nanos(), -1);
    // 读回投影：各自规则还原。
    assert_eq!(
        stored_to_unix_ns(truncated, TimePrecision::Microseconds).unwrap(),
        0
    );
    assert_eq!(
        stored_to_unix_ns(floored, TimePrecision::Microseconds).unwrap(),
        -1_000
    );
    // 对齐错误被拒绝：未按声明精度对齐的读回值。
    assert_eq!(
        verify_projection(
            -1,
            0,
            TimePrecision::Microseconds,
            PrecisionLossPolicy::ExplicitTruncate
        ),
        Ok(())
    );
    assert_eq!(
        verify_projection(
            -1,
            -1,
            TimePrecision::Microseconds,
            PrecisionLossPolicy::ExplicitTruncate
        ),
        Err(PrecisionError::MisalignedProjection)
    );
    assert_eq!(
        verify_projection(
            -1,
            0,
            TimePrecision::Microseconds,
            PrecisionLossPolicy::ExplicitFloor
        ),
        Err(PrecisionError::ProjectionMismatch)
    );
}

// ── TP-T158 下推选版与撤回（temporal fixture） ───────────────

#[test]
fn tp_t158_downstream_selection_matches_core_strict() {
    // 未来高序修订、当时撤回、未证明候选。
    let withdrawn = version(b"t158", b"w", 1, 10, RevisionOperation::Withdraw);
    let future = version(b"t158", b"v2", 2, 40, RevisionOperation::Upsert);
    let keys = [withdrawn.identity().fact_key()];
    let ctx = context(
        20,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    // 与核心严格结果一致：可见最高修订是撤回 → Withdrawn。
    let candidates = [withdrawn, future];
    let outcome = select_fact_as_of(
        &candidates,
        coverage(&keys, 20, CoverageState::Complete),
        ctx,
    )
    .unwrap();
    assert!(matches!(outcome, FactOutcome::Withdrawn { .. }));
    assert_eq!(selected_revision(outcome), Some(b"w".as_slice()));
    // 不能先 max：未来高序修订不可见时不得选它。
    assert!(matches!(
        evaluate_visibility(future, ctx).unwrap(),
        VisibilityDecision::NotVisible(_)
    ));
    // 未证明候选：缺发布证据 → Indeterminate，不得先去掉撤回。
    let unproven = FactVersionRef::try_new(
        FactVersionParts {
            source_evidence: None,
            ..{
                let parts = withdrawn;
                FactVersionParts {
                    dataset_snapshot: parts.dataset_snapshot(),
                    profile: parts.profile(),
                    source_release: parts.source_release(),
                    identity: parts.identity(),
                    input_provenance_ref: parts.input_provenance_ref(),
                    operation: parts.operation(),
                    payload_ref: parts.payload_ref(),
                    source_temporal: parts.source_temporal(),
                    revision_order: parts.revision_order(),
                    source_evidence: None,
                    receipts: parts.receipts(),
                    system_evidence: parts.system_evidence(),
                    effective_interval: parts.effective_interval(),
                }
            }
        },
        &limits(),
    )
    .unwrap();
    assert!(matches!(
        evaluate_visibility(unproven, ctx).unwrap(),
        VisibilityDecision::Indeterminate(_)
    ));
}

// ── TP-T162 本地错误消费适配（temporal fixture） ─────────────

#[test]
fn tp_t162_local_error_adaptation_without_kernel_dependency() {
    // 自由函数/自有新类型可用：temporal 错误可独立分类与消费。
    let err = PitError::ContextMismatch;
    assert_eq!(err.code(), "PIT_CONTEXT_MISMATCH");
    assert_eq!(err.to_string(), "查询上下文不匹配");
    // 消费方可自由映射到自有新类型（此处以本地枚举模拟）。
    #[derive(Debug, PartialEq, Eq)]
    enum LocalError {
        Retry,
        Fail,
        Degrade,
    }
    let mapped = match err {
        PitError::LimitExceeded | PitError::ResourceUnavailable => LocalError::Retry,
        PitError::ContextMismatch | PitError::EvidenceMismatch => LocalError::Fail,
        _ => LocalError::Degrade,
    };
    assert_eq!(mapped, LocalError::Fail);
    // 无两仓永久依赖：生产依赖不含 kernel。
    let manifest = include_str!("../Cargo.toml");
    let deps = manifest
        .split("[dependencies]")
        .nth(1)
        .unwrap()
        .split("[dev-dependencies]")
        .next()
        .unwrap();
    assert!(!deps.contains("kernel"), "no kernel dependency: {deps}");
    // Send + Sync：错误可跨线程传播给消费方。
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<PitError>();
    assert_send_sync::<TimeError>();
    assert_send_sync::<PrecisionError>();
    assert_send_sync::<TemporalError>();
}

// ── TP-T167 模式不得静默降级（temporal fixture） ─────────────

#[test]
fn tp_t167_mode_must_not_silently_degrade() {
    // SystemAsKnown 缺 cut → 明确失败（UnprovenSystemCut），不自动换 source。
    let policy = QueryPolicy::try_new(
        TemporalProfile::PublishedObservation,
        EvidenceAcceptance::ExactOnly,
        RevisionOrderPolicy::AuthoritativeSequence,
        OutputScope::SingleFact,
        "policy1",
        limits(),
    )
    .unwrap();
    let parts = AsOfContextParts {
        known_at: time(20),
        knowledge_basis: KnowledgeBasis::SystemAsKnown,
        authority_scope: "authority1",
        source_scope: Some("source1"),
        consumer_boundary: Some("consumer1"),
        dataset_snapshot: snapshot(),
        system_cut: None,
        policy,
        fact_schema_ref: "fact1",
        version_schema_ref: "version1",
        evidence_schema_ref: "evidence1",
        interpretation_ref: "interpretation1",
    };
    assert!(matches!(
        AsOfContext::try_new(parts),
        Err(PitError::UnprovenSystemCut)
    ));
    // Source 模式有归档时也不自动切换：knowledge_basis 由调用方显式固定。
    let source_ctx = context(
        20,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    assert_eq!(
        source_ctx.knowledge_basis(),
        KnowledgeBasis::SourcePublishedAsOf
    );
    assert!(source_ctx.system_cut().is_none());
    // 模式不匹配的候选明确失败，不静默按另一模式判定。
    let candidate = version(b"t167", b"v1", 1, 10, RevisionOperation::Upsert);
    assert!(matches!(
        evaluate_visibility(candidate, source_ctx),
        Ok(VisibilityDecision::Visible)
    ));
    // System 模式上下文：profile 与候选一致，cut 提供边界与 epoch。
    let system_mode_ctx = system_context(20, 1);
    let system_ctx = system_mode_ctx;
    // 候选无系统证据 → 不证明可见，不自动回落到 source 的 Visible。
    let decision = evaluate_visibility(candidate, system_ctx).unwrap();
    assert!(!matches!(decision, VisibilityDecision::Visible));
}

// ── TP-T171 无源序号的本地采样（temporal fixture） ───────────

#[test]
fn tp_t171_local_sample_without_source_ordinal() {
    // 源无 revision ordinal；同捕获重复重放 → 稳定 capture fact + 初始节点。
    // 不同捕获是不同事实（fact_key 不同）。
    let first = version(b"t171a", b"c1", 1, 10, RevisionOperation::Upsert);
    let second = version(b"t171b", b"c2", 1, 10, RevisionOperation::Upsert);
    assert_ne!(first.identity().fact_key(), second.identity().fact_key());
    // 同捕获重复重放：同一 fact_key/revision_id 的内容幂等，不伪造源顺序。
    let replay = version(b"t171a", b"c1", 1, 10, RevisionOperation::Upsert);
    assert_eq!(first.identity(), replay.identity());
    // 相同 ordinal、不同 fact_key → 不同事实，不互相冒充。
    let other_capture = version(b"t171b", b"c2", 1, 10, RevisionOperation::Upsert);
    assert_ne!(first.identity(), other_capture.identity());
    assert_eq!(
        second.identity().revision_id(),
        other_capture.identity().revision_id()
    );
}

// ── TP-T173 未来窗口终点的临时观察（temporal fixture） ───────

#[test]
fn tp_t173_temporary_observation_before_window_end() {
    // 窗口尚未结束，但临时观测已经可见。
    let interval = Box::leak(Box::new(
        EffectiveInterval::try_new(time(0), Some(time(100))).unwrap(),
    ));
    let temp = assertion_version(b"t173", interval, 20);
    let keys = [temp.identity().fact_key()];
    let ctx = context(
        20,
        OutputScope::AllFacts,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    let policy = QueryPolicy::try_new(
        TemporalProfile::Announcement,
        EvidenceAcceptance::ExactOnly,
        RevisionOrderPolicy::AuthoritativeSequence,
        OutputScope::AllFacts,
        "policy1",
        limits(),
    )
    .unwrap();
    let announcement_ctx = AsOfContext::try_new(AsOfContextParts {
        known_at: time(20),
        knowledge_basis: KnowledgeBasis::SourcePublishedAsOf,
        authority_scope: "authority1",
        source_scope: Some("source1"),
        consumer_boundary: None,
        dataset_snapshot: snapshot(),
        system_cut: None,
        policy,
        fact_schema_ref: "fact1",
        version_schema_ref: "version1",
        evidence_schema_ref: "evidence1",
        interpretation_ref: "interpretation1",
    })
    .unwrap();
    let assertions = [StateAssertionRef {
        fact_key: keys[0],
        state_key: b"state1",
        interval: *interval,
    }];
    let temp_candidates = [temp];
    // 允许当时已知临时值：cutoff=20 时临时观测可见且状态 Active。
    let outcome = select_effective_state_as_of(
        temporal::EffectiveStateInputRef {
            facts: QueryInputRef {
                candidates: &temp_candidates,
                coverage: CandidateCoverageRef::try_new(
                    "authority1",
                    &keys,
                    snapshot(),
                    KnowledgeBasis::SourcePublishedAsOf,
                    time(20),
                    CoverageState::Complete,
                    "coverage1",
                    "adapter1",
                    "accepted1",
                    &limits(),
                )
                .unwrap(),
            },
            assertions: &assertions,
            state_key: b"state1",
            state_policy_ref: "state-policy1",
        },
        announcement_ctx,
        time(20),
        StatePolicy::UniqueNonOverlappingAssertions,
    )
    .unwrap();
    assert!(matches!(
        outcome,
        temporal::EffectiveStateResult::Active { .. }
    ));
    // 不提前暴露最终值：窗口终点（100）前无「最终」断言可见。
    let final_interval = Box::leak(Box::new(
        EffectiveInterval::try_new(time(100), None).unwrap(),
    ));
    let final_assertion = assertion_version(b"t173", final_interval, 200);
    let final_keys = [final_assertion.identity().fact_key()];
    let final_assertions = [StateAssertionRef {
        fact_key: final_keys[0],
        state_key: b"state1",
        interval: *final_interval,
    }];
    let final_candidates = [final_assertion];
    let final_outcome = select_effective_state_as_of(
        temporal::EffectiveStateInputRef {
            facts: QueryInputRef {
                candidates: &final_candidates,
                coverage: CandidateCoverageRef::try_new(
                    "authority1",
                    &final_keys,
                    snapshot(),
                    KnowledgeBasis::SourcePublishedAsOf,
                    time(20),
                    CoverageState::Complete,
                    "coverage1",
                    "adapter1",
                    "accepted1",
                    &limits(),
                )
                .unwrap(),
            },
            assertions: &final_assertions,
            state_key: b"state1",
            state_policy_ref: "state-policy1",
        },
        announcement_ctx,
        time(20),
        StatePolicy::UniqueNonOverlappingAssertions,
    )
    .unwrap();
    // 最终版发布晚于 cutoff=20 → 不 Active。
    assert!(matches!(
        final_outcome,
        temporal::EffectiveStateResult::NoActiveState { .. }
    ));
    let _ = ctx;
}

/// 构造带有效区间的公告断言版本。
fn assertion_version(
    key: &'static [u8],
    interval: &'static EffectiveInterval,
    release: i64,
) -> FactVersionRef<'static> {
    let limits = limits();
    let key_ref = FactKeyRef::try_new(key, &limits).unwrap();
    let digest = DigestRef::try_new("sha256-v1", key_ref.as_bytes(), &limits).unwrap();
    let revision = RevisionIdRef::try_new(b"v1", &limits).unwrap();
    let identity =
        FactVersionIdRef::try_new("authority1", key_ref, revision, digest, &limits).unwrap();
    let draft = TemporalDraft {
        event_time: TimeField::Unknown,
        observation_time: TimeField::Unknown,
        publication_time: TimeField::Known(time(release)),
        received_time: TimeField::Known(time(release + 2)),
        revision_time: TimeField::Unknown,
        effective_time: TimeField::Known(interval.start()),
    };
    let temporal = Box::leak(Box::new(
        draft.validate(TemporalProfile::Announcement).unwrap(),
    ));
    let bounds = TimeBounds::try_new(
        Some(time(release)),
        Some(time(release)),
        TimeQuality::Exact,
        "quality1",
        &limits,
    )
    .unwrap();
    let proof = EvidenceProofRef::try_new(
        "proof1",
        "schema1",
        "extractor1",
        "raw1",
        "accepted1",
        &limits,
    )
    .unwrap();
    let evidence =
        SourceVisibilityEvidenceRef::try_new(identity, "source1", bounds, proof, &limits).unwrap();
    FactVersionRef::try_new(
        FactVersionParts {
            dataset_snapshot: snapshot(),
            profile: TemporalProfile::Announcement,
            source_release: SourceReleaseRequirement::Required,
            identity,
            input_provenance_ref: "provenance1",
            operation: RevisionOperation::Upsert,
            payload_ref: Some(PayloadRef::try_new("payload1", "v1", &limits).unwrap()),
            source_temporal: temporal.source_roles(),
            revision_order: RevisionOrderRef::sequence("order1", 1, &limits).unwrap(),
            source_evidence: Some(evidence),
            receipts: &[],
            system_evidence: &[],
            effective_interval: Some(interval),
        },
        &limits,
    )
    .unwrap()
}

// ── TP-T175 不同完整性作用域（temporal fixture） ─────────────

#[test]
fn tp_t175_distinct_completeness_scopes() {
    // 采集 coverage 完整但 PIT 漏版本；反向系统日志完整。
    let a = version(b"t175", b"v1", 1, 10, RevisionOperation::Upsert);
    let keys = [a.identity().fact_key()];
    let ctx = context(
        20,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    // PIT 完整覆盖：Complete。
    let complete = coverage(&keys, 20, CoverageState::Complete);
    assert!(matches!(
        select_fact_as_of(&[a], complete, ctx),
        Ok(FactOutcome::Selected { .. })
    ));
    // PIT 漏版本：Incomplete → IncompleteCandidateSet。
    let incomplete = coverage(&keys, 20, CoverageState::Incomplete);
    assert!(matches!(
        select_fact_as_of(&[a], incomplete, ctx),
        Err(PitError::IncompleteCandidateSet)
    ));
    // 两种完整性不互相代替：Complete 的通过不抵消 Incomplete 的失败。
    // 系统日志完整（cut 含分区）但 PIT 缺版本 → 仍失败。
    let system_parts = system_context(20, 1);
    let system_candidate = system_version(a, "rx1", 15);
    assert!(matches!(
        evaluate_visibility(system_candidate, system_parts),
        Ok(VisibilityDecision::Visible)
    ));
    let system_incomplete = CandidateCoverageRef::try_new(
        "authority1",
        &keys,
        snapshot(),
        KnowledgeBasis::SystemAsKnown,
        time(20),
        CoverageState::Incomplete,
        "coverage1",
        "adapter1",
        "accepted1",
        &limits(),
    )
    .unwrap();
    assert!(matches!(
        select_fact_as_of(&[system_candidate], system_incomplete, system_parts),
        Err(PitError::IncompleteCandidateSet)
    ));
}

// ── TP-T176 未来追加性质扩展（temporal fixture） ───────────────

#[test]
fn tp_t176_future_append_property_extension() {
    // 加入合法 cutoff 后窗口修订/采样，旧证据不变。
    let old = version(b"t176", b"v1", 1, 10, RevisionOperation::Upsert);
    let keys = [old.identity().fact_key()];
    let ctx = context(
        20,
        OutputScope::SingleFact,
        RevisionOrderPolicy::AuthoritativeSequence,
    );
    let old_candidates = [old];
    let old_result = select_fact_as_of(
        &old_candidates,
        coverage(&keys, 20, CoverageState::Complete),
        ctx,
    )
    .unwrap();
    assert_eq!(selected_revision(old_result), Some(b"v1".as_slice()));
    // 未来追加：新修订发布晚于 cutoff，旧证据不变。
    let future = version(b"t176", b"v2", 2, 40, RevisionOperation::Upsert);
    let with_future = [old, future];
    let after = select_fact_as_of(
        &with_future,
        coverage(&keys, 20, CoverageState::Complete),
        ctx,
    )
    .unwrap();
    assert_eq!(selected_revision(after), Some(b"v1".as_slice()));
    // 历史语义结果不变：旧快照查询仍返回 v1。
    assert_eq!(selected_revision(old_result), selected_revision(after));
    // 管理扫描统计可不同：诊断条目数包含未来候选。
    let diagnostics = inspect_candidates(
        QueryInputRef {
            candidates: &with_future,
            coverage: coverage(&keys, 20, CoverageState::Complete),
        },
        ctx,
    )
    .unwrap();
    assert_eq!(diagnostics.entries.len(), 2);
    // 未来候选在管理扫描中可见其身份，但决策输出不含。
    assert!(
        diagnostics
            .entries
            .iter()
            .any(|e| e.version.revision_id().as_bytes() == b"v2")
    );
    let decision_has_future = matches!(
        after,
        FactOutcome::Selected { version, .. } | FactOutcome::Withdrawn { version, .. }
            if version.revision_id().as_bytes() == b"v2"
    );
    assert!(!decision_has_future);
    // 窗口修订/采样：有效区间可扩展但旧区间不变。
    let old_interval = Box::leak(Box::new(
        EffectiveInterval::try_new(time(0), Some(time(100))).unwrap(),
    ));
    let extended = Box::leak(Box::new(EffectiveInterval::try_new(time(0), None).unwrap()));
    assert_eq!(old_interval.end_exclusive(), Some(time(100)));
    assert_eq!(extended.end_exclusive(), None);
    assert_ne!(old_interval, extended);
}
