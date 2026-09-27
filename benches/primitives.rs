//! 本地数值热点采样；正式基线仍须由受控机器冻结。

use std::hint::black_box;
use std::time::{Duration, Instant};

use temporal::{PrecisionLossPolicy, TimePrecision, UnixTimeNs, quantize_unix_ns};

fn measure(mut operation: impl FnMut(), iterations: usize) -> Duration {
    let started = Instant::now();
    for _ in 0..iterations {
        operation();
    }
    started.elapsed()
}

fn main() {
    let iterations = 100_000;
    let time = UnixTimeNs::from_unix_nanos(-1_234_567_890);
    let unit = measure(
        || {
            black_box(UnixTimeNs::from_unix_nanos(black_box(-1_234_567_890)));
        },
        iterations,
    );
    let arithmetic = measure(
        || {
            let _ = black_box(time.checked_add(black_box(Duration::from_nanos(1))));
        },
        iterations,
    );
    let precision = measure(
        || {
            let _ = black_box(quantize_unix_ns(
                black_box(-1_234_567_890),
                TimePrecision::Microseconds,
                PrecisionLossPolicy::ExplicitFloor,
            ));
        },
        iterations,
    );
    println!(
        "候选={} 平台={}/{} 次数={} 构造={unit:?} 算术={arithmetic:?} 精度={precision:?}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH,
        iterations,
    );
}
