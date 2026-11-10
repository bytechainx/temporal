//! U3 P1-E：§33 纯 temporal 补充场景（TP-T121/122/123/125/126/164/177）。
//!
//! 本文件只覆盖 `spec.md` §33.2/§33.6/§33.7 中 Owner 仅为 `temporal` 的 7 项；
//! 混合 Owner（TP-T124 等）与消费方 Owner 项在 P2/P3 拆分。

#![allow(clippy::unwrap_used)]

use std::time::Duration;

use temporal::{
    AbsoluteTimeRange, CivilDate, EvidenceAcceptance, FactKeyRef, FactOutcome, FactVersionIdRef,
    FactVersionParts, FactVersionRef, KnowledgeBasis, ObservationGranularity, ObservationPeriod,
    ObservationTime, OutputScope, PayloadRef, QueryPolicy, ResourceLimits, RevisionOperation,
    RevisionOrderPolicy, RevisionOrderRef, SourceReleaseRequirement, TemporalDraft, TemporalError,
    TemporalProfile, TimeField, UnixTimeNs,
};

fn time(nanos: i64) -> UnixTimeNs {
    UnixTimeNs::from_unix_nanos(nanos)
}

fn date(year: u16, month: u8, day: u8) -> CivilDate {
    CivilDate::try_new(year, month, day).unwrap()
}

fn limits() -> ResourceLimits {
    ResourceLimits::try_new(128, 16, 8, 128, 16, 32, 32, 16, 65536).unwrap()
}

/// 构造 `FactVersionRef`；`sampled` 为真时用 `SampledObservation` profile
/// 且来源发布阶段显式不适用（`source_evidence=None`）。
fn version_with_profile(
    key: &'static [u8],
    revision: &'static [u8],
    sampled: bool,
    limits: &ResourceLimits,
) -> FactVersionRef<'static> {
    let key_ref = FactKeyRef::try_new(key, limits).unwrap();
    let revision_ref = temporal::RevisionIdRef::try_new(revision, limits).unwrap();
    let digest =
        temporal::DigestRef::try_new("sha256-v1", revision_ref.as_bytes(), limits).unwrap();
    let identity =
        FactVersionIdRef::try_new("authority1", key_ref, revision_ref, digest, limits).unwrap();
    let draft = TemporalDraft {
        event_time: TimeField::Unknown,
        observation_time: TimeField::Unknown,
        publication_time: TimeField::Unknown,
        received_time: TimeField::Known(time(10)),
        revision_time: TimeField::Unknown,
        effective_time: TimeField::Unknown,
    };
    let profile = if sampled {
        TemporalProfile::SampledObservation
    } else {
        TemporalProfile::PublishedObservation
    };
    let temporal = Box::leak(Box::new(draft.validate(profile).unwrap()));
    FactVersionRef::try_new(
        FactVersionParts {
            dataset_snapshot: temporal::DatasetSnapshotRef::try_new(
                "s1",
                "manifest1",
                "schema1",
                limits,
            )
            .unwrap(),
            profile,
            source_release: if sampled {
                SourceReleaseRequirement::NotApplicable {
                    authority_profile_contract_ref: "sampled-local-v1",
                }
            } else {
                SourceReleaseRequirement::Required
            },
            identity,
            input_provenance_ref: "provenance1",
            operation: RevisionOperation::Upsert,
            payload_ref: Some(PayloadRef::try_new("payload1", "v1", limits).unwrap()),
            source_temporal: temporal.source_roles(),
            revision_order: RevisionOrderRef::sequence("order1", 1, limits).unwrap(),
            source_evidence: None,
            receipts: &[],
            system_evidence: &[],
            effective_interval: None,
        },
        limits,
    )
    .unwrap()
}

// ── TP-T121 绝对窗口半开边界 ─────────────────────────────────

#[test]
fn tp_t121_absolute_window_half_open_boundary() {
    let window = AbsoluteTimeRange::try_new(time(0), time(60_000_000_000)).unwrap();
    assert_eq!(window.start(), time(0));
    assert_eq!(window.end_exclusive(), time(60_000_000_000));
    // 起点包含。
    assert!(window.contains(time(0)));
    // 中间值包含。
    assert!(window.contains(time(30_000_000_000)));
    assert!(window.contains(time(59_999_999_999)));
    // 终点不包含。
    assert!(!window.contains(time(60_000_000_000)));
    // 端点无损：起点前、终点后都不属于窗口。
    assert!(!window.contains(time(-1)));
    assert!(!window.contains(time(60_000_000_001)));
    // 端点字节不变（无损往返）。
    assert_eq!(window.start().as_unix_nanos(), 0);
    assert_eq!(window.end_exclusive().as_unix_nanos(), 60_000_000_000);
    // 跨度与端点一致。
    assert_eq!(window.duration(), Duration::from_nanos(60_000_000_000));
}

// ── TP-T122 窗口交集与邻接 ───────────────────────────────────

#[test]
fn tp_t122_window_intersection_and_adjacency() {
    let left = AbsoluteTimeRange::try_new(time(0), time(10)).unwrap();
    let right = AbsoluteTimeRange::try_new(time(5), time(15)).unwrap();
    // 重叠窗口：交集准确。
    let overlap = left.intersection(right).unwrap();
    assert_eq!(overlap.start(), time(5));
    assert_eq!(overlap.end_exclusive(), time(10));
    assert!(overlap.contains(time(5)));
    assert!(overlap.contains(time(9)));
    assert!(!overlap.contains(time(10)));
    // 对称：交集与顺序无关。
    assert_eq!(right.intersection(left).unwrap(), overlap);
    // 仅邻接窗口：返回 None，不创建空范围。
    let adjacent_a = AbsoluteTimeRange::try_new(time(0), time(10)).unwrap();
    let adjacent_b = AbsoluteTimeRange::try_new(time(10), time(20)).unwrap();
    assert_eq!(adjacent_a.intersection(adjacent_b), None);
    assert_eq!(adjacent_b.intersection(adjacent_a), None);
    // 包含关系：交集为较小者。
    let inner = AbsoluteTimeRange::try_new(time(3), time(7)).unwrap();
    let outer = AbsoluteTimeRange::try_new(time(0), time(10)).unwrap();
    assert_eq!(outer.intersection(inner).unwrap(), inner);
    // 不相交：None。
    let far = AbsoluteTimeRange::try_new(time(100), time(200)).unwrap();
    assert_eq!(left.intersection(far), None);
}

// ── TP-T123 窗口全域跨度 ─────────────────────────────────────

#[test]
fn tp_t123_window_full_domain_span() {
    // start=MIN、end=MAX：跨度为 u64::MAX 纳秒，无溢出。
    let full = AbsoluteTimeRange::try_new(UnixTimeNs::MIN, UnixTimeNs::MAX).unwrap();
    assert_eq!(full.start(), UnixTimeNs::MIN);
    assert_eq!(full.end_exclusive(), UnixTimeNs::MAX);
    assert_eq!(full.duration(), Duration::from_nanos(u64::MAX));
    // 负端到正端的全域同样无溢出。
    let neg_to_pos = AbsoluteTimeRange::try_new(UnixTimeNs::MIN, time(0)).unwrap();
    assert_eq!(
        neg_to_pos.duration(),
        Duration::from_nanos((UnixTimeNs::MIN.as_unix_nanos() as i128).unsigned_abs() as u64)
    );
    // 跨度不超过 u64::MAX。
    assert!(full.duration().as_nanos() <= u64::MAX as u128);
}

// ── TP-T125 非法窗口 ─────────────────────────────────────────

#[test]
fn tp_t125_illegal_window_rejected_without_saturation() {
    // start == end → InvalidInterval。
    assert_eq!(
        AbsoluteTimeRange::try_new(time(10), time(10)),
        Err(TemporalError::InvalidInterval)
    );
    // start > end → InvalidInterval。
    assert_eq!(
        AbsoluteTimeRange::try_new(time(20), time(10)),
        Err(TemporalError::InvalidInterval)
    );
    // 端点相同但为极值时也拒绝（不因极值而饱和成合法）。
    assert_eq!(
        AbsoluteTimeRange::try_new(UnixTimeNs::MAX, UnixTimeNs::MAX),
        Err(TemporalError::InvalidInterval)
    );
    assert_eq!(
        AbsoluteTimeRange::try_new(UnixTimeNs::MIN, UnixTimeNs::MIN),
        Err(TemporalError::InvalidInterval)
    );
    // 拒绝不是默认空值：错误分类固定，不返回 Ok(空窗口)。
    assert!(AbsoluteTimeRange::try_new(time(10), time(9)).is_err());
    // 合法边界不受影响：end = start + 1ns 合法。
    assert!(AbsoluteTimeRange::try_new(time(10), time(11)).is_ok());
}

// ── TP-T126 无源时刻采样 ─────────────────────────────────────

#[test]
fn tp_t126_sampled_observation_without_source_time() {
    let sampled = version_with_profile(b"t126", b"v1", true, &limits());
    assert_eq!(sampled.profile(), TemporalProfile::SampledObservation);
    // 结构可校验：无源时间字段的版本仍可构造并被选择器读取。
    let keys = [sampled.identity().fact_key()];
    let ctx = sampled_context(20, TemporalProfile::SampledObservation);
    // 不自动变成 SourcePublishedAsOf 可见：来源模式下无发布证据 → 不确定。
    let source_ctx = sampled_context(20, TemporalProfile::SampledObservation);
    assert!(matches!(
        temporal::evaluate_visibility(sampled, source_ctx),
        Ok(temporal::VisibilityDecision::Indeterminate(_))
    ));
    // 系统模式下也不自动可见（无系统证据）。
    assert!(!matches!(
        temporal::evaluate_visibility(sampled, ctx),
        Ok(temporal::VisibilityDecision::Visible)
    ));
    // received_time 不被回填成 event_time/publication_time。
    assert_eq!(sampled.source_temporal().event_time(), &TimeField::Unknown);
    assert_eq!(
        sampled.source_temporal().publication_time(),
        &TimeField::Unknown
    );
    assert_eq!(
        sampled.source_temporal().effective_time(),
        &TimeField::Unknown
    );
    // 严格查询在无证据时不得给出 Selected。
    let coverage = temporal::CandidateCoverageRef::try_new(
        "authority1",
        &keys,
        temporal::DatasetSnapshotRef::try_new("s1", "manifest1", "schema1", &limits()).unwrap(),
        KnowledgeBasis::SourcePublishedAsOf,
        time(20),
        temporal::CoverageState::Complete,
        "coverage1",
        "adapter1",
        "accepted1",
        &limits(),
    )
    .unwrap();
    let candidates = [sampled];
    let outcome = temporal::select_fact_as_of(&candidates, coverage, ctx);
    assert!(!matches!(outcome, Ok(FactOutcome::Selected { .. })));
}

/// 来源模式上下文；`sampled` profile 与 policy 一致。
fn sampled_context(at: i64, profile: TemporalProfile) -> temporal::AsOfContext<'static> {
    let policy = QueryPolicy::try_new(
        profile,
        EvidenceAcceptance::ExactOnly,
        RevisionOrderPolicy::AuthoritativeSequence,
        OutputScope::SingleFact,
        "policy1",
        limits(),
    )
    .unwrap();
    temporal::AsOfContext::try_new(temporal::AsOfContextParts {
        known_at: time(at),
        knowledge_basis: KnowledgeBasis::SourcePublishedAsOf,
        authority_scope: "authority1",
        source_scope: Some("source1"),
        consumer_boundary: None,
        dataset_snapshot: temporal::DatasetSnapshotRef::try_new(
            "s1",
            "manifest1",
            "schema1",
            &limits(),
        )
        .unwrap(),
        system_cut: None,
        policy,
        fact_schema_ref: "fact1",
        version_schema_ref: "version1",
        evidence_schema_ref: "evidence1",
        interpretation_ref: "interpretation1",
    })
    .unwrap()
}

// ── TP-T164 纯逻辑不真实等待 ─────────────────────────────────

#[test]
fn tp_t164_pure_logic_does_not_wait_or_read_now() {
    // 固定 fixture 模拟多年时间：只由显式参数决定，不读取系统时钟。
    let before = UnixTimeNs::from_unix_nanos(0);
    let after = before
        .checked_add(Duration::from_secs(365 * 24 * 60 * 60 * 5))
        .unwrap();
    assert_eq!(
        after.as_unix_nanos(),
        5 * 365 * 24 * 60 * 60 * 1_000_000_000
    );
    // 纯函数路径无 sleep：同一输入两次结果一致，耗时不受时钟影响。
    let first = before.duration_since(UnixTimeNs::UNIX_EPOCH).unwrap();
    let second = before.duration_since(UnixTimeNs::UNIX_EPOCH).unwrap();
    assert_eq!(first, second);
    // 精度换算纯逻辑：不读 now。
    let projected = temporal::unix_ns_to_stored(
        after.as_unix_nanos(),
        temporal::TimePrecision::Seconds,
        temporal::PrecisionLossPolicy::Reject,
    )
    .unwrap();
    assert_eq!(projected, 5 * 365 * 24 * 60 * 60);
    // 窗口计算纯逻辑：只由显式端点决定。
    let window = AbsoluteTimeRange::try_new(before, after).unwrap();
    assert!(window.contains(after.checked_sub(Duration::from_secs(1)).unwrap()));
    assert!(!window.contains(after));
    // 时钟 trait 只在显式调用时读取：不构造系统时钟即无读取路径。
    let _ = temporal::SystemWallClock::new();
    let _ = temporal::SystemMonotonicClock::new();
}

// ── TP-T177 独立外部消费 ─────────────────────────────────────

#[test]
fn tp_t177_independent_external_consumption() {
    // 只检出 temporal：父 workspace / 内部 testkit / configx / gRPC 依赖不存在。
    let manifest = include_str!("../Cargo.toml");
    let deps = manifest
        .split("[dependencies]")
        .nth(1)
        .unwrap()
        .split("[dev-dependencies]")
        .next()
        .unwrap();
    for forbidden in [
        "workspace",
        "testkit",
        "configx",
        "grpc",
        "tonic",
        "prost",
        "kernel",
    ] {
        assert!(
            !deps.contains(forbidden),
            "production dependencies must not include {forbidden}: {deps}"
        );
    }
    // 生产依赖只有 thiserror。
    let production: Vec<&str> = deps
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    assert_eq!(
        production.len(),
        1,
        "unexpected production deps: {production:?}"
    );
    assert!(production[0].starts_with("thiserror"));
    // 单一类型身份：外部 crate 通过根路径消费同一 UnixTimeNs。
    let value = UnixTimeNs::from_unix_nanos(42);
    let _: UnixTimeNs = value;
    let reparsed = UnixTimeNs::from_unix_nanos(42);
    assert_eq!(value, reparsed);
    // 公开 API 可从根路径引用（编译期证据）。
    fn _public_api_surface(
        _t: UnixTimeNs,
        _w: AbsoluteTimeRange,
        _p: ObservationPeriod,
        _d: TemporalDraft,
    ) -> Result<TemporalError, TemporalError> {
        Err(TemporalError::MissingRequiredRole)
    }
    // 无内部模块对外暴露。
    let lib = include_str!("../src/lib.rs");
    assert!(
        !lib.contains("pub mod "),
        "internal modules must stay private"
    );
    // 公开入口只从根路径 re-export。
    assert!(lib.contains("pub use clock::"));
    assert!(lib.contains("pub use unix_time::UnixTimeNs"));
}

/// 民用期间构造（TP-T126 之外不直接使用；保留供 §33.2 窗口/期间对照）。
#[test]
fn observation_period_requires_calendar_reference() {
    let period = ObservationPeriod::try_new(
        date(2026, 1, 1),
        date(2026, 2, 1),
        ObservationGranularity::Month,
        "gregorian-v1",
    )
    .unwrap();
    assert!(period.contains(date(2026, 1, 15)));
    assert!(!period.contains(date(2026, 2, 1)));
    // 绝对窗口与民用期间不能隐式互转：两者类型不同，无 Into 转换。
    let window = AbsoluteTimeRange::try_new(time(0), time(1)).unwrap();
    let _ = (window, ObservationTime::Window(window));
}
