# ADR-0002：`LinearSupersedes` 的 chain digest 承诺对象（XH-TEMPORAL-REMEDIATION-002 / WP-03 / AUD-003）

**状态**：Proposed / Review Required（草案，含待 Owner 裁定项 R4，不构成 Accepted，不产生合入权）
**日期**：2026-11-08
**仓库与基线**：`bytechainx/temporal`，`goal/v0.2.0-pit-hardening`，基线 SHA `7cef81d`
**关联**：`.omc/drafts/1.md` AUD-003 / WP-03；`docs/audit/AUDIT-VALIDATION.md` §4（commit `942fb11d66f656057d8172322b4bba1c6289bc4f`）；`spec.md` §9.5（:489-500）、:475、:1039、§14、§16.3、§22（TP-T059、TP-T060）、§25.2、§25.3、§33.1；`goal.md` G-006；ADR-0001 §2、§4、§6
**下游**：T9（WP-03 代码强化）与 T16/T17 审阅

## 1. 背景与 T1 已核实事实

`RevisionOrderRef::LinearSupersedes`（`src/revision.rs:150-156`）携带 `predecessor: Option<RevisionIdRef>` 与 `chain_digest: DigestRef`。rustdoc 现写作「摘要固定权威链关系」（:150）与「固定完整权威链关系的摘要」（:154）。

T1 核实（`docs/audit/AUDIT-VALIDATION.md` §4）与本轮复核一致：

1. `select_chain`（`src/pit/selection.rs:733-803`）全程不解引用 `chain_digest`；链关系只靠拓扑与结构校验：前驱必须存在于候选集（:749-756）、同一前驱不得被重复指用（:757-759）、`root_count != 1`（:765-767）、多于一个 tip（:768-774）、`depth > max_depth`（:777-781）、`depth != versions.len()`（:799-801）；自环在构造期拒绝（`src/revision.rs:303-310`）。
2. `chain_digest` 在库内只有两个读取点：长度校验（`src/pit/selection.rs:366-375`，只查 `algorithm_version` 与 `as_bytes` 长度）与同版内容一致性（`same_revision_content` 经 `revision_order()` 的派生 `PartialEq`，`src/pit/selection.rs:490-504`；同 `revision_id` 而内容不同 → `ConflictingRevision`，:200-205、:240-242、:447-449）。全库检索 `chain_digest` 无其他读取点。
3. 现 `AuthoritativeSequence` 携带 `order_scope`（`src/revision.rs:144-149`），`LinearSupersedes` 不携带任何链作用域字段；链的身份只能由每个节点的 `authority_scope` + `fact_key`（`src/revision.rs:176-179`）与 `predecessor` 拓扑隐式推出。
4. 现有测试对同一链的所有节点复用同一个 `digest` 变量（`tests/pit_contract.rs:883-899`、`tests/p1b_revision.rs:514-584`、`tests/acceptance_align.rs:926-934`、:1692-1723），因此「节点 digest 不一致」从未被测试覆盖。
5. 草案 8 个建议测试中 5 个实质已存在；真正缺失的是 `chain_digest_contract_mismatch_rejected` 与「chain manifest 作用域不匹配被拒」。`spec.md` 与代码中都没有 chain manifest 定义（`spec.md:496` 只写「必要链闭包/权威链摘要」）。按 AGENTS.md 反过度设计纪律，实现方不得自行引入 chain manifest 类型，记为待裁定 R4。

因此本 ADR 的第一职责是裁定「digest 到底承诺什么」，再给出只依赖字节比较、不引入哈希与密码库的最小强化。

## 2. 三个候选与裁定

| 候选 | 含义 | 采纳 |
| --- | --- | --- |
| C1 单版本关系 | `chain_digest` 只摘要「本版本替换其前驱」这一条边（或该版本的语义内容） | 否 |
| C2 完整链清单 | `chain_digest` 摘要该 `authority_scope` + `fact_key` 内、该节点所属权威链的完整节点集合（同一连通链上所有节点必须声明同一 digest 与同一 `algorithm_version`） | 是（建议） |
| C3 链快照 | `chain_digest` 摘要查询边界（`known_at` / cut）截止的链前缀，随查询而变 | 否 |

裁定理由：

1. `spec.md:496` 的措辞是「必要链闭包/权威链摘要」：单数「摘要」与「闭包」指向整条权威链，不指向单条边。`spec.md:500` 要求「分叉、环、跨事实前驱、缺失必要链关系必须拒绝」，语义上需要一份覆盖全链的声明，而不是一条边的声明。
2. 否掉 C1：若 digest 只承诺单版本内容，`chain_digest` 与 `semantic_digest`（`src/revision.rs:179`）职责重叠，且 AUD-003 的缺口不可闭合——库内没有任何可校验的链级一致性，digest 退化为不参与判定的装饰字段。rustdoc「固定完整权威链关系」必须改写为 C1 措辞才能自洽，那是把承诺降级而非补齐。
3. 否掉 C3：C3 要求 `RevisionOrderRef` 知道查询边界。`temporal` 是纯函数选择器（`spec.md` §14），把 `known_at` / cut 注入版本身份会让同一 `revision_id` 在不同查询下携带不同 digest，与 `same_revision_content` → `ConflictingRevision` 的同版一致性（`src/pit/selection.rs:490-504`）直接冲突，并把输入模型变更扩散到所有既存调用方（breaking，`spec.md`:1426）。
4. C2 的代价必须承认：`temporal` 不实现哈希（`spec.md`:475「哈希/规范化由消费方的版本化合同实现，库只比较明确的算法/格式版本及摘要，不在本版加入密码库」），所以库无法验证 digest 值本身是否等于链内容。C2 只能让库校验跨节点一致性；digest 与链内容的对应关系属于 ADR-0001 的第 2–4 层。

## 3. C2 下 `temporal` 可校验的一致性（仅字节比较）

链身份（library 内可推导，不需要新类型）：一次查询内由 `predecessor` 拓扑构成的连通分量，其全部节点的 `authority_scope` 与 `fact_key` 必须相同（跨事实前驱已被 `src/pit/selection.rs:749-756` 拒绝），且恰有一个 `predecessor: None` 的根（`root_count == 1`，:765-767）。

必须新增或明示的规则（全部为 ADR-0001 第 1 层，Structural Validation）：

| 规则 | 内容 | 违反时的现有错误 | 现状态 |
| --- | --- | --- | --- |
| CH-1 | 同一连通链内所有候选的 `chain_digest.as_bytes()` 必须逐字节相等 | `InvalidRevisionChain` | 未实现 |
| CH-2 | 同一连通链内所有候选的 `chain_digest.algorithm_version()` 必须逐字节相等 | `InvalidRevisionChain` | 未实现 |
| CH-3 | 一致性只在单次查询的同一连通链内判定；不得跨查询、跨 `authority_scope` 或跨 `fact_key` 比较 digest | —— | 需在文档固定，防止实现出误拒 |
| CH-4 | 同一 `revision_id` 而 `chain_digest` 不同仍走 `ConflictingRevision`（现有行为，不得被 CH-1 吞掉或改码） | `ConflictingRevision` | 已实现（`src/pit/selection.rs:490-504`） |
| CH-5 | 不得因为两条不同链声明了相同 digest 字节就把它们合并；digest 不是链身份 | —— | 已由 `fact_key` 隔离（`spec.md`:487） |

CH-1 / CH-2 复用 `InvalidRevisionChain`，不新增 `PitError` 变体：`spec.md`:1426 禁止新枚举变体静默改变旧策略行为，新增错误码属 Gate C 的 API 变更，需单独评审。

## 4. 明文声明：普通字符串 digest 引用不是密码学验证

1. `DigestRef` 是 `(algorithm_version: &str, bytes: &[u8])`（`src/revision.rs:61-64`）。`try_new` 只校验两者非空与长度界限（`src/revision.rs:68-77`）。
2. `temporal` 不计算摘要、不校验摘要值、不校验摘要与链内容的对应关系、不校验签名、不引入密码库（`spec.md`:475、:1039）。
3. 因此 `chain_digest` 的一切库内检查都是字节一致性，属 ADR-0001 第 1 层。「链合法」不等于「链内容正确」。
4. `src/revision.rs:150` 与 `:154` 的「摘要固定权威链关系」「固定完整权威链关系的摘要」须改写为：「调用方声明的权威链全集摘要；本库只比较该声明在链内是否一致，不验证摘要内容」。改写属 rustdoc 变更，零语义变更；语义变更由 §3 的 CH-1 / CH-2 承担。
5. 按 `spec.md`:1436，禁止任何完成声明声称「证据或摘要结构确保来源真实」或「链摘要验证了修订内容」。

## 5. 不可变 chain manifest 的绑定方式（第 2–4 层，Adapter 侧）

`temporal` 不新增 chain manifest 类型。manifest 是消费方 Trusted Evidence Adapter（ADR-0001 §4）留存的外部工件，temporal 只通过 `chain_digest` 的声明与链拓扑与之对应。

1. chain identity（manifest 的键）：`authority_scope` + `fact_key` + 根 `revision_id`（`predecessor: None` 的节点）+ 节点数 + 有序 `revision_id` 列表。库内可从 `predecessor` 拓扑推出同一组字段（§3），因此两侧可比。
2. digest 合同：`chain_digest.algorithm_version()` 声明摘要算法与规范化版本；`chain_digest.as_bytes()` 是 Adapter 按该合同对完整链清单计算出的摘要。CH-1 / CH-2 保证同一链内的声明一致，不保证摘要值正确。
3. 不可变性：manifest 一经发布不得改写。链增长必须发布新 manifest 并保留旧的；旧查询必须能按当时的 manifest 身份复现。manifest 必须记录生成它的组件身份与版本，与 ADR-0001 §4 第 4 条的 `extractor_version` 同源。
4. 复现义务：消费者复现清单（`spec.md` §16.3）必须写入 chain identity 各字段、`chain_digest` 字节、`algorithm_version`、manifest 身份与版本、`acceptance_ref`。缺任一项，该次查询不可复现。
5. 禁止挪用现有字段：`payload_ref()` 与 `input_provenance_ref()` 不承载链清单（`spec.md`:1039 只允许绑定与比较）。
6. 若要在 `temporal` 侧显式绑定（例如给 `LinearSupersedes` 或 `AsOfContext` 增加 `chain_manifest_ref`）：这是公开 API 形状变更，必须先回答 AGENTS.md 反过度设计四问（哪个当前需求需要它、谁消费它、为什么现有字段不够、删除它失去什么），并通过 Gate C 与 `spec.md`:1426 兼容性评审。本草案不建议在 v0.2.0 引入。

## 6. 需代码强化的点（交 T9，rust-engineer）

| 编号 | 强化点 | 位置 | 约束 |
| --- | --- | --- | --- |
| S-1 | 实现 CH-1 / CH-2：同链 `chain_digest` 字节与 `algorithm_version` 一致性 | `src/pit/selection.rs:733-803`（在既有拓扑校验之后或同一遍历内） | 不新增 `Vec`/`HashMap`；比较为 O(n) 常数次字节比较；若需额外内存必须走 `:135-157` 的预算路径 |
| S-2 | 拒绝码统一 `InvalidRevisionChain`，不新增 `PitError` 变体 | 同上 | `spec.md`:1426；CH-4 的 `ConflictingRevision` 语义不得改动 |
| S-3 | 检查顺序在 `depth != versions.len()`（`:799-801`）之后或同一遍历，避免对不可达子图误报 digest 不一致 | 同上 | 现行为已先拒绝不可达与环 |
| S-4 | rustdoc 改写 `src/revision.rs:150`、`:154`，并同步 `docs/API.md` 对应条目 | `src/revision.rs`、`docs/API.md` | 使用 ADR-0001 §5 禁用语清单；零语义变更 |
| S-5 | 新增测试：`chain_digest_contract_mismatch_rejected`（T1 反例 D1：`v1 ← v2 ← v3` 且 `v3` 的 digest 改为任意字节 → 期望 `InvalidRevisionChain`）、`chain_digest_algorithm_version_mismatch_rejected`、`chain_partial_candidate_set_keeps_consistent_digest`（部分候选且 digest 一致 → 通过）、`chain_digest_shared_across_facts_not_merged` | `tests/pit_contract.rs`（邻近 `tp_t059_060_070`，:874-899）、`tests/p1b_revision.rs`（:506-584） | 现有用例对全链复用同一 `digest` 变量，CH-1 落地后应仍通过；测试测真实实现，不把判定逻辑抄进测试 |
| S-6 | 不得实现哈希计算、不得引入密码库或任何新依赖 | 全仓 | `spec.md`:475；`.omc/drafts/1.md` §1.1 |
| S-7 | `spec.md` §22 需要为 CH-1 / CH-2 新增 TP 条目：TP-T059 / TP-T060（`spec.md`:1213-1214）只覆盖拓扑与分叉，不覆盖 digest 一致性 | `spec.md`、`docs/测试矩阵.md` | 正本变更不在本 ADR 写范围内，交 captain 走 Spec Kit 流程与 `SCHEMA_VERSION` 评估 |

## 7. 验收映射与 gate 状态

状态词汇只允许 `PASS / FAIL / NOT_RUN / BLOCKED / NOT_APPLICABLE`（`spec.md` §25.2）。本 ADR 是草案，本轮未运行任何 cargo 命令。

| 验收项 | Owner | 落点 | gate_status | 交主 Agent 的命令 |
| --- | --- | --- | --- | --- |
| A：裁定 digest 承诺对象（R4-1） | 人类 Owner / captain | 本 ADR §2 给出建议 C2 | BLOCKED（等待 R4 裁决；裁决前 S-1/S-5 不得实现） | —— |
| B：CH-1 / CH-2 实现且非法 digest 被拒 | rust-engineer（T9）/ qa | S-1、S-2、S-5 | NOT_RUN | `cargo test --test pit_contract`、`cargo test --test p1b_revision` |
| C：rustdoc 与 `docs/API.md` 不再暗示摘要固定链关系 | rust-engineer | S-4 | NOT_RUN | `cargo test --test api_compile`（文档变更的编译面）+ 文档检查 |
| D：Adapter 留存不可变 manifest 并证明 digest 与链内容对应 | Adapter / Product Owner（`spec.md` §33.1） | 消费者仓 | BLOCKED（本机无消费者仓与 Trusted Evidence Adapter 实现） | 需消费者仓检出后由 Owner 执行 |
| E：`spec.md` §22 新增 digest 一致性 TP 条目 | captain / Owner | `spec.md`、`docs/测试矩阵.md` | NOT_RUN（本任务写范围只含 ADR-0002） | —— |

A 与 D 不得记为 `PASS`。`NOT_APPLICABLE` 不适用：这些义务真实存在，只是本机无法执行或等待裁决。

## 8. 后果、待裁定与非目标

正面：AUD-003 的缺口在不引入哈希、不引入新类型、不新增依赖的前提下变成可校验的字节一致性规则；digest 与 `semantic_digest` 的职责分开；rustdoc 与 `spec.md`:475、:1039、:1436 一致。

负面与风险：C2 之下，库仍不能证明候选集覆盖完整链——链完整性由 coverage 与 Adapter 保证（`spec.md` §12.3；AUD-001 的 R2 裁决独立于本 ADR）。若 Owner 最终选择 C1，则现行为即为正确行为，§3 的 CH-1 / CH-2 与 S-1、S-5 必须撤回，`src/revision.rs:150`、`:154` 改为单版本措辞。

约束：CH-1 / CH-2 是行为收紧——先前被接受的「同链 digest 不一致」输入将返回 `InvalidRevisionChain`。按 `spec.md`:1426 这属于校验行为变更，需要显式兼容性评审、`CHANGELOG` 记录与版本决策（Gate C）。`chain manifest` 若落地为库内类型则属 API 变更（R4-2）。

待 Owner 裁定：R4-1 承诺对象（建议 C2）；R4-2 是否在 `temporal` 新增 `chain_manifest_ref`（建议不在 v0.2.0 新增）；R4-3 CH-1 / CH-2 的拒绝码（建议复用 `InvalidRevisionChain`）；R4-4 `chain_digest.algorithm_version` 是否必须等于同节点 `semantic_digest.algorithm_version`（建议 v0.2.0 不强制，缺少当前消费需求）；R4-5 `spec.md` §22 的 TP 条目与 `SCHEMA_VERSION` 处理。

非目标：不实现哈希或密码学验证；不定义 chain manifest 库内类型；不处理 Derived lineage（WP-06）、Wire precision（WP-07）；不改动 coverage 与 `NotVisible` / `Indeterminate` 语义（AUD-001 / R2）；不改动 `AuthoritativeSequence`。
