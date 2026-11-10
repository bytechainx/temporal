//! TP-T118 零分配基础路径：用全局分配器计数实证单位构造、算术、精度换算与
//! 结构化数学错误路径不产生堆分配、不依赖全局锁。
//!
//! 本文件是独立的测试二进制：`#[global_allocator]` 计数必须独占进程，
//! 放在 `acceptance_align.rs` 里会与并行测试线程互相污染计数窗口。

#![allow(clippy::unwrap_used)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

use static_assertions::assert_not_impl_any;
use std::time::Duration;

use temporal::{
    EffectiveInterval, PitError, PrecisionError, PrecisionLossPolicy, TemporalError, TimeError,
    TimePrecision, UnixTimeNs, unix_ns_to_stored,
};

/// 进程级堆分配/释放计数。
static ALLOC_COUNTER: AtomicUsize = AtomicUsize::new(0);
static DEALLOC_COUNTER: AtomicUsize = AtomicUsize::new(0);

struct CountingAlloc;

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOC_COUNTER.fetch_add(1, Ordering::SeqCst);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        DEALLOC_COUNTER.fetch_add(1, Ordering::SeqCst);
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

/// 清零计数并返回清零前的读数。
fn reset_counters() -> (usize, usize) {
    (
        ALLOC_COUNTER.swap(0, Ordering::SeqCst),
        DEALLOC_COUNTER.swap(0, Ordering::SeqCst),
    )
}

fn counters() -> (usize, usize) {
    (
        ALLOC_COUNTER.load(Ordering::SeqCst),
        DEALLOC_COUNTER.load(Ordering::SeqCst),
    )
}

#[test]
fn tp_t118_zero_alloc_basic_path_stack_only() {
    // 类型性质：基础路径的输入输出都是 Copy 值，不引入堆容器。
    assert_not_impl_any!(UnixTimeNs: Default, From<i64>);

    // 单位构造与算术：零堆分配。
    reset_counters();
    let t = UnixTimeNs::from_unix_nanos(5);
    let added = t.checked_add(Duration::from_secs(1)).unwrap();
    assert_eq!(added.as_unix_nanos(), 1_000_000_005);
    let subtracted = added.checked_sub(Duration::from_secs(1)).unwrap();
    assert_eq!(subtracted.as_unix_nanos(), 5);
    let seconds = UnixTimeNs::try_from_unix_seconds(2).unwrap();
    assert_eq!(seconds.as_unix_nanos(), 2_000_000_000);
    let millis = UnixTimeNs::try_from_unix_millis(3).unwrap();
    assert_eq!(millis.as_unix_nanos(), 3_000_000);
    let micros = UnixTimeNs::try_from_unix_micros(4).unwrap();
    assert_eq!(micros.as_unix_nanos(), 4_000);
    let duration = added.duration_since(t).unwrap();
    assert_eq!(duration, Duration::from_secs(1));
    let (whole, sub) = added.to_seconds_nanos();
    assert_eq!((whole, sub), (1, 5));
    let (allocations, deallocations) = counters();
    assert_eq!(allocations, 0, "单位构造与算术不得堆分配");
    assert_eq!(deallocations, 0, "单位构造与算术不得堆释放");

    // 精度换算：非错误路径零堆分配。
    reset_counters();
    assert_eq!(
        unix_ns_to_stored(
            1_000_000,
            TimePrecision::Microseconds,
            PrecisionLossPolicy::Reject
        )
        .unwrap(),
        1_000
    );
    assert_eq!(
        unix_ns_to_stored(
            1_000_000,
            TimePrecision::Milliseconds,
            PrecisionLossPolicy::Reject
        )
        .unwrap(),
        1
    );
    assert_eq!(
        unix_ns_to_stored(5, TimePrecision::Nanoseconds, PrecisionLossPolicy::Reject).unwrap(),
        5
    );
    assert_eq!(
        unix_ns_to_stored(
            7,
            TimePrecision::Seconds,
            PrecisionLossPolicy::ExplicitTruncate
        )
        .unwrap(),
        0
    );
    assert_eq!(
        unix_ns_to_stored(
            -7,
            TimePrecision::Seconds,
            PrecisionLossPolicy::ExplicitFloor
        )
        .unwrap(),
        -1
    );
    let (allocations, deallocations) = counters();
    assert_eq!(allocations, 0, "精度换算非错误路径不得堆分配");
    assert_eq!(deallocations, 0, "精度换算非错误路径不得堆释放");

    // 结构化数学错误路径：错误分类不依赖堆结构。
    reset_counters();
    assert_eq!(
        unix_ns_to_stored(7, TimePrecision::Seconds, PrecisionLossPolicy::Reject),
        Err(PrecisionError::PrecisionLoss)
    );
    assert_eq!(
        UnixTimeNs::try_from_unix_seconds(i64::MAX),
        Err(TimeError::Overflow)
    );
    assert_eq!(
        UnixTimeNs::try_from_seconds_nanos(1, 1_000_000_000),
        Err(TimeError::InvalidSubsecond)
    );
    assert_eq!(t.duration_since(added), Err(TimeError::InvalidOrder));
    assert!(EffectiveInterval::try_new(time(10), Some(time(10))).is_err());
    assert_eq!(
        EffectiveInterval::try_new(time(10), Some(time(5))),
        Err(TemporalError::InvalidInterval)
    );
    assert_eq!(PitError::ContextMismatch.code(), "PIT_CONTEXT_MISMATCH");
    let (allocations, deallocations) = counters();
    assert_eq!(allocations, 0, "结构化数学错误路径不得堆分配");
    assert_eq!(deallocations, 0, "结构化数学错误路径不得堆释放");

    // SystemTime 转换基础路径：零堆分配。
    // Windows 的 SystemTime 不保留纳秒精度，故只对无损平台断言往返相等；
    // 精度受限平台断言返回 SystemTimePrecisionLoss（错误路径同样零分配）。
    reset_counters();
    let nanos = 1_000_000_005;
    match UnixTimeNs::from_unix_nanos(nanos).try_into_system_time() {
        Ok(system) => {
            assert_eq!(
                UnixTimeNs::try_from_system_time(system)
                    .unwrap()
                    .as_unix_nanos(),
                nanos
            );
        }
        Err(TimeError::SystemTimePrecisionLoss) => {}
        Err(other) => unreachable!("unexpected SystemTime error: {other:?}"),
    }
    let (allocations, deallocations) = counters();
    assert_eq!(allocations, 0, "SystemTime 转换基础路径不得堆分配");
    assert_eq!(deallocations, 0, "SystemTime 转换基础路径不得堆释放");
}

fn time(value: i64) -> UnixTimeNs {
    UnixTimeNs::from_unix_nanos(value)
}
