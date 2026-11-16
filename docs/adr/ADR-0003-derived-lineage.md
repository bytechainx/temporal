# ADR-0003：派生数据 lineage 合同归属（XH-TEMPORAL-REMEDIATION-002 / WP-06 / AUD-006）

**状态**：Proposed / Review Required（草案，含待 Owner 裁定项 R6，不构成 Accepted，不产生合入权）
**日期**：2026-11-08
**仓库与基线**：`bytechainx/temporal`，`goal/v0.2.0-pit-hardening`，基线 SHA `7cef81d`
**关联**：`.omc/drafts/1.md` AUD-006 / WP-06；`docs/audit/AUDIT-VALIDATION.md` §7（commit `942fb11d66f656057d8172322b4bba1c6289bc4f`）；`spec.md` §16.3（:818-830）、§17.1-§17.3（:840-863）、§18（TP-INV-016 / TP-INV-029 / TP-INV-030，:886、:899-900）、§22（TP-T038 :1182、TP-T101-TP-T106 :1280-1285）、§32.1（:1802）、§33.1；`goal.md` G-011；ADR-0001 §2、§4、§5；ADR-0002 §5
**Owner 侧**：消费者集 成范畴（Adapter / Product / Runtime Owner）

## 1. 背景与 T1 已核实事实

1. 库内没有 lineage 类型：`src/` 内无 `lineage`、`ActualExecution`、`Counterfactual`、`Simulation` 命中。唯一溯源字段是 `FactVersionParts.input_provenance_ref: &'a str`（`src/revision.rs:229`），构造时只做长度校验（`src/revision.rs:255`），读取点只有长度检查（`src/pit/selection.rs:348`）与同版一致性比较（`:495`）；不被解引用。
2. `PitManifestRef`（`src/pit/selection.rs:77-86`）只有 `context`、`coverage_manifest_ref`、`coverage_adapter_ref`、`coverage_acceptance_ref`，没有 lineage 字段；而 `spec.md:827`（§16.3 manifest 表）要求「lineage manifest | 派生查询使用」→ 规格与实现的记录缺口。
3. 该边界已由测试注释登记：`tests/acceptance_align.rs:1278-1282` 写明 TP-T101–TP-T106（派生下界、完成时间下界、历史伪造拒绝、反事实模拟、PIT join、模型实际可用性）均为消费方所有（`spec.md` §17），crate 不暴露派生数据 API；同文件 :1283-1318 只断言 crate 侧的 `Indeterminate` / `NotVisible` / `Visible`。
4. `spec.md` §17.1-§17.3 已完整定义草案要求的 lineage 字段、可用时间下界、两种执行模式与数据链控制，并在 `spec.md:863` 写明「这些训练/计算逻辑在消费方实现，temporal 只提供必要时间与版本合同」→ 草案「temporal 不实现 Feature Engine」与现行规格一致。
5. temporal 侧已有两项可核对的派生合同：`TemporalProfile::DerivedRecord`（`src/model.rs:29`）在构造期要求 `event_time` 或 `observation_time` 至少一项已知，否则 `ContextMismatch`（`src/revision.rs:267-275`），已由 `tests/profile_contract.rs:80` `tp_t038_derived_record_needs_semantic_anchor` 覆盖；`spec.md:407` 同时规定 `received_time` 是本地产物接纳边界，不冒充外部网络接收。
6. `checklist.md:25` G-011「派生数据继承知识边界」状态 NOT_RUN，Owner 与证据「待登记」。

缺口不在算法，而在记录与合同归属：`spec.md` 已经写了合同，仓内没有承载它的字段，也没有把 §17 的验收拆给消费方的合同文件。

## 2. 裁定：合同归属与是否新增库内类型

| 候选 | 含义 | 采纳 |
| --- | --- | --- |
| L-A | `temporal` 新增 `DerivedLineageManifest` 类型与 `PitManifestRef.lineage_manifest_ref` 字段，库内建模 lineage | 否 |
| L-B | `temporal` 不新增类型；把 `spec.md` §17 落成可核对的合同文档 + 现有字段承载 + 消费者实现 lineage adapter；lineage 记录落在消费者复现清单 | 是（建议） |
| L-C | 只在 ADR 文字里定义 lineage，不落任何可核对结构与测试归属 | 否 |

裁定理由：

1. `spec.md:863` 已把训练与计算逻辑归消费方，`spec.md` §32.1（:1802）明确要求 TP-T101–TP-T106 这类真实产品边界必须拆成 temporal 纯 fixture 断言与消费者集成证据。temporal 侧不存在可验证 lineage 内容的入口：`input_provenance_ref`、`manifest_ref`、`payload_ref` 一律不解引用（`spec.md`:1039；ADR-0001 §3）。
2. 否 L-A：新增 lineage 类型只搬运字段，不带来校验能力——链内容、计算完成事件、消费边界证据的核对属于 ADR-0001 的第 2–4 层。同时它会改变 `PitManifestRef` 公共 API（`spec.md`:1426、Gate C），并按 AGENTS.md 反过度设计纪律必须先回答「谁消费它、为什么现有字段不够、删除它失去什么」；`PitManifestRef` 当前只被构造并作为 `PitResult.manifest` 输出（`src/pit/selection.rs:258`、`:72`），库内没有对它自身的读取或校验点，新增字段不会进入任何判定路径。
3. 否 L-C：`spec.md:827` 的缺口会继续保持未登记，G-011 永远停在 NOT_RUN 且没有 Owner 归属，等于把 AUD-006 记成已处理。
4. L-B 的边界要说破：`temporal` 不验证「派生输出不早于输入可用性」，也不验证 lineage 是否存在。它只保证输入在给定边界不可见时不进入决策输出（现行为 `Indeterminate` / `NotVisible` / `NoVisibleVersion`，`tests/acceptance_align.rs:1283-1318`）。任何更强的断言必须在消费者 lineage adapter 内完成并出具证据。

R6（Owner 已裁，见 `docs/evidence/DECISIONS-20261108.md` R6 行）：WP-06 先定义版本化 lineage 记录义务与 Adapter 侧落点；`PitManifestRef` 新增 lineage 字段只能作为 Gate C 候选方案列出，不得直接改 `src/`。本文 §2 的 L-B 即该裁定的合同化表达；`spec.md:827` 要求 manifest 含 lineage manifest 与库内无该字段的冲突记录在案，v0.2.0 的处理方式是把 lineage 记录义务放在消费者复现清单，把 `spec.md` §16.3 该行的归属改写为「派生查询由消费者 lineage adapter 留存」（正本变更由 t15 执行）。

## 3. 版本化 lineage 记录义务（由消费者 lineage adapter 写入）

**谁能写**：只有消费方 Trusted Evidence Adapter / 派生计算组件（ADR-0001 §2 第 2–4 层）。`temporal` 的公开输出（`FactOutcome`、`PitResult`、`SelectedEvidenceRef`、`PitManifestRef`）不携带派生字段，也不生成 lineage。

一次派生记录 MUST 至少绑定以下项（`spec.md`:844 的展开）：

| 字段（消费者命名） | 内容 | 正本依据 |
| --- | --- | --- |
| 输出版本身份 | 派生记录 id、派生 schema 版本 | `spec.md`:844、:407 |
| 输入集合 | 每个实际使用输入的 `authority_scope` + `FactKey` + `RevisionId` + `semantic_digest`（链式事实另加 `chain_digest` 与 `algorithm_version`，ADR-0002 §5），或指向不可变输入清单的 `input_manifest_ref` | `spec.md`:844、ADR-0002 §5 |
| 输入查询上下文 | 该次查询的 `AsOfContext` 引用集：snapshot、knowledge basis、`known_at`、`authority_scope`、system cut、`policy_ref`、schema/interpretation 引用、limits | `spec.md` §16.3 :819-826 |
| 计算身份 | 计算代码 / 参数 / schema 版本、模型版本、训练数据快照引用 | `spec.md`:844、:863 |
| 执行模式 | `ActualExecution` 或 `CounterfactualSimulation`（二值，不得省略） | `spec.md`:852-857 |
| 完成边界 | 计算完成时间，以及该时间所用的受信时间引用 | `spec.md`:844、:846 |
| 消费边界证据 | 输出被目标边界允许消费的验收记录引用 | `spec.md`:844、ADR-0001 §4 |
| 验收 identity | `schema_version`、`extractor_version`、`acceptance_ref` 三元组，与 ADR-0001 §4 item 4 的验收记录同一版本化口径 | ADR-0001 §4；`docs/evidence/DECISIONS-20261108.md` R6 |

约束：

1. 复现清单必须能由上述字段唯一重建该派生结果：缺 `schema_version`、`extractor_version`、`acceptance_ref` 任一，该派生结果按不可复现处理（`spec.md` §16.3 :827-830）。
2. lineage 记录不可变：重新计算发布新记录并保留旧记录；历史查询必须能用其当时的记录复现，不得用当前重算覆盖（与 ADR-0002 §5 manifest 不可变性同一口径）。
3. 上述引用对 `temporal` 是 opaque：库只比较字节相等，不解引用、不验证内容（ADR-0001 §3 item 5；`spec.md`:1039）。
4. 多个 Owner 参与时（Adapter / Product / Runtime）必须分别出具自己的证据，不得互相代签（`spec.md` §33.1）。

## 4. 时间与边界规则（L-1..L-6）

| ID | 规则 | 判定 Owner |
| --- | --- | --- |
| L-1 | 真实系统派生产物的 `system_available` 保守界限不得早于：任一实际使用输入在对应边界的可用性、自身计算完成时间、输出被目标边界允许消费的时间三者中最晚者。仅设 `max(input.available_at)` 不够；用输入 `event_time` 最大值同样不合格 | Adapter / Runtime Owner |
| L-2 | 跨时钟比较必须使用同一受信时间解释，或显式的上下界证据；不得直接比较不同时钟的 instant | Adapter Owner（ADR-0001 §4） |
| L-3 | 今日离线重算不得标记为历史系统的 `ActualExecution` 可读。`CounterfactualSimulation` 可以今天运行，但必须分别保存生成时间、模拟知识时间、延迟模型、执行模式，禁止混成单一 timestamp | Product / Runtime Owner（TP-T103、TP-INV-030） |
| L-4 | PIT join 必须对每个输入取其在各自 cutoff 的可见修订；禁止先对全量最终值 join 再过滤 | Adapter Owner（TP-INV-016、`spec.md`:861、TP-T105） |
| L-5 | 滚动窗口使用当时可得输入；标准化与模型拟合不得无声明地使用决策之后的数据。标签可描述未来结果，但必须与该时点可用特征隔离并按训练数据策略管理 | Product Owner（`spec.md`:861） |
| L-6 | 模型版本另有训练数据快照、训练完成、审批 / 部署与可消费边界；不得因训练样本期间早于 T 就推断该模型当时可用 | Product / Runtime Owner（`spec.md`:863、TP-T106） |

`temporal` 侧对上述规则唯一可断言的事实是：输入在给定边界不可见时不进入决策输出（`Indeterminate` / `NotVisible` / `NoVisibleVersion`，现行为已由 `tests/acceptance_align.rs:1283-1318` 断言）。L-1..L-6 的判定全部在 Adapter / Product / Runtime 侧完成；`temporal` 测试通过不构成任何一条 L 规则的验收证据。

## 5. `temporal` 仓内可表达并可测的部分

| ID | 可表达内容 | 现有落点 | 状态 |
| --- | --- | --- | --- |
| S-1 | `DerivedRecord` 必须带语义锚点：`event_time` 或 `observation_time` 至少一项已知，否则 `ContextMismatch`；`received_time` 是本地产物接纳边界，不冒充外部发布 | `src/revision.rs:267-275`；`tests/profile_contract.rs:80` `tp_t038_derived_record_needs_semantic_anchor`；`spec.md`:407、TP-T038 :1182 | 已实现，测试 NOT_RUN（本会话未运行） |
| S-2 | `input_provenance_ref` 必须存在且长度受限；它是调用方声明的输入清单引用，库不解引用、不验证内容 | `src/revision.rs:229`、`:255`、`:371`；`src/pit/selection.rs:348`、`:495` | 已实现；rustdoc 待按 ADR-0001 §5 禁用语清单改写 |
| S-3 | 纯 fixture 断言：输入在截止点判 `Indeterminate` / `NoVisibleVersion` 时，该输入不进入决策安全输出；库输出不含「派生可用时间」字段，不能被当作 lineage 证据 | `tests/acceptance_align.rs:1283-1318`；`spec.md` §32.1（:1802）要求此类场景拆成 temporal 纯 fixture 与消费者集成证据 | 已实现，测试 NOT_RUN |
| S-4 | manifest 引用一致性只覆盖 `context` 与 `coverage_*_ref`；lineage 引用不在库内 | `src/pit/selection.rs:77-86`、`:258` | 现状即为合同边界，不新增 |

明确不做：`temporal` 不实现 Feature Engine，不计算派生可用时间，不执行 join、窗口、标准化、训练或回测逻辑（`spec.md`:863；草案要求 6）。

## 6. 后续落点（不直接改 `src/`，按 R6 口径）

1. 正本：`spec.md` §16.3 :827 行改写为「lineage manifest | 派生查询使用，由消费者 lineage adapter 留存并绑定复现清单」，并在 §17 加入 ADR-0003 指针；属正本变更，交 t15 / Owner 按 Spec Kit 流程与 `SCHEMA_VERSION` 评估执行。
2. `checklist.md:25` G-011 的 Owner 与证据登记：TP-T101–TP-T106 归 消费者集 成范畴（Adapter / Product / Runtime Owner，`spec.md` §33.1）；S-1..S-3 归 temporal Owner。登记前 G-011 保持 NOT_RUN。
3. rustdoc：`src/revision.rs:371`（`input_provenance_ref` 访问器）与 `docs/API.md` 相应条目按 ADR-0001 §5 清单改写，零语义变更。
4. Gate C 候选（仅候选，本 ADR 不建议在 v0.2.0 实施）：`PitManifestRef.lineage_manifest_ref`。列入理由：`spec.md:827` 要求 manifest 含 lineage 而库内无字段（`src/pit/selection.rs:77-86`）。实施前置条件：Gate C 通过、`spec.md`:1426 兼容性评审、AGENTS.md 反过度设计四问有答案、且出现真实的库内消费者。
5. 消费者侧：实现 lineage adapter 与反例 L1（`docs/audit/AUDIT-VALIDATION.md` §7：输入在 cutoff 判 `Indeterminate`，消费者把今日离线重算的输出标为 cutoff 时 system_available）。该拒绝判定需要 lineage 记录，在当前公共 API 内不可达 → 必须在消费者仓内实现与验收。

## 7. 验收映射与 gate 状态

状态词表只用 `PASS / FAIL / NOT_RUN / BLOCKED / NOT_APPLICABLE`（`spec.md` §25.2）。本会话未运行任何 cargo 命令，故无 `PASS`。

| 草案验收项 | 对应正本测试 | Owner | gate_status | test_commands |
| --- | --- | --- | --- | --- |
| 原始输入 revision 集合进入派生结果 manifest | TP-T101 一侧 | Adapter Owner（消费者集 成范畴） | BLOCKED（本机无 consumer 仓与 adapter） | 消费者仓集成命令，本仓不可表达 |
| 派生结果记录真实计算与消费边界 | TP-T102 | Adapter / Runtime Owner | BLOCKED | 同上 |
| 离线今日重算不得标为历史系统可读 | TP-T103、TP-INV-030 | Runtime / Product Owner | BLOCKED | 同上 |
| 派生可用时间不早于输入边界与自身完成边界 | TP-T101、TP-T102、TP-INV-029 | Adapter / Runtime Owner（判定）；temporal 仅可断言输入不可见不进输出 | 判定项 BLOCKED；temporal 侧可表达项 NOT_RUN | `cargo test --test acceptance_align`（主 Agent 执行） |
| 训练集 / 模型输入 / 回测特征绑定数据快照与 PIT policy | TP-T105、TP-T106、TP-INV-016 | Product Owner | BLOCKED | 消费者仓集成命令 |
| S-1 `DerivedRecord` 语义锚点 | TP-T038 | temporal Owner | NOT_RUN | `cargo test --test profile_contract` |
| S-3 输入不可见不进决策输出 | TP-T101/TP-T103 的 temporal 纯 fixture 半边 | temporal Owner | NOT_RUN | `cargo test --test acceptance_align` |
| S-2 rustdoc 改写 | ADR-0001 §5 | temporal Owner | NOT_RUN | `cargo test --test api_compile` |

`BLOCKED` 的理由统一：本仓无消费者检出、无 Trusted Evidence Adapter 实现、无 lineage adapter；`BLOCKED` 不得写成 `PASS`，也不得由 S-1..S-3 的通过推断（`spec.md` §32.1 :1802、§33.1）。不使用 `NOT_APPLICABLE`：这些义务真实存在，只是本机不可执行。

## 8. 后果、可推翻条件与非目标

- 后果（正面）：AUD-006 的记录缺口变成有 Owner、有字段清单、有状态词表的合同；`spec.md:827` 与实现的冲突被显式登记而不是被静默忽略；WP-06 与 WP-05（Trusted Evidence Adapter）共用同一 `schema_version` / `extractor_version` / `acceptance_ref` 版本化口径，避免两套 identity。
- 后果（负面与残留风险）：采纳 L-B 后，`temporal` 仍不能在库内拒绝「派生输出早于输入可用性」；该保护完全依赖消费者 adapter 是否实现与是否出具证据。若某消费者不实现 lineage adapter，本合同只构成违约记录，不构成技术阻断。此为 v0.2.0 的已知边界，须在发布记录中保留（`spec.md`:1432）。
- 可推翻条件：若 Gate C 批准 `PitManifestRef.lineage_manifest_ref`（§6 item 4），§2 的 L-B 与 §5 的 S-4 需重写，且新增字段须进入 `spec.md` §20 输入模型与 §22 验收矩阵；若 Owner 改为要求库内 lineage 校验（即让 `temporal` 判定 L-1），本 ADR 需要重开，因为那会引入新的拒绝路径与新的错误码，属行为变更。
- 待裁定：Gate C 是否受理 §6 item 4；`spec.md` §16.3 :827 与 §17 的正本改写口径由 t15 确认；G-011 的 Owner 登记文本由 Owner 确认。
- 非目标：不实现 Feature Engine、特征计算、join、窗口、训练或回测；不在 `temporal` 计算哈希或摘要；不新增依赖（`spec.md`:475；`.omc/drafts/1.md` §1.1）；不定义 chain digest 语义（ADR-0002）、证据信任分层（ADR-0001）、Wire 精度（WP-07）；不在本任务写 `spec.md`、`goal.md`、`checklist.md`、`src/`、`tests/`。
