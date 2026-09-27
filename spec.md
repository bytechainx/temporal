# temporal 完整设计与验收规格（SPEC）

| 元数据 | 内容 |
| --- | --- |
| 文档 ID | XH-TEMPORAL-SPEC-001 |
| 文档版本 | 0.1.0 |
| 整理日期 | 2026-09-27 |
| 状态 | 历史整合版评审候选；新 API 与接入规则均为目标合同，无实现、编译或生产验收声明 |
| 对应目标 | [goal.md](goal.md)，XH-TEMPORAL-GOAL-001 v0.1.0 |
| 模块 | 独立 Rust crate `temporal`；建议归属 `bytechainx/temporal` |
| 历史迁移基线 | kernel `0c895cbf8ce22f5435dd93d5f53eb03eeb57eb2f`，package 0.4.2 |
| 技术基线 | Edition 2024；MSRV 1.88；std；生产依赖初始仅 thiserror 2 |
| 消费方式 | 审核后的固定 Git commit；`publish = false` |
| 规范 Owner | temporal 模块维护者；具体审批身份由 CODEOWNERS 登记 |

> 本模块统一时间表示、六种时间角色、不可变修订的历史可见性和选版规则。它不保存数据、不连接数据库、不提供时间服务器，也不替代消费方的来源真实性、数据完整性、权限或业务契约。

## 目录

| 章节 | 内容 |
| --- | --- |
| 1 | [规范效力、阅读方式与术语](#s01) |
| 2 | [独立仓库与依赖边界](#s02) |
| 3 | [基础时间的唯一性与系统边界](#s03) |
| 4 | [UnixTimeNs 与算术合同](#s04) |
| 5 | [时钟接口与测试控制](#s05) |
| 6 | [民用日期、观测窗口、观测期间与有效区间](#s06) |
| 7 | [六角色模型、Draft 和 profiles](#s07) |
| 8 | [时间精度、舍入与存储能力](#s08) |
| 9 | [事实、来源修订、接收与可消费事件](#s09) |
| 10 | [可见性证据与时间不确定性](#s10) |
| 11 | [快照、历史知识切片与分布式顺序](#s11) |
| 12 | [查询上下文、策略与输入覆盖](#s12) |
| 13 | [可见性判定算法](#s13) |
| 14 | [PIT 选版、撤回与冲突处理](#s14) |
| 15 | [已知事实与有效状态的不同查询合同](#s15) |
| 16 | [结果、解释、缓存与复现清单](#s16) |
| 17 | [派生数据、特征、模型与模拟](#s17) |
| 18 | [必须冻结的不变量](#s18) |
| 19 | [错误分类与公共失败合同](#s19) |
| 20 | [公开 API 名册与输入模型](#s20) |
| 21 | [并发、资源、性能与安全边界](#s21) |
| 22 | [必须实现的验收测试矩阵](#s22) |
| 23 | [构建、测试与 CI 门禁](#s23) |
| 24 | [从 kernel 迁移的实施合同](#s24) |
| 25 | [交付证据、版本政策与完成定义](#s25) |
| 26 | [目标追踪与实施工作包](#s26) |
| 27 | [历史约束对账与统一术语](#s27) |
| 28 | [时间尺度、原始字段与映射溯源](#s28) |
| 29 | [Market/Macro 接入、窗口与采样](#s29) |
| 30 | [历史补齐、重放、水位线与覆盖](#s30) |
| 31 | [跨语言 DTO、gRPC 与存储合同](#s31) |
| 32 | [消费方接线、配置与时钟墙治理](#s32) |
| 33 | [新增验收矩阵、追踪与冻结项](#s33) |
| 34 | [核验来源、历史依据与规范区分](#s34) |

<a id="s01"></a>

## 1. 规范效力、阅读方式与术语

### 1.1 规范层次

MUST 表示必须，MUST NOT 表示禁止，SHOULD 表示有记录的理由才能偏离，MAY 表示可选。本文是拟议实现合同；审批后才能作为对应候选的验收基线。文档日期不代表远端创建时间，文档版本不等于 crate 版本。

源码基线事实转引 1.1.0 整合轮记录的历史 `kernel-to-temporal-analysis.md`，固定源码定位见 [K1]–[K8]；0.1.0 未重新审计远端最新 main，不把历史 SHA 当作今日最新版本。以下 API、目录、错误代码、查询规则、测试和 CI 均是目标设计，不得描述成该 kernel 提交已有的功能。旧源码中的 XH-TIME-MODEL-SPEC-001 引用只用于溯源；未读取其完整原件，不宣称无差异继承其全部条文。文档版本与 crate、wire schema、QueryPolicy 的版本分别管理。

规范冲突时暂停相关变更，由 Owner 通过 ADR/RFC 修订本 SPEC 与 GOAL。不能选择对当前实现最宽松的一段解释。

### 1.2 关键术语

| 术语 | 本规范定义 |
| --- | --- |
| 绝对时刻 | POSIX epoch 纳秒坐标（不累计闰秒）中的一个可表示值，不等于来源实际精确到纳秒 |
| 墙钟 | 对外部真实时间的采样，允许回退 |
| 单调时钟 | 用于进程内间隔测量的时钟；不提供跨进程历史身份 |
| 事实（fact） | 消费方业务契约定义的一个可独立修订的事实作用域 |
| 来源修订（revision） | 某事实的不可变内容版本，不是一次传输或一次入库 |
| 接收（receipt） | 一个边界实际接收某修订的事件；同版可以多次接收 |
| 可消费（availability） | 指定消费边界已经允许读取特定版本的证据 |
| PIT | Point-in-Time：仅基于指定知识截止点和视图当时可见版本选择结果 |
| 数据快照（snapshot） | 固定输入数据、元数据和证据的不可变集合或受信引用 |
| 历史切片（knowledge cut） | 指定消费边界在截止点的可读版本/事件集合证明 |
| 查询策略 | 对证据、知识口径、版本顺序、缺失及结果形态的版本化规则 |
| 语义结果 | 所选事实/版本/操作/有效状态，不包含随输入规模变化的扫描计数 |

### 1.3 合成示例说明

所有 CPI、发布时间、修订时间和策略示例均是合同测试数据，不是真实日历、经济指标值或交易建议。日期及时间的 Z 后缀表示示例明确采用 UTC；无时区民用日期不默认采用 UTC。

<a id="s02"></a>

## 2. 独立仓库与依赖边界

### 2.1 必须保持的依赖方向

```text
kernel                         temporal
  └── std / thiserror            └── std / thiserror

消费方产品契约 / 应用适配 / 组合根
  ├── 按需依赖 kernel
  └── 显式依赖 temporal

workspace 元仓库 ── 治理与候选登记，不是 Cargo 依赖
```

temporal MUST NOT 依赖 kernel、workspace、内部 testkit、业务 crate、存储驱动、网络、Tokio、gRPC、serde 或 chrono/time 时区库。这里的 serde 禁止指生产依赖；编译负向测试可以使用 serde dev-dependency。

测试与 benchmark 不允许要求相邻内部仓库存在。外部公开测试工具需审核，固定在本仓锁文件中。不得为了消除手写的少量类型校验引入新的内部基础库。

### 2.2 工程目录

```text
temporal/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── CHANGELOG.md
├── CONTEXT.md
├── CONTRIBUTING.md
├── LICENSE
├── goal.md
├── spec.md
├── checklist.md
├── docs/
│   ├── API.md
│   ├── 迁移指南.md
│   ├── 测试矩阵.md
│   ├── 性能基线.md
│   └── adr/
├── src/
│   ├── lib.rs
│   ├── unix_time.rs
│   ├── clock.rs
│   ├── error.rs
│   ├── precision.rs
│   ├── observation.rs
│   ├── model.rs
│   ├── revision.rs
│   ├── evidence.rs
│   └── pit/
│       ├── mod.rs
│       ├── context.rs
│       ├── visibility.rs
│       ├── selection.rs
│       └── effective.rs
├── tests/
│   ├── support/
│   ├── primitive_contract.rs
│   ├── clock_contract.rs
│   ├── observation_contract.rs
│   ├── precision_contract.rs
│   ├── profile_contract.rs
│   ├── evidence_contract.rs
│   ├── pit_contract.rs
│   ├── effective_contract.rs
│   ├── derived_contract.rs
│   ├── api_compile.rs
│   └── properties.rs
├── examples/
│   ├── event_record.rs
│   ├── published_observation.rs
│   └── pit_selection.rs
├── benches/
│   ├── primitives.rs
│   └── pit_selection.rs
└── .github/workflows/ci.yml
```

文件分区为内部实现细节；跨 crate 稳定入口为 crate 根明确列出的 re-export。`pub mod` 不得随意扩大消费面。公共 `docs/API.md` 是生成/检查清单，不是第二套规范。

### 2.3 Manifest 基线

下列为待建仓时采用的关键配置；包版本 0.1.0 为拟议初版，仓库地址仅在实际创建后填入。

```toml
[package]
name = "temporal"
version = "0.1.0"
edition = "2024"
rust-version = "1.88"
license = "MIT"
publish = false

[dependencies]
thiserror = "2"

[dev-dependencies]
proptest = "1"
static_assertions = "1"
serde = { version = "1", features = ["derive"] }

[lints.rust]
private_interfaces = "deny"
private_bounds = "deny"

[lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
todo = "deny"
unimplemented = "deny"
```

本版不引入 feature 开关；并不因不同产品接入而另建时间 primitive。crate 根 MUST 使用 `forbid(unsafe_code)`、`deny(missing_docs)`、`deny(unreachable_pub)`。测试中的断言许可限定在测试编译单元，不全局放宽生产 lint。

提交可由 MSRV 读取的 Cargo.lock；本仓 `--locked` 用于复现，不能假设库消费者会自动使用库仓库的锁文件。消费方也必须固定自己的锁文件和 Git 源。[R4]

<a id="s03"></a>

## 3. 基础时间的唯一性与系统边界

### 3.1 仅有三种基础概念

| 概念 | 类型 | 合法用途 | 禁止用途 |
| --- | --- | --- | --- |
| 绝对时刻 | `UnixTimeNs` | 事件/发布/接收/可见性时刻 | 隐式当作单调 timeout 起点 |
| 持续时间 | `std::time::Duration` | 非负长度、超时预算、检查加减 | 伪装 epoch 坐标 |
| 单调点 | `std::time::Instant` | 单进程耗时、deadline | 序列化、数据库主键、跨进程排序 |

MUST NOT 新增平行 `Timestamp(i64)`、`UnixMillis`、`UnixMicros`、`MonotonicNanos`。局部 import 别名可以使用 Timestamp，但必须仍是相同 Rust 类型。`CivilDate` 是日历标签，不是第四种绝对时刻 primitive。

### 3.2 数值、分辨率和准确度

纳秒数值表示不保证系统时钟、供应商或数据库具有纳秒准确度。来源精度、舍入方式、时钟误差和证据确定性必须独立表达。`TimePrecision` 表示单位分辨率，不作为质量评级。

SystemTime 不是单调时间，表示范围和精度依赖平台；Instant 不承诺跨进程或跨运行可比较。[R1] [R2]

### 3.3 纯规则边界

`UnixTimeNs` 运算、期间校验、precision、profile、evidence 校验和 PIT 选择 MUST NOT 读取系统时间、随机数、环境变量、文件、网络或数据库。它们只读取显式参数。

只有公开、明确命名的系统时钟实现可以读取 std 时间源；`try_from_system_time(value)` 只转换传入值，不读取 now。库不创建线程、不 sleep、不持有全局单例。

<a id="s04"></a>

## 4. UnixTimeNs 与算术合同

### 4.1 类型与 trait

```rust
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnixTimeNs(i64);
```

内部字段 MUST 私有；`UNIX_EPOCH`、`MIN`、`MAX` 为显式常量。MUST NOT 实现 `Default`、`From<i64>`、`Into<i64>`、`From<SystemTime>`、默认 serde 或将其格式化成人类日期的 `Display`。也不实现会产生隐式溢出或顺序语义的 `Add/Sub` 运算符。

### 4.2 必需公开方法

| 方法合同 | 返回语义 |
| --- | --- |
| `from_unix_nanos(i64) -> Self` | 所有 i64 值有效，调用方显式声明单位 |
| `as_unix_nanos(self) -> i64` | 无损导出原单位 |
| `try_from_unix_micros(i64) -> Result<Self, TimeError>` | 检查乘 1,000 |
| `try_from_unix_millis(i64) -> Result<Self, TimeError>` | 检查乘 1,000,000 |
| `try_from_unix_seconds(i64) -> Result<Self, TimeError>` | 检查乘 1,000,000,000 |
| `checked_add(self, Duration) -> Result<Self, TimeError>` | 超范围报错，不饱和、不 wrap |
| `checked_sub(self, Duration) -> Result<Self, TimeError>` | 同上 |
| `duration_since(self, Self) -> Result<Duration, TimeError>` | 反向顺序返回 InvalidOrder |
| `to_seconds_nanos(self) -> (i64, u32)` | Euclidean 标准化，nanos 在 0..1e9 |
| `try_from_seconds_nanos(i64, u32) -> Result<Self, TimeError>` | nanos 必须小于 1e9；宽整数检查 |
| `try_from_system_time(SystemTime) -> Result<Self, TimeError>` | 允许负 epoch，范围检查 |
| `try_into_system_time(self) -> Result<SystemTime, TimeError>` | 严格往返；平台范围或精度不足报错 |

不提供按数量级猜单位的构造器，也不在错误时返回 epoch、当前时间或最大值。

### 4.3 算术基准

计算中间值采用 i128/u128 或等价的检查运算。`max.duration_since(min)` 的纳秒数恰为 `u64::MAX`，必须成功；不能先用 i64 相减。`Duration::MAX` 参与加减必须报错而非 panic。

标准化例子：`-1ns → (-1s, 999_999_999ns)`；禁止结果 `(0s, -1ns)`。`try_from_seconds_nanos` 使用非负子秒字段，不能将秒字段和负子秒混用。

### 4.4 SystemTime 的严格往返

输入晚于 epoch 时取非负 duration 并检查为 i64；输入早于 epoch 时读取错误中的 duration，将其转换到宽整数再取负、检查 i64。不得把 `duration_since(UNIX_EPOCH)` 的 Err 直接解释成平台不支持负 epoch。

转出时按正负分别使用 `UNIX_EPOCH.checked_add` / `checked_sub`。绝对值在宽整数上计算，保证 `i64::MIN` 安全。构造成功后再读回，要求与原 `UnixTimeNs` 相等；不等则返回 `SystemTimePrecisionLoss`。不以强制取整冒充成功。[R1]

这比旧 kernel 的“拒绝所有负值”合同有意不同，必须在独立行为变更中批准并标注。[K2] 不对所有平台承诺 i64 全域可转换；平台不支持时返回结构化错误。需要有损转换的消费者先调用显式 precision 策略，再调用严格转换。

<a id="s05"></a>

## 5. 时钟接口与测试控制

### 5.1 目标接口

```rust
pub trait WallClock: Send + Sync {
    fn now(&self) -> Result<UnixTimeNs, TimeError>;
}

pub trait MonotonicClock: Send + Sync {
    fn now(&self) -> std::time::Instant;
}

pub trait RuntimeClock: WallClock + MonotonicClock {}
impl<T: WallClock + MonotonicClock + ?Sized> RuntimeClock for T {}
```

`SystemWallClock` 与 `SystemMonotonicClock` 保留 `new()`、Debug/Clone/Copy/Default；它们是不同对象，不虚构“同时原子读取”的墙钟/单调配对保证。组合 trait 的两个 now 调用必须用明确 trait 路径消除歧义。

公开 WallClock 只表达读时间能力，不保证非递减；MonotonicClock 使用同一时钟来源，测量异常由消费方用检查差值显式处理，不承诺绝对不会受平台异常影响。[R2]

### 5.2 注入边界

Application/Runtime 在入口采样，将时间值或上下文传给 Domain。Domain 不持有时钟 trait 对象。PIT API 不接受 Clock，它接受已经构造的 `AsOfContext`。

本仓 `tests/support` 提供 ManualWallClock，可显式 set/advance/rewind/fail；ManualMonotonicClock 基于一个测试 anchor 与检查偏移，只能非负推进。两者不是公共运行时服务，也不要求外部 testkit 依赖。

### 5.3 等待不是时钟

本模块不实现 sleep/timer/interval/scheduler。替换 ManualClock 不会唤醒 std Condvar，Tokio pause 也不控制 std Instant。[R3] 纯时间逻辑测试不用真实等待；少量系统时钟冒烟测试只验证可表示/接口，不断言固定年份或精确耗时。

<a id="s06"></a>

## 6. 民用日期、观测窗口、观测期间与有效区间

### 6.1 CivilDate

本版提供最小的 Gregorian `CivilDate`：私有 year/month/day，支持 year 1..=9999，按公历闰年规则校验日期；只提供 `try_new`、字段读取及比较。它不是日期解析库、不读取本地时区、不引入时区数据库。

输入超范围或构造下一边界超出范围必须报错，不能饱和。非公历来源必须由消费方先映射到带原始定义引用的明确日历坐标；不在本版假装完整支持任意历法。

### 6.2 观测点与期间

```rust
pub enum ObservationTime {
    At(UnixTimeNs),
    Window(AbsoluteTimeRange),
    Period(ObservationPeriod),
}
```

`At` 描述一个绝对观测点；`Window` 描述两个绝对时刻界定的有限半开窗口，例如一分钟 K 线；`Period` 描述按民用日期/日历定义的统计期间。三者不能依靠频率字符串相互猜测转换。

`AbsoluteTimeRange` 是私有字段的不可变范围值，不是新时间 primitive。必须具有 `try_new(start, end_exclusive)`、`start()`、`end_exclusive()`、`contains(t)`、`duration()`、`intersection(other)`。构造要求 `start < end_exclusive`，否则返回 `TemporalError::InvalidInterval`；相邻窗口的交集是 `None`，不得构造空窗口。`duration()` 使用宽整数中间值，完整 i64 坐标跨度也可返回标准库 Duration。范围值只做数学运算，不负责对齐交易时段、补齐缺口或选择 K 线最终版。

以下 `ObservationPeriod` 为不可变已验证值，字段合同如下。

| 字段 | 语义 |
| --- | --- |
| start | CivilDate，包含 |
| end_exclusive | CivilDate，不包含；必须大于 start |
| granularity | Day / Month / Quarter / Year / Custom |
| calendar_ref | 版本化日历/期间定义的非空引用，拥有者是消费方 |

Day 为一个日历日；Month 为月首到下月首；Quarter 为标准公历季度边界；Year 为年首到次年首。财政季度、财务年度、非规则期间用 Custom 并保留 calendar_ref，不用 Quarter/Year 谎报语义。

期间长度不固定为 30×24h，calendar_ref 不同的期间不能静默视为相同事实。期间转 UTC 需要消费方显式时区和定义版本；本库不提供隐式转换。

### 6.3 生效区间

`EffectiveInterval` 表示绝对时刻 `[start, end_exclusive)`；end 可以显式无界，不能用 i64::MAX 作为隐式无穷。有限 end MUST 大于 start。`contains(t)` 使用半开规则。

`effective_time` 表示开始生效时刻；当版本同时声明 EffectiveInterval 时必须等于 start。仅有 effective_time 不足以表示完整有效状态，结束规则由产品状态合同提供，且只能依据当时已知的信息推导。

<a id="s07"></a>

## 7. 六角色模型、Draft 和 profiles

### 7.1 输入与已验证模型

以下为目标类型形态，不是可直接编译的完整实现包。

```rust
pub enum TimeField<T> {
    Known(T),
    Unknown,
    NotApplicable,
}

pub struct TemporalDraft {
    pub event_time: TimeField<UnixTimeNs>,
    pub observation_time: TimeField<ObservationTime>,
    pub publication_time: TimeField<UnixTimeNs>,
    pub received_time: TimeField<UnixTimeNs>,
    pub revision_time: TimeField<UnixTimeNs>,
    pub effective_time: TimeField<UnixTimeNs>,
}
```

`TemporalDraft::validate(profile)` 返回字段私有的 `Temporal`；Temporal 只有只读访问器，不允许 post-validation setter。`received_time()` 对所有已验证 Temporal 返回 `UnixTimeNs`。其他访问器保留 Known/Unknown/NotApplicable，不降级成无法区分的 Option。

`TimeField`、Temporal、Draft 和查询策略均不提供静默 Default。需要的构造入口应按 profile 显式提供，不能缺省伪造发布/修订/接收。

### 7.2 字段角色

| 角色 | MUST 表示 | MUST NOT 表示 |
| --- | --- | --- |
| event_time | 该版本描述的事件发生坐标 | 本地解析完成、数据库写入完成 |
| observation_time | 该事实描述的观测点或参考期间 | 发布时间、latest revision 时间 |
| publication_time | 当前修订面向声明受众实际发布的时刻 | 整个序列初次发布、计划发布日期 |
| received_time | 本条投影所绑定接收边界的实际接收采样 | 来源发布日期、补采历史假接收 |
| revision_time | 来源定义的本修订行为时间 | 仅凭它定义版本身份或修订先后 |
| effective_time | 业务事实开始生效坐标 | 首次知晓、首次接收 |

Known 表示提供了来源/边界明确的时刻坐标，不表示无测量误差。误差和粗粒度通过证据字段表达。来源只给日期而没有实际发布时间，publication_time 必须保持 Unknown，附加日期证据；禁止伪造当天 00:00。

### 7.3 五个通用 profile

| Profile | 必需已知字段 | 其他合同 |
| --- | --- | --- |
| EventRecord | event_time、received_time | observation 可不适用；来源发布证据不能自动由 event_time 生成 |
| PublishedObservation | observation_time、received_time | publication 不得 NotApplicable；可以 Unknown，但查询另验 release 证据 |
| Announcement | effective_time、received_time | publication 不得 NotApplicable；允许生效时间晚于知识截止点 |
| DerivedRecord | received_time，且 event/observation 至少一项已知 | received 是本地产物接纳边界，不冒充外部网络接收；需派生 lineage |
| SampledObservation | received_time | 用于没有可证明源时刻的采样观察；保留响应/捕获证据，不把本地采样时间改写为源 event_time |

`SampledObservation` 的 event/observation/publication 缺失状态必须由端点映射明确判为 Unknown 或 NotApplicable，不能为了通过 EventRecord 校验补出一个时间。它只表明结构可被保存；严格来源历史查询仍须证明该观察版本的来源可用性，系统查询仍须 receipt、消费边界和 cut。采样事实与源事件的身份区别见 §29。

publication/revision 不确定的已发布观测可以通过结构校验，但不能因此获得 Visible。`TemporalDraft::validate(profile)` 保证类型与角色组合；`evaluate_visibility` 才验证查询所需证据。

MUST NOT 全局要求 `event <= received`、`observation <= known_at` 或 `effective <= known_at`。跨机器偏差、迟到、未来预测和提前公告有不同语义。质量问题保留证据并由 profile/查询策略处理，不通过修改原值掩盖。

### 7.4 热路径表示

六角色是统一逻辑视图，不强制每条 tick 携带六个大枚举、字符串和完整证据。实现 MAY 使用紧凑 EventRecord 与共享元数据/旁表，只要能构造等价只读视图并通过相同合同。优化不能改变字段含义、缺失状态或错误行为。

<a id="s08"></a>

## 8. 时间精度、舍入与存储能力

### 8.1 精度与损失策略

保留 `TimePrecision::{Seconds, Milliseconds, Microseconds, Nanoseconds}` 与 `nanos_per_unit()`。`PrecisionLossPolicy` 本版包含：

| 策略 | 定义 |
| --- | --- |
| Reject | 默认；非整单位拒绝 |
| ExplicitTruncate | 向零截断，继承旧策略的数学语义 |
| ExplicitFloor | 向负无穷取整；新增明确策略 |

不提供根据正负自动变换的“智能截断”。未来增加 nearest/ceil 需新规格和测试，未知策略外部匹配不能当成允许有损。

### 8.2 必需函数

保留具名单位入口 `unix_ns_to_stored(i64, TimePrecision, PrecisionLossPolicy)`、`stored_to_unix_ns(i64, TimePrecision)`、`quantize_unix_ns(i64, TimePrecision, PrecisionLossPolicy)`，返回本模块 `PrecisionError` 而不是 XError。它们是明确单位的适配边界，业务核心仍使用 UnixTimeNs。

`verify_projection(authoritative_ns, projected_ns, precision, policy)` 先检查 projected_ns 对精度对齐，再比较 authoritative 按明确策略得到的投影。不得先量化读回值，掩盖异常输入。

### 8.3 边界行为

`-1ns` 向微秒 ExplicitTruncate 得到 0；ExplicitFloor 得到 -1 个微秒。quantize 的还原乘法仍须检查：靠近 i64::MIN 的 floor 可能落到可表示范围之外，必须返回 Overflow。

Reject 策略下除法前检查余数是否为零；余数正负不影响“是否无损”。所有单位乘法使用检查算术，禁止 wrap。

### 8.4 存储能力声明不是证明

迁移 `TimeStorageCapabilities` 的 stored_precision、utc_normalized、lossless_unix_ns 三项含义。`require_lossless_unix_ns` 是窄检查，只检验 lossless_unix_ns，不声称证明实际后端读写。完整适配器验收还应检查范围、负值、精度、UTC 转换与驱动舍入。

MUST NOT 在 temporal 中保留 `verify_pg_dual_column`。PostgreSQL timestamp/timestamptz 的微秒分辨率是后端事实；具体读写、epoch 和投影规则由适配器测试。[R8] 本库只提供通用 projection 验证。

<a id="s09"></a>

## 9. 事实、来源修订、接收与可消费事件

### 9.1 四种身份 MUST 分开

| 对象 | 稳定身份 | 可变性 | 不能替代 |
| --- | --- | --- | --- |
| Fact | `(authority_scope, 消费方完整 FactKey)` | 由产品契约版本化 | 不能只用 series_id 或时间戳 |
| FactVersion（来源修订或产品观察版本） | `(authority_scope, FactKey, RevisionId)` | 内容和固有语义不可变；权威作用域明确 | 来源修订号、采样身份、规范化版本不能互相冒充 |
| Receipt | `(receiver_boundary, receiver_epoch, receipt_id)` | 追加事件 | 不证明目标消费者已可读取 |
| AvailabilityEvent | `(consumer_boundary, log_epoch, event_id)` | 追加事件 | 不等于来源公开时间 |

FactKey 的字节/字段规范化由产品契约负责。宏观通常包括 source、series、观测期间、单位、统计定义/季调口径；市场通常包括 venue、stream 和来源事件 ID/序号。temporal 消费完整键，不决定金融实体结构，也不引入第二套 canonical。同一 FactKey 的字节只在同一 authority_scope 内比较；不同作用域复用相同字节时仍是两个事实，不得因为 source_scope 或 consumer_boundary 名称相同而合并。

### 9.2 来源版本内容合同

每个来源 SourceRevision 至少绑定：fact_key、revision_id、operation、payload_ref、semantic_digest、来源时间角色、权威修订顺序/前驱关系和来源证据引用。`operation` 本版为 `Upsert` 或 `Withdraw`；恢复使用新的 Upsert 修订，不是删除 tombstone。

Upsert 是完整值版本或可解析到不可变完整值的引用，不能只给缺少前置依赖的增量补丁。Withdraw 无有效载荷值，但仍有身份、权威顺序、撤回语义和可见性证据。

`semantic_digest` 绑定载荷、业务 schema、操作、固有来源时间角色及修订关系；**不包含 received_time、receipt_id、接收边界或 system availability**。哈希/规范化由消费方的版本化合同实现，库只比较明确的算法/格式版本及摘要，不在本版加入密码库。

### 9.3 统一 Temporal 与多次接收并不冲突

Temporal 是一次具体接收/投影的统一时间视图。来源修订的 event/observation/publication/revision/effective 语义不变；received_time 来自本次绑定 Receipt。同一个 SourceRevision 可以投影为不同接收边界的 Temporal，而不制造“同版冲突”。

MUST NOT 把整份包含 received_time 的 Temporal 序列化后直接当作来源修订的唯一内容摘要。同版重复传输而接收时间不同是正常；同版载荷或固有来源语义不同才是冲突。新的来源时间更正必须版本化元数据并冻结旧快照；不得无痕修改历史发布时刻。

### 9.4 幂等、冲突与作用域

下列身份比较均在同一已验证authority_scope内；跨authority的同名键不能合并。同键同摘要重复记录幂等合并为一个版本，接收/可见性证据仍独立追加。相同 FactKey+RevisionId 的摘要冲突返回 `ConflictingRevision`，消费方隔离冲突，不覆盖任一证据。

不同事实的 RevisionId 可以相同，不能全库按 RevisionId 去重。相同 payload 在不同修订号或不同事实中也不自动合并。相同接收事件 ID 的不同内容是 evidence 冲突，不是新的 receipt。

### 9.5 修订顺序合同

第一版支持两种显式 `RevisionOrderPolicy`；来源版本依据来源权威，产品观察/派生产物依据该产品定义的版本权威，两种 authority scope 不能混用：

| 模式 | 必需证据 | 选择方式 |
| --- | --- | --- |
| AuthoritativeSequence | 来源认可的 order_scope 与非负 ordinal；同一事实内同 scope | 选可见版本中最大 ordinal |
| LinearSupersedes | 初始节点或同一事实的单一前驱引用；必要链闭包/权威链摘要 | 选可见版本中唯一未被可见后继替代的节点 |

来源序号是权威修订顺序，不是 Kafka offset、数据库插入序号或接收时间。日志重建/来源重置导致 scope 不同，不能直接比较 ordinal；需要消费方经版本化证明归一到同一 scope，否则 `IncomparableOrderScope`。

Sequence 相同 ordinal 却有不同有效 RevisionId 返回 `AmbiguousRevisionOrder`。LinearSupersedes 中分叉、环、跨事实前驱、缺失必要链关系必须拒绝；本版不自动合并分支。RevisionId 的 Ord 只可用于确定性输出排序，不用于决定哪个修订较新。

同版多个 receipt 的最早可用证据必须在该查询知识切片中选择，不能对整个未来日志取最小墙钟并回填历史。旧修订晚到仍保持旧修订顺序。

### 9.6 来源修订、规范化修订与本地观察

`FactVersionRef` 是通用不可变事实版本视图；`SourceRevision` 只表示其中来源真实定义的修订。源端没有 revision_id 时，产品可以定义确定性的本地版本标识，但必须记录 `authority_scope` 与身份算法版本，不能声称该标识来自供应商。

规范化器修复单位、字段映射或解析逻辑，应形成新的规范化版本/元数据快照，保留原 raw、来源身份和旧输出 digest；不得伪造供应商发布了新修订。SystemAsKnown 必须仍绑定历史消费边界当时实际读到的规范化内容；用新解析器重算的研究视图必须另建 snapshot。

同一个 `FactVersionRef` 的 `(authority_scope, FactKey, RevisionId)` 只属于一种不可变版本域。产品规范化版本与供应商原始版本不得复用同一身份却给出不同 semantic_digest。源修订号不变而规范化结果变化时，产品创建自己的新 RevisionId/明确的表示命名空间，并在 provenance 中保留原 `(source_scope, source_fact_key, source_revision_id, raw_digest)`；不得改写来源身份。库内 `semantic_digest` 绑定当前候选实际返回的不可变语义/载荷，raw_digest 是独立溯源摘要，二者不互换。

来源研究视图使用固定的转换/解释快照：每个候选与原来源版本、来源发布证据以及确定性的转换清单绑定；事后规范化不能用来引入原版本没有的未来输入。系统实际视图使用产品当时真实交付的规范化版本和对应availability/cut。不能将两个不同规范化版本塞入同一不可变版本身份，依赖“这是解析修复”豁免 ConflictingRevision。

对没有源版本顺序的单次采样，产品以稳定 capture/receipt 身份定义一个独立观察事实，并用 `LinearSupersedes` 的单一初始节点表示唯一版本；不伪造源 ordinal。重放同一捕获记录不能生成新事实。不同捕获记录不自动是同一源事实的连续修订。需要“当前最新观察”时，由产品先进行 PIT，再执行独立的观察选择合同。

<a id="s10"></a>

## 10. 可见性证据与时间不确定性

### 10.1 证据是引用和边界，不是一个 true 标志

证据模型至少记录 `evidence_id`、证据 schema/提取器版本、来源或边界作用域、目标版本身份/摘要、时间声明、质量/精度描述、原始证据引用。有效性审查由消费方受信证据入口执行；temporal 检查字段和规则一致性。只有已被该入口接受、且绑定当前版本和受众的证据，才能提供放行用的 TimeBounds；版本自身的 publication_time、revision_time、received_time 或任意计划日期不能自证可见。

`verified: true`、普通 `recorded_time` 或用户随意提供的提交号，不能独立构成可信证明。Rust 的已验证包装类型也只保证库执行了结构和合同校验，不使外部证据自动真实。

### 10.2 时间界限

逻辑模型 `TimeBounds` 表达：

```text
earliest_possible: Option<UnixTimeNs>
definitely_available_by: Option<UnixTimeNs>
precision / uncertainty_description_ref
```

已知上下界时必须 `earliest_possible <= definitely_available_by`；Exact 证据两者相等。两个边界均缺失表示无法判定，不是 epoch。保守上界可能晚于真实发布时刻，这会减少可见集合但不提前暴露。

在知识截止点 T 下，时间判定为：

```text
若 upper 存在且 upper <= T：时间维度 ProvenByCutoff
否则若 lower 存在且 lower > T：时间维度 NotYetVisible
否则：时间维度 Indeterminate
```

该三态只是时间判断；最终 Visible 还需要作用域、版本、快照和模式要求全部成立。`ExactOnly` 只接受已被证明为同一精确时刻的上下界；只有日期区间或单侧边界时，即使保守上界早于 T，也不能在该模式放行。混合多个必要条件时，任一明确 NotYetVisible 可以证明本次不应选该版本；没有此证明而存在 Unknown/Indeterminate，不能静默丢弃潜在的新修订。

### 10.3 日期粒度来源证据

仅知道“在当地日期 D 发布”，且有证据证明实际发布确在该日内时，适配器可以提供明确时区/日历版本下的区间。严格查询在该区间内返回 Indeterminate；采用保守上界策略时，不早于该日结束边界放行。

不能只看到一个计划发布日期就构造上述实际可用区间；不能假设一天始终等于 86,400 秒或未知时区是 UTC。日期边界转换、夏令时与来源闭区间端点由适配器处理。本库不内置 FRED 或任何供应商日期解释。FRED 的 real-time 与 vintage 日期是来源语义例证，不等价于精确发布时间。[R9] [R10]

### 10.4 来源可见性证据

`SourceVisibilityEvidence` 绑定 source_scope（包括来源/产品/频道及受众范围）、FactKey+RevisionId+semantic_digest、实际 release 的时间界限及证据引用。

候选若为产品规范化版本，受信证据入口必须验证其 `(authority_scope, FactKey, RevisionId, semantic_digest)` 与原来源版本身份、raw_digest、release 证据及固定转换清单的绑定；两个 digest 不要求相等。绑定缺失不能以原来源证据直接放行当前候选。

publication_time 已知时必须与证据一致，不能比证据证明的可用性更早放行。PublishedObservation/Announcement 的修订时间已知且表示实际修订行为时，该修订在此之前不能成为已发布版本。计划修订日期应放在业务日程 DTO，不得塞入当前版 revision_time。

来源发布型版本的 release gate 取所有已声明必要释放条件的交集，例如实际发布、实际修订、已知的受众解禁。不会因为某项缺失就默认零延迟；适配器必须证明当前版本的释放条件完整。

### 10.5 系统可见性证据

`SystemVisibilityEvidence` 绑定具体 consumer_boundary、日志/进程代次、版本身份、receipt 引用、system_available 的时间界限、可见性事件游标及证明引用。

持久化优先边界可以定义为提交后可读；流式边界可以定义为已验证记录进入消费者可读日志。两者都需要明确记录，不能把数据库提交强加给全部消费方，也不能把网络接收统一当作目标消费者可读。

同一版本面向不同消费者有不同 system availability，不能只存一个无作用域的 `available_at`。

<a id="s11"></a>

## 11. 快照、历史知识切片与分布式顺序

### 11.1 两种不可混同的固定对象

`DatasetSnapshotRef` 固定本次可用于重建的来源版本、元数据与证据集合，通常可以在回测之后生成。`SystemKnowledgeCut` 固定历史决策边界当时可消费的版本/事件集合，必须有历史可见性证明。

SystemAsKnown 查询需要两者交集。今天制作的完整历史快照不意味着其中所有数据在过去都已进入系统。

### 11.2 SystemKnowledgeCut 的最小声明

| 字段 | 合同 |
| --- | --- |
| boundary_id | 指定哪个产品/消费者的可见性边界 |
| boundary_epoch | 边界重建/日志代次，不同 epoch 不直接比较 |
| cut_id / manifest_ref | 不可变历史切片及其完整性证据引用 |
| known_at_binding | 与本次历史截止点的绑定关系和证据版本 |
| partitions | 每分区明确的可见性前沿或确切成员集合 |
| consistency_policy | 该 cut 如何保证是受控可读视图，及其版本 |
| evidence_snapshot_ref | 当时所需 receipt/visibility 证明的集合或可重建引用 |

单日志可以用可消费事件的前缀；多分区使用向量前沿或已验证一致 manifest。分区名称相同而 epoch 不同不能混用。前沿是“可见事件”的序号，不是未提交操作的任意序号。

### 11.3 Timestamp 无法取代 cut

两个事件可以有同一纳秒坐标；不同机器墙钟也可能错序。必须用 cut 判断事件是否属于历史可读集合，不能用 `available_time <= T` 独自放行。

系统墙钟回退导致未来提交带有较早数值时，历史 cut 仍必须排除它。不允许修改原始时间以维持假单调。无法把历史截止点与 cut 关联时，返回 `UnprovenSystemCut`，不以现在读取的最大 offset 猜测过去。

PostgreSQL 的 transaction_timestamp/now 反映事务起点，不自动代表提交对消费者可见的时刻；提交前采样的 wall clock 同样不是提交证明。[R11] 真实集成边界由消费方证明，库不实现数据库提交追踪。

### 11.4 来源视图允许事后证据，系统视图不得事后伪造

来源历史归档可在今天证明某旧版本确于 T 前发布，参与新 DatasetSnapshot 的 SourcePublishedAsOf 查询。SystemAsKnown 中新增的证据不能虚构历史 receipt/cut；只能重建已有受信日志证明的历史事实。

数据补全导致新的研究快照不同是允许的；用旧 snapshot 重放必须不变。将旧 snapshot 名称复用于新内容属于完整性错误。

<a id="s12"></a>

## 12. 查询上下文、策略与输入覆盖

### 12.1 AsOfContext

所有知识查询 MUST 显式传入不可变、已校验上下文，不提供隐式“现在/latest”。

| 字段 | 说明 |
| --- | --- |
| known_at | 知识截止点，UnixTimeNs |
| knowledge_basis | SourcePublishedAsOf 或 SystemAsKnown |
| authority_scope | 本次查询的唯一版本权威作用域；跨作用域请求须拆分，不能混排修订 |
| source_scope | 来源视图必需；系统视图可按 profile 需要保留 release scope |
| consumer_boundary | 系统视图必需 |
| dataset_snapshot | 不可变输入快照引用 |
| system_cut | 系统视图必需，来源视图不能拿它替代来源证据 |
| policy_ref | 完整策略内容/版本的稳定引用 |
| schema_refs | 产品键、版本、证据 schema 的明确版本 |
| interpretation_refs | 有转换时的日历/时区/提取器版本 |
| cut_boundary_rule | 本版固定为精确已证明时间的包含边界 `<=` |

`valid_at` 不放入所有查询默认条件；有效状态查询单独传入，见 §15。不得因为上层忘传 mode 就默认 source/system 其中之一。

`authority_scope` 确定事实版本身份与修订顺序，`source_scope` 确定来源证据的受众范围，`consumer_boundary` 确定系统可读边界；三者分别校验，不得互相代填。第一版单次严格查询只接受一个 authority_scope；跨权威聚合由产品分别完成 PIT 后再组合。

### 12.2 QueryPolicy

策略值经构造器一次校验后不可修改；必须至少固定 profile、证据接受模式、修订顺序模式、严格失败行为、输出范围和资源上限。

证据接受模式：

- `ExactOnly`：必要可用性事件必须有精确、经接受的证据；粗粒度不放行。
- `ConservativeBounds`：允许真实区间的保守上界，不允许伪造或计划日程。

选版本版使用严格行为：潜在影响结果的证据不足、分支或冲突不返回一个貌似成功的最新值。允许独立评估版本状态，但批量严格查询不得把部分成功列表作为完整答案。

### 12.3 CandidateCoverage

库不能从一组输入证明“数据库里没有遗漏”。消费方必须提供可审计的 coverage 声明：本次 authority_scope、完整 FactKey 集合、snapshot、查询模式及截至 cutoff 的所有潜在候选已完整枚举，或具有等价可验证的索引/上界证明。空候选集合也须有同一作用域的完整性证明；空列表本身不证明 NoVisibleVersion。

`Complete` 是受信适配器提供的合同，不是任意调用者能写一个 bool 就证明的事实。候选截断、分页尚未结束、仅 latest、遗漏 tombstone 或证据查询不完整时，覆盖为 Incomplete，严格入口返回 `IncompleteCandidateSet`。

如果 API 接受事先按来源可见性过滤的候选，必须附与同一策略等价的 coverage/filter 证明，并与 Rust 参考选择器做一致性测试。不能把 SQL pushdown 当成天然正确。

### 12.4 不同输出范围

Facts 查询按每个完整 FactKey 返回当时版本。LatestObservation 是产品层在 Facts 结果上进行的第二次选择，需要明确系列作用域、期间顺序、状态和缺失策略，不在 temporal 中默认将不同期间合并。

例如按 series_id 返回“最新一条”可能丢失同一期间的来源差别或采用错误发布版本，因此不得作为库级默认 API。

<a id="s13"></a>

## 13. 可见性判定算法

### 13.1 结果模型

`VisibilityDecision` 必须区分：

| 状态 | 含义 |
| --- | --- |
| Visible | 本查询所需条件全部得到证明 |
| NotVisible(reason) | 已有充分证据证明该版本不属于本次可见集合 |
| Indeterminate(reason) | 尚不能证明可见，也不能证明对结果无影响 |

无效上下文、版本身份错配、证据篡改/冲突、结构错误通过 `Result::Err(PitError)` 返回，不藏进 NotVisible。缺失质量证据不能借“安全起见排除”悄悄让旧修订成为确定答案。

### 13.2 SourcePublishedAsOf

按顺序检查：context 与 snapshot/scope 一致；候选版本绑定准确；来源 release 证据被消费方接受且对应当前修订；所有必要 release gate 的时间判定；本版已知的 publication/revision 声明与证据不矛盾。

来源视图不要求本系统历史 received_time <= known_at，因为允许事后补采。但结构中的 received_time 仍必须真实保留，不允许回填。资料只有最新最终值而没有历史修订内容时，不能借一个旧 publication_time 放行。

event_time、observation_time 和 effective_time 不是通用可见性前提。对预测或公告，它们可以晚于 known_at。

### 13.3 SystemAsKnown

必须同时满足：

```text
1. version 属于 dataset snapshot；
2. source release 条件满足当前 profile 的要求；
3. receipt 属于声明的接收链，且绑定相同版本；
4. availability event 属于指定 consumer boundary/epoch；
5. event 属于历史 SystemKnowledgeCut；
6. system availability 的保守上界 <= known_at；
7. context、cut 与证据的时钟解释/误差合同一致。
```

对于没有独立 publication 的 EventRecord，以及以本地捕获事实为权威的 SampledObservation，独立来源发布条件可以经明确的 profile/authority 合同标记为不适用；不能仅因字段缺失自动豁免，也不能因此免除接收、可消费证据和历史 cut。该豁免不允许推断源状态在捕获前何时成立。

若上下游原始墙钟坐标与因果顺序不一致，不强制把时间改写为单调；采用已证明的时间界限/边界解释。无法建立可靠界限或 cut 时返回 Indeterminate/UnprovenSystemCut，不根据错序原值放行。

### 13.4 原因的安全性

详细管理员诊断可以记录“某不可见版本缺少证据”，但回测决策数据流不能接收到未来版本 ID、数量、内容或未来发布时间等信息。核心内部可计算排除原因，公开给策略的输出必须由 §16 的决策安全投影限制。

<a id="s14"></a>

## 14. PIT 选版、撤回与冲突处理

### 14.1 权威执行顺序

```text
校验查询、作用域、snapshot 与 coverage
    → 读取候选的最小身份和证据
    → 判定可见性
    → 对本次可能影响结果的候选进行一致性检查
    → 按 (authority_scope, 完整 FactKey) 分组
    → 在可见候选中按已声明的权威修订关系选唯一版本
    → 若所选为 Withdraw，返回 Withdrawn
    → 否则返回所选完整值版本的引用
    → 产生固定版本清单与决策安全输出
```

MUST NOT 先全局 max(revision) 再过滤发布时间。MUST NOT 先过滤撤回。MUST NOT 以接收时间、入库时间、数值大小、UUID 字典序或 hash 值决定修订优先级。

### 14.2 单事实选择

无可见候选且无会影响结论的 Indeterminate：返回 NoVisibleVersion。存在可能更改当前选版的 Indeterminate：严格入口返回 `IndeterminateSelection`，不能返回旧版并声称它就是当时最新。

为了第一版容易审计，默认每个请求 FactKey 中任何尚未被可靠排除的 Indeterminate 都阻止该事实的严格选版。以后优化“证明该未知候选一定被可见新版本支配”的路径必须保持等价并有独立测试。

Sequence 模式对可见、合法且相同 order_scope 的版本取最大 ordinal；重复同版去重；同序号冲突拒绝。LinearSupersedes 模式要求可证明链关系，取唯一最大可见节点；遇到分叉/循环/必要关系缺失拒绝，不搜索一个任意可用路径。

### 14.3 因果闭包与完整性

单条 Upsert 的 payload 完整不等于选版候选完整。Sequence 模式的序号存在间隙本身不必报错，但适配器必须证明 gap 不隐藏一个截至 cutoff 可见、会改变结果的版本。

链式模式需要比较的祖先关系必须来自查询允许的关系证据；不能使用未来新增的 supersedes 边来提前判断旧版无效。后续关系更正进入新元数据快照，不原地重写旧快照。

### 14.4 不可见未来候选不污染过去

合法、确定在 cutoff 后才可见的修订不参与过去的版本优先级、冲突选择和有效状态判断。对全量归档进行未来数据质量审计属于另一个管理结果，不改变历史语义结果。

本条不要求在已有历史输入被证实篡改时继续返回可信结果，也不允许忽略无法解析的危险输入。未来追加不污染不变量的前提是合法、同策略、历史证据未被修改的扩展集合。

### 14.5 撤回、恢复与批量完整性

所选版本 Withdraw 时返回显式 Withdrawn，不返回最后一个非撤回值。后续 Upsert 可在其可见后恢复。撤回事实身份或期间纠正涉及旧键撤回与新键新增，由产品契约以明确原子组/证据处理，不在库里悄悄改 FactKey。

`select_facts_as_of` 对已声明完整作用域提供严格结果；任一潜在影响完整性的错误返回 Err，不附一份可被误认完整的成功列表。调用方可以主动拆分独立事实分别查询，但不能将混合状态包装为“全部成功”。

### 14.6 伪代码

```text
select_facts_as_of(input, context):
    validate_context_snapshot_coverage(input, context)
    candidates = evaluate_visibility_for_scope(input, context)
    if any_uncertain_candidate_can_affect_requested_facts(candidates):
        return IndeterminateSelection
    groups = group_visible_candidates_by_authority_and_full_fact_key(candidates)
    for each requested fact in canonical_key_order:
        validate_visible_revision_identity_and_order(groups[fact])
        chosen = unique_authoritative_latest(groups[fact])
        if no chosen: emit NoVisibleVersion
        else if chosen.operation == Withdraw: emit Withdrawn(chosen.id)
        else: emit Selected(chosen.id, chosen.payload_ref)
    return result_with_selected_manifest_and_safe_projection
```

此伪代码不是用于绕过前置证据校验的实现；外部存储选择器必须证明等价，而不是只实现最后一行 SQL 排序。

<a id="s15"></a>

## 15. 已知事实与有效状态的不同查询合同

### 15.1 知识事实查询

`select_fact_as_of` 回答当前 FactKey 截至 known_at 的权威可见版本，不把 effective_time 作为可见性截止条件。未来生效公告可以被返回。

### 15.2 有效状态查询

`select_effective_state_as_of` 额外接收 valid_at 与 StatePolicy。先对每个独立事实/断言完成 PIT 修订选择和撤回处理，再将存活断言映射到明确的 state_key 与 EffectiveInterval，最后查询包含 valid_at 的有效断言。

本版通用 StatePolicy 仅支持 `UniqueNonOverlappingAssertions`：同 state_key、同 valid_at 零个断言返回 NoActiveState；一个返回 Active；多个返回 `ConflictingEffectiveState`。不按最新发布时间自动化解业务冲突。

本次输入中每个已选存活事实都必须有明确的状态断言映射；缺映射返回 ContextMismatch，不能以漏映射推断 NoActiveState。

### 15.3 修订不等于未来生效替换

示例：A 表示当前合同状态，B 表示已公告的未来替换。如果 B 不应在尚未生效时删除 A，A/B 必须由产品契约作为不同状态断言或完整版本化时间线表达；不能简单把 B 存为 A 的单值修订，然后要求库猜测是否应该复活旧版。

A 的结束边界若由 B 推导，只能在 B 当时已知且产品策略允许时使用；不能用未来尚未公告的 B 提前截断 A 的历史区间。需要复杂优先级、区间分裂或法规适用逻辑时，由产品模块持有规则并版本化，temporal 只提供可见版本和区间工具。

### 15.4 有效状态的可复现条件

输出同时记录 known_at、valid_at、StatePolicy 版本和参与状态断言的修订集合。valid_at 可以早于或晚于 known_at；查询未来状态只允许利用当时已知的公告，不构成对未公布未来事实的访问许可。

<a id="s16"></a>

## 16. 结果、解释、缓存与复现清单

### 16.1 事实结果

`FactOutcome` 本版包含 Selected、Withdrawn、NoVisibleVersion。Indeterminate 和结构/完整性错误在严格选择入口通过 Err 返回；不要把 Indeterminate 序列化成 `value=null`。

Selected 至少返回 authority_scope、FactKey、RevisionId、operation、semantic_digest、payload_ref 和用于本次可见性判定的证据引用。Withdrawn 同样返回作用域、撤回修订和有效证据，不退回旧 payload。NoVisibleVersion 绑定所请求的 authority_scope 与完整 FactKey，但不暴露任何未来候选的身份或数量。

### 16.2 决策安全输出与管理解释

默认输出是 `DecisionSafe`：只包含当前知识边界可提供的事实和为其背书的引用。不包含“未来还有三个修订”“下个月撤回”等排除明细。行政审计可以显式请求独立的 `AdministrativeDiagnostics`，该对象不得进入策略/特征计算输入。

即使核心选择结果正确，给策略暴露未来存在性也会引入泄漏，因此管理解释与决策接口必须分开。结构化 reason_code 由消费方记录指标；本库不写日志或发送 telemetry。

### 16.3 复现清单

| 必需项 | 作用 |
| --- | --- |
| 输入 DatasetSnapshotRef | 固定版本、元数据和证据集合 |
| Knowledge basis / known_at | 固定知识口径和截止点 |
| boundary / SystemKnowledgeCut | 固定实际系统视图；仅适用于系统模式 |
| 所选 authority_scope+FactKey+RevisionId+digest | 固定真实使用内容，不能只保留日期 |
| 来源/接收/可消费 evidence refs | 说明可见性结论的依据 |
| policy/schema/calendar/timezone/extractor refs | 固定解释规则 |
| temporal commit 与消费方 commit | 固定实现 |
| StatePolicy / valid_at | 有效状态查询使用 |
| lineage manifest | 派生查询使用 |
| completeness/acceptance evidence refs | 说明输入完整性和信任边界 |

manifest 的编码、哈希和签名由消费方版本化实现，库输出确定性字段顺序和引用，不把 std 默认 Hash 实现当作跨版本稳定的内容摘要。

### 16.4 缓存与分页

缓存键 MUST 包含 snapshot、knowledge basis、known_at、authority_scope、来源或消费边界作用域、system cut、policy/schema 版本和有效状态参数。仅按 `(series_id, date)` 缓存不合格。

分页必须绑定相同快照和查询合同。不得在每页之间重新取 latest 或切换 cut；未完成枚举不形成 Complete coverage。快照过期/数据缺失返回 `SnapshotUnavailable`，不得静默退回最新数据。

<a id="s17"></a>

## 17. 派生数据、特征、模型与模拟

### 17.1 Lineage 合同

派生记录至少绑定输出版本、输入 revision/digest 集合或不可变清单、输入查询上下文、计算代码/参数/schema 版本、实际或模拟执行模式、完成事件和消费边界证据。

对真实系统产物，派生 system_available 的保守界限不得早于任一实际使用输入在对应边界的可用性，也不得早于自身计算完成和输出被目标边界允许消费的时间。跨时钟比较使用相同受信时间解释或明确的上下界证据。

仅设置 `max(input.available_at)` 不够，因为计算和交付还可能发生在之后。使用输入的 event_time 最大值作为可用时间同样不合格。

### 17.2 两种执行模式

| 模式 | 含义 | 禁止行为 |
| --- | --- | --- |
| ActualExecution | 真实系统曾经计算并在边界可消费 | 用今日离线重算伪造过去的 system availability |
| CounterfactualSimulation | 基于历史可得输入和声明延迟模型进行研究性模拟 | 声称模拟时间是当年真实系统交付证据 |

反事实模拟可以今天运行，但必须证明每个模拟决策的输入符合其 declared knowledge basis/cutoff。模拟结果保存当前生成时间、模拟知识时间、延迟模型与执行模式，不能混成一个 timestamp。

### 17.3 数据链控制

PIT join 必须针对各输入自身可见修订，而非先对全量最终值 join 再过滤。滚动窗口必须使用当时可得输入；标准化/模型拟合不得无声明地使用决策之后的数据。标签可描述未来结果，但必须与该时点可用特征隔离，并按训练数据策略管理。

模型版本也有训练数据快照、训练完成、审批/部署和可消费边界；不能因为训练样本期间早于 T，就推断该模型当时已经可用。这些训练/计算逻辑在消费方实现，temporal 只提供必要时间与版本合同。

<a id="s18"></a>

## 18. 必须冻结的不变量

| ID | 不变量 |
| --- | --- |
| TP-INV-001 | 一个最终消费闭包只有一个 UnixTimeNs 类型权威 |
| TP-INV-002 | 任何整数时间输入必须由 API/字段名明确单位 |
| TP-INV-003 | 溢出、反向差值和范围错误不得 wrap、饱和或回填 now |
| TP-INV-004 | 未知不是 epoch，未知也不是不适用 |
| TP-INV-005 | 墙钟、单调点、日历标签互不冒充 |
| TP-INV-006 | 时间坐标分辨率不等于采样准确度 |
| TP-INV-007 | 纯规则不读取系统时间或外部状态 |
| TP-INV-008 | publication/revision 描述当前版本，不描述整个序列 |
| TP-INV-009 | source revision 与 receipt/availability 是不同身份 |
| TP-INV-010 | 相同来源修订的载荷及固有语义不可无痕覆盖 |
| TP-INV-011 | 多次接收同版不制造内容冲突或新权威修订 |
| TP-INV-012 | SourcePublishedAsOf 与 SystemAsKnown 不隐式切换 |
| TP-INV-013 | 系统 receipt 不自动证明目标消费者可见 |
| TP-INV-014 | 系统 cut 必须绑定 boundary、epoch、截止点和一致视图 |
| TP-INV-015 | 日期粒度或未知时区不能伪造精确日内发布时间 |
| TP-INV-016 | 在可见候选中选版，不能全量 latest 后过滤 |
| TP-INV-017 | 撤回参与选版，不能先去掉 tombstone |
| TP-INV-018 | 旧版晚到不能击败权威较新的可见修订 |
| TP-INV-019 | 不可比/冲突顺序不能由 UUID 或接收顺序裁决 |
| TP-INV-020 | 潜在改变结果的 Indeterminate 不能静默降为不存在 |
| TP-INV-021 | 不完整候选集合不能声称严格最新版本 |
| TP-INV-022 | 合法的未来可见追加不改变历史语义结果 |
| TP-INV-023 | 固定快照、策略、上下文与实现，结果可复现 |
| TP-INV-024 | 输入排列/合法重复不改变所选版本 |
| TP-INV-025 | 不同 authority_scope、事实/来源/期间不得因相同 FactKey 字节、timestamp 或 revision_id 合并 |
| TP-INV-026 | known_at 与 valid_at 是不同查询维度 |
| TP-INV-027 | 有效区间推导不能使用当时不可见的未来替换 |
| TP-INV-028 | 决策输出不携带未来候选的存在性或内容 |
| TP-INV-029 | 派生真实可用性不早于输入和实际计算/交付 |
| TP-INV-030 | 研究性模拟不冒充真实系统历史 |
| TP-INV-031 | 默认 Reject 精度损失；负数向零/floor 不混用 |
| TP-INV-032 | 读回投影必须先对齐检查，再与权威投影比较 |
| TP-INV-033 | kernel 与 temporal 最终无永久依赖 |
| TP-INV-034 | 错误分类本地化，消费方决定重试/降级 |
| TP-INV-035 | API/测试/包构建与部署证据不能相互冒充 |
| TP-INV-036 | 绝对观测窗口与民用日历期间必须区分；所有标准区间明确闭开规则 |
| TP-INV-037 | 缺乏源时间的采样不得用 received_time 伪造 event_time 或 publication_time |
| TP-INV-038 | POSIX、UTC 闰秒标签、TAI/GPS 与 smear 坐标未经明确解释不能混用 |
| TP-INV-039 | 原始时间字段不可覆盖；单位、精度和映射变更须可追溯且版本化 |
| TP-INV-040 | 时间戳不是来源事件身份、订单簿连续性或跨分区总序 |
| TP-INV-041 | K 线窗口终点、到期或结算时点不自动证明最终值已可消费 |
| TP-INV-042 | 临时/最终、来源修订、规范化修订及重传分别有合同 |
| TP-INV-043 | 历史补齐固定时间轴与 [start,end)，分页不能不断扩大目标范围 |
| TP-INV-044 | 当前快照不能证明历史状态路径或回填历史深度覆盖 |
| TP-INV-045 | Checkpoint、CoverageLedger 与 PIT CandidateCoverage 不互相代替 |
| TP-INV-046 | 水位线仅按声明策略解释流处理进度，不天然证明来源完整性 |
| TP-INV-047 | 重放保留原 receipt/cut；重放接收与系统消费证据另记 |
| TP-INV-048 | wire 中缺字段、Unknown、NotApplicable 与已知 epoch 必须可区分 |
| TP-INV-049 | 纳秒权威整数在 JSON/跨语言链路中不得经浮点中转 |
| TP-INV-050 | 产品 DTO/后端投影不得反向成为 temporal 的运行时或序列化依赖 |
| TP-INV-051 | 查询进行中策略、schema、mapping 与 snapshot 不能被热更新暗改 |
| TP-INV-052 | 今日重新规范化/重算不能冒充历史系统已消费的内容版本 |
| TP-INV-053 | 管理诊断、未知修订与档案质量错误不得成为策略的未来信息特征 |
| TP-INV-054 | 库级、消费方、迁移、真实系统验收的 Owner 与证据必须分开 |
| TP-INV-055 | 历史约束、当前提案、文档检查及真实执行证据不得混称已完成 |

### 18.1 未来追加不污染的精确定义

固定历史集合 S、策略 P 和 cutoff T。S′ 是 S 的合法扩展，新增版本在相同知识口径中都被证明只在 T 之后可见；旧版本、旧证据和解释规则未修改，历史 coverage 成立。比较 S 与 S′ 各自不可变快照下、除 snapshot 标识外相同的查询：

```text
semantic_result(select(S, T, P)) == semantic_result(select(S′, T, P))
```

新增快照引用、扫描计数、管理审计明细可以不同；所选事实/修订/操作/状态必须相同。对同一个旧 snapshot 重放则连固定 manifest 引用都应保持一致。

如果 S′ 新增的是“确实在 T 之前已发布但当时档案缺失”的来源历史版本，它不符合“未来可见追加”的前提，可以改善新的来源研究快照；不得修改原 S。这一例外不允许回填系统历史接收。

### 18.2 候选完整性的前提

库只能对其受信、完整且已固定的输入作上述保证。源档案缺失、适配器漏候选或伪造时间证据会破坏前提，必须在产品验收中覆盖，不能通过标注一个 PIT 字段来免除。

<a id="s19"></a>

## 19. 错误分类与公共失败合同

### 19.1 四组本地错误

所有错误必须实现 Debug/Display/std::error::Error，且为 Send+Sync+'static。Display 用于人类阅读，不构成机器协议；使用稳定 `code() -> &'static str` 或明确枚举匹配。不提供全局可重试策略。

| 错误组 | 必须可区分的情况 | 示例稳定 code |
| --- | --- | --- |
| TimeError | Overflow、InvalidOrder、InvalidSubsecond、SystemTimeOutOfRange、SystemTimePrecisionLoss、SourceUnavailable | TIME_OVERFLOW / TIME_INVALID_ORDER / TIME_SYSTEM_RANGE |
| PrecisionError | PrecisionLoss、Overflow、MisalignedProjection、ProjectionMismatch、LosslessRequired | PREC_LOSS_REJECTED / PREC_OVERFLOW / PREC_MISALIGNED |
| TemporalError | MissingRequiredRole、RoleNotApplicable、InvalidDate、InvalidPeriod、InvalidInterval、ProfileMismatch、MetadataMismatch | TEMP_ROLE_REQUIRED / TEMP_PERIOD_INVALID / TEMP_PROFILE_MISMATCH |
| PitError | ContextMismatch、EvidenceMismatch、ConflictingRevision、AmbiguousRevisionOrder、IncomparableOrderScope、InvalidRevisionChain、IndeterminateSelection、UnprovenSystemCut、IncompleteCandidateSet、SnapshotUnavailable、ConflictingEffectiveState、LimitExceeded、ResourceUnavailable | PIT_EVIDENCE_MISMATCH / PIT_INDETERMINATE / PIT_INCOMPLETE / PIT_LIMIT |

所有已列入本版的公开错误变体按下表固定机器 code；API.md 必须与之逐项一致。新增错误需要规范修订，不允许共用一个模糊 `OTHER`。

| 错误变体 | 固定机器 code |
| --- | --- |
| TimeError::Overflow | TIME_OVERFLOW |
| TimeError::InvalidOrder | TIME_INVALID_ORDER |
| TimeError::InvalidSubsecond | TIME_INVALID_SUBSECOND |
| TimeError::SystemTimeOutOfRange | TIME_SYSTEM_RANGE |
| TimeError::SystemTimePrecisionLoss | TIME_SYSTEM_PRECISION_LOSS |
| TimeError::SourceUnavailable | TIME_SOURCE_UNAVAILABLE |
| PrecisionError::PrecisionLoss | PREC_LOSS_REJECTED |
| PrecisionError::Overflow | PREC_OVERFLOW |
| PrecisionError::MisalignedProjection | PREC_MISALIGNED |
| PrecisionError::ProjectionMismatch | PREC_PROJECTION_MISMATCH |
| PrecisionError::LosslessRequired | PREC_LOSSLESS_REQUIRED |
| TemporalError::MissingRequiredRole | TEMP_ROLE_REQUIRED |
| TemporalError::RoleNotApplicable | TEMP_ROLE_NOT_APPLICABLE |
| TemporalError::InvalidDate | TEMP_DATE_INVALID |
| TemporalError::InvalidPeriod | TEMP_PERIOD_INVALID |
| TemporalError::InvalidInterval | TEMP_INTERVAL_INVALID |
| TemporalError::ProfileMismatch | TEMP_PROFILE_MISMATCH |
| TemporalError::MetadataMismatch | TEMP_METADATA_MISMATCH |
| PitError::ContextMismatch | PIT_CONTEXT_MISMATCH |
| PitError::EvidenceMismatch | PIT_EVIDENCE_MISMATCH |
| PitError::ConflictingRevision | PIT_REVISION_CONFLICT |
| PitError::AmbiguousRevisionOrder | PIT_REVISION_ORDER_AMBIGUOUS |
| PitError::IncomparableOrderScope | PIT_ORDER_SCOPE_INCOMPARABLE |
| PitError::InvalidRevisionChain | PIT_REVISION_CHAIN_INVALID |
| PitError::IndeterminateSelection | PIT_INDETERMINATE |
| PitError::UnprovenSystemCut | PIT_SYSTEM_CUT_UNPROVEN |
| PitError::IncompleteCandidateSet | PIT_INCOMPLETE |
| PitError::SnapshotUnavailable | PIT_SNAPSHOT_UNAVAILABLE |
| PitError::ConflictingEffectiveState | PIT_EFFECTIVE_STATE_CONFLICT |
| PitError::LimitExceeded | PIT_LIMIT |
| PitError::ResourceUnavailable | PIT_RESOURCE_UNAVAILABLE |

NotVisible 的稳定 reason_code 至少固定为 `RELEASE_AFTER_CUTOFF`、`SYSTEM_AVAILABLE_AFTER_CUTOFF`、`OUTSIDE_SYSTEM_CUT`。Indeterminate 的原因固定为 `SOURCE_TIME_UNPROVEN`、`SYSTEM_TIME_UNPROVEN`、`PRECISION_INSUFFICIENT`、`RECEIPT_UNPROVEN`、`RELEASE_CONDITION_UNPROVEN`。身份/快照不匹配是 Err，不属于合法 NotVisible。原因代码只表达判定类别，不包含未来载荷；行政明细仍遵守 §16。

### 19.2 行为要求

输入无效返回 Err；明显未来版本返回 NotVisible；证据不足返回 Indeterminate；确无当时可见事实返回 NoVisibleVersion；已撤回返回 Withdrawn。用户取消批量外部任务与库计算失败由调用方分别表示，本库不自创全平台 CancellationToken。

错误不能返回完整敏感载荷或授权 token。正常数学错误路径不分配字符串；将事实键和证据引用附加到详细报告时，受资源上限控制。不得 `unwrap/expect/panic` 处理可由输入触发的错误。

### 19.3 与 kernel 的适配

不提供 `kernel` feature，不把 TimeError 自动转换为 XError。消费方可定义自由转换函数，或在自己的 Error 新类型中包裹错误。第三方桥接 crate 不拥有 kernel::XError、temporal::TimeError 和 std::From，不能直接实现二者的 From 映射。[R5]

外部 non_exhaustive 枚举的匹配必须有保守兜底，未知错误不能被当作成功、可见或允许截断。[R6] 基础转换溢出究竟是 Invalid、Unavailable 或 Invariant 由实际调用上下文决定，不在时间库中统一猜测。

<a id="s20"></a>

## 20. 公开 API 名册与输入模型

### 20.1 稳定入口原则

crate 根只 re-export 已登记类型/函数。内部文件布局不成为公开路径合同。新 API、语义变化、feature 和生产依赖必须先修改 SPEC/API 清单，再实现并测试。

以下签名为目标合同清单，尚未编译；实现需补齐 Rust 生命周期、必要 trait bounds、构造器及 rustdoc，不得将此文档代码片段冒充完整源码。

### 20.2 类型族

| 类型族 | 必须公开的概念 | 关键限制 |
| --- | --- | --- |
| 基础 | UnixTimeNs、TimeError | 唯一绝对时刻 |
| 时钟 | WallClock、MonotonicClock、RuntimeClock、SystemWallClock、SystemMonotonicClock | 不公开测试运行时服务 |
| 期间 | CivilDate、ObservationTime、AbsoluteTimeRange、ObservationPeriod、ObservationGranularity、EffectiveInterval | 字段通过检查构造 |
| 模型 | TimeField、TemporalDraft、Temporal、TemporalProfile、SourceTemporalRef | 已验证模型不可变 |
| 精度 | TimePrecision、PrecisionLossPolicy、TimeStorageCapabilities、PrecisionError | 具名单位入口 |
| 身份/版本 | FactKeyRef、RevisionIdRef、DigestRef、PayloadRef、RevisionOperation、RevisionOrderRef、FactVersionRef | 不拥有业务实体定义或载荷内容 |
| 证据 | TimeBounds、SourceVisibilityEvidenceRef、ReceiptEvidenceRef、SystemVisibilityEvidenceRef、DatasetSnapshotRef、SystemKnowledgeCutRef | 不把结构校验当真实性证明 |
| 查询 | AsOfContext、QueryPolicy、KnowledgeBasis、CandidateCoverageRef、QueryInputRef、VisibilityDecision、FactOutcome、PitResult | 无隐式 now/latest/default |
| 有效状态 | EffectiveStateInputRef、StatePolicy、EffectiveStateResult | 不自动裁定业务冲突 |
| 错误/诊断 | TemporalError、PitError、DecisionReason、AdministrativeDiagnostics | 不泄漏未来到决策输入 |

`Ref` 后缀表示借用外部不可变数据或标识；不是网络远程引用解析器。库不自行获取 ref 指向的内容。

### 20.3 规范化标识

FactKeyRef / RevisionIdRef 是带验证的非空规范化字节串借用，限制最大长度，由 QueryPolicy 确定。必须进行精确字节相等比较，不能 trim、大小写折叠或 Unicode 自动归一导致身份变化。消费方 schema 决定如何生成规范化键。

DigestRef 至少有算法/编码版本和摘要字节，PayloadRef 有存储引用身份/版本；它们只用于绑定和比较，不证明原始内容可用。解引用和摘要校验在受信适配器执行。

### 20.4 FactVersionRef 的逻辑字段

| 字段 | 类型/约束 |
| --- | --- |
| authority_scope | 当前候选版本域的明确非空引用；查询作用域与顺序证据必须匹配 |
| fact_key / revision_id | 经过格式/大小校验的引用；事实完整键与authority共同确定身份 |
| input_provenance_ref | 产品版本到原始来源/capture、mapping、schema和转换清单的不可变绑定；库不自行解引用 |
| operation | Upsert / Withdraw |
| semantic_digest | 当前候选不可变语义与实际载荷的摘要；与原始raw_digest区分 |
| payload_ref | Upsert 必需；Withdraw 不携带值 |
| source_temporal | SourceTemporalRef，含五种固有时间角色，不含 received_time；其来源/产品含义由authority明确 |
| revision_order | 明确 scope+sequence 或合法前驱关系 |
| source_evidence | 对应版本的来源可用性证据 |
| receipts | 独立接收事件引用集合 |
| system_evidence | 按消费边界分组的可消费事件证据 |
| effective_interval | 可选；与 effective_time 一致 |

`Temporal::source_roles()` 产生只读 SourceTemporalRef，用于将六角色接收视图中不含received的五个固有角色投影出来，不创建另一套时间语义。SourceTemporalRef这个名称不证明外部来源真实性；产品观察/派生版本的角色仍按其authority与profile解释。系统可见性从本次 cut 中的 receipts/system_evidence 选择，不能误用某个事后重投影的 received_time 提前或推迟历史可见性。

### 20.5 必需函数合同

| 入口 | 返回 | 失败与保证 |
| --- | --- | --- |
| `AbsoluteTimeRange::try_new(start, end_exclusive)` | Result<AbsoluteTimeRange, TemporalError> | 有限非空半开范围；端点为 UnixTimeNs |
| `AbsoluteTimeRange::intersection(other)` | Option<AbsoluteTimeRange> | 无交集/仅相邻返回 None；不创建空范围 |
| `TemporalDraft::validate(profile)` | Temporal | 角色合法；不承诺可见性证据充分 |
| `TimeBounds::try_new(lower, upper, quality_ref)` | TimeBounds | 检查区间与来源说明 |
| `AsOfContext::try_new(parts)` | AsOfContext | mode 所需 scope/cut/snapshot 完整 |
| `FactVersionRef::try_new(parts)` | FactVersionRef | 格式、操作/载荷和本地身份一致 |
| `evaluate_visibility(version, context)` | Result<VisibilityDecision, PitError> | 只评估本版；不声明全组最新 |
| `select_fact_as_of(candidates, coverage, context)` | Result<FactOutcome, PitError> | 单一 authority_scope + 完整 FactKey；跨作用域或跨事实输入拒绝 |
| `select_facts_as_of(input, context)` | Result<PitResult, PitError> | 单一完整 authority_scope 内的事实集合；严格全量结果 |
| `inspect_candidates(input, context)` | Result<AdministrativeDiagnostics, PitError> | 显式、数量有界的行政诊断；含未来候选身份，不能进入决策输入 |
| `select_effective_state_as_of(input, context, valid_at, state_policy)` | Result<EffectiveStateResult, PitError> | 先 PIT、后状态；重叠冲突拒绝 |
| `verify_projection(authoritative, projected, precision, policy)` | Result<(), PrecisionError> | 对齐与明确舍入一致 |
| `require_lossless_unix_ns(capabilities)` | Result<(), PrecisionError> | 只验证能力声明中的此项 |

Parts 输入结构必须在 API.md 中逐字段登记，不得通过无版本 `HashMap<String, Value>` 把必须条件转移给运行时猜测。所有快照、cut、coverage 和 context 在实际调用时重新检查绑定关系，不能复用在不同查询验证过的输入来绕过条件。

### 20.6 编译合同

正向：根路径可用；UnixTimeNs 是 Copy/Send/Sync；时钟 trait 可用于 dyn；组合对象真实同时实现两类接口；所有错误 Send/Sync。负向：私有字段不可构造、无 Default/serde/无单位 From、Instant 无库提供的 wire 入口、内部模块路径不可依赖。

所有 compile_fail 必须配套合法路径的正向样例；不得因导入不存在而假通过。可演进枚举使用 non_exhaustive，外部消费者采用保守未知分支；TimeField 的三态闭合集合在本版显式固定，增补状态属于协议变更。

<a id="s21"></a>

## 21. 并发、资源、性能与安全边界

### 21.1 并发模型

基础与模型值不可变；选择器无全局状态、无后台任务、无隐藏锁、无连接池。可由任意同步/异步应用并行调用，但本库不提供无实际等待点的 async 包装，也不引入独立线程池。

输入 payload 不复制；可以借用只读结构、使用索引排序。不同线程必须对同一不可变 snapshot 和相同策略得到等价结果；调用方的可变外部 store 不能通过共享引用伪装成已固定输入。

### 21.2 复杂度与上限

标准批量入口目标为 O(n log n) 时间、O(n) 工作内存；Sequence 分组后比较 O(n)，链校验/排序不得退化到不受控 O(n²)。输出按规范化 FactKey 稳定排序，HashMap 迭代顺序不得决定答案。

QueryPolicy 必须含候选总数、每事实版本数、键/引用最大字节数、证据数、链深、诊断条目和允许工作内存预算。达到上限返回 LimitExceeded，不静默截断并返回完整结果。可分批处理的调用方必须保留跨页完整性与 snapshot，或在完整性声明前不输出最终结论。

本版不承诺流式提前产生“最终 latest”；尚未证明后续没有更优可见修订时，只能输出非最终内部状态。

工作内存预算需要在申请前检查；实现应使用可失败预留，能捕获的分配失败返回 ResourceUnavailable，与请求超配额的 LimitExceeded 区分。操作系统/进程分配器的致命 OOM 不属于可恢复性保证，不得声称资源上限使进程永不耗尽内存。

### 21.3 分配与基准

UnixTimeNs、区间数值检查、单个精度换算和结构化数学错误路径应零堆分配。单条可见性检查可借用证据完成；批量索引、结果与管理诊断允许有界分配。

基准至少覆盖：单位构造、checked add/sub、SystemTime 转换、三种舍入、六角色校验、单条可见性、1/10/100 个修订选版、10³/10⁵ 候选批量、乱序/重复/撤回/证据不足。记录 CPU、OS、target、工具链、优化级别、样本、分配次数与峰值工作内存。

固定环境的中位耗时或内存较已冻结基线回退超过 10% 时阻断并审议；噪声大的共享 CI 结果只能作为信号，不能用一次跑分虚构硬件无关 SLO。不提出没有测量依据的固定 ns/QPS 承诺。

### 21.4 信任与安全

标识、证据引用和诊断是外部输入，必须限制大小、链深和集合规模。禁止递归遍历无限链、无界日志输出或恶意分支导致资源耗尽。库禁止 unsafe；错误不包含凭证/敏感载荷。

source_scope 是时间可见性作用域，不是授权票据。消费方仍须在读取数据前证明授权/许可，不因 PIT 通过而扩大访问权限。不能把不可验证数据标成受信，也不能以“库不做 I/O”为理由省略集成信任边界。

<a id="s22"></a>

## 22. 必须实现的验收测试矩阵

本版共 180 个必选验收场景：本章保留 TP-T001–TP-T120，§33 补充 TP-T121–TP-T180。它们是需要落地的合同场景，不是本次已经运行的 Rust 测试。其中消费方适配、调度、存储及回测测试由对应消费仓执行；本仓只承担自身能力和纯合同 fixture，具体归属见 §32–33。初始执行状态全部为 `NOT_RUN`；实现时须映射具体测试函数、断言、工具链和日志。一个场景可拆成多个测试函数，但不得只用测试名或源码字符串证明行为。

### 22.1 独立性与公开 API

| 测试 ID | 场景 | 输入/前提 | 必须断言 | 不变量 |
| --- | --- | --- | --- | --- |
| TP-T001 | 单仓冷启动 | 仅检出 temporal，无相邻内部仓库 | 构建、测试、示例均不依赖 kernel/workspace/testkit | TP-INV-033, TP-INV-035 |
| TP-T002 | 生产依赖检查 | 检查 cargo metadata 全闭包 | 除已审核公开依赖外无业务/运行时/存储依赖 | TP-INV-033 |
| TP-T003 | 唯一类型身份 | 一个真实消费者依赖多个产品 crate | 同一 UnixTimeNs 来源/版本；无平行 Timestamp 新类型 | TP-INV-001 |
| TP-T004 | 禁止无单位构造 | 外部 crate 尝试 From<i64>/Into<i64> | 编译失败且合法具名构造对照编译通过 | TP-INV-002 |
| TP-T005 | 未知时间无默认值 | 外部 crate 尝试 UnixTimeNs/Temporal/AsOfContext::default | 编译失败；epoch 仅显式构造 | TP-INV-004 |
| TP-T006 | 序列化边界 | dev 环境确有 serde trait | UnixTimeNs/Instant 包装无库提供的默认 wire 旁路 | TP-INV-005 |
| TP-T007 | 私有字段边界 | 外部尝试构造 UnixTimeNs/已验证 Temporal 字段 | 编译失败，Draft 合法构造通过 | TP-INV-001, TP-INV-004 |
| TP-T008 | 内部模块不可依赖 | 尝试内部实现文件路径与 crate 根路径 | 前者失败、后者成功；负向测试不靠失效导入假通过 | TP-INV-035 |
| TP-T009 | 非穷尽枚举消费 | 外部消费 TimeError/PrecisionLossPolicy | 未知分支保守报错，不允许默认可见或截断 | TP-INV-034 |
| TP-T010 | 根导出名册一致 | 枚举 API.md 与实际公开 surface | 每个稳定入口有正向编译及对应行为测试 | TP-INV-035 |

### 22.2 数值基础与算术

| 测试 ID | 场景 | 输入/前提 | 必须断言 | 不变量 |
| --- | --- | --- | --- | --- |
| TP-T011 | epoch 与任意纳秒 | 0、1、-1、i64::MIN/MAX | 具名纳秒构造与导出完全相等 | TP-INV-002, TP-INV-003 |
| TP-T012 | 单位正负构造 | 正负 seconds/millis/micros 的安全范围 | 精确乘 scale，无符号丢失 | TP-INV-002, TP-INV-003 |
| TP-T013 | 单位乘法溢出 | 各单位正负边界内外各一值 | 边界内成功、边界外 Overflow | TP-INV-003 |
| TP-T014 | 普通加减可逆 | 随机 t 与可表示 Duration | 成功加 d 后减 d 返回原 t | TP-INV-003 |
| TP-T015 | 极值加减 | MAX+1ns、MIN-1ns | 返回 Overflow，不 wrap/panic | TP-INV-003 |
| TP-T016 | 全 i64 跨度 | MAX.duration_since(MIN) | Duration::from_nanos(u64::MAX) | TP-INV-003 |
| TP-T017 | 反向与相等差 | earlier.duration_since(later)、t.duration_since(t) | 前者 InvalidOrder，后者 ZERO | TP-INV-003 |
| TP-T018 | 最大 Duration | Duration::MAX 加减与跨界值 | 结构化错误，无中间值溢出 panic | TP-INV-003 |
| TP-T019 | 负子秒标准化 | -1ns、-1000000001ns | Euclidean 秒/非负子秒；可精确重建 | TP-INV-003 |
| TP-T020 | 子秒输入校验 | nanos=999999999、1000000000 和极端 seconds | 合法边界成功；非法子秒或结果范围报错 | TP-INV-003 |

### 22.3 SystemTime 与时钟

| 测试 ID | 场景 | 输入/前提 | 必须断言 | 不变量 |
| --- | --- | --- | --- | --- |
| TP-T021 | SystemTime 正负 epoch | 平台支持的 epoch 前后可表示点 | 无损转换，不再一律拒绝负数 | TP-INV-003 |
| TP-T022 | SystemTime 极负值 | i64::MIN 转平台后读回 | 成功即严格相等；否则明确范围/精度错误 | TP-INV-003 |
| TP-T023 | SystemTime 平台精度 | 构造平台不能保留的亚单位 ns | SystemTimePrecisionLoss；不能成功静默丢位 | TP-INV-003, TP-INV-006 |
| TP-T024 | SystemTime 超范围 | 平台或 i64 ns 不能表示的值 | SystemTimeOutOfRange，不默认 epoch | TP-INV-003 |
| TP-T025 | 墙钟允许回退 | ManualWallClock 连续给 100、90、110 | 原样保留采样，不能强制递增 | TP-INV-005, TP-INV-006 |
| TP-T026 | 单调测试推进 | ManualMonotonicClock 正常推进及负向请求 | 合法前进；后退或 anchor+duration 溢出拒绝 | TP-INV-005 |
| TP-T027 | 组合接口真实成立 | 同一测试对象实现 WallClock+MonotonicClock | RuntimeClock 成立；两种 now 显式调用 | TP-INV-005 |
| TP-T028 | 单类时钟不冒充组合 | 只实现 WallClock 的对象 | 不能当 RuntimeClock；负向外部编译通过检测 | TP-INV-005 |
| TP-T029 | PIT 不读取 now | 固定输入改变系统/测试时钟状态 | PIT 结果不变；纯函数未发生外部读取 | TP-INV-007 |
| TP-T030 | 虚拟推进不冒充等待 | 推进测试时钟，另有真实运行时等待 | 测试文档/接口不宣称控制该等待；无长 sleep 纯规则测试 | TP-INV-007, TP-INV-035 |

### 22.4 时间角色与日历

| 测试 ID | 场景 | 输入/前提 | 必须断言 | 不变量 |
| --- | --- | --- | --- | --- |
| TP-T031 | 日期合法性 | 闰年 2/29、非闰年 2/29、非法月日 | 仅合法 CivilDate 通过 | TP-INV-004 |
| TP-T032 | 期间半开与边界 | start=end、反向、合法月首到下月首 | 空/反向拒绝；合法 [start,end) 通过 | TP-INV-026 |
| TP-T033 | 粒度不是固定秒数 | 二月、三月、财政季度 | 公历期间正确；财政季度 Custom，不用固定 30 天 | TP-INV-006 |
| TP-T034 | 日历身份 | 相同起止但 calendar_ref 不同 | 不能无声当作相同观测事实 | TP-INV-025 |
| TP-T035 | EventRecord 必需项 | 分别删除 event_time/received_time | 结构校验拒绝；publication 可明确不适用 | TP-INV-004 |
| TP-T036 | PublishedObservation 粗时间 | 观测已知、publication Unknown 且有日期证据 | 结构可合法；日内 PIT 仍需单独证据判断 | TP-INV-015 |
| TP-T037 | Announcement 提前公告 | publication < known_at < effective_time | 知识视图可见，状态视图尚未生效 | TP-INV-026 |
| TP-T038 | DerivedRecord 语义 | 本地产物接纳时间和观测字段 | 不将本地接纳冒充外部发布，缺语义锚点拒绝 | TP-INV-009, TP-INV-029 |
| TP-T039 | Unknown 与 NotApplicable | 必需字段 NA、可未知字段 Unknown、epoch Known | 三种结果不混同，无自动补零 | TP-INV-004 |
| TP-T040 | 六角色不是排序链 | 未来预测观测、source/client 时钟偏差 | 不使用统一 event<=received 或 observation<=known_at 规则 | TP-INV-005, TP-INV-026 |

### 22.5 精度、量化与投影

| 测试 ID | 场景 | 输入/前提 | 必须断言 | 不变量 |
| --- | --- | --- | --- | --- |
| TP-T041 | 默认拒绝精度损失 | 无显式策略、含亚毫秒 ns | 默认 Reject，返回 PrecisionLoss | TP-INV-031 |
| TP-T042 | 向零正负对称 | ±1500000001ns 转 ms | ExplicitTruncate 分别得到 ±1500 | TP-INV-031 |
| TP-T043 | floor 负边界 | -1ns 转 us | 得到 -1us，不是向零策略的 0 | TP-INV-031 |
| TP-T044 | 无损整单位 | 正负精确毫秒/微秒值 | Reject 下成功并可还原 | TP-INV-031 |
| TP-T045 | 存储乘法溢出 | 极值 stored×scale | Overflow，不 wrap | TP-INV-003, TP-INV-031 |
| TP-T046 | floor 还原下溢 | MIN 附近量化到秒 | 若还原小于 i64::MIN，返回 Overflow | TP-INV-003, TP-INV-031 |
| TP-T047 | ns 单位恒等 | 全域随机 i64 转 ns | 所有策略都保持原值 | TP-INV-031 |
| TP-T048 | 未对齐投影 | authoritative=1ns、readback=999ns、us | 先判读回未对齐并拒绝，不能双重量化通过 | TP-INV-032 |
| TP-T049 | 投影值不一致 | 对齐但不等于指定策略的权威投影 | ProjectionMismatch | TP-INV-032 |
| TP-T050 | 能力声明边界 | lossless=true 但 UTC/范围未证明 | 窄函数只验证声明；完整适配器验收不能标为已通过 | TP-INV-006, TP-INV-035 |

### 22.6 事实身份与不可变修订

| 测试 ID | 场景 | 输入/前提 | 必须断言 | 不变量 |
| --- | --- | --- | --- | --- |
| TP-T051 | 同版同内容重复 | 相同 FactKey+RevisionId+semantic_digest | 幂等一个版本；传输证据可以多条 | TP-INV-010, TP-INV-011 |
| TP-T052 | 同版载荷冲突 | 同键同版、不同 semantic_digest | ConflictingRevision，不覆盖 | TP-INV-010 |
| TP-T053 | 同版不同接收时间 | 内容不变、两个边界分别 receipt | 不是内容冲突，也不生成新来源修订 | TP-INV-009, TP-INV-011 |
| TP-T054 | 相同标识跨事实或权威作用域 | 两个 source/period 使用相同 revision_id；两个 authority_scope 复用相同 FactKey+RevisionId | 分别保存与选择，不跨事实或权威作用域去重；混合单作用域查询拒绝 | TP-INV-025 |
| TP-T055 | 同纳秒多事件 | event_time 相同、源事件 ID 不同 | 保持两个事实 | TP-INV-025 |
| TP-T056 | 旧版晚到 | 先收到 ordinal=2，再收到 ordinal=1 | 当前可见选择仍为 2 | TP-INV-018 |
| TP-T057 | 同序号冲突 | 同事实同 order_scope、ordinal 相同且修订不同 | AmbiguousRevisionOrder | TP-INV-019 |
| TP-T058 | 来源代次重置 | 同名 source、不同 order_scope/epoch | 不直接比较 ordinal，缺映射则拒绝 | TP-INV-019 |
| TP-T059 | 非法前驱关系 | 跨事实前驱、循环或缺必要链关系 | InvalidRevisionChain，不无限遍历 | TP-INV-019 |
| TP-T060 | 分支修订 | 同一前驱的两个不同可见后继 | 不按 UUID 选一个，返回明确冲突 | TP-INV-019 |

### 22.7 来源可见性与不确定性

| 测试 ID | 场景 | 输入/前提 | 必须断言 | 不变量 |
| --- | --- | --- | --- | --- |
| TP-T061 | CPI 发布前 | 2026-08 观测，v1 于 9/10 发布；9/1 查询 | NoVisibleVersion | TP-INV-008, TP-INV-016 |
| TP-T062 | CPI 初值窗口 | 同事实 v2 于 10/10；9/20 查询 | 只选 v1，不能返回最终修订值 | TP-INV-008, TP-INV-022 |
| TP-T063 | CPI 修订之后 | 10/11 查询同一合成数据 | 选择 v2 | TP-INV-008, TP-INV-016 |
| TP-T064 | 精确边界 | release=T-1ns、T、T+1ns | 前两者满足时间门禁，最后不满足 | TP-INV-015 |
| TP-T065 | 日内不确定区间 | 实际发布在 [Dstart,Dend]，T 在区间内 | Indeterminate，不能按 Dstart 提前放行 | TP-INV-015, TP-INV-020 |
| TP-T066 | 保守上界与精确模式 | 已接受的实际发布区间 upper<=T，分别用两种证据策略 | ConservativeBounds 可通过；ExactOnly 即使 cutoff 已过仍为 Indeterminate，不借上界放行 | TP-INV-015 |
| TP-T067 | 计划日期不能充证据 | 只有 scheduled_release，没有实际发布证明 | Indeterminate 或缺少必需证据 | TP-INV-008, TP-INV-015 |
| TP-T068 | 旧版内容缺失 | 只有最新 B，却提供 v1 的旧发布日期 | 不得生成 v1 可见值，严格查询失败 | TP-INV-010, TP-INV-021 |
| TP-T069 | 来源补采口径 | 11 月获得真实 9 月版本归档证据 | 新来源快照可重建 9 月；不可伪造系统 receipt | TP-INV-012, TP-INV-022 |
| TP-T070 | 证据作用域或转换绑定不匹配 | 证据绑定另一版本/source audience/digest；规范化候选缺原来源与转换清单绑定 | 明确错配返回 EvidenceMismatch；缺绑定返回 Indeterminate；均不 Visible | TP-INV-008, TP-INV-012 |

### 22.8 系统可见性、cut 与覆盖

| 测试 ID | 场景 | 输入/前提 | 必须断言 | 不变量 |
| --- | --- | --- | --- | --- |
| TP-T071 | 接收不等于消费 | release 08:30:00、receipt 02、available 03 | 系统查询 01/02 不可见，03 需同时符合 cut | TP-INV-013 |
| TP-T072 | 同纳秒不同序号 | 两事件时间相同、cut 只含第一事件 | 第二事件不可见 | TP-INV-014 |
| TP-T073 | 未来提交旧墙钟 | 新事件 wall_time<T、但事件不属于旧 cut | 旧系统视图仍排除 | TP-INV-014, TP-INV-022 |
| TP-T074 | 边界隔离 | 边界 A 已可见、B 未可见 | A/B 查询不同，不能共享无 scope available_at | TP-INV-013, TP-INV-014 |
| TP-T075 | 日志代次隔离 | 分区名相同但 epoch 变化 | 不能错误比较旧新 offset | TP-INV-014 |
| TP-T076 | 多分区切片 | 只有部分分区达到约定 frontier | 缺完整性/一致性证明时不可声称完整系统视图 | TP-INV-014, TP-INV-021 |
| TP-T077 | 事务起点假提交 | recorded_time 早于 T，实际提交晚于 T | 系统视图排除；now 字段不充当提交证明 | TP-INV-013, TP-INV-014 |
| TP-T078 | 流式边界先于持久化 | 有真实可消费日志证据，DB 尚未持久化 | 按该边界策略判断，不强加 DB 完成条件 | TP-INV-013 |
| TP-T079 | 缺历史 cut | 只有当前快照和几个时间字段 | UnprovenSystemCut，不切换到来源模式 | TP-INV-012, TP-INV-014 |
| TP-T080 | 截断候选/证据分页 | 只返回 first page、latest 或缺 tombstone | IncompleteCandidateSet，无完整成功结果 | TP-INV-021 |

### 22.9 严格选版与撤回

| 测试 ID | 场景 | 输入/前提 | 必须断言 | 不变量 |
| --- | --- | --- | --- | --- |
| TP-T081 | 先可见再选修订 | v1 可见、较新 v2 尚未发布 | 返回 v1，不是空结果 | TP-INV-016 |
| TP-T082 | 撤回不能先过滤 | 最新可见修订为 Withdraw | 返回 Withdrawn，不复活旧 Upsert | TP-INV-017 |
| TP-T083 | 撤回后恢复 | 新 Upsert 的 release 晚于撤回 | 恢复前 Withdrawn；恢复可见后 Selected | TP-INV-017 |
| TP-T084 | 不确定更高修订 | 旧版确定可见、新修订证据不足 | IndeterminateSelection，不声称旧版确定最新 | TP-INV-020 |
| TP-T085 | 空可见集合 | 覆盖完整且所有候选明确未来 | NoVisibleVersion，不泄漏未来版本细节 | TP-INV-016, TP-INV-028 |
| TP-T086 | 合法输入乱序 | 对候选作多种排列 | 所选事实/修订与输出键顺序一致 | TP-INV-024 |
| TP-T087 | 合法重复输入 | 复制版本及同一证据的合法重复 | 结果幂等，不产生新修订或不同选择 | TP-INV-011, TP-INV-024 |
| TP-T088 | 链关系不能偷看未来 | 未来才出现的 supersedes 边 | 不能提前让旧修订失效 | TP-INV-016, TP-INV-022 |
| TP-T089 | 部分失败不能伪装全绿 | 批量请求中一个事实有潜在影响冲突 | 严格批量返回 Err，不返回貌似完整成功列表 | TP-INV-020, TP-INV-021 |
| TP-T090 | Facts 不是 LatestObservation | 同系列有两个观测期间 | 分别返回各事实版本，不默认折叠为一条 | TP-INV-025 |

### 22.10 知识与有效状态

| 测试 ID | 场景 | 输入/前提 | 必须断言 | 不变量 |
| --- | --- | --- | --- | --- |
| TP-T091 | 公告与生效分离 | 9/1 公告，10/1 生效，9/15 已知查询 | 知识 Selected；9/15 状态不激活新断言 | TP-INV-026 |
| TP-T092 | 区间起点包含 | valid_at == interval.start | Active | TP-INV-026 |
| TP-T093 | 区间终点排除 | valid_at == interval.end_exclusive | 不属于该断言 | TP-INV-026 |
| TP-T094 | 非法有效区间 | start>=end 或 effective_time 与 start 不同 | 拒绝构造/元数据不一致 | TP-INV-026 |
| TP-T095 | 未来状态已公告 | valid_at>known_at，只有已知未来公告 | 允许查询已知计划，不访问未公布版本 | TP-INV-026 |
| TP-T096 | 多个有效断言 | 同 state_key/valid_at 两个存活区间重叠 | ConflictingEffectiveState，不随便取最新 | TP-INV-019, TP-INV-026 |
| TP-T097 | 未来替换保留当前 | A 当前状态，B 不同断言未来生效 | 在 B 生效前不删除 A | TP-INV-026, TP-INV-027 |
| TP-T098 | 未来公告不截断历史 | B 尚不可见，却能推导 A 的未来 end | 不得使用 B 提前截断 A | TP-INV-027 |
| TP-T099 | 有效状态撤回 | 相关断言最新可见版 Withdraw | 该断言不 Active，也不复活旧版 | TP-INV-017, TP-INV-026 |
| TP-T100 | 预测观测晚于截止 | 今天已发布对未来期间的预测 | 不因 observation>known_at 自动判泄漏 | TP-INV-026 |

### 22.11 派生数据与复现性质

| 测试 ID | 场景 | 输入/前提 | 必须断言 | 不变量 |
| --- | --- | --- | --- | --- |
| TP-T101 | 派生下界 | 一个输入尚未可见，输出被标为可用 | lineage 校验拒绝 | TP-INV-029 |
| TP-T102 | 完成时间下界 | 所有输入已知，但计算/交付尚未完成 | 真实系统输出不可消费 | TP-INV-029 |
| TP-T103 | 今日重算历史 | 今天生成特征而写历史 system_available | 拒绝 ActualExecution 历史伪造 | TP-INV-029, TP-INV-030 |
| TP-T104 | 反事实模拟 | 今天运行但每步只用当时许可输入 | 可标 CounterfactualSimulation，不能标真实历史执行 | TP-INV-030 |
| TP-T105 | PIT join | 一侧存在未来修订，另一侧已知 | 关联选择仅使用两侧各自当时版本 | TP-INV-016, TP-INV-029 |
| TP-T106 | 模型实际可用性 | 样本截止早于 T、模型部署晚于 T | 不能进入 T 的真实系统决策 | TP-INV-029 |
| TP-T107 | 固定快照重放 | 相同 context/policy/revisions/实现多次执行 | manifest 和语义结果一致 | TP-INV-023 |
| TP-T108 | 合法未来追加 | 新增都在 cutoff 后才可见，旧证据不变 | S/S′ 所选事实版本语义一致 | TP-INV-022 |
| TP-T109 | 来源历史补全例外 | 新增确于 T 前发布的旧版本，使用新 snapshot | 新研究结果允许改善，旧 snapshot 不变 | TP-INV-022, TP-INV-023 |
| TP-T110 | 跨快照混页 | 分页中途切换 snapshot 或 policy | ContextMismatch，不合并为完整结果 | TP-INV-021, TP-INV-023 |

### 22.12 诊断、错误、缓存与资源

| 测试 ID | 场景 | 输入/前提 | 必须断言 | 不变量 |
| --- | --- | --- | --- | --- |
| TP-T111 | 未来信息隔离 | 管理诊断知道未来版本存在 | DecisionSafe 不含未来 ID/数量/载荷/发布时间 | TP-INV-028 |
| TP-T112 | 三种空值语义 | 无数据、撤回、证据不足 | NoVisibleVersion/Withdrawn/Err 明确区分 | TP-INV-017, TP-INV-020 |
| TP-T113 | 错误机器合同 | 遍历全部公开错误变体 | 唯一稳定 code；Display 文案变化不改变分类 | TP-INV-034 |
| TP-T114 | 上下文错误适配 | 同 TimeError 出现在外部输入与系统源失败 | 消费方可以不同映射，库无 kernel 依赖 | TP-INV-033, TP-INV-034 |
| TP-T115 | 缓存隔离 | 同 key/T 但 source/system/snapshot/cut 不同 | 不得命中同一个不完整缓存键 | TP-INV-012, TP-INV-023 |
| TP-T116 | 资源上限 | 超候选数、键长、证据数或链深；可注入的预留失败 | 超配额 LimitExceeded；可恢复预留失败 ResourceUnavailable；不裁剪伪成功 | TP-INV-021 |
| TP-T117 | 链/集合规模 | 构造大规模合法链与恶意循环 | 有界内存与时间；无无限递归/二次退化 | TP-INV-019 |
| TP-T118 | 零分配基础路径 | allocator 计数测单位/算术/precision/数学错误 | 基础路径无堆分配、无全局锁 | TP-INV-003 |
| TP-T119 | 后端等价对照 | 同 snapshot/coverage 用 Rust 和真实存储选择器 | 选版、撤回、边界、Indeterminate 处理一致 | TP-INV-016, TP-INV-017, TP-INV-021 |
| TP-T120 | 交付证据分级 | 只有文档检查或 Python 参考通过 | Rust/平台/集成 gate 仍 NOT_RUN，不宣称生产完成 | TP-INV-035 |

<a id="s23"></a>

## 23. 构建、测试与 CI 门禁

### 23.1 独立仓库必需门禁

在干净的 temporal 仓库根目录执行，依赖和工具链已经按候选初始化并记录。以下命令是实施验收要求，本交付未执行它们。

```bash
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo doc --locked --no-deps
cargo package --locked
cargo metadata --locked --format-version 1
cargo tree --locked --edges all
```

rustdoc 在 CI 配置中使用 `RUSTDOCFLAGS=-D warnings`。`cargo test` 必须保留 doctest；不能仅用 `--all-targets` 的其他命令替代 doctest 合同。`cargo package` 默认验证解包构建；`--no-verify` 不构成包构建通过。[R12]

初次建立锁文件是单独可审查的开发步骤，不在 locked gate 内自动解锁升级依赖。缓存只优化速度，不能掩盖缺少的本地源码、未登记依赖或不可重现构建输入。

### 23.2 工具链与平台矩阵

| 维度 | 必需合同 |
| --- | --- |
| MSRV | Rust 1.88.0，至少完整测试与库构建；所有公开 API 均可用 |
| stable | 固定并记录当次 stable 解析到的具体版本，fmt/clippy/test/doc/package |
| Linux | x86_64-unknown-linux-gnu |
| macOS | aarch64-apple-darwin |
| Windows | x86_64-pc-windows-msvc |
| 确定性 | 不同平台基础数值/期间/PIT 结果一致；SystemTime 平台限制明确不同 |
| 外部消费 | 无 parent workspace 的临时消费 crate 编译、示例和 trait 合同 |

SystemTime 冒烟/平台精度测试不得要求所有 OS 都精确表示 -1ns；要求的是“成功则无损，否则明确报错”。其他平台或 no_std 不在本版支持承诺中。

### 23.3 依赖与公开面门禁

检查 cargo metadata 的实际依赖与构建输入，不只 grep Cargo.toml 的一层。禁止未审批内部仓库、父工作区继承、跨仓 path、构建脚本读取相邻仓库或网络生成核心代码。测试产物也不能偷偷依赖内部 testkit。

检查 API 名册与实际导出；确认根类型身份唯一、non_exhaustive 外部使用正确、负向样例有合法导入对照。检查中文规范、README、示例、include 包清单中无失效旧 kernel 路径。

### 23.4 性质测试与差分测试

除了 §22 固定场景，必须用属性生成覆盖全域数值、随机合法修订顺序、输入排列、重复、迟到、撤回和证据边界。失败保存 seed 与最小反例。性质测试不是“测试次数达到阈值即证明无 bug”，需配合确定性边界合同。

消费者的 SQL/列式/流式选择器必须对相同固定 snapshot、policy、coverage 与核心参考结果做差分；未来字段仅增加到历史数据集的变异测试必须执行。剥离后旧 kernel 门禁也要真实执行，不能只验证新仓。

### 23.5 测试时钟与真实性分层

纯规则、合同和性质测试不使用真实 sleep。平台时钟冒烟测试与真实消费者时序集成分开；长时运行证据不能由逻辑时钟推进代替。测试 helper 不成为生产依赖，也不虚构跨运行时的统一暂停开关。

没有新增共享可变并发原语时，不为形式完整强制引入 loom；若后续引入此类原语，另立规范与模型检验要求。工具选择必须对应实际风险。

<a id="s24"></a>

## 24. 从 kernel 迁移的实施合同

### 24.1 固定来源，不重新猜测最新 main

迁移开始时读取实际最新 main 并记录 SHA，与本分析基线 `0c895cbf8ce22f5435dd93d5f53eb03eeb57eb2f` 做差异审查。本文的旧行为判断只针对固定基线，不能直接套用到之后修改的代码。

源仓库基线是 kernel 0.4.2，含时间、生命周期与错误职责；新的 temporal 版本独立管理。[K1] 删除 kernel 时间公开项是破坏性变化，拟按 0.5.0 级别管理，不作为无破坏的 0.4.3 补丁。最终版本需经发布审批。[R7]

### 24.2 代码与测试归属矩阵

| 旧位置/内容 | 最终处置 |
| --- | --- |
| src/time/unix_time.rs | 迁往 temporal 的 UnixTimeNs/TimeError；单列负 epoch/精度新合同 |
| src/time/clock.rs | 迁往 temporal 时钟接口与系统实现 |
| src/time.rs 与根 time re-export | 在 temporal 重建明确公开面；从最终 kernel 删除 |
| time_storage 通用单位/换算/能力 | 迁往 temporal precision；返回本地 PrecisionError |
| verify_pg_dual_column | 移到实际 PostgreSQL 消费适配器，调用通用 verify_projection |
| error.rs 的 From<TimeError> for XError | 删除；消费方显式适配 |
| error.rs 中时间相关 compile_fail | 迁到 temporal，修复 import 并保留正向对照 |
| 混合 clock_contract/public_api/api_compile 测试 | 按语义拆分，不整文件无差别搬迁 |
| kernel::lifecycle、ShutdownSignal/Guard、WaitTimeoutError | 保留在 kernel |
| kernel::XError.retry_after 的 std Duration | 保留；标准库时间长度不是 temporal 依赖 |
| 原 bench/example/docs 中时间入口 | 分拆或修改；不得遗留声称旧 API 仍可用的样例 |

### 24.3 必须独立审查的行为变化

原 kernel 的负 SystemTime 输入和负 UnixTimeNs 转 SystemTime 会拒绝；temporal 目标允许平台支持的负值并严格检查精度。原 time_storage 返回 XError，目标返回本地结构化错误。新增 ExplicitFloor 和严格 projection 校验均是明确新行为。[K2] [K3]

这些修改必须有新旧测试对照和 CHANGELOG。机械迁移 PR 不得把行为差异伪装成格式化；新包阶段性存在时，不允许同一个最终应用闭包同时引用两套来源不同的绝对时间新类型。

### 24.4 多仓候选与合入顺序

先固定 GOAL/SPEC/API，再实现独立基础和模型/PIT，之后在消费者组合候选中验证，最后清理 kernel。候选 manifest 记录每仓 SHA、Cargo.lock、schema、策略、fixture 和外部证据版本；元仓 SHA 不能代替实际代码闭包。

main-first 是受信基线与候选管理，不代表直接修改保护分支。每仓各自 PR 和 required checks；跨仓集成通过不免除本仓检查，本仓通过不替代消费者集成。

若确实无法同步迁移，兼容过渡需单独例外记录：Owner、允许的单向依赖、唯一类型身份方案、截止候选和移除条件。不得形成循环；不得默认留下永久 `kernel -> temporal` re-export。

### 24.5 Git 消费合同

正式消费只使用实际存在且审核过的 temporal Git 源与完整 commit，并记录消费者锁文件。未创建的建议仓库不提供假安装指令或虚构 rev。开发中的临时本地 patch 只用于候选调试，必须从独立性验收环境移除。[R4]

保留 publish=false，不执行 cargo publish；cargo package 仅验证独立包内容与构建。不宣称 crates.io/docs.rs 已有发布。

<a id="s25"></a>

## 25. 交付证据、版本政策与完成定义

### 25.1 三个互不替代的完成状态

| 完成状态 | 必须证明 |
| --- | --- |
| temporal 模块本版完成 | 本 SPEC 标为 temporal Owner 的 MUST 实现；本仓 Rust/平台/API/资源门禁通过；文档与代码一致；消费方条款不冒充本仓已实现 |
| kernel 时间迁移完成 | 上项成立；已登记消费者迁移完成；旧公开项和依赖清理；双仓及组合候选通过 |
| 业务系统 PIT 验收完成 | 上述规则真实接入产品数据与证据；后端/派生/回测结果对照通过；信任边界可解释 |

任何一个不能由另一个状态推断。只有 Python 参考模型、文档检查或人工阅读不能令 Rust gate 变为 PASS。

### 25.2 Evidence manifest

每个必需 gate 记录 gate_id、要求/测试 ID、仓库 SHA、工作树状态、工具链/target、命令、退出码、时间、输入快照/seed、日志引用与摘要，以及执行状态。

执行状态仅允许 `PASS / FAIL / NOT_RUN / BLOCKED / NOT_APPLICABLE`。NOT_APPLICABLE 必须有受审查的适用性理由；缺日志、被取消、无工具链或环境失败不能标 PASS。机器全绿不产生人审/合入权限。

### 25.3 版本兼容性

删除/重命名 API、改变默认拒绝行为、时间单位、日期/区间边界、source/system 可见性、修订顺序、错误机器 code 或策略解释，均需明确兼容性审查和版本变化。新增 enum 变体也不能悄悄改变旧策略行为。[R6] [R7]

PolicyRef 和 schema 版本独立于 crate SemVer：同一个 API 实现支持新策略时，历史查询仍绑定旧策略具体内容。禁止把 policy_ref 不变而内部语义改写称作兼容升级。

### 25.4 检查表初始状态

所有实现与集成检查初始为未执行。提交代码时只对本候选确实执行成功的条目打勾；文档本身完整不是代码完成证据。发布记录必须保留残余风险、平台边界和未覆盖的消费方，不用“生产级”三个字替代证据。

### 25.5 不可做的完成声明

不得声称本库已经消除整个系统的一切前视偏差；不得声称证据结构确保来源真实；不得声称模拟时间等于长时真实运行；不得声称 package 可构建就等于所有测试通过；不得声称独立新仓通过就说明旧消费者已迁移。

<a id="s26"></a>

## 26. 目标追踪与实施工作包

| GOAL | 核心 SPEC 章节 | 主要测试 ID | 工作包 |
| --- | --- | --- | --- |
| G-001 | §3–4 | TP-T004、TP-T011–TP-T024 | Foundation |
| G-002 | §5 | TP-T025–TP-T030 | Clock |
| G-003 | §7 | TP-T035–TP-T040 | Temporal profiles |
| G-004 | §6、§15 | TP-T031–TP-T034、TP-T091–TP-T100 | Observation/effective |
| G-005 | §8 | TP-T041–TP-T050 | Precision |
| G-006 | §9、§14 | TP-T051–TP-T060、TP-T086–TP-T089 | Revision identity/order |
| G-007 | §10–13 | TP-T061–TP-T070 | Source/System visibility |
| G-008 | §13–14、§18 | TP-T061–TP-T063、TP-T081–TP-T090、TP-T108 | PIT selector |
| G-009 | §10–12 | TP-T071–TP-T080 | Knowledge cut/evidence |
| G-010 | §16、§20 | TP-T086–TP-T087、TP-T107、TP-T110–TP-T111、TP-T115 | Manifest/API |
| G-011 | §17 | TP-T101–TP-T106 | Derived integration |
| G-012 | §21 | TP-T116–TP-T118 | Resource/performance |
| G-013 | §19 | TP-T009、TP-T113–TP-T114 | Local errors |
| G-014 | §24 | TP-T001–TP-T003、TP-T008、TP-T119 | Kernel migration |
| G-015 | §22–23、§25 | TP-T010、TP-T050、TP-T119–TP-T120 | Gates/evidence |

G-016–G-020 的新增工作包与测试追踪见 §33。每个工作包的 Issue 必须关联 GOAL、具体 SPEC 条款、测试 ID、影响仓库和验收证据。Draft/Spec/Code/Test/Evidence 不得只互相链接而缺少实际断言；修订范围、默认策略或保护边界变化必须重新评审。

建议按 M0 契约、M1 独立基础、M2 模型证据、M3 PIT、M4 消费接线、M5 迁移收口交付。此表不是已创建的 GitHub 任务，也不授权自动合入或发布。

<a id="s27"></a>

## 27. 历史约束对账与统一术语

### 27.1 历史输入与处理原则

本版依据 2026-09-26 的 `goal(1).md`、`spec(1).md`、`kernel-to-temporal-analysis.md`，结合数据平台历史文件及后续 Market Data、binancex、REST 补齐和模块解耦讨论整理。旧文档是历史设计输入，不是已实施事实。明确用户约束继续保留；本版补全的类型、测试和接入规则仍须评审。

| 历史输入 | 本版处理 | 权威位置 |
| --- | --- | --- |
| 六个时间角色 | 全部保留，区分 Unknown/NotApplicable；接收必须真实 | §7 |
| 时间能力从 kernel 剥离 | 一个独立 temporal crate；最终两个仓库互不依赖 | §2、§24 |
| 元仓库 + 独立 crate | 不把源码搬回 workspace，不建立全仓 Cargo workspace | §2、§32 |
| Market 主要 event + received | 真实事件保持该要求；没有源时刻的采样明确使用另一 profile | §7、§29 |
| Macro 使用观测/发布/修订/生效 | 支持这些语义，不要求每一条记录所有角色均 Known | §6–7 |
| 早期 `effective_time <= T` 示例 | 不作为通用可见性条件；known_at 与 valid_at 分开 | §15 |
| 后续 envelope 中出现 transaction_time | 先确定是源交易时刻还是库事务时刻，不新增含混的第七统一角色 | §27.2 |
| 历史补齐 Raw→Canonical→Committed→Coverage | durable-first 产品保留该顺序；独立流式边界另有证据，不强制全平台等数据库 | §30 |
| Checkpoint 不等于完整覆盖 | 正式区分采集进度、数据覆盖与 PIT 候选完整性 | §12、§30 |
| gRPC 用于产品模块通信 | 产品合同拥有 wire schema，temporal 不拥有服务或协议栈 | §31 |
| configx 统一加载、显式配置 | 应用传入冻结策略；temporal 不读取全局配置 | §32 |
| 开发受到真实等待影响 | 时间计算不读 now；timer/阻塞等待测试归运行时/生命周期 Owner | §5、§32 |

### 27.2 易混淆时间词汇的唯一解释

| 名称 | 必须如何使用 | 禁止混用 |
| --- | --- | --- |
| source transaction time | 来源确实定义的成交/事务事件时刻；由映射选为 event_time，其他源字段保留在 provenance | 数据库提交时间 |
| database transaction time | 某后端事务的开始/提交/系统时间轴，名称必须明确其边界 | 直接代替 source event 或 consumer availability |
| recorded_time | 具体审计/记录动作的时刻，必须绑定动作类型与边界 | 万能的“已经可用”字段 |
| committed_time | 只有真实提交及读取边界已证明时才使用；仍需 cursor/cut | 事务开始或提交前的墙钟采样 |
| source_available_time | 来源/受众在某个界限前已可取得当前版本的证据 | 整个序列初次公开日期 |
| system_available_time | 指定消费者边界实际允许读取当前版本的证据 | 无边界的一份全局时间 |
| observation/snapshot time | 来源定义的状态采样坐标；没有时刻则保留 Unknown | HTTP 响应时间直接改名 |
| replayed_time | 本次重放的审计动作时间 | 原始 received_time |
| scheduled_release | 计划日程，属于产品 DTO | 实际 publication_time |
| ingestion_time/processing_time | 必须声明是哪个 pipeline 阶段，不能作为无定义通用名 | 代替六角色、历史 cut 或完整性 |

六角色仍是统一语义视图；receipt、commit、availability、normalization 和 replay 可以是独立审计对象。不得为“字段齐全”把所有阶段压成一个超大 struct，也不得因精简 struct 而删除可见性证据。

### 27.3 规格权威与目录位置

`temporal` 独立仓库的 `goal.md/spec.md` 拥有模块行为合同。`ZoneCNH/workspace` 可以登记模块、固定文档版本/摘要及跨仓采纳规则，但不得复制另一份可以单独改动的同级权威 SPEC。

历史讨论提出的 `standards/temporal.md` 应是跨产品语义约束与采纳索引，引用 temporal 的已批准版本，并登记产品映射 Owner；它不是把源代码迁回 workspace 的理由。实际目录路径须以迁移时读取的元仓名册为准，本交付不宣称已创建该路径。已有 `specs/market_data/adapter` 内的供应商约束继续归其产品 Owner，不在 temporal 下重建 binance/okx 等平行目录。

<a id="s28"></a>

## 28. 时间尺度、原始字段与映射溯源

### 28.1 epoch、时间尺度和闰秒边界

本版 `UnixTimeNs` 的权威数值是 POSIX epoch 纳秒，不累计闰秒；不是 TAI/GPS 计数器，也不是承诺物理绝对准确的 UTC 测量。Rust SystemTime 不计闰秒，其闰秒附近行为受操作系统与配置影响。[R1]

来源的 UTC 标签、显式 leap-second 文本、smear 坐标、TAI/GPS 或未知尺度，必须由消费方在清晰的转换合同下解释；保存原始值、尺度、解释规则和版本。不支持的尺度或无法无歧义转换的时间不能由 temporal 自动接受为同一坐标。库内没有闰秒表更新、NTP/PTP 或 smear 转换引擎。

绝对坐标范围为 i64 纳秒全域。CivilDate 的 1..=9999 年范围不意味着所有日期都可无损转换为该绝对坐标；转换超范围必须拒绝。不得为了保存远期计划日期而饱和到 UnixTimeNs::MAX，可在产品 DTO 中保留 civil date，直到获得合法转换。

### 28.2 端点时间映射清单

每个实际数据产品/端点/流版本必须提供 `TimeMappingContract`。这是消费方版本化合同，不是要求 temporal 新增供应商配置结构。

| 字段 | 必需说明 |
| --- | --- |
| mapping_id / version / digest | 不可变规则版本及内容摘要 |
| source_scope / endpoint_or_stream | 来源、环境、市场、数据类型及实际参数作用域 |
| raw_field_path | 原始字段路径；嵌套字段必须无歧义 |
| raw_unit | seconds / milliseconds / microseconds / nanoseconds / civil-date / documented other |
| time_scale | 源坐标尺度及转换规则；未知不得默认为 POSIX |
| target_role | event/observation/publication/revision/effective 或独立 provenance |
| clock_domain / audience_scope | 谁采样、对谁发布；不把不同边界混同 |
| field_presence_rule | 必须存在、来源未提供、语义不适用分别如何处理 |
| precision_and_rounding | 源分辨率、量化策略和可能误差，不由整数长度推断 |
| interval_convention | 参考窗口的起止点、闭开规则、时区/日历/频率含义 |
| evidence_rule | 哪份原始响应/归档/发布证据能绑定当前版本 |
| schema / extractor / timezone refs | 发生解释时固定全部依赖版本 |

同一供应商不同端点、文件与流可能由不同映射处理；不能设置一个供应商级 `timestamp_unit = ms` 然后覆盖全部接口。此条是接入校验规则，不是对任何特定交易所当前端点行为的声明。

### 28.3 接收时间的采样点

Runtime 必须定义 `receiver_boundary`：例如完整网络帧进入受控缓冲、完整 HTTP body 被接纳、或归档文件内容被受控入口接纳。选择哪种边界由产品契约决定，但同一边界不能随重试而改含义。

HTTP 请求开始、首字节、完整响应、解码完成可能分别有审计记录。原始响应中的每一条业务记录可以共享该响应的 receipt 证据；不得谎称它们是分别在源事件发生时被收到。批量文件行的 event_time 不改变当前实际文件接纳时间。

接收事件绑定进程/日志代次、receipt_id、原始内容引用/摘要、明确单位的墙钟采样与时钟质量。采样点之后的解析、校验、提交、发布耗时由独立边界记录。重试收到相同内容是新 receipt；重新解析同一 raw 不必虚构新网络 receipt。

### 28.4 原值与校正值并存

Raw 时间原值与原始字节 MUST 保留；归一化值由 mapping 固定生成。采集时发现时钟偏差、单位异常或不合理未来值时，质量诊断记录原值和规则，不通过改写 event_time/received_time 使其“看起来正常”。

时钟校准只可依据该查询解释合同允许的校准证据。今日测得的 offset 不得回写为当时已知误差；新历史研究快照可以采用明确标注的事后校正，但不得冒充原 SystemAsKnown 视图。单次远端时间请求也不自动证明链路对称或得到了准确单向延迟。

跨机器 `received - event` 是带时钟误差的表观差，可能为负，不能直接当作准确网络延迟并取绝对值“修复”。本进程耗时使用检查过的 monotonic 差；跨边界延迟保留其不确定性和坐标来源。

### 28.5 精度降低与可见性

原始毫秒时间换成纳秒整数只改变表示单位，不增加源精度。构造 `TimeBounds` 时，时间戳所代表的是采样点、量化桶还是可用性上界，必须有明确来源合同；不能一律把每个毫秒值变成 exact 纳秒证据。

来源窗口边界、发布时间和接收时刻可以具有不同精度。`PrecisionLossPolicy` 负责数学投影；`QueryPolicy` 负责证据是否足以支持可见性，二者不能互相替代。

<a id="s29"></a>

## 29. Market/Macro 接入、窗口与采样

### 29.1 产品映射矩阵

以下是跨产品接入合同；不是某供应商所有端点已实现能力的清单。具体字段、保留窗口及市场适用性由源适配器在自己的版本证据中核验。

| 数据形态 | 推荐 profile 与语义 | 必须保留的非时间信息 |
| --- | --- | --- |
| 逐笔成交/真实源事件 | EventRecord；event_time 为明确的源事件时刻 | 来源身份、市场、实际标的、事件 ID/序列 |
| 带明确源时间的报价或指标事件 | EventRecord 或 PublishedObservation，依据真实语义 | 数值定义、源字段映射和质量 |
| 无源 as-of 时刻的 REST 当前快照 | SampledObservation；received 已知，源时间保持 Unknown/NA | 端点、参数、请求/响应捕获身份 |
| 日内 K 线 | observation=Window([open,end))；更新版本的 event/publication 据来源映射 | 频率、对齐规则、临时/最终标识、版本身份 |
| 日/月/季度宏观统计 | PublishedObservation；observation=Period | 统计定义、季调/单位、来源修订关系 |
| 深度快照/增量 | 有真实事件时间则按映射，否则采样；时间不判断连续性 | snapshot/update ID、范围、序号与来源协议状态 |
| 滚动统计/OI/人数或持仓比 | 来源观察点或实际统计窗口；未提供则采样 | 采样周期、聚合方法、覆盖范围，禁止反推时间窗 |
| 结算、规则更新或到期公告 | Announcement 或 EventRecord，区分生效点与实际事件 | 计划/实际状态、产品适用范围与版本 |
| 自建 K 线/重建状态/特征 | DerivedRecord；窗口和实际产出可用性分开 | 完整 lineage、输入版本与计算版本 |

BTC、ETH、BNB、SOL 与 Spot、USDⓈ-M、COIN-M、Options 的组合，由 Market Data 动态标的与适用性登记决定。temporal 不创建这些枚举，不要求不存在的币种-市场组合产出数据，也不推断供应商端点支持历史补齐。

### 29.2 K 线窗口、临时版与最终版

一根 K 线的 `observation_time` 可以是 `[10:00:00,10:01:00)`；这是它描述的窗口，不是数据在 10:01:00 已被收到或可消费的证明。源 close_time 若使用末个时间格点，只有明确掌握格点/端点规则才允许归一成 end_exclusive，禁止机械 `close_time + 1ns`。

临时更新必须保存为当时版本，完整 OHLCV 最终值不能在窗口尚未结束时被读取。临时版可以描述一个终点晚于 known_at 的预定窗口，但只包含当时已观察内容，并显式标记非最终；这不等于允许读取窗口余下部分的未来值。

产品 `FinalOnly` 策略应先进行 PIT 选版，再检查当前可见版本的 finality；不合条件返回明确的产品级 InProgress/NotFinal，不返回未来最终版，也不随意复活被当前版本替代的旧版。Finality 是产品载荷/合同属性，不是第七个统一时间字段。

同一窗口不同构建方法、对齐方式、时区或源系列可以是不同事实。时间窗口相同不构成跨供应商或跨市场等价性证据。

### 29.3 当前采样没有源发生时刻

无源时间的状态响应只能证明“在本次受控捕获中观察到此内容”，不能据此证明状态首次发生于何时。不得将请求开始、响应结束或本机 now 填到 event_time。

产品可以用 `(source_scope, endpoint_identity, normalized_params, capture_id)` 等规范化组合定义采样事实。这里只规定身份完整性要求，不在 temporal 里固定金融实体键。再次解析同一 capture 使用同一事实身份；新请求得到相同值也可以是另一条合法采样观察，不按 payload 相等删除观察历史。

只有产品额外持有可信的源时间/修订证据时，才能映射到对应源事件或来源版本。不能因为 REST 请求成功就标 `SourcePublishedAsOf @ 过去T` 可见。

### 29.4 订单簿连续性与重建边界

源 snapshot、源 delta、partial depth 与重建的 local book state 是不同对象，不能只因都有 bids/asks 就共用未经区分的时间合同。重建器使用其源协议规定的序列/前驱/区间关系；不得先按 event_time 排序，再把序列缺口判成已修复。

重建状态的系统可用性不早于所需 snapshot 和 delta 在重建边界可用、连续性校验通过、计算完成以及产物发布到消费者边界。状态保留输入游标/序列闭包、重建 epoch、算法版本和质量，不仅保存“最后一条 delta 的时间”。

当前 REST snapshot 只能成为某个重建起点；没有历史 anchor 和完整差分链时，不证明过去任意时刻的完整深度。订单簿算法和缓冲恢复属于 runtime/product，不纳入 temporal 实现。

### 29.5 规范化更正与源修订

例如旧 normalizer 把源微秒按毫秒读取：更正时保留 raw、旧 mapping、旧输出与新 mapping、新输出。来源 revision_id 不变；规范化版本变化。旧错误数据是否撤销、如何公布产品更正，由产品权威合同决定。

SourcePublishedAsOf 新研究快照可以反映经明确说明的修复。SystemAsKnown 回放旧决策必须能取得当时真实可消费的规范化版本，不能只保留修复后的正确值并声称还原了历史系统。

### 29.6 Macro 与有效状态

每个统计期间的初值、修订、撤回分别持有不可变载荷和当版发布证据。只保存 latest 值与最初 publication_time 不支持该期间的历史选版。

PublishedObservation 不默认要求 effective_time 已知；统计口径未定义业务生效时刻时应 NotApplicable，而非复制 observation 或 publication。政策公告等有独立有效时间的产品，按 Announcement 与有效状态合同处理，不让时间库猜测其业务适用规则。

<a id="s30"></a>

## 30. 历史补齐、重放、水位线与覆盖

### 30.1 冻结补齐时间轴

补齐任务必须固定 source、environment、实际市场、series/subject、原生参数、时间轴角色、映射版本及有限 `[start,end)`。`end` 在任务启动时通过显式输入冻结；分页途中不重新取 now 使任务永久追赶扩大的区间。

event 轴历史任务不等于按 received 轴导出；publication 轴修订补齐不等于 observation 轴统计期补齐。任务可声明两种坐标限制，但不得用一个无角色的 start_time 混用。

### 30.2 原生闭区间与单位映射

对于已证明其原生时间坐标是 `k * δ`、查询两端均包含的端点，目标 `[s,e)` 可映射为：

```text
native_start = ceil_div(s, δ)
native_end   = ceil_div(e, δ) - 1
```

该公式只适用于已声明的离散坐标/包含边界合同；所有运算采用宽整数并处理负值和范围错误。`native_end < native_start` 表示该网格没有命中点，不发出起止颠倒的请求。

端点支持更宽的保守请求时，可以重叠拉取后按固定目标坐标筛选，但必须证明没有漏掉边界记录。若原生字段是量化桶、窗口终点或另一个时间角色，先按其合同映射，不套用上式。请求成功或数学公式正确不等于端点全部数据可枚举。

### 30.3 同时间戳分页与幂等

不能通用地使用 `last_timestamp + 1` 作为下一页起点：同一时间格点可能有多条记录。优先使用端点认可的 cursor/事件 ID/复合顺序；只能时间分页时，必须重叠、按完整身份去重，并证明分页上限不会永远截断同一时间桶。无法证明时记录阻塞/不支持，不能宣布覆盖完整。

跨页固定 snapshot 或来源读取一致性合同。来源查询没有稳定快照时，消费方必须描述一致性限制、重扫或差分策略；不得伪造 snapshot。重复相同源版本保留幂等语义，多次 receipt 仍可保存。

### 30.4 五种完整性与进度对象

| 对象 | 回答的问题 | Owner |
| --- | --- | --- |
| Checkpoint | 任务处理到哪个可恢复位置 | collector/runtime |
| CoverageLedger | 某来源/产品/时间范围实际具备哪些数据，哪些缺失或不适用 | data platform/product |
| CandidateCoverage | 本次 PIT 作用域的潜在候选与所需证据是否完整枚举 | 受信查询适配器，temporal 检验绑定 |
| Watermark | 在某流处理策略下，事件时间处理到哪个前沿 | stream runtime |
| SystemKnowledgeCut | 指定历史边界实际可消费哪些版本/事件 | 消费边界/日志或存储适配器 |

`checkpoint advanced ≠ coverage complete`；`watermark passed ≠ history complete`；`query candidates complete ≠ source archive complete`。来源查询与系统查询的候选覆盖声明必须各自说明其 universe：系统日志里没有记录，可证明该系统没看见；不自动证明来源当时没有发布。

产品覆盖至少区分 complete、partial、unknown、source-unavailable、unsupported、not-applicable，实际机器状态由产品合同拥有。本库只接收需要的绑定与完整性证明，不实现覆盖账本。空响应、断连、请求失败或保留窗口外都不得自动转成“已证明无事件”。

### 30.5 持久化优先任务的提交顺序

对明确采用 durable-first 的历史补齐产品，确认顺序是原始内容保全、规范化/校验、持久提交与可读证明、覆盖登记/恢复 checkpoint。实际原子边界由后端 Owner 设计并测试。

仅收到响应但保存失败，不推进到完整。仅保存 raw 但规范化失败，可记录 raw coverage，不能冒充 canonical coverage。任务状态 Finished 可以与 data coverage Partial 并存，最终报告必须同时展示。

允许持久化前流式消费的产品另有明确 consumer boundary，不能将其可见性证据混入 durable-first 边界。反过来也不要求所有流式产品统一等待后端落库。

### 30.6 重放的双重时间

归档重放保存原始 SourceRevision、Receipt、原始 mapping/cut 引用，并新增本次 replay run、replay intake、执行代码/策略和输出可用性证据。不得直接覆盖历史 received_time 为重放时刻，也不得把本次 received 写回成旧 publication_time。

同一个来源版本在两个接收边界的 Temporal 视图可以不同：原投影 received 为原接收，新投影 received 为重放接纳。哪个视图参与查询必须由 boundary/cut 明确选择，而不是“永远取最小 received”或“永远取最后一次”。

ActualExecution 回放过去系统日志与 CounterfactualSimulation 研究模拟保持显式模式。人工推进测试时间可以执行模拟事件，不产生真实网络/数据库历史证明。

### 30.7 水位线与迟到

水位线策略至少标明分区、epoch、事件时间角色、乱序预算、空闲分区规则及更正路径。超出迟到预算的记录不能直接被丢弃后仍宣称该范围数据完整；由产品记录迟到、重算或不可恢复缺口。

水位线前沿不是权威修订次序；已关闭窗口仍可能收到来源合法更正，需按修订/派生产物合同另建版本。时间库提供区间和版本可见性工具，不拥有水位线调度或自动重算任务。

<a id="s31"></a>

## 31. 跨语言 DTO、gRPC 与存储合同

### 31.1 语义统一，不强制同一 wire 实现

本章 Owner 为产品 contracts、协议适配器与存储适配器。temporal 不直接依赖 serde、prost、tonic、数据库驱动，不实现序列化 trait，也不建立 temporal gRPC 服务。产品模块通过经版本化的 DTO 传输，显式检查后构造 temporal 值。

每个协议版本必须明确单位、epoch/时间尺度、缺失状态、窗口闭开规则、键/版本身份、接收与消费边界、策略及快照绑定。不能只因为字段名是 timestamp 就宣称与本库兼容。

### 31.2 建议 wire 形状与解码合同

以下为消费者 DTO 目标形状示例，不是 temporal 默认序列化格式，也不是已编译的协议实现。JSON 中已知时刻使用规范十进制字符串；Unknown 和 NotApplicable 使用显式状态。

```json
{
  "schema_id": "example.temporal-envelope",
  "schema_version": 1,
  "time_scale": "POSIX",
  "event_time": {"state": "known", "unix_ns": "0"},
  "observation_time": {
    "state": "known",
    "kind": "absolute_window",
    "start_unix_ns": "0",
    "end_exclusive_unix_ns": "60000000000"
  },
  "publication_time": {"state": "unknown"},
  "received_time": {"state": "known", "unix_ns": "60005000000"},
  "revision_time": {"state": "not_applicable"},
  "effective_time": {"state": "not_applicable"},
  "mapping_ref": "example:mapping:1"
}
```

示例只展示时间投影。完整接入还需源/产品身份、原始内容引用、receipt、版本、证据和查询上下文；不能仅凭该 JSON 证明 PIT 可见性。示例 epoch 值是合法已知值，并非缺省值。

| 输入状态 | 解码要求 |
| --- | --- |
| known + 合法整数 | 具名单位转换；检查范围，保留 epoch=0 |
| known 但缺值 | 协议校验失败，不转换为 Unknown |
| unknown 或 not_applicable | 无 known 值；按 profile 保留三态并校验 |
| 字段缺失 | 默认协议不合法；仅在有明确旧版本迁移规则时显式映射，不自动等于零或 NA |
| state 同时带多个互斥值 | 拒绝，而不是任选一个 |
| 未知枚举、schema 或 profile | 按兼容性规则拒绝/隔离；不默认当作事件、可见或不适用 |
| observation 绝对窗口 | 保留两个纳秒端点；检查严格 start<end |
| observation 民用期间 | 保留 CivilDate、granularity、calendar_ref，不偷偷转成 UTC |

JSON 十进制整数字符串的消费者规范应明确为 `0` 或可选负号加无前导零的数字串，不接受浮点、小数、科学计数法、前导加号、空白或负零。解析可使用更宽整数做范围检查，不能先转 f64。二进制 i64 通道同样必须检查上游原始类型范围。

### 31.3 Protobuf presence、范围与时间尺度

消费者可以使用有明确 presence 的整型字段/消息或 oneof 表示已知值与缺失状态。仅有 proto3 无 presence 的标量字段，不能区分未提供与显式零；因此不得据它把 missing 解释为 Unix epoch。

使用自有纳秒整数字段时，schema 名称/注释固定 POSIX 与单位，所有端点保持一致。使用 `google.protobuf.Timestamp` 时，必须显式执行 seconds/nanos 规范化及 i64 纳秒范围检查；其文档范围比本库坐标更宽，且有自己的闰秒 smear 解释，不能直接假定所有值与本库无损等价。[R13]

```text
-1 纳秒 → seconds=-1, nanos=999999999
seconds/nanos → i128(seconds) × 1_000_000_000 + nanos
检查 nanos 范围 → 检查总和属于 i64 → 构造 UnixTimeNs
```

负子秒不能把 nanos 写成负数。将宽范围日期缩入 i64 纳秒失败时返回消费者转换错误，不饱和、不回退 now。ProtoJSON 将 64 位整数表示为字符串；实际链路必须验证网关与客户端没有把它们转成浮点。[R14]

### 31.4 gRPC 请求、响应与分页绑定

产品查询 API 必须显式携带或通过不可变 query token 绑定：knowledge_basis、known_at、source/consumer scope、dataset_snapshot、system_cut（需要时）、policy/schema/mapping refs，以及有效状态查询的 valid_at/StatePolicy。请求字段缺失不能被服务端补成 latest 或 now。

分页 token 绑定同一查询的快照、策略、作用域和枚举位置；服务端验证绑定，过期返回明确定义错误。传输成功不等于服务端查询完整，服务端返回完整不等于客户端当时已经接收并可消费。

服务端提交边界、服务端响应、客户端完整接收、客户端应用可消费必须按产品需要分别记录。`market_data → market_regime` 的消息传输不自动继承上游 system_available_time 成为下游可用性；消费方 B 需要自身证据。gRPC timeout/deadline 是运行时预算，不是历史知识截止点。

### 31.5 存储中的权威值与投影

建议由产品适配器保留权威 `*_unix_ns` 整数列或等价的无损字段，再生成数据库日期时间投影供索引/运维使用。具体列类型、范围、驱动和时区规则属于后端实现，不是 temporal 内建行为。

有损投影必须固定精度与策略。恢复时以权威值为准，投影用于验证或查询优化；不得把已经舍入的时间重新宣称为原纳秒值。PostgreSQL 的日期时间精度与实际驱动转换应按其官方合同和真实往返测试验证。[R8]

例如 `-1ns` 的向零微秒投影是 `0us`，floor 投影是 `-1us`。写入读取双方都必须使用同一明确策略。`verify_projection` 先检查读回值是否对齐，不能对异常读回值再量化以掩盖错误。

### 31.6 下推查询的等价性

正确的存储选择器必须与 §13–15 纯选择器语义等价：固定快照与完整作用域、验证证据和 cut、处理不确定性、筛出可见候选、按完整 FactKey/权威顺序选择、解释 Withdraw，再进行有效状态查询。

禁止以下近似替代严格语义：仅 `event_time <= cutoff`、仅 `received_time <= cutoff`、先 `max(revision)` 后筛时间、先删 tombstone、用事务开始时间当作消费者提交可见性，或仅取 latest 表。PostgreSQL 的事务时间函数并不自动给出提交时刻。[R11]

不能在此给出一段省略 evidence/cut/coverage 的 SQL 并称作生产 PIT 实现。实际后端的窗口函数、索引、分区裁剪或列式算子，必须对相同固定 fixture 与 Rust 参考结果做差分；部分下推未满足的语义仍须在受信层执行。

### 31.7 保留期与不可恢复数据

旧 payload、元数据、receipt 或 cut 被删除后，原 snapshot 可能无法重建。消费方必须返回 SnapshotUnavailable/明确的产品错误，不能以新数据填补同名快照。存储保留期、归档、恢复和证据完整性由数据产品负责。

本库不承诺访问权限、数据授权、加密或长期存储可靠性；通过时间规则检查不扩大这些权限。真实系统验收必须包括恢复后版本/摘要/边界一致性，而不是仅证明数据库里还有同一时间戳。

<a id="s32"></a>

## 32. 消费方接线、配置与时钟墙治理

### 32.1 按所有权验收，不按文档所在位置揽权

| Owner | 必须承担 | 不应承担 |
| --- | --- | --- |
| temporal | 基础值、时钟接口/系统采样适配、时间模型、精度、纯证据校验与选版、区间工具、本地错误 | I/O、网络协议、全局配置、后台任务、数据库、业务聚合 |
| kernel | 生命周期、关停及本仓错误能力；标准 Duration/Instant 的合理使用 | 永久转导出 temporal、统一时间错误映射 |
| 源适配器/规范化器 | 原始字节、端点字段映射、单位/尺度、来源版本与证据、源协议顺序 | 宣称所有来源字段天然具备 PIT 证明 |
| 产品 contracts / application | FactKey、观察/修订 authority、边界、profile、QueryPolicy、DTO、明确结果语义 | 让 temporal 依赖本产品 |
| Runtime / collector | 连接、限流、重试、timer、接收/消费边界、checkpoint、排队、关闭与恢复 | 用手动时钟模拟冒充真实链路证明 |
| 存储/查询适配器 | 持久性、snapshot/cut/coverage 的实际证据、协议和存储转换、差分验证 | 用能力声明替代往返与历史可读证明 |
| Feature / backtest / model | PIT join、lineage、训练/部署可用性、实际与模拟模式隔离 | 用 today/latest 重算替代过去真实输入 |
| workspace 治理 | 受信基线、模块名册、任务、规则、候选与跨仓证据登记 | 承载 crate 源码或成为构建前置 |

§22 中 TP-T050、TP-T069、TP-T071–TP-T080、TP-T101–TP-T106、TP-T110、TP-T115、TP-T119 等包含真实产品边界的场景，必须分成 temporal 纯 fixture 断言与相应消费者集成证据。纯 fixture 通过只证明规则，不能替代外部证据真实性。其余场景若涉及编译/后端，也按实际能力 Owner 记录，禁止“写在 temporal/spec.md 所以全由 temporal 实现”。

### 32.2 显式配置和 QueryPolicy 生命周期

configx 可以由应用负责加载统一配置，但不进入 temporal 依赖图。组合根将外部配置解析为明确版本的 QueryPolicy、profile、资源上限、边界与 schema refs；解析/校验失败阻止该查询启动。

禁止读取全局环境变量、静态可变配置、隐式本地时区，或在选择器内读取当前 feature flag。配置的业务默认值可以由产品批准；进入 temporal 时必须是可审计的明确值，而不是省略字段后由库猜测。

每个查询和分页过程持有不可变策略快照。热更新产生新的 policy_ref，仅作用于之后启动的查询；已有查询不改变。在运行记录中保留完整策略内容或可恢复引用，而不仅是可能被覆盖的配置文件路径。

### 32.3 推荐调用边界

```text
源连接器 / 归档读取器
  → Raw + Receipt（实际采样与边界）
  → 版本化 Decoder / Normalizer / Mapping
  → 产品 FactKey / Version / Provenance
  → TemporalDraft::validate(profile)
  → 原始内容、规范化内容及证据保全
  → 明确 consumer availability / snapshot / historical cut
  → QueryInput + CandidateCoverage + AsOfContext
  → temporal 的可见性与选版纯函数
  → 产品 DecisionSafe 投影 / 结果 DTO
```

这是职责顺序示例，不强制单一同步流水线。durable-first 和允许先流消费的产品分别证明自己的边界；错误、重试和隔离不应改写原有时间。派生产物通过自身 lineage 与接纳边界重新进入该过程，不跳过版本和可用性合同。

### 32.4 消除开发时钟墙，而不是新增时间中心

纯数值、区间、profile、revision、visibility、selection 测试只使用显式时间值、查询上下文和本仓 fixture；覆盖数小时或多年的逻辑场景无需等真实墙钟。禁止为了测试 cutoff 而 sleep 到指定时间。

系统时钟适配器测试与纯逻辑分离；时钟对象测试允许故障、回退和边界注入。测试替换时钟只替换被显式注入的采样点，不控制任意组件内部的直接 now，也不自动唤醒真实条件变量。

运行时 timer、真实网络、数据库提交和长时稳定性是其他 Owner 的集成验收。Tokio 的暂停时钟只作用于其时间体系，不控制 std::time::Instant；不能因此宣称整个系统时间被冻结。[R3] 禁止为避免真实测试等待，把其状态伪改为 PASS。

### 32.5 可观测性与错误隔离

本库返回稳定 code 与必要结构，不自动打日志、注册 metrics 或发送 telemetry。消费方可以记录时间转换失败、证据不足、cut 错配、冲突修订、覆盖不完整、超资源、质量隔离等指标；标签大小与敏感内容须受控。

回测/训练的数据通道不能看到“某个未来版本存在但被排除”的诊断，也不能把离线档案中的未来质量问题当作可交易特征。严格选择因不可证明而失败时，该次研究结果应标记无效/不可完成，而不是让策略凭这个错误决定跳过某些样本并宣称无偏。

实际在线系统可以按其当时真实可得的质量状态执行已声明的降级规则，但必须记录该状态的历史证据；不能把今天归档审计发现的问题回写为过去系统已知。管理诊断与策略输入分开，错误细节的访问边界也须在消费方验收。

### 32.6 关停、恢复与重试

temporal 无后台线程、连接或队列，不需要 shutdown/flush/reconnect 生命周期 API。Runtime 负责停止接入、处理在途记录、证明提交/消费、保存可恢复进度；kernel 负责其既有生命周期原语。一个共享库“生产级”不意味着它必须拥有连接池或 async 接口。

失败重试可以新增 receipt/attempt，不改写已存在版本。恢复之后若 boundary_epoch 改变，需要新的 cut 或明确跨 epoch 映射，不沿用相同 offset 假定历史连续。是否允许丢弃/重试/降级由消费方上下文决定，本地 TimeError 不内置平台重试策略。

### 32.7 多仓采纳、回滚与阶段完成

M0 固定契约/Owner/消费者；M1 独立基础；M2 模型/证据；M3 PIT；M4 消费接线；M5 kernel 清理。每阶段均登记真实代码 SHA、锁文件、策略、schema、mapping、fixture 和 required checks，不以文档日期或元仓 SHA 代替。

回滚必须选择已验证的兼容候选组合，并固定数据快照/解释规则。不能仅回滚 temporal 代码却继续使用语义已变的 DTO、缓存或 mapping，然后宣称重现成功。不可逆的数据覆盖或旧版删除不是库版本回滚能够恢复的。

本规格本身不授权创建 Issue/PR/Milestone/Project 或修改远端 main。实现任务应由各仓 PR 承载，跨仓关系在元仓登记；merge 权限和 required checks 保持原有治理流程。

<a id="s33"></a>

## 33. 新增验收矩阵、追踪与冻结项

### 33.1 新增测试的证据归属

以下 TP-T121–TP-T180 与 §22 的 120 项共同组成 180 项验收场景。所有执行状态初始为 **NOT_RUN**。Owner 为 temporal 的条目要求本仓 Rust 行为或编译证据；其他条目要求对应消费者仓库测试，必要时引用同一纯规则 fixture。

表中 Adapter 指源/协议/存储适配器；Product 指 contracts/application；Runtime 指采集与运行时；Platform 指数据产品/回测/特征集成；Governance 指元仓候选与发布治理。多个 Owner 的条目必须拆分证据，不能互相代签。

### 33.2 窗口与行情语义

| 测试 ID | 场景 | 输入/前提 | 必须断言 | Owner | 不变量 |
| --- | --- | --- | --- | --- | --- |
| TP-T121 | 绝对窗口半开边界 | [0ns,60s)；测试起点、终点和中间值 | 起点/中间包含、终点不包含；端点无损 | temporal | TP-INV-036 |
| TP-T122 | 窗口交集与邻接 | 两个重叠窗口、两个仅邻接窗口 | 交集准确；仅邻接返回 None，不创建空范围 | temporal | TP-INV-036 |
| TP-T123 | 窗口全域跨度 | start=MIN，end=MAX | duration 为 Duration::from_nanos(u64::MAX)，无溢出 | temporal | TP-INV-003, TP-INV-036 |
| TP-T124 | 绝对窗口与民用期间 | 同名一天分别用 Window 和 Period | 不能隐式互转；显式解释才生成绝对边界 | temporal / Adapter | TP-INV-036 |
| TP-T125 | 非法窗口 | start=end、start>end | InvalidInterval；无饱和或默认空值 | temporal | TP-INV-003, TP-INV-036 |
| TP-T126 | 无源时刻采样 | SampledObservation，只有合法 received | 结构可校验；不自动变成 SourcePublishedAsOf 可见 | temporal | TP-INV-037 |
| TP-T127 | 本地采样不冒充源事件 | 请求开始/响应收到已知，源状态时刻未知 | event/publication 不得被自动回填，保留采样证据 | Adapter / Product | TP-INV-037 |
| TP-T128 | K 线临时版与最终版 | 同窗口临时版先到，最终版在 cutoff 后才可见 | 过去只选当时版本，不因 closed 标志忽略发布时间 | Product / temporal | TP-INV-041, TP-INV-042 |
| TP-T129 | 来源 close 时间与可用性 | 源 close 边界已知，最终响应/消费晚于该边界 | 按源粒度转换窗口；close 不直接证明最终值可消费 | Adapter / Product | TP-INV-036, TP-INV-041 |
| TP-T130 | 订单簿连续性 | 时间递增但来源序列缺口，随后收到当前快照 | 不得用时间排序弥补缺失路径；执行产品失效/重建规则 | Product / Runtime | TP-INV-040, TP-INV-044 |

### 33.3 原始映射与时间尺度

| 测试 ID | 场景 | 输入/前提 | 必须断言 | Owner | 不变量 |
| --- | --- | --- | --- | --- | --- |
| TP-T131 | 端点单位分别固定 | 两个端点分别给 ms 和 us，值形态相似 | 按各自 mapping 转换；供应商级统一猜测不得通过 | Adapter | TP-INV-002, TP-INV-039 |
| TP-T132 | 禁止位数猜单位 | 未声明单位的 10/13/16 位整数 | 拒绝或隔离映射，不按数量级选择秒/毫秒 | Adapter | TP-INV-002, TP-INV-039 |
| TP-T133 | 原始无符号范围溢出 | u64 大于 i64::MAX 或乘单位后超范围 | 先检查后转换；raw 保留；不使用 as 强转 wrap | Adapter / temporal | TP-INV-003, TP-INV-039 |
| TP-T134 | 零值缺失约定 | 来源A明确零为缺失，来源B明确零为epoch | 仅依版本化 mapping 区分；核心0仍合法已知值 | Adapter / Product | TP-INV-004, TP-INV-048 |
| TP-T135 | 未知时区与重复民用时间 | 无时区日期，或 DST 模糊时刻 | 无明确解释不生成唯一精确 UTC/POSIX 时间 | Adapter | TP-INV-015, TP-INV-038 |
| TP-T136 | 不支持的时间尺度 | leap 标签、TAI/GPS/smear 未配转换规则 | 不能直接当 POSIX ns；保留原值与明确失败状态 | Adapter | TP-INV-038 |
| TP-T137 | 映射可复现 | 相同 raw、mapping、schema、依赖版本 | 规范化坐标与内容摘要完全一致 | Adapter | TP-INV-023, TP-INV-039 |
| TP-T138 | 事后时钟校准 | 今日 offset 被用于纠正过去 received | 不得修改旧 SystemAsKnown 证据；研究另建快照 | Platform | TP-INV-039, TP-INV-052 |
| TP-T139 | 规范化器修复 | 源 payload 不变，解析器修复字段/单位 | 新规范化版本，非伪造来源修订；旧产物仍可重放 | Adapter / Product | TP-INV-042, TP-INV-052 |
| TP-T140 | transaction_time 含义 | 同时有来源成交时间和后端事务开始时间 | 两者映射分开；后者不能替换源event或提交可见性 | Adapter / Product | TP-INV-005, TP-INV-013 |

### 33.4 历史补齐与重放

| 测试 ID | 场景 | 输入/前提 | 必须断言 | Owner | 不变量 |
| --- | --- | --- | --- | --- | --- |
| TP-T141 | 补齐终点固定 | 任务开始后 wall clock 持续前进 | 各页保持同一 end_exclusive 和时间轴，不扩大目标 | Runtime | TP-INV-043 |
| TP-T142 | 闭区间原生网格换算 | 正负 s/e、非整单位、δ>0 与无网格点范围 | ceil_div 公式精确；无点是空请求结果，不溢出 | Adapter / temporal | TP-INV-003, TP-INV-036, TP-INV-043 |
| TP-T143 | 同时间戳跨页记录 | 多条记录时间相同且跨分页边界 | 使用源游标/复合键/已证明方法，不用 last_time+1 跳过 | Adapter / Runtime | TP-INV-025, TP-INV-043 |
| TP-T144 | 分页重叠和重传 | 两页重叠同源记录，多次网络 receipt | 版本幂等；真实不同 receipt 保留；不虚增源修订 | Adapter / Product | TP-INV-011, TP-INV-042 |
| TP-T145 | 当前快照不能补历史 | 历史时段缺少深度，只得到今天当前快照 | 历史路径覆盖仍缺失，不据时间字段伪造过去状态 | Platform | TP-INV-044, TP-INV-045 |
| TP-T146 | 空响应的完整性 | 空页但来源保留/过滤/能力证据不足 | coverage 不自动 complete；未知与已证明无事件区分 | Adapter / Platform | TP-INV-021, TP-INV-045 |
| TP-T147 | raw成功规范化失败 | raw已保存，canonical校验或提交失败 | raw/canonical coverage 分开；不得推进 canonical 完成 | Runtime / Platform | TP-INV-045 |
| TP-T148 | 归档重放双时间 | 同 raw 以原历史 receipt 和今天 replay 接纳 | 原视图不覆盖；新接纳边界另记，查询明确选择 | Runtime / Platform | TP-INV-047 |
| TP-T149 | 重建日志代次 | 同名分区 offset 重置且新事件墙钟较早 | epoch/cut 隔离；不能误放进旧历史视图 | temporal / Runtime | TP-INV-014, TP-INV-047 |
| TP-T150 | 水位线之后更正 | watermark 已过，收到合法旧窗口来源修订 | 记录/更正策略明确；不能丢弃后仍声称历史完整 | Product / Runtime | TP-INV-042, TP-INV-046 |

### 33.5 协议与存储

| 测试 ID | 场景 | 输入/前提 | 必须断言 | Owner | 不变量 |
| --- | --- | --- | --- | --- | --- |
| TP-T151 | Protobuf缺失与零 | 未提供字段、显式零、明确Unknown | presence保留三种差别；不默认epoch | Adapter | TP-INV-048 |
| TP-T152 | 二进制纳秒往返 | 0、-1、i64::MIN/MAX 跨服务传输 | 位值和单位无损；异常范围拒绝 | Adapter | TP-INV-002, TP-INV-049 |
| TP-T153 | JSON整数精确性 | 大于2^53的纳秒、MIN/MAX、科学计数法 | 规范字符串精确往返；权威路径拒绝浮点中转 | Adapter | TP-INV-049 |
| TP-T154 | 三态wire校验 | known缺值、unknown带值、合法NA、合法epoch | 互斥/存在性正确校验；合法三态互不混同 | Adapter / temporal | TP-INV-004, TP-INV-048 |
| TP-T155 | 未知协议变体 | 未支持schema/profile/enum值 | 保守拒绝/隔离，不默认为可见或不适用 | Adapter | TP-INV-048, TP-INV-050 |
| TP-T156 | Protobuf宽时间范围 | 合法Timestamp但超出i64纳秒、负子秒 | 超范围失败；-1ns规范化为(-1,999999999) | Adapter / temporal | TP-INV-003, TP-INV-038 |
| TP-T157 | 后端有损时间投影 | -1ns分别使用truncate/floor到us并读回 | 权威ns不变；投影按各自规则，对齐错误被拒绝 | Adapter / temporal | TP-INV-031, TP-INV-032 |
| TP-T158 | 下推选版与撤回 | 未来高序修订、当时撤回、未证明候选 | 与核心严格结果一致，不能先max或先去撤回 | Adapter / temporal | TP-INV-016, TP-INV-017, TP-INV-020 |
| TP-T159 | 跨页缓存绑定 | 相同T但snapshot/cut/policy/mapping改变 | 旧token或缓存不能混入新语义，明确失败或新查询 | Adapter / Product | TP-INV-023, TP-INV-051 |
| TP-T160 | 存储能力与实测区分 | 声明支持ns但驱动实际降精度 | 能力检查不能替代端到端测试；集成状态不得PASS | Adapter | TP-INV-006, TP-INV-055 |

### 33.6 运行时与信息隔离

| 测试 ID | 场景 | 输入/前提 | 必须断言 | Owner | 不变量 |
| --- | --- | --- | --- | --- | --- |
| TP-T161 | 查询中配置热更新 | 同一查询分页间更新policy | 已有查询保持原策略，后续新查询才采用新引用 | Product | TP-INV-051 |
| TP-T162 | 本地错误消费适配 | 消费者同时使用kernel和temporal错误 | 自由函数/自有新类型可用，无两仓永久依赖 | Product / temporal | TP-INV-033, TP-INV-034, TP-INV-050 |
| TP-T163 | gRPC上下游可见边界 | 服务端早已可读，客户端稍后接收/可消费 | 下游SystemAsKnown不能照抄上游available时间 | Product / Runtime | TP-INV-013, TP-INV-014 |
| TP-T164 | 纯逻辑不真实等待 | 固定fixture模拟多年时间；替换系统时钟 | 无需sleep；不读取now；结果只由显式参数决定 | temporal | TP-INV-007 |
| TP-T165 | 虚拟时间控制范围 | Tokio暂停与std/阻塞等待同时存在 | 测试只声明被实际控制的范围；真实I/O验收另记 | Runtime / kernel | TP-INV-005, TP-INV-055 |
| TP-T166 | 错误与管理诊断侧信道 | 未来归档质量问题导致严格查询失败 | 研究标为无效，不让策略把未来存在性/错误当特征 | Platform | TP-INV-028, TP-INV-053 |
| TP-T167 | 模式不得静默降级 | SystemAsKnown缺cut，Source模式有归档 | 明确失败，不自动换source或simulation再声称实际回放 | Product / temporal | TP-INV-012, TP-INV-030 |
| TP-T168 | 多个消费者的版本绑定 | 同源修订在不同边界用不同规范化版本 | receipt和availability准确绑定实际内容/摘要/映射 | Product / Adapter | TP-INV-009, TP-INV-052 |
| TP-T169 | 跨时钟表观负延迟 | received-event为负但原值有各自证据 | 保留带符号质量信息，不取绝对值伪装网络延迟 | Runtime / Product | TP-INV-005, TP-INV-039 |
| TP-T170 | 关停中提交与checkpoint | 收到数据后关停，部分在途尚未提交 | Runtime按实际边界保存进度，不让temporal自建队列/池 | Runtime / kernel | TP-INV-045, TP-INV-054 |

### 33.7 端到端与治理

| 测试 ID | 场景 | 输入/前提 | 必须断言 | Owner | 不变量 |
| --- | --- | --- | --- | --- | --- |
| TP-T171 | 无源序号的本地采样 | 源无revision ordinal；同捕获重复重放 | 稳定capture fact+初始节点，不伪造源顺序或新事实 | Product / temporal | TP-INV-019, TP-INV-037, TP-INV-042 |
| TP-T172 | 今日重解析与历史实际 | 新解析器纠正过去曾误解析的值 | 实际回放保留当时产物；更正研究另有snapshot/模式 | Platform | TP-INV-030, TP-INV-052 |
| TP-T173 | 未来窗口终点的临时观察 | 窗口尚未结束，但临时观测已经可见 | 允许当时已知临时值；不提前暴露最终OHLC等内容 | Product / temporal | TP-INV-026, TP-INV-041 |
| TP-T174 | 派生与模型全链 | 输入可见，但计算/部署/交付时间不同 | 各自实际可用性得到验证；不能只比较样本event_time | Platform | TP-INV-029, TP-INV-030 |
| TP-T175 | 不同完整性作用域 | 采集coverage完整但PIT漏版本；反向系统日志完整 | 分别验证，任何一种不能替代另一种证明 | Platform / temporal | TP-INV-021, TP-INV-045 |
| TP-T176 | 未来追加性质扩展 | 加入合法cutoff后窗口修订/采样，旧证据不变 | 历史语义结果不变；管理扫描统计可不同 | temporal / Platform | TP-INV-022, TP-INV-024 |
| TP-T177 | 独立外部消费 | 只检出temporal，外部crate消费公开API | 无父workspace/内部testkit/configx/gRPC依赖；单一类型身份 | temporal | TP-INV-001, TP-INV-033, TP-INV-050 |
| TP-T178 | 兼容候选回滚 | temporal/product/schema/mapping不同版本组合 | 只认可已验收闭包；固定快照重现，不暗换解释 | Governance / Platform | TP-INV-023, TP-INV-051 |
| TP-T179 | 冻结参数与Owner缺失 | 资源预算/映射证据/消费者Owner等发布必需项未定 | 关联任务Blocked，不默认值补齐后伪称验收完成 | Governance | TP-INV-054, TP-INV-055 |
| TP-T180 | 文档检查不冒充实现 | 仅本MD结构检查或外部参考算术检查通过 | Rust/协议/集成场景仍NOT_RUN；不宣称180项行为通过 | Governance | TP-INV-035, TP-INV-055 |

### 33.8 目标到新增场景追踪

| GOAL | SPEC | 新增验收场景 | 工作包/Owner |
| --- | --- | --- | --- |
| G-016 | §6–7、§9、§29 | TP-T121–TP-T130、TP-T171、TP-T173 | 窗口/采样 pure core；产品映射与订单簿接入 |
| G-017 | §11–12、§30 | TP-T141–TP-T150、TP-T175 | 补齐/覆盖/重放；Runtime/Platform |
| G-018 | §3–8、§28 | TP-T131–TP-T140、TP-T169、TP-T172 | 源映射与时间质量；Adapter/Product |
| G-019 | §8、§20、§31 | TP-T151–TP-T160、TP-T163、TP-T168 | wire/存储/查询差分；协议和后端适配器 |
| G-020 | §5、§23–25、§32 | TP-T161–TP-T170、TP-T174、TP-T176–TP-T180 | 配置/边界/测试/候选；各Owner分层 |

新增场景扩展 G-001–G-015 的边界覆盖，并不取消旧测试或更改其历史执行状态。测试实现应绑定唯一断言和真实证据，不要求为凑数量创建一百八十个空壳函数。

### 33.9 实施前必须冻结的参数

以下不是放任实现者猜测的空白；这是需在对应阶段获得证据并批准的项目环境参数。纯数学与选版语义已由本 SPEC 固定，不依赖这些参数重新定义。

| 冻结项 | 最迟冻结阶段 | 责任与未满足时行为 |
| --- | --- | --- |
| temporal 实际仓库地址、LICENSE 和 CODEOWNERS | M0 | 维护者确认；不能把建议地址当已创建 |
| 实际 kernel main SHA 与历史 SHA 差异 | M0 | kernel Owner审查；不以旧行为直接覆盖新代码 |
| 完整消费者与公开入口清单 | M0，M5前收口 | 元仓登记；未知外部消费者不宣称全部迁移 |
| MSRV锁文件、stable具体版本、平台targets | M1 | 模块维护者；历史MSRV1.88为拟保留基线，不称最新版 |
| 每查询候选/键/证据/链深/内存预算 | M3前 | 产品与模块维护者提供实际整数；未批准预算不运行生产查询 |
| 来源字段/单位/尺度/精度/闭开规则证据 | 每源接入前 | Adapter Owner；缺失字段映射隔离，不猜测 |
| FactKey、source/capture/normalization authority | 每产品接入前 | Product Owner；不可比修订不强排序 |
| reception/consumer boundary、epoch与cut证明 | M4 | Runtime/Store Owner；未证明则SystemAsKnown不可验收 |
| wire schema、codec、持久精度、保留与恢复策略 | M4 | Product/Adapter Owner；差分/往返未执行不得放行 |
| 实际/模拟模式、lineage与回测失败策略 | M4 | Backtest/Feature Owner；不自动降级 |
| 性能冻结基线与10%回归门禁的受控机器 | 性能验收前 | 模块Owner；无基线不声称已满足性能SLO |
| 过渡例外、整组回滚候选与移除旧入口条件 | M5前 | 跨仓Owner；禁止无期限兼容依赖 |

尚未收集的环境参数必须在任务中标明 Owner 与 BLOCKED 条件。不能把它们变成运行时静默默认值，也不能因此将所有基础实现工作无限期阻塞；按阶段完成并如实分别记录。

### 33.10 文档自检与实施验收分开

本交付可以验证 Markdown 结构、章节/测试/不变量编号、引用和下载包一致性。这只对应文档质量。Rust compile/test/clippy、跨平台、真实协议、数据库、采集恢复、历史数据和回测验收都需要后续实际执行证据。

任何自动生成的 checklist 默认未勾选。没有对应 SHA、命令、退出码、日志和输入清单的 PASS 无效；文档体量、测试条目数或“生产级”措辞不构成通过证明。

<a id="s34"></a>

## 34. 核验来源、历史依据与规范区分

### 34.1 整合轮使用的历史材料

1.1.0 整合轮记录了对下列三个历史文档的完整读取，原件未被改写；0.1.0 未重新取得原件。本版仍是评审候选，不宣称历史整理稿已经批准或实现。历史原稿的 1.0.0 与整合轮的 1.1.0 仅用于溯源；本叶文档版本从 0.1.0 开始，不承接这些历史标识的版本序列。

| 历史文件 | 历史文档版本/日期 | 原件 SHA-256 |
| --- | --- | --- |
| goal(1).md | 1.0.0 / 2026-09-26 | f53dbc72b9ad3da34c843b690c08912077c84a570da4df9f04ff35a7f2d69edb |
| spec(1).md | 1.0.0 / 2026-09-26 | c10b29ce3d8852ffe1cb01a6d9f22e7ed84775cec539df5fc42bf7d891b86916 |
| kernel-to-temporal-analysis.md | 2026-09-26 | db6a60c8dba6debf2a239de085dcac0c4922757e790433b676c7c811def64b0e |

1.1.0 整合轮还记录了读取 `xhyper-data-platform-complete.md` / 同内容副本中 EventEnvelope、六角色、Market/Macro、早期 PIT 和订单簿段落，并检索历史讨论的独立模块、kernel剥离、市场采集/REST补齐、配置加载及测试时间约束。该历史输入用于确认需求，不充当今日供应商API或远端仓库状态证据。

### 34.2 历史代码事实与候选设计

K1–K8 指向历史 kernel 固定提交 `0c895cbf8ce22f5435dd93d5f53eb03eeb57eb2f`。0.1.0 未重新读取远端最新 main，也未重新执行该提交的 Rust 测试；旧源码事实由上述剥离报告转引。开始迁移前必须按 §24、§33 重新固定实际候选并审查差异。

相对 1.0.0，候选新增的 Window/AbsoluteTimeRange、SampledObservation、消费方映射/补齐/wire/配置合同及60个场景均为拟议设计。0.1.0 在工作区元仓库隔离分支修订文档，没有上传或合入 temporal crate 源码，也没有证明任何实现完成。

### 34.3 外部官方技术资料

原稿 R1–R12 是 2026-09-26 的历史参考。1.1.0 整合轮记录了对 Rust SystemTime/Instant、Tokio pause、Cargo依赖、FRED real-time、PostgreSQL时间函数、Protobuf Timestamp 与 ProtoJSON 的复核；0.1.0 未对全部外部资料重新复核。未重新查阅的旧参考只作为历史出处保留，不宣称其当前版本已全量复核。

官方资料只支持对应语言/平台/协议事实；本文件的时间角色、证据、查询政策、架构和测试是本项目目标合同，不是 Rust、Binance、FRED 或 PostgreSQL 对项目的保证。本文不包含对交易所最新历史保留窗口、速率限制或全部币种上市状态的断言。

| 引用 | 支持的事实范围 |
| --- | --- |
| K1–K8 | 历史package、类型、错误、公开面与CI的定位，不是今日main |
| R1–R3 | 标准库时钟、平台精度及Tokio虚拟时间边界 |
| R4–R7、R12 | Git依赖、语言trait限制、兼容与打包 |
| R8、R11 | PostgreSQL精度与事务时间函数语义 |
| R9–R10 | FRED real-time/vintage 日期语义，不等于精确日内发布 |
| R13–R14 | Protobuf Timestamp范围/子秒/尺度合同与ProtoJSON整数表示 |

[K1]: https://github.com/bytechainx/kernel/blob/0c895cbf8ce22f5435dd93d5f53eb03eeb57eb2f/Cargo.toml "kernel 固定 package 基线"
[K2]: https://github.com/bytechainx/kernel/blob/0c895cbf8ce22f5435dd93d5f53eb03eeb57eb2f/src/time/unix_time.rs "旧时间 primitive 与转换行为"
[K3]: https://github.com/bytechainx/kernel/blob/0c895cbf8ce22f5435dd93d5f53eb03eeb57eb2f/src/time_storage.rs "旧精度与 PostgreSQL 投影规则"
[K4]: https://github.com/bytechainx/kernel/blob/0c895cbf8ce22f5435dd93d5f53eb03eeb57eb2f/src/error.rs "旧错误映射"
[K5]: https://github.com/bytechainx/kernel/blob/0c895cbf8ce22f5435dd93d5f53eb03eeb57eb2f/src/time/clock.rs "时钟接口"
[K6]: https://github.com/bytechainx/kernel/blob/0c895cbf8ce22f5435dd93d5f53eb03eeb57eb2f/tests/api_compile.rs "编译边界"
[K7]: https://github.com/bytechainx/kernel/blob/0c895cbf8ce22f5435dd93d5f53eb03eeb57eb2f/src/lib.rs "旧公开面"
[K8]: https://github.com/bytechainx/kernel/blob/0c895cbf8ce22f5435dd93d5f53eb03eeb57eb2f/.github/workflows/ci.yml "旧 CI 范围"
[R1]: https://doc.rust-lang.org/std/time/struct.SystemTime.html "SystemTime"
[R2]: https://doc.rust-lang.org/std/time/struct.Instant.html "Instant"
[R3]: https://docs.rs/tokio/latest/tokio/time/fn.pause.html "Tokio pause"
[R4]: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html "Cargo Git dependency"
[R5]: https://doc.rust-lang.org/reference/items/implementations.html "Rust coherence/orphan rules"
[R6]: https://doc.rust-lang.org/reference/attributes/type_system.html "Rust non_exhaustive"
[R7]: https://doc.rust-lang.org/cargo/reference/semver.html "Cargo SemVer"
[R8]: https://www.postgresql.org/docs/current/datatype-datetime.html "PostgreSQL 日期时间类型"
[R9]: https://fred.stlouisfed.org/docs/api/fred/realtime_period.html "FRED Real-Time Periods"
[R10]: https://fred.stlouisfed.org/docs/api/fred/series_vintagedates.html "FRED Vintage Dates"
[R11]: https://www.postgresql.org/docs/current/functions-datetime.html "PostgreSQL 时间函数"
[R12]: https://doc.rust-lang.org/cargo/commands/cargo-package.html "cargo package"

[R13]: https://protobuf.dev/reference/protobuf/google.protobuf/#timestamp "Protobuf Timestamp 官方合同"
[R14]: https://protobuf.dev/programming-guides/json/ "ProtoJSON 类型表示"
