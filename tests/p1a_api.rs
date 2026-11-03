//! P1-A / 22.1：独立性与公开 API 的 10 个场景。
#![allow(clippy::unwrap_used)]

use static_assertions::{assert_impl_all, assert_not_impl_any};
use std::hash::Hash;
use std::mem::size_of;

use temporal::{
    PrecisionLossPolicy, Temporal, TemporalDraft, TemporalProfile, TimeError, TimeField, UnixTimeNs,
};

const MANIFEST: &str = include_str!("../Cargo.toml");
const LIB: &str = include_str!("../src/lib.rs");
const MODEL: &str = include_str!("../src/model.rs");
const ERROR: &str = include_str!("../src/error.rs");
const PRECISION: &str = include_str!("../src/precision.rs");
const API: &str = include_str!("../docs/API.md");

fn dependencies_section() -> &'static str {
    MANIFEST
        .split("[dependencies]")
        .nth(1)
        .unwrap()
        .split("[dev-dependencies]")
        .next()
        .unwrap()
}

#[test]
fn tp_t001_single_crate_cold_start() {
    assert!(MANIFEST.contains("name = \"temporal\""));
    assert!(!MANIFEST.contains("[workspace]"));
    assert!(!MANIFEST.contains("path = \""));
    assert!(!MANIFEST.contains("git = \""));
    assert!(!MANIFEST.contains("testkit"));
}

#[test]
fn tp_t002_production_dependency_closure() {
    let deps: Vec<&str> = dependencies_section()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#') && line.contains('='))
        .collect();
    assert_eq!(deps.len(), 1, "production dependencies: {deps:?}");
    assert!(deps[0].starts_with("thiserror"));
    for forbidden in ["tokio", "reqwest", "serde", "kernel", "tracing", "sqlx"] {
        assert!(!dependencies_section().contains(forbidden), "{forbidden}");
    }
}

#[test]
fn tp_t003_unique_type_identity() {
    assert_eq!(size_of::<UnixTimeNs>(), 8);
    assert_impl_all!(UnixTimeNs: Copy, Send, Sync, Eq, Ord, Hash);
    assert!(std::any::type_name::<UnixTimeNs>().ends_with("UnixTimeNs"));
    assert!(LIB.contains("pub use unix_time::UnixTimeNs;"));
    assert!(!LIB.contains("struct Timestamp"));
}

#[test]
fn tp_t004_no_unitless_conversion() {
    assert_not_impl_any!(UnixTimeNs: From<i64>, Into<i64>, From<u64>, From<usize>);
    assert_eq!(
        UnixTimeNs::from_unix_nanos(7).as_unix_nanos(),
        7,
        "named construction remains available"
    );
}

#[test]
fn tp_t005_unknown_time_has_no_default() {
    assert_not_impl_any!(UnixTimeNs: Default);
    assert_not_impl_any!(Temporal: Default);
    assert_eq!(UnixTimeNs::UNIX_EPOCH.as_unix_nanos(), 0);
}

#[test]
fn tp_t006_serialization_boundary() {
    assert_not_impl_any!(UnixTimeNs: serde::Serialize);
    assert!(!API.contains("Serialize"));
}

#[test]
fn tp_t007_private_fields_and_valid_draft() {
    assert!(MODEL.contains("pub struct Temporal {"));
    let temporal_block = MODEL
        .split("pub struct Temporal {")
        .nth(1)
        .unwrap()
        .split('}')
        .next()
        .unwrap();
    assert!(!temporal_block.contains("pub "));
    let draft = TemporalDraft {
        event_time: TimeField::Known(UnixTimeNs::from_unix_nanos(1)),
        observation_time: TimeField::NotApplicable,
        publication_time: TimeField::Unknown,
        received_time: TimeField::Known(UnixTimeNs::from_unix_nanos(2)),
        revision_time: TimeField::Unknown,
        effective_time: TimeField::Unknown,
    };
    let validated = draft.validate(TemporalProfile::EventRecord).unwrap();
    assert_eq!(validated.received_time().as_unix_nanos(), 2);
}

#[test]
fn tp_t008_internal_modules_are_private() {
    assert!(LIB.contains("mod unix_time;"));
    assert!(!LIB.contains("pub mod unix_time"));
    assert!(LIB.contains("mod pit;"));
    assert!(!LIB.contains("pub mod pit"));
    let _ = UnixTimeNs::from_unix_nanos(0);
}

#[test]
fn tp_t009_non_exhaustive_enums_require_wildcard() {
    assert!(ERROR.contains("#[non_exhaustive]"));
    assert!(PRECISION.contains("#[non_exhaustive]"));
    fn classify(error: TimeError) -> u8 {
        match error {
            TimeError::Overflow => 1,
            _ => 0,
        }
    }
    assert_eq!(classify(TimeError::Overflow), 1);
    let policy = PrecisionLossPolicy::Reject;
    assert_eq!(policy, PrecisionLossPolicy::Reject);
}

#[test]
fn tp_t010_root_export_roster_consistency() {
    for name in [
        "UnixTimeNs",
        "WallClock",
        "MonotonicClock",
        "RuntimeClock",
        "TemporalDraft",
        "PrecisionLossPolicy",
    ] {
        assert!(API.contains(name), "API.md missing {name}");
        assert!(LIB.contains(name), "lib.rs missing {name}");
    }
    let _: UnixTimeNs = UnixTimeNs::UNIX_EPOCH;
    let _ = TemporalProfile::EventRecord;
}
