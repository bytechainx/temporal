# BRIEF-T5 — WP-01 AUD-001 consumer evidence isolation 修复（t5 执行简报）

正本契约 = 任务 t5 description；本文件是执行要点浓缩版，供新会话直接读取，避免长派工消息撑爆上下文。

## 1. 缺陷链（已由 t4 复核确认，verdict pass）
- src/pit/visibility.rs:153-159：扫描 `version.system_evidence()` 时 `evidence.boundary_id() != cut.boundary_id()` → `continue`，只消费 Cut 自身 boundary 的证据。
- src/pit/visibility.rs:217-221：任一 revision 满足 `!saw_in_cut && !version.system_evidence().is_empty()` → 确定性 `NotVisible(DecisionReason::OutsideSystemCut)`。
- 后果：目标消费者缺覆盖证明时返回确定性 NotVisible，正确语义应为 Indeterminate + 覆盖未证明原因（spec.md §22 / §12.3 / §13.3）。
- t4 复核报告：docs/reviews/REVIEW-REPRO-AUD001.md（可分段读，勿全读）。

## 2. Owner 裁定（已登记 docs/evidence/DECISIONS-20261108.md）
- R1：DecisionReason 枚举扩展 TARGET_COVERAGE_UNPROVEN（本任务只在 src/pit/visibility.rs 内新增该枚举变体与 code() 分支；spec.md 文本同步归 t15）。
- R2：覆盖判定复用既有 CandidateCoverageRef，不新建类型。
- 主候选形态 A：visibility 内部三态 contains（经 SystemKnowledgeCutRef 判定 InCut / OutsideCut / Unproven）。

## 3. 最小修复形态（起点，遇阻回报 captain，不自行扩大）
1. 证据扫描不再跳过其他 boundary 的 SystemVisibilityEvidenceRef；按其与当前 Cut 的关系三态化。
2. :217 的「存在系统证据但无一在 Cut 内」拆分：
   - 确证在 Cut 外（cursor 越过已证 frontier 之类）→ 保持 NotVisible（既有正当路径不得放宽，如 tests/acceptance_align.rs tp_t073 与 repro1/3/4 场景）。
   - 覆盖未证明 → Indeterminate + TARGET_COVERAGE_UNPROVEN。
3. repro 测试 tests/repro_audit001.rs 的 cross_consumer_evidence_is_indeterminate 期望由此转绿。

## 4. 硬纪律（违反即返工）
- 只写 src/pit/visibility.rs。不新增依赖，不跑 cargo（门禁由 captain 代跑），不 push。
- 小步执行：每回合 ≤1 次 edit（≤40 行）→ 立即提交；提交必须带 pathspec：`git commit -q -m "fix(temporal): ..." -- src/pit/visibility.rs`。
- 读文件必须 offset/limit 有界（每次 ≤120 行）；visibility.rs 分两段读。禁止整读大测试文件。
- 禁止 compress 调用（成员侧故障主因之一）。
- 遇语义歧义（如既有 NotVisible 路径归属不明）：发一条短消息给 captain 裁决，勿自行扩大改动。
- 回报格式：改动摘要 + 提交 SHA 列表 + 对 t6 测试接口说明 + 对 tp_t074/075/076 影响预期。gate_status 一律 NOT_RUN。
