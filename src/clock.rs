use std::time::{Instant, SystemTime};

use crate::{TimeError, UnixTimeNs};

/// 读取可回退的墙钟时间。
pub trait WallClock: Send + Sync {
    /// 采样当前墙钟时刻。
    fn now(&self) -> Result<UnixTimeNs, TimeError>;
}

/// 读取同一进程内测量间隔的单调时间点。
pub trait MonotonicClock: Send + Sync {
    /// 采样当前单调时间点。
    fn now(&self) -> Instant;
}

/// 同时提供墙钟与单调时间点的对象。
pub trait RuntimeClock: WallClock + MonotonicClock {}

impl<T: WallClock + MonotonicClock + ?Sized> RuntimeClock for T {}

/// 使用标准库系统墙钟的采样器。
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemWallClock;

impl SystemWallClock {
    /// 构造系统墙钟采样器。
    pub const fn new() -> Self {
        Self
    }
}

impl WallClock for SystemWallClock {
    fn now(&self) -> Result<UnixTimeNs, TimeError> {
        UnixTimeNs::try_from_system_time(SystemTime::now())
    }
}

/// 使用标准库单调时钟的采样器。
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemMonotonicClock;

impl SystemMonotonicClock {
    /// 构造系统单调时钟采样器。
    pub const fn new() -> Self {
        Self
    }
}

impl MonotonicClock for SystemMonotonicClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}
