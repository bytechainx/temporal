# ADR-0004：跨语言与存储精度 DTO 合同（WP-07 / AUD-007）

- 状态：Proposed / Review Required
- 日期：2026-11-08
- 关联：`.omc/drafts/1.md` AUD-007 与 WP-07；`docs/audit/AUDIT-VALIDATION.md` §8；`spec.md` §8（:422-452）与 §31（:1694-1783）；ADR-0001 §4（Trusted Evidence Adapter）；ADR-0003 §3（复现清单 identity）
- 写范围：仅本文件。不改 `src/`、`tests/`、`spec.md`、`goal.md`、`checklist.md`、`docs/API.md`。

## 1. 背景与已核实事实

T1 判定（`docs/audit/AUDIT-VALIDATION.md` §8）：AUD-007 为 **partially-confirmed**。草案的 5 项要求中，要求 3 与要求 4 已在库内实现（判定 refuted）；要求 1 与要求 2 在库内 **NOT_APPLICABLE**（库没有序列化与传输代码）；要求 5 拆成两半——读回校验在库内已有，gRPC 请求绑定属消费侧。

已核实事实（行号对基线 `7cef81d` 与本次复核点同时有效）：

1. 依赖面：`Cargo.toml` 的 `[dependencies]` 只有 `thiserror = "2"`；`serde` 只出现在 `[dev-dependencies]`。`src/` 没有任何序列化实现。
2. 编译期反向断言：`tests/api_compile.rs`:14 `assert_not_impl_any!(UnixTimeNs: Default, From<i64>, Into<i64>, serde::Serialize);`；`tests/p1a_api.rs`:63、:73、:80 同向断言。结论：库不提供隐式整数转换与序列化旁路，JSON 与二进制编解码 100% 在消费侧。
3. 未知时间没有默认值：`TimeField::{Known, Unknown, NotApplicable}`（`src/model.rs`:9-16）；`UnixTimeNs::from_unix_nanos` 必须显式给值（`src/unix_time.rs`:19）。
4. 单位换算已带范围检查：`try_from_unix_micros` / `try_from_unix_millis` / `try_from_unix_seconds` 用 `checked_mul` → `TimeError::Overflow`（`src/unix_time.rs`:29、:37、:45）；`try_from_seconds_nanos` 拒绝 `nanos >= 1_000_000_000`（`TimeError::InvalidSubsecond`）并按 i128 检查范围（`:90`）；`to_seconds_nanos` 用 Euclid 除法，亚秒恒非负（`:82`）；`try_from_system_time` / `try_into_system_time` 往返不复原 → `TimeError::SystemTimePrecisionLoss`（`:101`、`:112`）。
5. 降精度策略与读回校验已存在：`PrecisionLossPolicy::{Reject(默认), ExplicitTruncate, ExplicitFloor}`（`src/precision.rs`:31）、`unix_ns_to_stored`（`:53`）、`stored_to_unix_ns`（`:72`）、`quantize_unix_ns`（`:79`）、`verify_projection` 先检查对齐再比对权威投影（`:89`）、`require_lossless_unix_ns` 只读能力声明（`:106`）。
6. 查询侧绑定字段已存在但不参与判定：`AsOfContextParts` 携带 `dataset_snapshot`、`knowledge_basis`、`system_cut`、`policy` 与三个 `*_schema_ref`（`src/pit/context.rs`:206-260）；这些 schema 引用当前不进入判定（AUD-002 第 3 点）。
7. 正本无冲突：`spec.md`:448（能力声明不是读写证明）与 `spec.md`:1783（库不承诺访问控制、加密、长期存储可靠性）已划清边界。

## 2. 裁定：WP-07 的交付物形状

- **W-A（否决）**：在 `temporal` 内实现 JSON / Protobuf / gRPC DTO 或序列化实现。理由：`spec.md`:1698 规定库不依赖 serde / prost / tonic / DB 驱动、不实现序列化 trait、不承载 temporal gRPC 服务；库内不存在读取该 DTO 的消费者；不满足 AGENTS.md 反过度设计四问。
- **W-B（采纳）**：库只保留「解码后置合同」——消费者把 wire 值解析成整数后，必须用库的具名单位入口与 `verify_projection` 完成范围检查与读回校验。DTO 与 wire schema 归消费方 contracts 仓；本文件给出合同条款与库内可测子集。
- **W-C（否决）**：在 `temporal` 新增 DTO 类型（例如 `TimeEnvelope`）。理由：`spec.md`:1706-1724 的示例信封是消费方样例；库内复制它只增加无人读取的类型。
- **明文更正**：草案把要求 3 与要求 4 记为「缺失」。两者已在库内实现并有测试。WP-07 不得据此新增抽象（`docs/audit/AUDIT-VALIDATION.md` §8）。

## 3. 合同条款（W-1..W-10）

「消费者必须」由产品 contracts、协议适配器、存储适配器与 Runtime Owner 落实（`spec.md`:1798 §32.1）。「`temporal` 侧对应」只描述库已提供、消费者应当使用的入口，不描述新能力。

| 条款 | 消费者必须 | `temporal` 侧对应 | 正本出处 |
| --- | --- | --- | --- |
| W-1 JSON 纳秒编码 | 纳秒写成十进制整数字符串：`0` 或可选 `-` 加数字，无前导零。禁止浮点、小数、科学计数法、前导 `+`、空白、负零。可用更宽整数做范围检查，但 MUST NOT 先转成 `f64` 再取整。二进制 i64 通道必须校验上游原始类型范围。 | 库无 JSON 代码；`UnixTimeNs` 无 `Default`、`From<i64>`、`Into<i64>`、`serde::Serialize` 旁路（`tests/api_compile.rs`:14），消费者不能借用隐式转换 | `spec.md`:1739；TP-INV-049（:919） |
| W-2 缺失值与哨兵 | `known` 但缺值 → 协议校验失败，不得降级为 `Unknown`。字段缺失在默认协议下无效，只能由显式旧版本迁移规则映射，不得自动补 `0` 或 `NotApplicable`。`epoch = 0` 与负纳秒是合法已知值，不得当哨兵。 | 三态只由协议显式声明驱动：`TimeField::{Known, Unknown, NotApplicable}`（`src/model.rs`:9-16）；`from_unix_nanos` 必须显式给值（`src/unix_time.rs`:19） | `spec.md`:1726、:1731；TP-INV-048（:918） |
| W-3 Protobuf presence | 用 `optional` 或 `oneof` 表达显式 presence。proto3 无 presence 的标量无法区分「未提供」与显式 `0`，因此缺失值 MUST NOT 读成 Unix epoch。自定义纳秒字段须在 schema 名与注释固定 POSIX 时间尺度与单位，并与所有端一致。 | 库无 Proto 类型；解码后消费者调用具名单位入口 | `spec.md`:1741-1743 |
| W-4 Timestamp 归一化与范围 | 归一化顺序：先检查 `nanos` 属于 `[0, 1_000_000_000)`，再算 `i128(seconds) × 1_000_000_000 + nanos`，最后检查和是否属于 `i64` 范围。负亚秒不得写成负 `nanos`（`-1 纳秒 → seconds = -1, nanos = 999999999`）。超范围必须返回消费者转换错误，不得饱和、不得回退 `now`。不得假定 `google.protobuf.Timestamp` 与本库无损等价——其文档范围更宽，且自带 leap-second smear [R13]。ProtoJSON 链路必须验证网关与客户端未把 64 位整数变成浮点 [R14]。 | 归一化后的落点：`UnixTimeNs::try_from_seconds_nanos`（`src/unix_time.rs`:90）拒绝 `nanos >= 1_000_000_000`（`TimeError::InvalidSubsecond`）并按 i128 检查范围；`to_seconds_nanos`（`:82`）的 Euclid 除法保证亚秒非负 | `spec.md`:1745、:1747-1753 |
| W-5 单位换算入口 | 必须使用与 wire 单位同名的入口（`try_from_unix_seconds` / `try_from_unix_millis` / `try_from_unix_micros`），不得自行乘系数。 | `src/unix_time.rs`:29、:37、:45 使用 `checked_mul` → `TimeError::Overflow`；库内所有单位乘法均为 checked 算术 | `spec.md`:446；§8.3 |
| W-6 存储降精度 | 权威值与投影分开保存：保留权威 `*_unix_ns` 整数或无损列，另有供索引与运维使用的 datetime 投影。降精度必须固定 `TimePrecision` 与 `PrecisionLossPolicy`，写侧与读侧使用同一策略。恢复只使用权威值；舍入后的时间不得再声称是原纳秒值。列类型、范围与驱动行为属后端实现。 | `unix_ns_to_stored`（`src/precision.rs`:53）、`stored_to_unix_ns`（`:72`）、`quantize_unix_ns`（`:79`）；`Reject` 为默认策略；`ExplicitTruncate` 向零、`ExplicitFloor` 向负无穷 | `spec.md` §8.1（:426-434）、§31.5（:1765-1769） |
| W-7 读回校验 | 读回校验必须先看读回值是否按声明精度对齐，再与权威值的投影比较。MUST NOT 先量化读回值以掩盖异常输入。 | `verify_projection`（`src/precision.rs`:89-103）：先 `MisalignedProjection`，后 `ProjectionMismatch` | `spec.md`:440、:1769 |
| W-8 能力声明不是证明 | 适配器必须自行验证范围、负值、驱动舍入、UTC 转换与真实往返。`lossless_unix_ns` 一类声明只是声明。MUST NOT 在 `temporal` 内保留 `verify_pg_dual_column` 之类的后端专用校验。 | `TimeStorageCapabilities`（`src/precision.rs`:43-50）；`require_lossless_unix_ns`（`:106-114`）只读一个布尔声明 | `spec.md`:450、:452 [R8] |
| W-9 gRPC 请求绑定 | 请求必须携带、或经不可变查询 token 绑定：`knowledge_basis`、`known_at`、来源与消费作用域、`dataset_snapshot`、需要时的 `system_cut`、policy / schema / mapping 引用；有效状态查询还须绑定 `valid_at` 与 `StatePolicy`。请求缺字段 MUST NOT 由服务端补 `latest` 或 `now`。分页 token 必须绑定同一查询的快照、策略、作用域与枚举位置；过期 token 返回已定义错误。gRPC timeout 与 deadline 是运行时预算，不是历史知识截止点。 | `AsOfContextParts` 已承载 `dataset_snapshot`、`knowledge_basis`、`system_cut`、`policy` 与三个 `*_schema_ref`（`src/pit/context.rs`:206-260）；这些引用当前不参与判定，其强度归 WP-02（AUD-002 / R3），本 ADR 不重复实现 | `spec.md`:1757-1761 |
| W-10 快照不可重建 | payload、metadata、receipt、cut 被删除时，原快照可能无法重建。消费者必须返回 `SnapshotUnavailable` 或明确的产品错误，MUST NOT 用新数据填充同名快照。真实验收包含恢复后的版本、摘要与边界一致性。 | 库不承诺访问权、数据授权、加密或长期存储可靠性（`spec.md`:1783） | `spec.md`:1779-1783 |

## 4. `temporal` 侧可测试子集（交主 Agent 门禁）

下列测试均已存在；本会话按成员纪律不运行 cargo，因此 §6 记录为 **NOT_RUN**。这些命令只证明 W-1 的前提、W-2、W-4、W-5、W-6、W-7、W-8 的库内合同，不证明任何 wire 往返。

| 子集 | 命令 | 已存在的测试锚点 | 覆盖条款 |
| --- | --- | --- | --- |
| S-1 原始值与亚秒归一化 | `cargo test --test p1a_numeric` | `tests/p1a_numeric.rs`:9 `tp_t011_epoch_and_arbitrary_nanos`、:17 `tp_t012_signed_unit_construction`、:53 `tp_t013_unit_multiplication_overflow`、:133 `tp_t019_negative_subsecond_normalization`、:152 `tp_t020_subsecond_and_range_validation`、:101 `tp_t016_full_i64_span` | W-2、W-4、W-5 |
| S-2 SystemTime 往返 | `cargo test --test p1a_clock` | `tests/p1a_clock.rs`:104 `tp_t021_system_time_signed_epoch`、:118 `tp_t022_extreme_negative_system_time`、:144 `tp_t024_system_time_out_of_range_is_explicit` | W-4 |
| S-3 精度策略与投影 | `cargo test --test p1b_precision` | `tests/p1b_precision.rs`:16 `tp_t041_default_policy_rejects_subunit_loss`、:36 `tp_t042_truncate_is_symmetric_around_zero`、:66 `tp_t043_floor_keeps_negative_subunit`、:94 `tp_t044_lossless_units_round_trip_under_reject`、:114 `tp_t045_stored_multiplication_does_not_wrap`、:136 `tp_t046_floor_restore_underflow_is_explicit`、:164 `tp_t047_nanosecond_identity_holds_for_all_policies`、:184 `tp_t048_alignment_is_checked_before_projection`、:206 `tp_t049_aligned_but_wrong_projection_mismatches`、:237 `tp_t050_capability_check_only_reads_declaration` | W-6、W-7、W-8 |
| S-4 精度契约 | `cargo test --test precision_contract` | `tests/precision_contract.rs`:9 `默认拒绝精度损失`、:32 `负值截断与_floor_不同`、:76 `换算溢出和纳秒恒等`、:100 `投影先检查读回对齐`、:131 `能力函数只检查声明`、:146 `精度错误码固定` | W-6、W-7、W-8 |
| S-5 无序列化旁路 | `cargo test --test api_compile` | `tests/api_compile.rs`:14 `assert_not_impl_any!(UnixTimeNs: Default, From<i64>, Into<i64>, serde::Serialize)`；`tests/p1a_api.rs`:63、:73、:80 同向断言 | W-1 前提 |

补充判据（可选，需 Owner 批准后才写进 `spec.md` §22）：`wire_guard_negative_and_extreme_seconds_nanos_round_trip`——`-1`、`i64::MIN`、`i64::MAX` 经 `to_seconds_nanos` 再经 `try_from_seconds_nanos` 必须恒等往返，落点 `tests/p1b_precision.rs`（邻近 :206）。理由：`spec.md`:1747 的归一化配方是消费者必须复刻的算法，库内先用极端值锁住该形状，成本是一次既有测试文件内的断言。未批准前不得声称它是 `spec.md` 验收项。

## 5. 消费者验收条款（V-1..V-6）

Owner 归属按 `spec.md`:1798 §32.1。全部 6 项本机不可执行：本仓库无 consumer 检出，无 serde / prost / tonic / PostgreSQL 链路，且 `temporal` 禁止引入这些依赖。状态一律 **BLOCKED（需外部输入：消费者仓与适配器实现）**。

| 项 | 必须证明 | Owner | 解除 BLOCKED 的前置 |
| --- | --- | --- | --- |
| V-1 | JSON 往返 fixture：正例 `0`、`-1`、`i64::MIN`、`i64::MAX`、`2^53 + 1` 逐位精确；反例 `"1.0"`、`"1e9"`、`"+1"`、`" 1"`、`"01"`、`"-0"`、`"9223372036854775808"` 必须被拒绝。必须证明解析路径不经 `f64`。 | 产品 contracts + 协议适配器 | 消费者仓的 wire schema 与解码实现 |
| V-2 | proto3 presence：同一字段「缺失」与「显式 `0`」必须产生不同解码结果；缺失必须报协议校验失败，不得产出 `UnixTimeNs(0)`。 | 协议适配器 | `.proto` 定义与解码测试 |
| V-3 | `google.protobuf.Timestamp` 边界：`nanos` 属于 `[0, 1_000_000_000)`；`seconds` 超出本库 i64 纳秒可表示范围时必须返回转换错误；leap-second smear 差异必须在适配器文档声明 [R13]。ProtoJSON 链路必须证明网关与客户端未把 64 位整数变成浮点 [R14]。 | 协议适配器 | 真实 gRPC / ProtoJSON 往返记录 |
| V-4 | 后端往返：PostgreSQL `timestamp` / `timestamptz` 微秒分辨率下，`-1` 纳秒在 `ExplicitTruncate` 与 `ExplicitFloor` 下的读回值必须分别可复现；写侧与读侧同一策略；`verify_projection` 使用权威值；驱动舍入与时区规则按官方合同核验并有真实往返记录 [R8]。 | 存储适配器 | 可用数据库与驱动、往返夹具 |
| V-5 | gRPC 绑定：请求缺 `dataset_snapshot`、`knowledge_basis`、`system_cut`、policy 或 schema 引用时必须被拒绝；| V-5 | 服务端绑定：请求缺少 `dataset_snapshot`、`knowledge_basis`、`system_cut`、policy 或 schema 引用时必须拒绝，服务端不得补 `latest` 或 `now`。分页 token 必须绑定同一快照、策略、作用域与枚举位置；过期 token 返回已定义错误。gRPC timeout / deadline 只是运行时预算，不是历史知识截止点。 | 协议适配器 + Runtime | 服务端实现与契约测试 |
| V-6 | 下推等价：SQL 选择器必须与 `spec.md` §13-15 纯选择器在同一固定夹具上产生同一结果，包含证据与 cut 校验、完整 `FactKey` / `authority_scope` 排序、tombstone 保留、Withdraw 解释。禁止 `event_time <= cutoff` 近似、`max(revision)` 前筛、先删 tombstone、把事务开始时间当作消费提交可见性 [R11]。 | 存储适配器 | 真实数据库与差分夹具 |

约束：V-1..V-6 的任何一项 MUST NOT 由 S-1..S-5 通过来推断（`spec.md`:1802 §32.1）。`temporal` 侧测试通过只证明规则合同，不证明 wire 或存储往返（TP-INV-048..TP-INV-050，`spec.md`:918-920）。

## 6. gate 状态汇总

状态词表只允许 `PASS / FAIL / NOT_RUN / BLOCKED / NOT_APPLICABLE`（`spec.md` §25.2，:1420）。本会话不运行 cargo，也不声称任何 `PASS`。`NOT_APPLICABLE` 只用于要求 1 与要求 2 的**库内**判定（库无序列化与传输代码），不用于 V-1..V-6——那六项是真实义务，只是本机不可执行。

| 项 | 状态 | 说明 |
| --- | --- | --- |
| S-1..S-5（`cargo test --test p1a_numeric` / `p1a_clock` / `p1b_precision` / `precision_contract` / `api_compile`） | **NOT_RUN** | 测试已存在；由主 Agent 执行并记录命令、退出码与时间 |
| V-1..V-6 | **BLOCKED（需外部输入）** | 本机无 consumer 检出、无 serde / prost / tonic / PostgreSQL 链路；`temporal` 禁止引入这些依赖 |
| §4 可选判据与 `spec.md` §22 新增 TP 条目 | **BLOCKED（需 Owner 裁定）** | 正本变更走 Spec Kit 流程与 `SCHEMA_VERSION` 评估 |
| 草案要求 1、2 的库内实现义务 | **NOT_APPLICABLE（库内）** | 理由见 §1 事实 1 与 2；对应 wire 义务改记为 V-1、V-2 |
| 草案要求 3、4、5 的库内读回校验 | 已在库内实现并有测试（S-1..S-4） | 判定为 refuted / 部分已有；执行状态仍是 NOT_RUN，直到主 Agent 运行 |

## 7. 与其他正本和工作包的关系

1. **WP-02（AUD-002 / `docs/evidence/DECISIONS-20261108.md` R3）**：`AsOfContext` 的 `evidence_schema_ref` 等引用是否参与判定、以何错误码拒绝，归 WP-02。本 ADR 的 W-9 只定义绑定义务，不实现比较。
2. **WP-05（可信适配器与 coverage）**：V-4 与 V-6 的往返与差分证据归存储适配器 Owner。coverage 完整性判定不在本 ADR。
3. **ADR-0001 §4**：DTO 与协议版本身份沿用同一 identity 口径——`schema_version`、`extractor_version`、`acceptance_ref`。wire schema 版本是 Acceptance 层记录的 `schema_version` 的来源之一。
4. **ADR-0003 §3**：复现清单已要求 policy / calendar / timezone / extractor 引用（`spec.md`:827）。DTO 版本与 wire schema 版本按同一口径进入复现清单。
5. **正本侧待办（不在本任务写范围，交 captain 与 t15）**：`spec.md` §31 与 §22 是否需要新增跨语言往返 TP 条目；`docs/API.md` 是否加一节 §31 指针（避免形成第二正文）；`checklist.md` 中 G-012 / §31 相关 Owner 登记。

## 8. 后果、可推翻条件、非目标与待裁定

- **正面后果**：草案的两处误判被明文更正；WP-07 不新增库内类型、不新增依赖；门禁可立即用 S-1..S-5 执行；消费者义务与 Owner 归属明确。
- **残留风险**：真正的精度风险全在消费者侧。`verify_projection`（`src/precision.rs`:89-103）只检查调用方传入的读回值，不证明后端真的按声明精度存储（`spec.md`:450）。`AsOfContext` 的 schema 引用当前不进入判定，所以 W-9 的绑定在库内不可强制。两项残留必须写进发布记录（`spec.md`:1432）。
- **禁止的完成声明**（`spec.md`:1436）：不得声称「库内精度测试通过 ⇒ wire 无损」、「能力声明证明后端精度」、「SQL 下推与纯选择器等价」。
- **可推翻条件**：若 Owner 决定让 `temporal` 提供 feature-gated 序列化（例如 `serde` feature），须 Gate C、`spec.md`:1426 兼容性评审与 TP-INV-050 重审；§2 的 W-B 需重写。
- **待 Owner 裁定**：R7-1 `spec.md` §22 是否新增跨语言与存储往返 TP 条目及 `SCHEMA_VERSION` 处理；R7-2 §4 的可选判据是否入库；R7-3 `docs/API.md` 是否加 §31 指针；R7-4 V-1..V-6 的 Owner 登记位置。
- **非目标**：不实现 JSON / Protobuf / gRPC 编解码；不新增 serde / prost / tonic / chrono 依赖；不实现 `verify_pg_dual_column`；不定义 wire schema；不改 `temporal` 公共 API 与错误码；不在本 ADR 内判定 coverage 或 schema 引用一致性。
