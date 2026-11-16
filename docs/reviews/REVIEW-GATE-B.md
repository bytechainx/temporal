# REVIEW-GATE-B：Evidence 信任边界独立审查（ADR-0001 / AUD-002 措辞）

**状态**：审查已完成（只读静态审查；本轮未运行任何 cargo 命令）
**Reviewer**：`security-reviewer`（`.omc/drafts/1.md` §8：负责 Evidence boundary；不可批准自己的 trust adapter）
**复审（t16 attempt 2）**：`release-reviewer`。§1–§8 是 attempt 1 记录，未被推翻；attempt 2 的独立核验、状态更新与新增 findings 见 §9 与 §10。attempt 2 对 §5 表中 `DEF-1` 与 §7 第 7 项的 `DEF-1` 做了编号勘正（该编号在 attempt 1 文本中无定义，实指 F-7），并补齐 §4 F-7 前向引用的 §9。
**日期**：2026-11-08
**仓库 / 基线 / 审查对象**：`bytechainx/temporal`，分支 `goal/v0.2.0-pit-hardening`，基线 `7cef81d`。审查对象 `docs/adr/ADR-0001-evidence-trust-boundary.md`（122 行，最后修改于 `cca9a4e`，此后未变）。审查期间 HEAD 由 `0820462` 前进到 `115ea53`（新增 ADR-0003 文档与 t20 措辞改写 commit）。
**改动面核验**：`git diff 7cef81d..f91126f -- src/` 为空 → 该行号核验对 `src/` 的基线版本成立。至 `115ea53`，`src/` 相对基线的唯一差异是 t20 的 14 行 rustdoc 改写（`src/evidence.rs` 10 行、`src/revision.rs` 4 行，纯注释、行数不变），因此本审查引用的全部 `src/` 行号与判定语义在两个 HEAD 上均有效；`src/pit/` 与 `src/error.rs` 与基线一致。
**并发环境**：本 worktree 由多名成员共用同一 git index。本次审查的 `docs/reviews/REVIEW-GATE-B.md` 曾被并发 commit `3924dc2`（t13 的 ADR-0003 提交）带入仓库；文件内容未受影响。记录此事实以免下游误读提交归属。
**措辞改写 commit 核对（依 captain 指引）**：审查期间 t20 的改写已 commit —— `7813b6f docs(evidence): restate proof wording as structural reference checks per ADR-0001 5`（`src/evidence.rs`）与 `115ea53 docs(revision): restate chain digest and source release wording per ADR-0001 5`（`src/revision.rs`）。该 diff 经核验为**纯注释变更**：`git diff -U0 f91126f..115ea53 -- src/` 中除 `///` 行外无任何代码行改动（`14 insertions(+), 14 deletions(-)`）。「零语义变更」要求满足。实施覆盖度核对见 §9，结论为 F-7。
**verdict**：**NEEDS_REVISION**（findings：high 1、medium 5、low 1）。Gate B 当前不可记 `PASS`。

## 1. 审查方法

取证只读：`docs/adr/ADR-0001-evidence-trust-boundary.md`（122 行）、`docs/audit/AUDIT-VALIDATION.md` §3（:97-134）与 §9（:296-300）、`docs/evidence/DECISIONS-20261108.md`（32 行）、`src/evidence.rs`、`src/revision.rs`、`src/pit/selection.rs`、`src/pit/context.rs`、`src/pit/visibility.rs`、`spec.md` §10.1/§11.2/§11.4/§12.3/§16.3/§21.4/§22.7/§25.2-25.5/§32.1/§32.3/§33.1、`docs/API.md`（52 行）、`tests/` 目录与 `tests/pit_contract.rs:874-931`。未运行 build/test/clippy（任务约束「不 build」）。ADR 的 §1、§3、§5、§6、§7 引用点逐条对源码与规格核对，结果见 §3。

## 2. Gate B 五维度结论

| # | Gate B 条目（`.omc/drafts/1.md` §4） | 结论 | 依据 |
| --- | --- | --- | --- |
| 1 | 结构校验与真实性验证职责分离 | **满足（合同层）** | ADR §2 四层表 + §3 不执行清单 + §4 适配器义务，与 `spec.md:522-524`（§10.1）、`spec.md:1039`、`spec.md:1440`（§25.5）一致；`temporal` 不实现摘要、签名、解引用、验收存续判定、适配器受信判定 |
| 2 | proof refs 绑定版本身份 | **部分满足 → 需修订** | 版本身份绑定确实存在且 ADR 描述准确（`src/revision.rs:314-318`、`:319-323`、`:335-339` → `EvidenceMismatch`）；`schema_version`/`extractor_version` 的等值绑定已裁定不在 v0.2.0 实现（`docs/evidence/DECISIONS-20261108.md:23`），但 ADR 未记录该裁决、未交代 `spec.md:1229`（TP-T070）的归属后果，且审计的 R3 与裁决记录中的 R3 指向不同事物 → F-1 |
| 3 | Cut manifest 与证据快照绑定 | **未定义 → 需修订** | ADR 全文未处理 `cut.manifest_ref()` / `cut.evidence_snapshot_ref()` / `known_at_binding_ref()` / `consistency_policy_ref()`；库内它们只做 label 长度检查（`src/pit/context.rs:287-290`、`src/pit/selection.rs:307-311`），与实际消费证据集合无任何比较；ADR:7 的排除清单也未列该项 → F-2 |
| 4 | consumer / boundary / epoch / partition scope 完整 | **部分满足（如实标注缺口）** | ADR §3 准确列出 `src/evidence.rs:522-540` 与 `src/pit/selection.rs:577-587` 的边界/代次/分区/`known_at` 判定；`validate_for`（`src/pit/context.rs:458-470`）只重绑 `authority_scope`、`dataset_snapshot`、`knowledge_basis`、`known_at`，不含 consumer/boundary/epoch 覆盖。该缺口属 AUD-001 / R2（WP-01），不在本 ADR 范围；ADR 未越权声称已解决 → 不计 finding |
| 5 | 外部受信适配器合同可被消费者独立验证 | **部分满足 → 需修订** | ADR §4 九条义务方向正确（含 §33.1 多 Owner 不互签、§16.3 复现项、§21.4 授权独立）；但合同的规范正本与 Owner 未定（F-5），验收/覆盖证据相对 `known_at` 的时效义务缺失（F-4） |
| — | 外部受信适配器集成测试通过 | **BLOCKED（如实）** | ADR §7 D 项 `BLOCKED` 判定成立：本机无消费者仓与适配器实现；不得由 temporal 结构测试推断（ADR:114） |

## 3. ADR 事实引用核验（全部命中）

逐条核对 `7cef81d` 源码，ADR 的 §1、§3、§5、§6 引用点**全部准确**，未发现虚构行号或错误归属：

- `label()` 非空 + 字节上限：`src/evidence.rs:8-16`；`src/revision.rs:12-24`（`checked_bytes`/`checked_label`）；`src/pit/context.rs:9`（`label`）。
- `TimeBounds::try_new` 区间顺序与 `Exact` 上下界相等：`src/evidence.rs:49-69`（`:57-59` 顺序、`:60-62` Exact 相等）。
- `EvidenceProofRef::try_new` 五字段仅经 `label()`：`src/evidence.rs:116-131`；字段 `evidence_id/schema_version/extractor_version/raw_evidence_ref/acceptance_ref`：`src/evidence.rs:106-112`。
- 证据与版本身份绑定：`src/revision.rs:314-318`（source_evidence）、`:319-323`（receipts）、`:335-339`（system_evidence）→ `EvidenceMismatch`；重复不一致判定：`:324-334`、`:340-350`；选择期去重：`src/pit/selection.rs:600-654`。
- `proof` 各字段仅长度检查：`src/pit/selection.rs:408-426`（`validate_evidence_refs`）。
- schema 引用不参与判定（ADR:49 的结论成立）：`src/pit/context.rs:228`、`:244-245`（仅 label）、`:343-348`（仅访问器）；`src/pit/selection.rs:416`（`proof.schema_version()` 仅长度检查）。全 `src/` grep 证实 `evidence_schema_ref` 与 `interpretation_ref` 无比较读取点。
- coverage 重绑：`src/pit/context.rs:458-470`（`validate_for`：`state != Complete → IncompleteCandidateSet`，只比较四项）；`src/pit/selection.rs:127`（`select_impl` 调用它）；`:260-262` 输出 manifest 只是回显 `input.coverage.*_ref()`。
- 措辞清单 20 行 / 27 处的引用文本与基线 rustdoc 逐字一致（核对 `src/evidence.rs:21/30/32/188/192/342/420/435/505`、`src/revision.rs:131/150/154/222`、`src/pit/selection.rs:16/34/38/45/57/62/71/82`、`src/pit/context.rs:121/355/445`、`src/pit/visibility.rs:50/52/54`）。
- 正面样例判定正确：`src/evidence.rs:48`、`:104`、`:115` 后半句、`src/revision.rs:59`、`:89`、`:248` 确实已带限定语；`docs/API.md` 四条免责声明存在（`:15`、`:19`、`:32`、`:42`）。
- 规格引用存在且语义相符：`spec.md:520-527`、`647-653`、`815-830`、`1114-1120`、`1229`、`1420-1424`、`1426-1430`、`1436-1440`、`1789`、`1812`、`1863-1867`；`goal.md` G-007/G-009/G-010/G-015 存在（`goal.md:123/135/141/171`）。
- gate 词汇符合 `spec.md:1422-1424`；ADR §7 只用 `NOT_RUN`/`BLOCKED`，无 `PASS`，并声明本轮未运行 cargo（ADR:105）。

**结论**：ADR 对基线代码事实的描述准确、无夸大；未违反 `spec.md:1440`（§25.5）禁止的完成声明。下列 findings 是合同覆盖与内部一致性问题，不是事实错误。唯一滞后项：ADR §6 仍以「待 Owner 裁定」呈现 D-1/D-2/D-3（ADR:95-101），而裁决已在 `docs/evidence/DECISIONS-20261108.md:23-25` 落定 → 见 F-1。

## 4. Findings

### F-1（medium）§6 未记录已落定裁决，且 TP-T070 与审计 R3 的归属未闭合

- **位置**：ADR §4 第 5 条（ADR:56）、§6 D-1（ADR:99）、§7 C（ADR:111）、§8 负面项（ADR:120）。
- **证据**：裁决已落定：`docs/evidence/DECISIONS-20261108.md:23`（D-1：v0.2.0 不做库内 `evidence_schema_ref`↔`proof.schema_version()`、`interpretation_ref`↔`extractor_version()` 等值比较，一致性属 Adapter 第 4 层）、`:24`（D-2 不改名）、`:25`（D-3 归 t15）。ADR 仍写「待 Owner 裁定」，未引用该登记文件。同时存在 ID 冲突：`docs/audit/AUDIT-VALIDATION.md:298` 的 **R3** 定义为「证据/快照 schema 版本绑定校验的强度与错误码（TP-T070 要求 `EvidenceMismatch` / `Indeterminate`）」并给出处置「按 `spec.md:1229` 直接实现，不改规格」；`docs/evidence/DECISIONS-20261108.md:26` 的 **R3** 却是「AUD-002 措辞 → t20」。审计 R3 的处置方向（库内实现绑定校验）与 D-1 裁决（库内不做）方向相反，两份文件都没有记录这次覆盖。
- **未闭合的规格后果**：`spec.md:1229`（§22.7 TP-T070）是 temporal 侧场景，期望「证据绑定另一版本/source audience/digest → 明确错配返回 `EvidenceMismatch`；缺绑定返回 `Indeterminate`；均不 `Visible`」。按 D-1 定稿口径，「schema 版本错配」分支不由 `temporal` 实现；已实现的是「绑定另一版本」分支（`src/revision.rs:314-338`）与缺绑定 `Indeterminate`。ADR 与裁决记录都没写明 TP-T070 是否拆分（哪一段归第 4 层、哪一段仍是库内义务），也未记 §25.3 兼容性/CHANGELOG 结论。若下游按 `spec.md:1229` 原文验收，Gate D/E 会把该项读成 `FAIL`（规格要求而实现缺席），而不是如实的 `BLOCKED`/`NOT_APPLICABLE`。
- **影响**：Gate B 第 2 项无法在当前文本上判定；t15（spec 正本）与 t18（Gate E 前置）缺少可执行口径。
- **requiredFix**：(1) §6 三行改为「已裁定」并引用 `docs/evidence/DECISIONS-20261108.md:23-25` 与 commit；(2) ADR 增加一条 TP-T070 拆分说明：`schema_version`/`extractor_version` 错配 → 第 4 层义务（列 Owner 与验收落点），`版本身份`错配 → 库内既有 `EvidenceMismatch`（`src/revision.rs:314-338`），并注明需 t15 在 `spec.md` §22.7 与 CHANGELOG 记录该拆分（按 `spec.md:1426` §25.3）；(3) 裁决登记侧消解 R3 ID 冲突：把「措辞改写」与「schema 绑定校验处置」记为两个不同编号，或在 `:26` 显式写明它取代 `AUDIT-VALIDATION.md:298` 的 R3。

### F-2（high）Cut manifest 与证据快照绑定未定义，也未声明排除

- **位置**：ADR:7（范围排除清单）、§3 检查表（ADR:30-38）、§4 第 6 条（ADR:57）。
- **证据**：`spec.md:588`（`cut_id / manifest_ref` = 不可变历史切片及其完整性证据引用）与 `spec.md:592`（`evidence_snapshot_ref` = 当时所需 receipt/visibility 证明的集合或可重建引用）已定义语义；`spec.md:589` 要求 `known_at_binding` 表达「绑定关系和证据版本」、`spec.md:591` 要求 `consistency_policy` 表达「该 cut 如何保证是受控可读视图，及其版本」。基线库内 `src/evidence.rs:498`、`:506`、`:514`、`:518` 只暴露 getter；`src/pit/context.rs:287-290` 与 `src/pit/selection.rs:307-311` 只做 label/长度检查；`src/evidence.rs:522-540` 的 `contains` 只比较 `boundary_id`、`known_at`、`log_epoch`、`partition`、`cursor <= inclusive_frontier`，不涉及 `manifest_ref` 或 `evidence_snapshot_ref`。`docs/audit/AUDIT-VALIDATION.md:226` 已判定「实现把这些字段当作不透明标签」并记入 R5；`:300` 的 R5 要求「至少要求 `contains` 三态化」。ADR §3 列出 cut 判定却未陈述这四个引用不参与判定；§4 只覆盖 `coverage_manifest_ref` / `coverage_acceptance_ref`；ADR:7 的排除清单只列 WP-03/WP-06/WP-07，未列该项，读者会认为已覆盖。`docs/evidence/DECISIONS-20261108.md:12` 宣布「R5 与 R2 为同一裁决」，但 R2 的裁决内容是扩展 `CandidateCoverageRef` 的 boundary/epoch 覆盖与改写三处 `NotVisible` 断言，未给 `manifest_ref` / `evidence_snapshot_ref` 任何比较规则 → R5 的该项仍未回答。
- **影响**：Gate B 第 3 项无落点：消费者无法据本 ADR 判断「本次查询实际使用的证据集合」与 cut 声明的证据快照是否同一集合；`AUD-005` 修复要求 4（验证同一 Cut 的一致性策略和证据快照引用）失去合同入口。
- **requiredFix**：三处补写：(1) §3 增加事实行——`manifest_ref` / `known_at_binding_ref` / `consistency_policy_ref` / `evidence_snapshot_ref` 库内仅长度检查，与实际消费证据无比较；(2) §4 增加一条适配器义务——适配器交付给 `temporal` 的 receipt/visibility 证据集合必须可由该 cut 声明的 `evidence_snapshot_ref` 重建，缺口或不可重建时适配器必须拒绝，不得写入 `temporal` 输入；(3) 在 §6 或范围声明显式写明该项落 `WP-04` / `AUD-005` 要求 4 并引用 R5（注意 WP 标签冲突：`docs/audit/AUDIT-VALIDATION.md:22`、`:300` 写作 `WP-05`，`.omc/drafts/1.md` §5 的 WP-04 = Cut 完整性，需 Owner 一并纠正；`docs/evidence/DECISIONS-20261108.md:27` 已把 `src/evidence.rs:535-539` 的 confirmed 缺陷纳入 t9/WP-04）。

### F-3（medium）§5 措辞清单不完整，与 §5 自设禁用语规则冲突，且正在被实施复制

- **位置**：ADR §5 禁用范围（ADR:66）、§5 清单（ADR:68-90）。
- **证据**：对 `src/` 全量 rustdoc 扫描（`///` 行命中「证明 / 受信 / 可信 / 已验证 / 确保真实」）得 52 处，清单只处理 27 处。与已列行同族的漏项：`src/evidence.rs:249`「受信入口验收记录」（与已列 `:192`、`:342` 完全同句）；`src/pit/selection.rs:80`「受信枚举器的覆盖清单引用」与 `src/pit/context.rs:449`「受信适配器身份与版本」（与已列 `src/pit/selection.rs:82` 同族）；`src/pit/context.rs:302`「包含已证明的相同时刻」。其余未分类命中：`src/evidence.rs:38`、`:149`；`src/revision.rs:128`、`:133`、`:224`、`:366`；`src/pit/context.rs:124`、`:352`、`:357`、`:361`；`src/pit/visibility.rs:19`、`:21`、`:25`、`:27`；`src/error.rs:142`；`src/model.rs:30`。`docs/API.md` 侧未处理：`:26`「已验证时间 profile」（对应已标记的 `src/revision.rs:222`）、`:52`「本次证明可见的引用」（对应已标记的 `src/pit/selection.rs:38`）、`:34`「…和一致性证明」（对应已标记的 `src/evidence.rs:420`）。**实施侧已在复制该缺口**：工作树未提交的 `src/evidence.rs` 改写只覆盖 §5 已列位置（`:21`、`:30`、`:32`、`:115`、`:188`、`:192`、`:342`、`:420`），上述漏项原文未变（核对工作树 `src/evidence.rs:38`、`:149`、`:249` 与 `src/pit/selection.rs:80`、`src/pit/context.rs:302`、`:449` 原文仍在）。
- **影响**：rust-engineer 按 §5 表实施后，rustdoc 与 `docs/API.md` 仍留禁用语，改写 PR 直接违反 ADR 自设规则，或被迫扩大未经评审的改动面（违反 AGENTS.md「精准修改」与 `spec.md:1426` 兼容性评审边界）。
- **requiredFix**：把 §5 落点从「固定 27 处清单」改为可复现判定规则：对禁用语列表全量扫描 `src/**/*.rs` 的 `///` 行与 `docs/API.md`，逐命中分类为「改写」或「保留 + 一句理由」，并把该扫描命令写入 §7 的验收证据要求；或在 t20 完成前补全上述漏项。

### F-4（medium）验收与覆盖证据缺少相对 `known_at` 的时效义务

- **位置**：ADR §4 第 3、6、7 条（ADR:54、:57、:58）。
- **证据**：§4 第 3 条要求验收记录包含「验收时间」，但没有任何一条要求验收时刻、`acceptance_ref` 或 `coverage_acceptance_ref` 相对被查询的 `known_at` 是历史可复核的，也未要求适配器保留做出该验收时的日志/manifest 状态引用。`spec.md:606`（§11.4）规定「`SystemAsKnown` 中新增的证据不能虚构历史 receipt/cut；只能重建已有受信日志证明的历史事实」。ADR:44 已正确声明 `temporal` 不判验收记录的存续与有效性，因此该义务只能由 §4 承担。ADR:7 的排除清单与 §7 映射也未覆盖该项。
- **影响**：今天生成的 `acceptance_ref` 可被用于一次 `known_at` 在过去的系统视图查询而通过全部库内检查（仅长度检查），信任边界在第 4 层出现回填空洞；消费者无法据 §4 独立复核验收证据的历史有效性。
- **requiredFix**：§4 增加一条：验收记录必须记录验收执行时刻与被验收日志/manifest 的状态引用（可重建的序号、epoch 或 manifest 版本）；`SystemAsKnown` 查询引用的验收与覆盖证据必须可追溯到 `known_at` 前已存在的受信日志条目；不可追溯时适配器必须拒绝，不得写入 `temporal` 输入。

### F-5（medium）适配器合同的规范正本与 Owner 未定

- **位置**：ADR:3（状态行）、§4 标题（ADR:48）、§6 D-3（ADR:101）。
- **证据**：ADR 自述「Proposed / Review Required（草案，不构成 Accepted）」；`docs/audit/AUDIT-VALIDATION.md:113` 记录「草案要求的外部 Trusted Evidence Adapter 合同在本仓无落点：仓内无该合同文件」。裁决只落定 `docs/API.md` 与 `spec.md` 正本由 t15 交付（`docs/evidence/DECISIONS-20261108.md:25`），未指定 §4 这份合同的正本文件、版本化方式、以及按 `spec.md:1789`（§32.1）与 `spec.md:1863-1867`（§33.1）的 Owner 归属；ADR 自身也未把合同正本列为 §6 待裁定项。
- **影响**：Gate B 第 5 项要求的「合同可被消费者独立验证」缺少可验证对象：消费者现在只能读到一份未 Accepted 的 ADR。
- **requiredFix**：§4 开头补一条落点决策并列入 §6：合同正本（新增 `spec.md` 小节或 `docs/` 下独立合同文件，与 t15 交付边界对齐）、版本化方式（按 `spec.md:1428` 的 PolicyRef/schema 版本独立于 crate SemVer 原则）、Owner（Adapter / Product / Runtime），并声明 ADR Accepted 之前的效力状态。

### F-6（low）§7 验收映射与既有测试覆盖不一致

- **位置**：ADR §7 C（ADR:111）。
- **证据**：`tests/pit_contract.rs:874` `tp_t059_060_070_missing_proof_and_invalid_chain_fail_closed` 名称声称覆盖 TP-T070，但断言只覆盖分叉拒绝（`InvalidRevisionChain`，`:913-920`）与缺证据 `Indeterminate`（`:921-931`），不含 schema/转换绑定错配的 `EvidenceMismatch`；`docs/audit/AUDIT-VALIDATION.md:112` 明确草案两条测试「针对的行为当前不存在」。§7 用 `NOT_RUN` 描述 C 项并把命令写为 `cargo test --test pit_contract`。
- **影响**：`NOT_RUN` 会让下游 Release Reviewer 把「pit_contract 全绿」读成 C 项已有证据，而实际状态是「按 D-1 定稿口径库内不实现该判定 + 第 4 层测试待写」。
- **requiredFix**：在 §7 区分「既有测试未运行」（`NOT_RUN`）与「需先新增合同/测试」（`BLOCKED`，注明被 F-1 拆分说明阻塞），并注明 `tp_t059_060_070_*` 只覆盖 TP-T070 的缺证据与链错分支。

### F-7（medium）措辞改写实施只覆盖 §5 清单的一半，并新引入一处完成式声明

- **位置**：commit `7813b6f`（`src/evidence.rs`）与 `115ea53`（`src/revision.rs`）；对照 ADR §5 清单（ADR:68-90）。
- **证据**：见 §9 的逐文件核对。两个 commit 的 diff 为纯注释变更（无代码行改动、行数不变），「零语义变更」成立；正面样例未被误改（`src/evidence.rs:48`、`:104`、`:115` 后半句、`src/revision.rs:59`、`:89`、`:248` 无 hunk）。但 §5 声明的 27 处只落地 13 处：`src/evidence.rs` 10 处与 `src/revision.rs` 4 处已完成；`src/pit/selection.rs`（§5 列 `:16`、`:34`、`:38`、`:45`、`:57`、`:62`、`:71`、`:82` 共 8 处）、`src/pit/context.rs`（`:121`、`:355`、`:445` 共 3 处）、`src/pit/visibility.rs`（`:50`、`:52`、`:54` 共 3 处）在两个 commit 中**无任何改动**，`docs/API.md` 亦未处理。另 `src/evidence.rs:188` 的新文本「适配器声明并已验收的实际发布界限。」在 `temporal` 的 rustdoc 中断言外部验收已完成，属 ADR:66 禁用语族的完成式声明（`spec.md:1440`），且与同族 `:192`、`:342` 的新文本「外部验收记录引用；本库不验证该记录」自相矛盾。
- **影响**：ADR §5 的目标状态（rustdoc 与 `docs/API.md` 不再使用禁用语）尚未达成；按 ADR 字面判定，WP-02 的措辞交付不完整；Gate B 第 1 项不能转 `PASS`。`已验收` 措辞若合入，会把「验收已完成」写进库内 API 文档，正是 AUD-002 指控的形态。
- **requiredFix**：(1) 补齐 `src/pit/selection.rs`、`src/pit/context.rs`、`src/pit/visibility.rs` 的 14 处与 `docs/API.md`（与 F-3 的补全清单合并执行，按修订后的 §5 全量扫描规则）；(2) 把 `src/evidence.rs:188` 改为「适配器声明的发布界限；其验收由外部记录承载，本库不核验」，并把「已验收」写入 §5 禁用语列表。

## 5. AUD-002 修复要求逐项覆盖（`.omc/drafts/1.md`:99-106）

| 要求 | ADR 落点 | 覆盖判定 |
| --- | --- | --- |
| 1 文档明确区分 Structural Validation / Integrity / Authenticity / Acceptance | §2 四层表（ADR:17-24） | 覆盖，质量高 |
| 2 `temporal` 只负责结构、引用绑定、时间边界、选择规则 | §2（ADR:24）、§3（ADR:26-46） | 覆盖；引用绑定的深度与 cut 侧引用未闭合 → F-1、F-2 |
| 3 外部 Trusted Evidence Adapter 负责原始校验、摘要、签名/来源、验收 | §4 第 1-3 条（ADR:52-54） | 覆盖；时效与正本缺口 → F-4、F-5 |
| 4 不允许 API 文档把 `try_new` 描述成真实性证明 | §5（ADR:62-93） | 规则正确，清单不完整且正在被复制 → F-3、F-7 |
| 5 对所有 proof refs 保留版本化 schema / extractor / acceptance identity | §4 第 4 条（ADR:55）、§6 D-1/D-2 | 字段层覆盖（`src/evidence.rs:106-112` 三字段齐备）；判定归属需与 TP-T070 对齐 → F-1；cut 级引用无 identity 合同 → F-2 |

## 6. Gate B 清单状态（词汇依 `spec.md:1422-1424`）

| Gate B 项 | gate_status | 说明 |
| --- | --- | --- |
| 结构校验与真实性验证职责分离 | NOT_RUN（合同已定，实施待 WP-02 代码 PR） | 待 t20 措辞 commit 后复审（F-7） |
| proof refs 绑定版本身份 | BLOCKED | 被 F-1（TP-T070 拆分与 R3 记录）阻塞；库内实现已按 `DECISIONS-20261108.md:23` 裁定不做 |
| Cut manifest 与证据快照绑定 | BLOCKED | 本 ADR 未定义（F-2）；R5 该子项未回答 |
| consumer / boundary / epoch / partition scope 完整 | BLOCKED | 属 AUD-001 / R2（WP-01），不在 ADR-0001 范围 |
| 外部受信适配器集成测试通过 | BLOCKED | ADR §7 D 判定成立；本机无消费者仓与适配器实现，不得由 temporal 结构测试推断 |

本审查未运行：`cargo fmt --check`、`cargo clippy --all-targets --all-features -- -D warnings`、`cargo test --all-targets --locked`、`cargo doc --no-deps --locked`、`cargo package --locked`、MSRV 1.88、三平台矩阵 → 全部 `NOT_RUN`（依 `.omc/drafts/1.md` §7「Agent 不得自行把 NOT_RUN 改成 PASS」）。主 Agent 在 `0820462` 的 gate 日志显示 `tests/repro_audit001.rs` 四测试全绿，属他人执行证据，本审查不据此改变任何 gate_status。

## 7. 复审进入条件

ADR-0001 修订版需同时满足，security-reviewer 才考虑 `PASS`：

1. §6 按 F-1(1) 记录 D-1/D-2/D-3 的已落定裁决与来源，并按 F-1(2) 写明 TP-T070 的拆分与 t15 正本/CHANGELOG 义务。
2. §3 与 §4 补齐 F-2 的 cut manifest / evidence snapshot 事实、适配器重建义务与 WP-04 / R5 指向（含 WP 编号冲突纠正）。
3. §5 给出可复现的全量禁用语扫描规则（含 `docs/API.md`），或补全 F-3 列出的漏项。
4. §4 新增 F-4 的验收/覆盖时效义务。
5. §4 或 §6 写明适配器合同正本、版本化方式与 Owner（F-5）。
6. §7 按 F-6 区分 `NOT_RUN` 与「需先新增合同/测试」。
7. t20 措辞 commit 落地后，按 §5 修订版核对 diff（F-7）；此项不构成 ADR 修订的阻塞，但构成 Gate B 第 1 项转 `PASS` 的前提。

## 8. Owner / captain 已落定项（本审查不重开）

依 `docs/evidence/DECISIONS-20261108.md`，本审查接受以下口径并仅检查其记录一致性：D-1 不做库内 schema/extractor 等值比较（`:23`）、D-2 v0.2.0 不改名（`:24`）、D-3 `docs/API.md` 归 t15（`:25`）、措辞改写单列 t20（`:26`）、`src/evidence.rs:535-539` 缺陷纳入 t9/WP-04（`:27`）。在这些口径下，Gate B 第 1、2 项的 `PASS` 只能来自「§4 合同被 Accepted + 消费者集成验收（ADR §7 D）」，而 D 项在本机为 `BLOCKED`；因此即便 ADR 修订完成，Gate B 也不能在 temporal 单仓内被判定为 `PASS`。这一点 ADR §7 已如实表达，属正确做法。

本文件为审查记录，不构成 Accepted，不产生合入权（`spec.md:1424`、AGENTS.md「机器全绿不代替人审」）。

## 9. 措辞改写逐文件落地核对（§4 F-7 前向引用本节；attempt 2 补齐）

核对基准：基线 `7cef81d` → HEAD `c438d40`。取证命令：`git diff -U0 7cef81d..c438d40 -- src/`、`git diff --stat 7cef81d..c438d40`、`git grep -n -E "^\s*///.*(证明|受信|可信|已验证|确保真实)" c438d40 -- src/`、`git grep -n -E "证明|受信|可信|已验证" c438d40 -- docs/API.md`、`git diff --stat 7cef81d..c438d40 -- docs/API.md`（输出为空）。全部 diff 均为 `///` 注释行改动，行数不变 → 「零语义变更」在 `c438d40` 仍成立。

| 文件 | ADR §5 列出位置（逐位置清点 28 处） | 落地 commit | `c438d40` 时未落地 |
| --- | --- | --- | --- |
| `src/evidence.rs` | `:115`、`:21`、`:30`、`:32`、`:188`、`:192`、`:342`、`:420`、`:435`、`:505` 共 10 | `7813b6f` | 无（`:188` 文本缺陷见 F-8） |
| `src/revision.rs` | `:131`、`:150`、`:154`、`:222` 共 4 | `115ea53` | 无 |
| `src/pit/selection.rs` | `:16`、`:34`、`:38`、`:45`、`:57`、`:62`、`:71`、`:82` 共 8 | `0fcc4bc`（另含 F-3 漏项 `:80`） | 无 |
| `src/pit/context.rs` | `:121`、`:355`、`:445` 共 3 | `c438d40`（另含 F-3 漏项 `:124`、`:302`、`:352`、`:357`、`:361`、`:449`） | 无 |
| `src/pit/visibility.rs` | `:50`、`:52`、`:54` 共 3 | 无 | 全部 3 处 |
| `docs/API.md` | 未列入 ADR §5 | 无 | `:26`、`:34`、`:52`（见下） |

落地进度：25 / 28。attempt 1 核对时为 13 / 28。

禁用语全量残留（`c438d40`，`src/` 内 `///` 命中 26 处）：

- 判定需处置（16 处）：`src/error.rs:142`；`src/evidence.rs:38`、`:149`、`:249`；`src/model.rs:30`；`src/pit/visibility.rs:19`、`:21`、`:25`、`:27`、`:50`、`:52`、`:54`；`src/revision.rs:128`、`:133`、`:224`、`:366`。
- 判定保留（10 处，均带否定或限定语，与 ADR:91 的正面样例一致）：`src/evidence.rs:48`、`:104`、`:115`、`:198`；`src/model.rs:168`；`src/pit/context.rs:449`；`src/pit/selection.rs:82`；`src/precision.rs:41`；`src/revision.rs:59`、`:248`。
- `docs/API.md`（自基线起未改动）：需处置 3 处 —— `:26`「已验证时间 profile」、`:34`「…和一致性证明」、`:52`「本次证明可见的引用」；已带否定限定、可保留 3 处 —— `:15`、`:32`、`:42`。

保留判定本身也需按 F-3 的 requiredFix 逐条附一句理由，否则「保留」与「漏项」在文本上不可区分。

## 10. attempt 2 独立复审（`release-reviewer`）

**复审对象与 HEAD 快照**：`docs/adr/ADR-0001-evidence-trust-boundary.md`（122 行）。`git log -- docs/adr/ADR-0001-evidence-trust-boundary.md` 显示该文件只有 4 个 commit，最后为 `cca9a4e`，此后无任何修订 → §1–§8 指出的 ADR 侧缺口在 `c438d40` 全部仍在。复审窗口内 HEAD 由 `6e947a5` 经 `0fcc4bc` 前进到 `c438d40`；`src/pit/selection.rs` 与 `src/evidence.rs` 先后处于未提交状态，属 `t20`（`rust-engineer`）进行中工作，本节行号与判定只对 `c438d40` 有效。

**方法**：只读取证（`git log` / `git diff` / `git grep` / `git show` / 定行段读取）。未运行 build、test、clippy、doc、package（任务约束「不 build」）。gate 词汇依 `spec.md:1422-1424`。

### 10.1 attempt 1 findings 复核

| finding | 复核结论 | attempt 2 独立证据 |
| --- | --- | --- |
| F-1（medium） | 仍成立 | `docs/evidence/DECISIONS-20261108.md:23-25` 已落定 D-1/D-2/D-3，ADR §6（ADR:97-101）仍写「待 Owner 裁定」且未引用该登记文件。ID 冲突确证：`docs/audit/AUDIT-VALIDATION.md:298` 的 R3 = 「证据/快照 schema 版本绑定校验的强度与错误码（TP-T070 要求 `EvidenceMismatch` / `Indeterminate`）…按 `spec.md:1229` 直接实现，不改规格」，而 `docs/evidence/DECISIONS-20261108.md:26` 的 R3 = 「AUD-002 措辞 → t20」。前者方向与 `:23` 的 D-1（库内不做）相反，两份文件都未记录这次覆盖。`spec.md:1229` 原文已核对：「证据绑定另一版本/source audience/digest；规范化候选缺原来源与转换清单绑定 → 明确错配返回 `EvidenceMismatch`；缺绑定返回 `Indeterminate`；均不 `Visible`」 |
| F-2（high） | 仍成立，独立复核通过 | `c438d40` 的 `SystemKnowledgeCutRef::contains`（`src/evidence.rs:522-540`）只比较 `boundary_id`、`known_at`、`event.consumer_boundary`、`event.log_epoch`、`part.partition`、`part.log_epoch`、`event.cursor <= part.inclusive_frontier`；返回 `Result<bool, PitError>`，非三态；边界不符即 `UnprovenSystemCut`。`manifest_ref`、`known_at_binding_ref`、`consistency_policy_ref`、`evidence_snapshot_ref` 只有 getter（`src/evidence.rs:497-520`）与长度检查（`src/pit/context.rs:284-291` 的 `label`、`src/pit/selection.rs:303-311` 的 `check`），与实际消费证据集合无任何比较。四项语义在 `spec.md:588`、`:589`、`:591`、`:592` 已定义。ADR:7 排除清单仍只列 WP-03/WP-06/WP-07。`docs/evidence/DECISIONS-20261108.md:12` 把 R5 并入 R2，而 R2 的裁决内容（`:10`）只覆盖 `CandidateCoverageRef` 的 boundary/epoch 覆盖与三处 `NotVisible` 断言改写，未给这四项任何比较规则 |
| F-3（medium） | 部分消解，仍成立 | 已消解：`src/pit/selection.rs:80`、`src/pit/context.rs:124`、`:302`、`:352`、`:357`、`:361`、`:449`（`0fcc4bc`、`c438d40`）。仍成立：§9 的 16 处 `src/` 残留 + `docs/API.md` 3 处 |
| F-4（medium） | 仍成立 | ADR §4 九条（ADR:52-60）无「验收执行时刻 / 被验收日志或 manifest 状态引用 / 相对 `known_at` 可追溯」义务；`spec.md:606`（§11.4）原文已核对：「`SystemAsKnown` 中新增的证据不能虚构历史 receipt/cut；只能重建已有受信日志证明的历史事实」 |
| F-5（medium） | 仍成立 | `docs/audit/AUDIT-VALIDATION.md:113` 记录「草案要求的外部 Trusted Evidence Adapter 合同在本仓无落点：仓内无该合同文件」；ADR 未把合同正本、版本化方式与 Owner 列入 §6 |
| F-6（low） | 仍成立，并被 F-9 加强 | `tests/pit_contract.rs:874` 的 `tp_t059_060_070_missing_proof_and_invalid_chain_fail_closed` 断言只有 `Err(PitError::InvalidRevisionChain)`（`:913-920`）与 `VisibilityDecision::Indeterminate(_)`（`:921-931`）。精确化：`EvidenceMismatch` 在库内测试确有覆盖，但覆盖的是别的绑定维度 —— `tests/acceptance_align.rs:562-585`（`tp_t070_evidence_scope_or_binding_mismatch_never_visible`，`source_scope` 错配）、`tests/pit_contract.rs:652`、`:842`、`:869`、`tests/evidence_contract.rs:49`、`:59`（`TimeBounds` 顺序与 `Exact` 上下界不等）。缺的只是 schema / extractor 绑定错配的断言 |
| F-7（medium） | 部分消解，仍成立 | 见 §9：25 / 28 落地；`src/pit/visibility.rs:50`、`:52`、`:54` 与 `docs/API.md` 的 3 处未做；`已验收` 措辞缺陷的根因见 F-8 |

### 10.2 attempt 2 新增 findings

#### F-8（high）ADR 自身授权的替换文本违反 ADR 的分层与禁用规则

- **位置**：ADR:73（`src/evidence.rs:188` 行的「替换」列）；对照 ADR:19（第 1 层「不得声称」列含「已验收」）与 ADR:66（禁用语列表）。
- **证据**：ADR:73 把「已证明的实际发布界限」替换为「适配器声明并已验收的实际发布界限」。`t20` 逐字照做：`c438d40` 的 `src/evidence.rs:188` 为「适配器声明并已验收的实际发布界限。」。同一 `impl` 相邻的 `:192` 新文本却是「外部验收记录引用；本库不验证该记录」。`spec.md:1440`（§25.5）禁止「声称证据结构确保来源真实」一类完成式声明。
- **与 F-7 的关系**：attempt 1 把该文本记为实施侧缺陷。复核结论是它由 ADR 正文授权 —— 只改代码不改 ADR:73，任何重做都会再次产生同一措辞。这是根因差异，不是同一缺陷的重复登记。
- **影响**：`temporal` 的公开 rustdoc 声称外部验收已完成，正是 AUD-002 指控的形态（`.omc/drafts/1.md`:94-95、要求 4）。
- **requiredFix**：ADR:73 的替换列改为「适配器声明的发布界限；其验收由外部记录承载，本库不核验」；把「已验收」并入 ADR:66 禁用语；`t20` 按修订后的 ADR 同步该行。

#### F-9（medium）AUD-002 要求 5 无测试证据，ADR §7 C 指定的命令不产出该证据

- **位置**：ADR §7 C（ADR:111）、§4 第 4 与第 7 条（ADR:55、:58）；`.omc/drafts/1.md`:105-106（修复要求 5）。
- **证据**：全 `tests/` 检索 `schema_version` 与 `extractor_version` 命中数为 0 —— 测试既不调用 `proof.schema_version()` / `proof.extractor_version()`，也不变更这两个夹具值（恒为 `"schema1"` / `"extractor1"`：`tests/pit_contract.rs:106-107`、`:214-215`、`:813-814`；`tests/acceptance_align.rs:174-175`、`:282-283`、`:1059-1060`、`:1236-1237`、`:1845-1846`；`tests/effective_contract.rs:65-66`；`tests/p1b_revision.rs:117-118`；`tests/repro_audit001.rs:76-77`；`tests/evidence_contract.rs:73` 为 `"schema-v1"`），无错配用例，也无「字段被保留」断言。唯一 identity 回显断言是 `tests/pit_contract.rs:388`（`result.manifest.context.fact_schema_ref()`）。输出侧：`PitManifestRef`（`src/pit/selection.rs:75-86`）只携带 `context` 与 `coverage_manifest_ref` / `coverage_adapter_ref` / `coverage_acceptance_ref`，不记录 per-evidence 的 `schema_version` / `extractor_version` / `acceptance_ref`；这三者只经 `FactOutcome::Selected { evidence }` 内嵌的 `SelectedEvidenceRef`（`src/pit/selection.rs:18-24`）间接可达。`docs/API.md:52` 已把复现清单责任整体推给消费者。
- **影响**：ADR §7 C 以 `cargo test --test pit_contract` 作为 C 项证据，但该命令即使全绿也不产生 C 项证据 —— Gate B 第 2 项与 AUD-002 要求 5 没有可登记为 `PASS` 的证据。`docs/audit/AUDIT-VALIDATION.md:110-112` 已判定草案两条测试「针对的行为当前不存在」。
- **requiredFix**：二选一并写入 ADR §7：(a) C 项 gate_status 改 `BLOCKED`，写明需要的断言（错配用例属第 4 层合同；库侧只保留「identity 字段被保留」断言及其落点）；或 (b) 在 §4 第 7 条写明 per-evidence identity 由消费者从结果对象读取、库不写入 manifest，并由 `t15` 在 `spec.md` §16.3 正本写明该口径及 CHANGELOG 结论（`spec.md:1426`）。

#### F-10（low）ADR §5 自报计数错误

- **证据**：ADR:120 写「§5 表的 20 行、覆盖 27 处公开 rustdoc 位置」。逐位置清点 ADR:70-89 的 20 行，实为 28 处（其中 8 行各含 2 个以上位置：`:21-22`、`:30`+`:32`、`:192`+`:342`、`:150`+`:154`、`:38`+`:45`、`:57`+`:62`、`:355`+`:445`、`:50`+`:52`+`:54`）。
- **影响**：下游按「27 处」核对会把未完成项读成已完成；§9 的实际口径是 25 / 28。
- **requiredFix**：ADR:120 与 §5 表头改为逐位置计数，并与 §9 的落地进度对齐。

### 10.3 Gate B 维度与 AUD-002 要求（attempt 2 判定）

| 维度 / 要求 | attempt 1 | attempt 2 | 变化原因 |
| --- | --- | --- | --- |
| 结构校验与真实性验证职责分离 | 满足（合同层） | 合同层满足，但 F-8 使实施层出现反向声称 | `src/evidence.rs:188` 已按 ADR:73 落地 |
| proof refs 绑定版本身份 | 部分满足 → 需修订 | 需修订，且新增「无测试证据」判定 | F-1 未修订 + F-9 |
| Cut manifest 与证据快照绑定 | 未定义 → 需修订 | 未定义 → 需修订（`high`，独立复核通过） | 无变化 |
| consumer / boundary / epoch / partition scope | 部分满足（如实标注缺口） | 不变（属 AUD-001 / WP-01，不在本 ADR 范围） | 无变化 |
| 适配器合同可被消费者独立验证 | 部分满足 → 需修订 | 不变 | F-4、F-5 未修订 |
| AUD-002 要求 1、2、3 | 覆盖 | 覆盖（要求 2 的深度受 F-1、F-2 约束） | 无变化 |
| AUD-002 要求 4 | 规则正确、清单不完整 | 规则正确；清单不完整（F-3）且 ADR 自身授权了违规文本（F-8） | 新增 `high` |
| AUD-002 要求 5 | 字段层覆盖 | 字段层覆盖（`src/evidence.rs:106-112` 三字段齐备，可经 `SelectedEvidenceRef` 读出）；无测试证据（F-9） | 判定收紧 |

### 10.4 verdict（attempt 2）

**NEEDS_REVISION**（与 attempt 1 一致；新增一条 `high`：F-8）。Gate B 五项在 `c438d40` 仍为 `NOT_RUN` 或 `BLOCKED`，无一项可记 `PASS`。

解锁条件：

1. F-8、F-2、F-1 是 ADR-0001 修订的必需项（`t15` 的 spec 正本同步依赖 F-1 的 TP-T070 拆分口径与 F-9 的 identity 口径）。
2. F-3 / F-7 的剩余项（`src/pit/visibility.rs` 3 处 + `docs/API.md` 3 处 + §9 的 16 处 `src/` 残留）是 Gate B 第 1 项转 `PASS` 的前提。
3. F-9 是 AUD-002 要求 5 的证据前提；未闭合前 `t18` 的 Evidence Manifest 只能记 `NOT_RUN` 或 `BLOCKED`。
4. F-5 是 Gate B 第 5 项的证据前提。
5. 复审进入条件沿用 §7 的 7 条，并追加 F-8、F-9、F-10 三条。

### 10.5 attempt 2 未运行的检查

`cargo fmt --check`、`cargo clippy --all-targets --all-features -- -D warnings`、`cargo test --all-targets --locked`、`cargo doc --no-deps --locked`、`cargo package --locked`、MSRV 1.88、Linux / macOS / Windows 矩阵 → 全部 `NOT_RUN`（任务约束「不 build」；`.omc/drafts/1.md` §7 禁止把 `NOT_RUN` 改成 `PASS`）。attempt 2 运行的取证命令全部为 `git log` / `git diff` / `git grep` / `git show` / 定行段读取，退出码均为 `0`。

§9 与 §10 同为审查记录，不构成 Accepted，不产生合入权（`spec.md:1424`）。`release-reviewer` 不对自己的实现做批准：本节只评价 `t2`（`architect`）的 ADR 与 `t20`（`rust-engineer`）的改写，未评价自身产出。
