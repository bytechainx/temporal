# AUD-001..AUD-007 逐条核实报告

| 项 | 值 |
| --- | --- |
| 文档 ID | TEMPORAL-AUDIT-VALIDATION-20261108 |
| 对应提案 | `.omc/drafts/1.md`（XH-TEMPORAL-REMEDIATION-002 v0.1.0） |
| 仓库 / 分支 | `bytechainx/temporal` / `goal/v0.2.0-pit-hardening` |
| 语义基线 | `7cef81d35d9d8bf23be9d3221097222832911e60` |
| 核查提交点 | `b71aa17`（`git diff 7cef81d..b71aa17` 仅新增 `docs/evidence/BASELINE-20261108.md`；`src/`、`tests/` 与基线逐字节相同，行号对两者同时有效） |
| 核查方式 | 只读源码与测试静态核查。未运行 cargo（build / test / clippy / bench 全部 NOT_RUN，见文末门禁表）。 |
| 判定词汇 | confirmed（草案事实成立且有可复现代码证据）；partially-confirmed（部分成立，其余不成立或仅缺证据）；refuted（与代码事实不符）；needs-owner-ruling（语义未定义，须 Owner 裁决后才能实现） |
| 撰写 | pit-researcher（T1） |

## 1. 结论汇总

| 条目 | 判定 | 阻塞下游 | 需 Owner 裁决 | 建议 WP |
| --- | --- | --- | --- | --- |
| AUD-001 跨消费者证据被推断为确定不可见 | confirmed | 是（P0） | 是（R1、R2） | WP-01 |
| AUD-002 证据引用只校验结构 | partially-confirmed | 否 | 是（R3） | WP-02 |
| AUD-003 链摘要不参与链关系验证 | confirmed（事实）+ needs-owner-ruling（语义） | 否（P1） | 是（R4） | WP-03 |
| AUD-004 工作内存预算只估算、未测量 | partially-confirmed | 否（P1） | 否 | WP-04 |
| AUD-005 Cut 语义与分区覆盖证明 | partially-confirmed（含 1 处 confirmed 缺陷） | 否（P1） | 是（R5） | WP-05 |
| AUD-006 派生数据 lineage | partially-confirmed（缺口在合同，不在算法） | 否（P1） | 是（R6） | WP-06 |
| AUD-007 跨语言与存储精度 | partially-confirmed（3/5 项 refuted 或不在库内） | 否（P1） | 否 | WP-07 |

一句话结论：草案对代码事实的描述总体准确。只有 AUD-001 是必须立刻修复的 P0 语义缺陷。AUD-002 与 AUD-007 的指控有相当部分与现行代码和 `spec.md` 不符，应改写为「消费者合同缺口」，不得按缺陷修复处理。

## 2. AUD-001 —— 其他消费者的证据被当成目标消费者的确定结论

### 草案主张

`src/pit/visibility.rs` 在目标消费者没有任何证据时返回确定的 `NotVisible(OutsideSystemCut)`。

### 源码事实

1. `src/pit/visibility.rs:153-159`：循环只处理 `event.consumer_boundary() == boundary && event.log_epoch() == cut.boundary_epoch()` 且 `cut.contains(...)` 为真的事件；其余事件 `continue`，全部 `saw_*` 标志保持 false。
2. `src/pit/visibility.rs:217-221`：`if !saw_in_cut && !version.system_evidence().is_empty()` → `NotVisible(DecisionReason::OutsideSystemCut)`。
3. 库内没有任何「目标消费者证据覆盖完整性」模型。`CoverageState` / `CandidateCoverageRef`（`src/pit/context.rs:354-471`，`validate_for` 见 `src/pit/context.rs:458-470`）只校验 `authority_scope`、`dataset_snapshot`、`knowledge_basis`、`known_at` 与事实枚举完整性，不校验 consumer / boundary / epoch 的证据覆盖。
4. 现行测试把该行为固定为断言，共三处：
   - `tests/acceptance_align.rs:641-696` `tp_t074_boundary_isolation_no_shared_available_at`：证据绑定 `consumer1`（夹具 `system_version`，`tests/acceptance_align.rs:315-330`），查询上下文 `boundary_id=consumer2`（`tests/acceptance_align.rs:650-691`）→ `tests/acceptance_align.rs:692-695` 断言 `NotVisible(_)`。
   - `tests/acceptance_align.rs:698-734` `tp_t075_log_epoch_isolation`：事件 `log_epoch="log2"` 与 `cut.boundary_epoch="log1"` 不符 → `:730-733` 断言 `NotVisible(DecisionReason::OutsideSystemCut)`。
   - `tests/acceptance_align.rs:736-789` `tp_t076_multi_partition_cut_needs_completeness_proof`：事件在 `part2`、Cut 只列 `part1` → `:766-769` 断言 `NotVisible(_)`。
5. `OutsideSystemCut` 的全部出现点：`src/pit/visibility.rs:18`（枚举定义）、`src/pit/visibility.rs:37`（`OUTSIDE_SYSTEM_CUT` 编码）、`src/pit/visibility.rs:219`（唯一返回点）；测试断言仅 `tests/acceptance_align.rs:636` 与 `:732`。

判定：**confirmed**。代码行为、触发条件、无覆盖证明三点均与草案一致。

### 与 `spec.md` 的冲突点（停下请求 Owner 裁决）

- `spec.md:665-675`（§13.1）规定 `NotVisible(reason)` 表示「已有充分证据证明该版本不属于本次可见集合」，并规定「缺失质量证据不能借『安全起见排除』悄悄让旧修订成为确定答案」。AUD-001 的修复方向与 §13.1 一致。
- `spec.md:994` 把 `OUTSIDE_SYSTEM_CUT` 列为合法 `NotVisible` 原因码，并规定 Indeterminate 的原因「固定为」`SOURCE_TIME_UNPROVEN`、`SYSTEM_TIME_UNPROVEN`、`PRECISION_INSUFFICIENT`、`RECEIPT_UNPROVEN`、`RELEASE_CONDITION_UNPROVEN` 五项。修复若引入新的 Indeterminate 原因（例如 `TARGET_COVERAGE_UNPROVEN`），直接违反 §22 该条固定枚举 → 见裁决项 R1。
- 规格 §22 期望列比测试断言弱：`spec.md:1238`（TP-T074）只要求「A/B 查询不同，不能共享无 scope available_at」，`spec.md:1239`（TP-T075）只要求「不能错误比较旧新 offset」，`spec.md:1240`（TP-T076）只要求「缺完整性/一致性证明时不可声称完整系统视图」。三条都没有要求 `NotVisible`。把结果改为 `Indeterminate` 与规格期望列不冲突，但会使上述三个测试失败 → 见裁决项 R2。
- AUD-001 修复要求 2（目标消费者覆盖证明）在 `spec.md` 无落点：§12.3（`spec.md:647-653`）的完整性证明只覆盖「截至 cutoff 的潜在候选已完整枚举」，不覆盖消费者证据覆盖 → 见裁决项 R2。

### 最小反例设计（可编译级）

前置：复用 `tests/acceptance_align.rs` 夹具 `limits()`（`:29-31`）、`snapshot()`（`:33-35`）、`version()`（`:122-207`）、`system_context(at, frontier)`（`:229-272`，`boundary_id="consumer1"`、`boundary_epoch="log1"`、`partitions=[part1/log1]`）、`with_system_evidence(...)`。

```text
反例 V1（他消费者证据）
  1. base   = version(b"v1", b"r1", 1, 10, Upsert)
  2. events = [SystemVisibilityEvidenceRef(identity=base.identity(),
                consumer_boundary="consumerA", log_epoch="log1",
                partition="part1", cursor=1, bounds=Exact(16))]
  3. v      = with_system_evidence(base, receipts, events, &limits())
  4. 断言现行为：evaluate_visibility(v, system_context(20, 1))
                == NotVisible(DecisionReason::OutsideSystemCut)
  5. 期望行为：== Indeterminate(<目标边界证据未证明>)
```

```text
反例 V2（同边界、他 epoch）
  与 V1 相同，仅 consumer_boundary="consumer1"、log_epoch="log2"。
  现行为 NotVisible(OutsideSystemCut)（visibility.rs:154 跳过 → :217）。
  该场景已由 tests/acceptance_align.rs:698-734 固定。
```

```text
反例 V3（同边界同 epoch、他分区）
  与 V1 相同，仅 partition="part2"，Cut 只列 part1。
  evidence.rs:535-539 返回 Ok(false) → visibility.rs:157 跳过 → :217 确定 NotVisible。
  该场景已由 tests/acceptance_align.rs:736-789 固定。
```

```text
对照反例 C1（必须保持 NotVisible，防止修复过度）
  事件 consumer_boundary="consumer1"、log_epoch="log1"、partition="part1"、
  cursor=2，Cut 前沿 inclusive_frontier=1 → 事件确证在 Cut 前沿之外。
  期望保持 NotVisible(OutsideSystemCut)。已由 tests/acceptance_align.rs:604-638 覆盖。
```

区分 V1/V2/V3 与 C1 需要新的判定输入：库必须能区分「事件确证位于 Cut 之外」与「目标边界没有任何证据证明该事件的归属」。建议实现方向（供 WP-01 Owner 采纳，本报告不改代码）：`SystemKnowledgeCutRef::contains` 返回三态（`InCut` / `OutsideCut` / `Unproven`），并在 `visibility.rs` 把 `Unproven` 路由到 Indeterminate；`OutsideSystemCut` 只在至少一个事件确证 `OutsideCut` 且无任何 `InCut` 事件时返回。

### 建议测试名（草案给定的名单经核查均可落地）

`system_visibility_other_consumer_evidence_is_indeterminate`、`_target_consumer_event_in_cut_is_visible`、`_target_consumer_event_after_frontier_is_not_visible`、`_empty_target_coverage_is_not_visible_only_if_complete`、`_incomplete_target_coverage_is_indeterminate`、`_mixed_consumers_do_not_cross_prove`。

## 3. AUD-002 —— 证据引用只校验结构

### 源码事实

1. `src/evidence.rs:114-131` `EvidenceProofRef::try_new(evidence_id, schema_version, extractor_version, raw_evidence_ref, acceptance_ref, limits)`：六个字段只经 `label()`（`src/evidence.rs:8-16`）做非空与长度校验。无签名、无内容解析、无外部校验。草案事实成立。
2. 草案隐含的「文档把结构校验描述成真实性证明」指控与现行文档不符，以下四处已明确免责：
   - `src/evidence.rs:104`「外部受信证据入口的验收记录引用；本库仅验证结构与绑定」。
   - `src/evidence.rs:115`「登记受信入口验收结果的稳定引用；任意字符串不自动构成可信证明」。
   - `src/evidence.rs:48`「检查区间顺序与质量声明，不证明原始证据真实」。
   - `src/revision.rs:248`「已进行结构校验的事实版本借用；证据真实性仍由外部证明」。
   - `spec.md:522-524`（§10.1）与 `spec.md:1039`：解引用与摘要校验在受信适配器执行；已验证包装类型不使外部证据自动真实。
3. 真实缺口（confirmed）：版本绑定字段从未互相比较。
   - `src/pit/selection.rs:280-285`：`snapshot.id()/manifest_ref()/schema_version()` 只做长度检查（`check(...)`）。
   - `src/pit/selection.rs:412-422`：`proof.schema_version()`、`proof.extractor_version()`、`proof.acceptance_ref()` 只做长度检查。
   - `AsOfContext` 的 `fact_schema_ref` / `version_schema_ref` / `evidence_schema_ref` 只有访问器（`src/pit/context.rs:335-344`），`src/` 内无任何读取点，即查询声明的 evidence schema 版本与候选证据的 `schema_version` 不匹配也不会被拒绝。
   - 结论：草案测试 `evidence_version_binding_mismatch_rejected` 与 `evidence_schema_version_mismatch_rejected_by_adapter_contract` 针对的行为当前不存在。
4. 草案要求的「外部 Trusted Evidence Adapter 合同」在本仓无落点：仓内无该合同文件；`checklist.md` 相关项（例如 TP-T002 生产依赖检查）全部状态 NOT_RUN。

判定：**partially-confirmed**。代码事实成立；「文档误导」部分 refuted；「版本绑定未校验」部分 confirmed。

### 与 `spec.md` 的冲突点

`spec.md:1229`（TP-T070）已规定「证据绑定另一版本/source audience/digest → 明确错配返回 `EvidenceMismatch`；缺绑定返回 `Indeterminate`；均不 `Visible`」。现行代码不实现该期望。这是实现落后于规格，不是规格冲突，无需 Owner 裁决语义；但绑定检查会影响现有候选放行路径 → 记为 R3（决定绑定强度与错误码）。

### 最小反例设计

```text
反例 E1（schema 版本错配当前被放行）
  1. v = version(...)  # tests/acceptance_align.rs:122-207
     其中 EvidenceProofRef::try_new("proof1","schema1", ...)（:172-180）
  2. ctx = context 但 evidence_schema_ref = "schema9"
  3. 现行为：select_fact_as_of 正常返回 Selected
  4. 期望行为（TP-T070）：Err(PitError::EvidenceMismatch)
```

### 建议处置

WP-02 只做两件事：（a）按 TP-T070 增加 `evidence_schema_ref` / `version_schema_ref` 与候选证据字段的绑定校验；（b）在仓内新增消费者侧 Trusted Evidence Adapter 合同文档。不得把「真实性校验」加入本库（`Cargo.toml` 无密码学依赖，`spec.md:475` 明确本版不加入密码库）。

## 4. AUD-003 —— 链摘要不参与链关系验证

### 源码事实

1. `src/pit/selection.rs:733-803` `fn select_chain<'a>(versions: &[FactVersionRef<'a>], max_depth: usize)` 全程不解引用 `chain_digest`：`744-747` 只取 `predecessor`；`749-756` 前驱必须存在于候选集，否则 `InvalidRevisionChain`；`757-759` 同一前驱被重复指用 → `InvalidRevisionChain`（分叉）；`765-767` `root_count != 1` → `InvalidRevisionChain`；`768-774` 多于一个 tip → `InvalidRevisionChain`；`777-781` `depth > max_depth` → `LimitExceeded`；`799-801` `depth != versions.len()` → `InvalidRevisionChain`（捕获不可达子图与环）。
2. 自环在构造期拒绝：`src/revision.rs:303-310`（`predecessor == 自身 revision_id` → `InvalidRevisionChain`）。
3. `chain_digest` 在库内只有两个用途：长度校验（`src/pit/selection.rs:366-375`）与版本内容一致性比较（`revision_order()` 参与 `same_revision_content`，`src/pit/selection.rs:490-504`）。同一 `revision_id` 而 digest 不同 → `ConflictingRevision`（`src/pit/selection.rs:200-205`、`:240-242`、`:447-449`）。
4. 库内无任何密码学验证，符合 `spec.md:475`「哈希/规范化由消费方的版本化合同实现，库只比较明确的算法/格式版本及摘要，不在本版加入密码库」与 `spec.md:1039`「它们只用于绑定和比较，不证明原始内容可用」。
5. 草案的 8 个建议测试中，5 个在实质内容上已存在：
   - `chain_predecessor_missing_rejected`、`chain_multiple_roots_rejected`、`chain_fork_rejected`、`chain_cycle_rejected`：`tests/pit_contract.rs:874` `tp_t059_060_070_missing_proof_and_invalid_chain_fail_closed`、`tests/p1b_revision.rs:506` `tp_t059_linear_chain_accepts_single_predecessor`、`tests/acceptance_align.rs:1692-1733`（恶意环 `v1→v2→v1` → `InvalidRevisionChain`）。
   - `chain_depth_limit_enforced`：`select_chain` 的 `max_depth`（`:779-781`）+ `tests/p1b_revision.rs` 链深场景。
   - `chain_input_permutation_invariant`：`tests/acceptance_align.rs:868` `tp_t086_legal_input_permutation_keeps_consistent_order`。
   - 真正缺失：`chain_digest_contract_mismatch_rejected`、`chain_manifest_scope_mismatch_rejected`。

判定：**confirmed**（「digest 未参与链关系验证」事实成立）；草案要求的「digest 承诺对象」与「不可变 chain manifest 绑定」在当前 `spec.md` 与代码中都无定义 → 该部分 **needs-owner-ruling**。

### 与 `spec.md` 的冲突点

`spec.md:496`（§9.5 表）要求 `LinearSupersedes` 携带「初始节点或同一事实的单一前驱引用；必要链闭包/权威链摘要」，但未定义链摘要的验证语义、承诺范围或快照作用域。草案要求 2（受信入口绑定不可变 chain manifest）新增了一个库内不存在的概念。按 AGENTS.md 反过度设计纪律（新增抽象须回答「谁消费它、删除它失去什么」），不得由实现方自行引入 chain manifest 类型 → 记为 R4。

### 最小反例设计

```text
反例 D1（digest 与链内容脱钩）
  1. 构造合法线性链 v1 ← v2 ← v3（RevisionOrderRef::LinearSupersedes）
  2. 将 v3 的 chain_digest 改成任意字节串（DigestRef::try_new("sha256-v1", b"whatever", &limits)）
     predecessor 字段保持不变
  3. 现行为：select_chain 正常返回 v3（digest 内容不进入判断）
  4. 若 Owner 裁决 digest 承诺「完整链清单」：该输入必须被拒绝
  5. 若 Owner 裁决 digest 只承诺「单版本语义内容」：该输入必须只影响
     same_revision_content 比较，链校验不变（现行为即为正确行为）
```

结论：D1 的正确期望取决于 R4 的裁决结果。裁决前不得实现 `chain_digest_contract_mismatch_rejected`。

## 5. AUD-004 —— 工作内存预算只估算、未测量

### 源码事实

1. 预算在申请前计算：`src/pit/selection.rs:135-157`
   `per_candidate = size_of::<FactVersionRef>()*4 + size_of::<u64>() + size_of::<bool>()`；
   `per_fact = size_of::<FactKeyRef>() + size_of::<FactOutcome>()`；
   `index_bytes = candidates*per_candidate + fact_keys*per_fact`；
   `reserved_bytes = index_bytes*2 + 1024`；`reserved_bytes > max_working_bytes` → `LimitExceeded`。
2. 第二道校验：`validate_evidence_identity`（`src/pit/selection.rs:506-587`）累加 source / receipt / event 的 scratch 字节（仅当 `system_cut` 存在才计 receipt 与 event，`:571-587`），并检查 `reserved_bytes + scratch_bytes*2 > max_working_bytes` → `LimitExceeded`。
3. 全部堆分配使用 `try_reserve_exact` 并把失败映射为 `ResourceUnavailable`：`src/pit/selection.rs:159-160`（keys）、`:165`（ordered）、`:183`（visible）、`:187`（uncertain）、`:245`（outcomes）、`:556-567`（sources/receipts/events）、`:439-442`（distinct）、`:717-725`（ordinals `Vec<u64>`）、`:737-741`（has_successor `Vec<bool>`）；构造期另有 `src/evidence.rs:459-462`（cut names）与 `src/pit/context.rs:393-403`（coverage sorted）。
4. 排序为 `sort_unstable`（原地 pdqsort），无堆 scratch。草案「明确 O(n log n) 排序与 O(n) 索引内存」的要求在代码上成立，但仓内文档未记载该复杂度。
5. 已存在的预算测试：`tests/pit_contract.rs:958` `tp_t116_working_budget_rejects_before_allocating_selection_index`（`max_working_bytes` 取 1 与 3217 均 `LimitExceeded`）。
6. 明确的自证缺口：`tests/acceptance_align.rs:1739` `tp_t118_zero_alloc_basic_path_stack_only` 注释写明「此处以类型性质断言（未做 allocator 计数）」。`docs/性能基线.md` 写明基线未冻结、分配与峰值工作内存 NOT_RUN。

判定：**partially-confirmed**。预算机制存在且先于分配生效（草案要求 1、5 部分满足）；「覆盖所有临时分配的清单」「峰值同时存活证明」「allocator overhead 处理」「分配计数」四项确实缺失（草案要求 3、4、6、7）。未发现具体超预算反例，不得声称已发生内存超限缺陷。

### 与 `spec.md` 的冲突点

无冲突。`spec.md:1096-1112`（§21.2、§21.3）已经要求申请前检查、`ResourceUnavailable` 与 `LimitExceeded` 区分、基准记录「分配次数与峰值工作内存」。AUD-004 的缺口是 §21.3 尚未完成，属证据缺失，不是语义变更。

### 最小反例设计（用于证伪或确认，交给 WP-04）

```text
探针 A（计数法）
  自定义 GlobalAlloc 计数 + 峰值统计，跑 10^3 / 10^5 候选，
  对比 selection.rs:135-157 的 reserved_bytes 预测值。
  若实测峰值 > 预测值 → 预算模型不足，AUD-004 升级为 confirmed。
  若实测峰值 <= 预测值 → 草案要求 3 只需文档化，不须新增机制。

探针 B（scratch 路径覆盖）
  构造带 system_cut 且 receipts/events 数量逼近 max_evidence_per_version 的输入，
  验证 validate_evidence_identity 的 scratch 估算是否覆盖 :556-567 的全部 Vec 预留。

不新增生产 allocator 抽象：计数只在 benches 或 tests 的 dev-profile 内实现。
```

## 6. AUD-005 —— Cut 语义、分区覆盖与绑定引用

### 源码事实

1. `src/evidence.rs:522-540` `SystemKnowledgeCutRef::contains`：先做四项一致性检查（`boundary_id`、`known_at`、`event.consumer_boundary`、`event.log_epoch`），不符 → `Err(UnprovenSystemCut)`（`:528-534`）；随后 `Ok(partitions.iter().any(|p| p.partition == event.partition && p.log_epoch == event.log_epoch && event.cursor <= p.inclusive_frontier))`（`:535-539`）。
2. 缺陷（confirmed）：`:535-539` 把两种不同情况合并成同一个 `Ok(false)`——「事件 cursor 超过该分区已证明的前沿」与「Cut 未列出该事件所在 partition，因而没有任何前沿证明」。后者经 `src/pit/visibility.rs:157` 跳过，落到 `src/pit/visibility.rs:217` 的确定 `NotVisible(OutsideSystemCut)`。这等于把「缺 partition 覆盖证明」推断为「该分区没有事件」。草案要求 5 被违反。测试证据：`tests/acceptance_align.rs:736-789`。
3. 前沿闭区间已正确实现：`event.cursor <= part.inclusive_frontier`（`src/evidence.rs:538`；字段 `inclusive_frontier` 见 `src/evidence.rs:389`）。草案要求 3 满足，无需新增语义。
4. 分区唯一性已实现：`src/evidence.rs:469-472` 排序后查重 → `UnprovenSystemCut`；分区顺序无关（`sort_unstable` 后比较）→ 草案测试 `cut_duplicate_partition_rejected` 与 `cut_multi_partition_permutation_invariant` 已满足。
5. 单一代次约束：`src/evidence.rs:463-466` 要求每个 `CutPartitionRef.log_epoch == cut.boundary_epoch`，否则 `UnprovenSystemCut`。一个 Cut 无法表达跨 epoch 的分区集合。
6. `known_at` 绑定：`src/pit/context.rs:277-280` 要求 `cut.boundary_id() == boundary` 且 `cut.known_at() == parts.known_at`，否则 `UnprovenSystemCut`。但 `known_at_binding_ref` 只做 label 长度校验（`src/pit/context.rs:284-292`），`src/` 内无任何读取点 → 「绑定关系与证据版本」未参与判定。
7. `consistency_policy_ref` 与 `evidence_snapshot_ref` 同样只存储与长度校验（`src/pit/context.rs:284-292`、访问器 `src/evidence.rs:518-520`），`src/` 内无读取点。草案要求 4 未实现。
8. `CutPartitionRef` 只有 `partition` / `log_epoch` / `inclusive_frontier`（`src/evidence.rs:385-390`），没有 `spec.md:590`（§11.2）允许的「确切成员集合」表示形式。

判定：**partially-confirmed**，其中第 2 点为 confirmed 缺陷，第 3、4 点为 refuted（草案要求的保护已经存在），第 6、7、8 点为实现缺口。

### 与 `spec.md` 的冲突点

- `spec.md:590`（§11.2）允许「每分区明确的可见性前沿或确切成员集合」。实现只支持前沿。是否本版实现成员集合 → 记为 R5。
- `spec.md:589`（§11.2 表）要求 `known_at_binding` 表达「绑定关系和证据版本」，`consistency_policy` 表达「该 cut 如何保证是受控可读视图，及其版本」。实现把这些字段当作不透明标签。补齐校验强度需要 Owner 决定校验对象（仅与 `known_at` 比较，还是要求 coverage 层的对应引用相等）→ 记为 R5。
- 草案测试 `cut_partition_missing_is_indeterminate` 与 `tests/acceptance_align.rs:766-769` 现有断言直接冲突；该断言对应的规格期望列（`spec.md:1240`）只要求「不可声称完整系统视图」，未要求 `NotVisible` → 与 AUD-001 的 R2 同一裁决。

### 最小反例设计

```text
反例 P1（缺 partition 证明被当成确定结论）
  events: partition="part2", cursor=1, consumer_boundary="consumer1", log_epoch="log1"
  cut.partitions = [part1/log1 frontier=u64::MAX]
  现行为：NotVisible(OutsideSystemCut)
  期望行为：Indeterminate（该分区无前沿证明）

对照 C2（必须保持 NotVisible）
  events: partition="part1", cursor=2；cut.partitions=[part1/log1 frontier=1]
  期望保持 NotVisible(OutsideSystemCut)。已由 tests/acceptance_align.rs:604-638 覆盖。
```

P1 与 C2 需要 `contains` 三态返回才能区分，与 AUD-001 建议的 `InCut / OutsideCut / Unproven` 同一改造，建议合并进 WP-01 与 WP-05 的共同实现单元。

## 7. AUD-006 —— 派生数据 lineage

### 源码事实

1. 库内没有 lineage 类型：`grep` 在 `src/` 内无 `ActualExecution`、`Counterfactual`、`Simulation`、`lineage` 命中。唯一的溯源字段是 `FactVersionParts.input_provenance_ref: &'a str`（`src/revision.rs:229`），只做长度校验（`src/revision.rs:255`），在 `src/pit/selection.rs:348`（长度）与 `:495`（同版一致性比较）使用，不被解引用。
2. `PitManifestRef`（`src/pit/selection.rs:77-86`）没有 lineage 字段，而 `spec.md:827`（§16.3 manifest 表）要求「lineage manifest | 派生查询使用」→ 规格要求与实现的记录缺口。
3. 该边界已由测试注释明确登记：`tests/acceptance_align.rs:1278-1282`「T101–T106 属派生数据下界 / 完成时间下界 / 历史伪造拒绝 / 反事实模拟 / PIT join / 模型实际可用性，均为消费方所有（规格 §17）；crate 不暴露派生数据 API」。测试 `tests/acceptance_align.rs:1283-1318` 只断言 crate 侧的 Indeterminate / NotVisible / Visible。
4. `spec.md:840-863`（§17.1-§17.3）已经完整定义草案要求的 1、2、3、5 内容，并在 `spec.md:863` 写明「这些训练/计算逻辑在消费方实现，temporal 只提供必要时间与版本合同」→ 草案要求 6（temporal 不实现 Feature Engine）与现行规格一致。
5. `checklist.md` 的 G-011「派生数据继承知识边界」状态 NOT_RUN。

判定：**partially-confirmed**。缺口不在算法，而在记录与合同：temporal 承诺「提供必要时间与版本合同」（`spec.md:863`），但仓内没有承载 §17 lineage 的最小合同类型或 manifest 字段，也没有把 §17 的验收拆给消费方的合同文件。

### 与 `spec.md` 的冲突点

`spec.md:827` 要求 manifest 含 lineage manifest 项，实现没有该字段。补齐会改变 `PitManifestRef` 公共 API → 记为 R6（本版是否加字段，或由消费者在 manifest 之外自行绑定）。

### 最小反例设计

```text
反例 L1（派生输出声称早于输入可用性）
  输入版本 I 在 cutoff 判为 Indeterminate；
  消费者把「今日离线重算」的输出标为 cutoff 时 system_available。
  crate 侧唯一可断言的事实：输入不可进真实决策（现行为 Indeterminate，
  tests/acceptance_align.rs:1283-1318 已覆盖）。
  拒绝「输出早于输入」需要 lineage 记录 → 不在当前公共 API 内可达。
```

结论：AUD-006 的验收测试（`derived_feature_cannot_predate_input_availability` 等）不能由本仓单元测试完成。建议 WP-06 交付范围限定为：合同文档 + R6 裁决 + 把 §17 测试项按 `spec.md` §32.1 拆分 Owner 并登记到 `checklist.md`。

## 8. AUD-007 —— 跨语言与存储精度

按草案的 5 个要求逐条判定。

| 草案要求 | 判定 | 证据 |
| --- | --- | --- |
| 1. JSON 纳秒整数不得经过 IEEE-754 浮点 | 不在库内（NOT_APPLICABLE in-crate） | `Cargo.toml`：`[dependencies]` 只有 `thiserror = "2"`；`serde` 仅在 `[dev-dependencies]`。`src/` 内无任何 serde 实现；测试反向断言 `tests/api_compile.rs:14` 与 `tests/p1a_api.rs:80`：`assert_not_impl_any!(UnixTimeNs: ..., serde::Serialize)`。JSON 处理 100% 在消费者侧。 |
| 2. Protobuf 缺失值不得默认成 Epoch | 不在库内（NOT_APPLICABLE in-crate） | 仓内无 wire/Proto/DTO 类型。库内未知时间的表示是 `TimeField::Unknown`，无默认值构造（`UnixTimeNs::from_unix_nanos` 必须显式给值，`src/unix_time.rs:19-21`）。 |
| 3. 秒/纳秒转换必须检查范围 | refuted（已实现） | `src/unix_time.rs:29-47`：`try_from_unix_micros` / `try_from_unix_millis` / `try_from_unix_seconds` 全部 `checked_mul` → `TimeError::Overflow`。`:90-99`：`try_from_seconds_nanos` 拒绝 `nanos >= 1_000_000_000`（`TimeError::InvalidSubsecond`）并用 i128 检查范围。`:101-124`：`try_from_system_time` / `try_into_system_time` 往返校验，不无损则 `TimeError::SystemTimePrecisionLoss`。`checked_add` / `checked_sub` 走 i128 → `TimeError::Overflow`。 |
| 4. 数据库降精度必须显式声明舍入策略 | refuted（合同已实现） | `src/precision.rs`：`TimePrecision`（Seconds/Millis/Micros/Nanos）、`PrecisionLossPolicy`（Reject/Truncate/Floor）、`TimeStorageCapabilities`（注释「存储适配器声明的时间能力；声明本身不是读写证明」）、`unix_ns_to_stored`、`stored_to_unix_ns`、`quantize_unix_ns`、`require_lossless_unix_ns`。 |
| 5. DB 读回值与权威值分开保存；gRPC 请求必须绑定 Snapshot/KnowledgeBasis/Cut/Policy/schema 引用 | 部分实现、部分不在库内 | 读回校验已有：`src/precision.rs:88-104` `verify_projection`（先检查读回值对齐，再比较权威值的预期投影）。gRPC 绑定不在库内（无 RPC 代码）；查询侧的绑定合同存在（`AsOfContextParts` 携带 `dataset_snapshot`、`knowledge_basis`、`system_cut`、`policy`、三个 `*_schema_ref`，`src/pit/context.rs:206-260`），但这些 schema 引用当前不参与判定（见 AUD-002 第 3 点）。 |

判定：**partially-confirmed**。要求 3、4 应改判 refuted（草案描述的风险在本库已被处理）；要求 1、2 与要求 5 的传输部分属消费者 DTO/服务合同，本库无对应代码面，只能作为 WP-07 的消费者合同交付项。草案把「temporal 已提供的能力」记成「缺失」的部分需要更正，否则 WP-07 会造出仓库不需要的新抽象，违反 AGENTS.md 反过度设计纪律。

### 与 `spec.md` 的冲突点

无。`spec.md:448`（§8.4 存储能力声明不是证明）与 `spec.md:1783`（本库不承诺访问权限、加密或长期存储可靠性）已经划清边界。

## 9. 需要 Owner 裁决的清单（本任务在此停下，不改语义）

| ID | 问题 | 阻塞 | 建议裁决方向 |
| --- | --- | --- | --- |
| R1 | `spec.md:994` 固定 Indeterminate 的原因码为 5 项。AUD-001 若引入新原因（如 `TARGET_COVERAGE_UNPROVEN`）即违反该枚举；复用 `RECEIPT_UNPROVEN` 或 `SYSTEM_TIME_UNPROVEN` 会损失可解释性 | WP-01 | 二选一：(a) 扩充 §22 原因枚举并同步 `docs/标准.md` 与 `DecisionReason::code()`；(b) 复用现有码并在诊断层给出明细 |
| R2 | 「目标消费者证据覆盖完整性」合同落点：扩展 `CandidateCoverageRef`，还是新增按 boundary/epoch 的覆盖证明类型；`tests/acceptance_align.rs:692-695`、`:730-733`、`:766-769` 三处断言是否允许改写 | WP-01、WP-05 | 规格先行：先改 §12.3 或 §13.3，再定测试改写清单 |
| R3 | 证据/快照 schema 版本绑定校验的强度与错误码（TP-T070 要求 `EvidenceMismatch` / `Indeterminate`） | WP-02 | 按 `spec.md:1229` 直接实现，不改规格 |
| R4 | `chain_digest` 的承诺对象（单版本语义 / 完整链清单 / 链快照）与是否需要不可变 chain manifest 概念 | WP-03 | 若不裁决，WP-03 只做文档化 + 现有链校验测试补齐，不加新类型 |
| R5 | Cut 的「确切成员集合」表示本版是否实现；`known_at_binding_ref` / `consistency_policy_ref` / `evidence_snapshot_ref` 是否参与判定以及与谁比较 | WP-05 | 至少要求 `contains` 三态化；成员集合可推后 |
| R6 | 本版是否在 `PitManifestRef` 增加 lineage 字段以满足 `spec.md:827` | WP-06 | 公共 API 变更，需 Owner 决定 |

## 10. 门禁状态与 test_commands

本任务未运行任何 cargo 命令（纪律要求由主 Agent 统一执行门禁）。

| 门禁 | 状态 |
| --- | --- |
| `cargo fmt --check` | NOT_RUN |
| `cargo clippy --all-targets --all-features -- -D warnings` | NOT_RUN |
| `cargo test --all-targets --locked` | NOT_RUN |
| `cargo doc --no-deps --locked` | NOT_RUN |
| `cargo bench` | NOT_RUN |
| MSRV 1.88 校验 | NOT_RUN |
| 三平台（Linux/macOS/Windows） | NOT_RUN（本机仅 Linux） |

供主 Agent 执行、用于验证本报告结论的命令（只读验证，不改动代码）：

```bash
cargo test --all-targets --locked --test acceptance_align tp_t074
cargo test --all-targets --locked --test acceptance_align tp_t075
cargo test --all-targets --locked --test acceptance_align tp_t076
cargo test --all-targets --locked --test pit_contract tp_t116
cargo test --all-targets --locked --test p1b_revision tp_t059
cargo test --all-targets --locked --test acceptance_align tp_t117
cargo test --all-targets --locked
```

说明：前六条用于固定 AUD-001 / AUD-005 / AUD-003 / AUD-004 的现行为基线。任何把 `NotVisible` 改为 `Indeterminate` 的修复都会使 `tp_t074`、`tp_t075`、`tp_t076` 三个测试失败，这是预期信号，不是回归；改写前需要 R2 裁决。

## 11. 核查覆盖记录

已读：`src/pit/visibility.rs`、`src/evidence.rs`、`src/revision.rs`、`src/pit/selection.rs`、`src/pit/context.rs`、`src/unix_time.rs`、`src/precision.rs`、`Cargo.toml`、`spec.md`（§9-§17、§21-§22、§16.3）、`checklist.md`、`docs/性能基线.md`、`docs/evidence/BASELINE-20261108.md`、`tests/acceptance_align.rs`（夹具与 §22.7-§22.12）、`tests/pit_contract.rs`、`tests/p1b_revision.rs`、`tests/api_compile.rs`、`tests/p1a_api.rs`。

未逐行读：`src/clock.rs`、`src/model.rs`、`src/observation.rs`、`src/error.rs`、`src/pit/effective.rs`、`src/pit/diagnostics.rs`、`goal.md` 全文、`docs/API.md`、`docs/测试矩阵.md`、`docs/迁移指南.md`、其余测试文件。本报告结论不依赖这些文件的细节；若 WP 实现需要其中内容，须另行核查。

本任务只写本报告一个文件，未修改 `src/` 或 `tests/`。
