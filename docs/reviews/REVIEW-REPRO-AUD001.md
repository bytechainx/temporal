# REVIEW-REPRO-AUD001 — t4 独立确认：`tests/repro_audit001.rs` 是否捕获 AUD-001

审查人：security-reviewer-2（reviewer）。日期：2026-11-08。方法：静态追踪夹具与判定路径 + 交叉核对门禁日志与构建产物来源。未运行 cargo。

## 0. 裁定

- 裁定：**pass**。`tests/repro_audit001.rs` 确实捕获 AUD-001，夹具没有缺陷。
- 测试 2 `cross_consumer_evidence_is_indeterminate` 在提交基线上必然失败：路径为 `src/pit/visibility.rs:153-159` 的 `continue` → `:217-221` 的 `NotVisible(DecisionReason::OutsideSystemCut)`。静态追踪与 115ea53 上的实际失败逐字一致。
- 测试 1、3、4 在未修复代码上必然通过，三个修复边界锁定有效。
- 绿→红翻转与夹具无关，成因是**构建来源污染**：17:21 与 17:28 两次 gate 运行复用了兄弟隔离树 `fix-v0.2.0-pit-consumer-isolation` 中未提交的 WP-01 补丁所编译的产物（见 §3）。两次「绿」不得作为 Gate C 证据。
- repro 可作为 WP-01 的确定性回归门。附带 3 条非阻塞观察（§6）。

## 0.1 审查对象与基线

| 项 | 值 |
| --- | --- |
| 审查树 | `/home/workspace/bytechainx/.worktrees/temporal/goal-v0.2.0-pit-hardening`，分支 `goal/v0.2.0-pit-hardening`，审查时 HEAD `0fcc4bc` |
| 被测夹具 | `tests/repro_audit001.rs` 273 行；`4f719d7` 新增，`0820462` rustfmt；`git diff 0820462..HEAD` 对该文件为空 |
| 被测实现 | `src/pit/visibility.rs` 228 行；自 `07374f8` 起未改动；`git diff 0820462..HEAD` 为空 |
| 辅助正本 | `src/evidence.rs:88-101`（`TimeBounds::at`）、`src/evidence.rs:522-540`（`SystemKnowledgeCutRef::contains`）、`src/revision.rs:296-345` 与 `:399-401`（`FactVersionParts` 校验与 `system_evidence` 访问器） |
| 对照日志 | `.scratch/temporal-gates/repro-red-20261108.log`（17:21，HEAD 4f719d7）、`gates-0820462.log`（17:28）、`msrv-1.88-115ea53.log`（18:16，HEAD 115ea53） |

行号说明：本文所有 `src/pit/visibility.rs` 行号对 `7cef81d`、`4f719d7`、`0820462`、`115ea53`、`0fcc4bc` 同时有效——该文件自创建提交起未变。

## 1. 夹具数据流静态追踪（问题一）

结论：夹具确实把跨消费者证据挂到 `version.system_evidence`，判定确实触达 `:153-159` → `:217-221`。逐环节证据：

1. 挂载点：`tests/repro_audit001.rs:148-162` 用 `SystemVisibilityEvidenceRef::try_new(base.identity(), boundary, "log1", "event1", "receiver1", "rx1", "receipt1", "part1", cursor, available_bounds, proof, &limits)` 构造 1 个事件，`boundary` 参数即事件的 `consumer_boundary`；`:176` 把它写入 `FactVersionParts.system_evidence`。
2. 不被吞掉：`src/revision.rs:296-345` 只做「数量 ≤ `max_evidence_per_version`」「`evidence.version() == parts.identity`」「同 `(consumer_boundary, log_epoch, event_id)` 或同 `(partition, cursor)` 不得互相矛盾」三类检查，全部通过（夹具 1 个事件、`base.identity()` 同一身份、无重复）。`src/revision.rs:399-401` 的访问器原样返回该切片，不做过滤、不去重。
3. 前置门禁不改变结论：`evaluate_visibility` 的 `:63-68`（`validate_version_limits`）、`:69-74`（分区数）、`:75-80`（authority / snapshot / profile 一致性）对夹具全部通过——夹具的 `authority1`、`DatasetSnapshotRef("s1","manifest1","schema1")`、`TemporalProfile::PublishedObservation` 与 `query_context` 完全同源构造。
4. 来源发布门禁：`source_release = Required`、`source_evidence` 为 `Some`（`tests/repro_audit001.rs:83-96`），`source_scope "source1"` 与上下文 `Some("source1")` 相等；`TimeBounds[10,10] Exact` 与 `publication_time = 10` 不冲突；`src/evidence.rs:88-101` 的 `at(20, ExactOnly)` 因 `definitely_available_by 10 ≤ 20` 且 `quality == Exact` 返回 `ProvenByCutoff` → `release = Visible`（`src/pit/visibility.rs:104`）。
5. 早退分支均不触发：`:124-130`（`publication 10 > 20` 为假）、`:131-137`（`revision_time` 为 `Unknown`）、`:138`（`release` 不是 `NotVisible`）、`:141-143`（`knowledge_basis` 为 `SystemAsKnown`）。
6. 进入系统可读判定：`:144-146` 取到 `consumer_boundary = consumer2`（`tests/repro_audit001.rs:218`），`:147` 取到 cut（`:191-203`，`boundary_id = consumer2`，`boundary_epoch = "log1"`，`part1` 前沿 `frontier`）。
7. 关键分支：`:153` 遍历该 1 个事件；`:154` 比较 `event.consumer_boundary() != boundary` → `"consumer1" != "consumer2"` 为真 → `:155` `continue`。因此 `:157` 的 `cut.contains(...)` 在该测试中**从未被调用**，`saw_in_cut` 保持 `false`。
8. 收尾分支：`saw_proven`（`:199`）、`saw_unknown`（`:202`）、`saw_unmatched_receipt`（`:207`）、`saw_future`（`:212`）全为 `false` → `:217` 条件 `!saw_in_cut && !version.system_evidence().is_empty()` 为真 → `:218-220` 返回 `NotVisible(DecisionReason::OutsideSystemCut)`。
9. 唯一性核对：`OutsideSystemCut` 在整个 `src/pit/visibility.rs` 中只由 `:217-221` 产生（grep 核实），所以红观测的原因码只能来自该分支。
10. 动态印证：`msrv-1.88-115ea53.log:301-302` 记录 `panicked at tests/repro_audit001.rs:247:5: AUD-001 repro: 目标消费者缺少覆盖证明时必须 Indeterminate，实际为 NotVisible(OutsideSystemCut)`，与第 7-9 步的静态预测逐字一致。

补充：`src/evidence.rs:522-540` 的 `contains` 对「事件 `consumer_boundary` 与 cut `boundary_id` 不一致」返回 `Err(PitError::UnprovenSystemCut)`。当前实现靠 `visibility.rs:154` 提前 `continue` 而从未把这个 `Err` 暴露出来；这一点决定 WP-01 的落点选择（见 §5）。

## 2. 四个测试的路径判定（问题三上半）

| 测试 | 夹具输入 | 判定路径 | 未修复代码结果 | 断言 | 结论 |
| --- | --- | --- | --- | --- | --- |
| 1 `own_boundary_evidence_in_cut_is_visible` | 事件绑 `consumer1`，查询 `consumer1`，前沿 1，cursor 1 | `:154` 通过 → `:157` `contains` 为真（`src/evidence.rs:527-538`：`boundary_id`、`known_at = 20`、`log_epoch`、分区 `part1` 且 `cursor 1 ≤ frontier 1` 全匹配）→ `:161-168` 命中 receipt（`receiver1/rx1/receipt1`）→ `:169` 与 `:172-179` 通过 → `:180-187` receipt bounds `[15,15]` `ProvenByCutoff` → `:188-194` 事件 bounds `[16,16]` `ProvenByCutoff` → `saw_proven` → `:199-201` 返回 `release` | `Visible` | `Visible` | 绿（对照组有效） |
| 2 `cross_consumer_evidence_is_indeterminate` | 事件绑 `consumer1`，查询 `consumer2` | 见 §1 第 7-9 步 | `NotVisible(OutsideSystemCut)` | `Indeterminate(_)` | **红（预期红，正是 AUD-001 的缺陷证据）** |
| 3 `cross_consumer_evidence_never_grants_visibility` | 与测试 2 相同 | 同测试 2 | `NotVisible(OutsideSystemCut)` | `!Visible` | 绿 |
| 4 `event_after_proven_frontier_stays_not_visible` | 事件绑 `consumer1`，查询 `consumer1`，前沿 1，cursor 2 | `:154` 通过 → `:157` `contains` 返回 `Ok(false)`（`src/evidence.rs:535-538`：`cursor 2 > inclusive_frontier 1`）→ `:158` `continue` → `:217` | `NotVisible(OutsideSystemCut)` | `NotVisible(OutsideSystemCut)` | 绿（钉住 `OUTSIDE_SYSTEM_CUT` 的合法用途） |

对照动态记录：`msrv-1.88-115ea53.log` 对 repro 记录 `test result: FAILED. 3 passed; 1 failed`（失败项即测试 2），与上表完全一致。该日志同时记录 `tests/acceptance_align.rs` 跑 **35** 个测试，与提交树该文件的 `#[test]` 计数相同（`0820462` 与 `HEAD` 均为 35）。

## 3. 绿→红翻转机制（问题二）

结论：夹具与实现都不是原因。两次「绿」测量的二进制**不是提交树源码编译所得**，而是复用了兄弟隔离树里未提交的 WP-01 补丁产物。

动态证据（可复核）：

1. `gates-0820462.log:9-13` 记录 `tests/acceptance_align.rs` 跑 **37** 个测试，其中 `system_selection_other_consumer_coverage_is_indeterminate` 与 `system_visibility_other_consumer_evidence_is_indeterminate` 两项存在并通过；`repro-red-20261108.log:17`（HEAD `4f719d7`）同样记录 37。
2. 这两个测试名在任何提交中都不存在：`git grep -n "other_consumer" 0820462 -- tests/` 无命中，`HEAD` 的 `tests/` 亦无命中；提交树该文件只有 35 个 `#[test]`。所以 37 个测试的二进制必然来自未提交源码。
3. 未提交源码就在同一 `temporal` 的兄弟隔离树：`.worktrees/temporal/fix-v0.2.0-pit-consumer-isolation`（其 `HEAD` 为 `7cef81d`，脏文件 `spec.md`、`src/evidence.rs`、`src/pit/visibility.rs`、`tests/acceptance_align.rs`、`tests/evidence_contract.rs`）。该树 `tests/acceptance_align.rs` 恰有 **37** 个 `#[test]`，且包含上述两个测试名。
4. 该树的 `src/pit/visibility.rs` 补丁正是 WP-01 形态：新增 `DecisionReason::CutCoverageUnproven`（码 `CUT_COVERAGE_UNPROVEN`）与 `saw_frontier_excluded`，把 `if !saw_in_cut && !version.system_evidence().is_empty()` 改为「仅当切片已声明该分区且读数越出前沿才 `NotVisible(OutsideSystemCut)`；否则 `Indeterminate(CutCoverageUnproven)`」。用这份实现跑 repro，测试 1 仍 `Visible`、测试 2 变 `Indeterminate`、测试 3 仍非 `Visible`、测试 4 仍 `NotVisible(OutsideSystemCut)` —— 4/4 绿，与两次观测逐项吻合。
5. 两棵树共用同一构建目录 `/home/workspace/bytechainx/.cargo/wt/temporal`（两份日志里的 `deps/` 路径相同；两棵树内均无 `.cargo/config.toml`，说明 target 目录由外部环境注入）。cargo 的 unit 元数据哈希不含 worktree 路径，新鲜度按源文件 mtime 判断：goal 树 `src/pit/visibility.rs` 的 mtime 为 `14:56:59`（自 worktree 创建后未改），fix 树该文件 mtime 为 `16:21:55`。fix 树在 16:2x 的构建产物晚于 goal 树全部源码 mtime，因此 17:21 与 17:28 的 goal 树运行判定这些单元为「新鲜」并直接复用。
6. 复用痕迹与运行日志吻合：`gates-0820462.log` 只有 **1** 行 `Compiling temporal v0.1.0`（20 余个测试单元里只有 1 个被重编译 —— 即 17:27:54 被 `cargo fmt` 改写的 `tests/repro_audit001.rs`）。于是该轮 repro 测试源码来自 goal 树，却链接了 fix 树编译出的、已含 `CutCoverageUnproven` 的 `libtemporal`。
7. 18:16 的 MSRV 1.88 运行强制全新编译（工具链不同 → 单元哈希由 `repro_audit001-06c6caffd13acfeb` 变为 `repro_audit001-3162b0b0fafd4925`，日志同时重编译 `proc-macro2`/`syn`/`temporal`），才第一次量到提交树的真实行为：35 个 acceptance 测试、repro `3 passed; 1 failed`。`115ea53` 上 1.99 的十连发红与之一致。
8. 因此「红/绿机制未定论」可以结案：**红是提交基线的真实行为，绿是跨 worktree 构建产物复用造成的假绿**。`docs/evidence/DECISIONS-20261108.md` 登记的「repro 4/4 绿 @ `4f719d7` / @ `0820462`」应作废并注明成因，不得作为 Gate C 的绿证据。

## 4. 被排除的其它机制

| 候选机制 | 判定 | 依据 |
| --- | --- | --- |
| `HashSet`/`HashMap` 随机迭代（`RandomState`） | 排除 | 判定路径无哈希容器：`src/pit/visibility.rs` 只用切片 `iter()`/`find()`/`any()`（`:153`、`:161`、`:172`），`src/evidence.rs:535` 为切片 `any()`，`src/evidence.rs:469` 为 `sort_unstable()` 定序。repro 每测试只有 1 个系统事件与 1 个分区，任何顺序都得到同一结果。 |
| 更早的 `Indeterminate` 分支先命中 | 排除 | 能在 `:217` 之前产出 `Indeterminate` 的位置只有 `:109` 与 `:113`（来源侧）。夹具 `source_evidence` 为 `Some` 且 bounds `[10,10] Exact` 使 `:104` 取 `Visible`；即使它们变为 `Indeterminate`，`:217` 仍在 `:222` 之前先行返回 `NotVisible`。 |
| 其它 `Indeterminate` 触发路径 | 排除 | `:202`（`saw_unknown`）要求 bounds 判为 `Indeterminate`，夹具 bounds 全为 `TimeQuality::Exact` 且上界 ≤ 20；`:207`（`saw_unmatched_receipt`）要求事件先通过 `:154` 的边界与代次检查，夹具不通过；`:225` 只在 `system_evidence` 为空或 `saw_in_cut` 为真时可达。 |
| 夹具未把证据挂到 `version.system_evidence` | 排除（且与观测矛盾） | 若 `system_evidence` 为空，`:217` 不触发而 `:225` 生效，测试 1 只能得到 `Indeterminate(SystemTimeUnproven)` 而非 `Visible`；两次绿观测里测试 1 是绿的，故该假设不成立。挂载链路见 §1 第 1-2 步。 |
| cut 不含该证据导致提前 `Err` | 排除 | `src/evidence.rs:527-531` 的 `Err(UnprovenSystemCut)` 只在 `contains` 被调用时产生，而 repro 测试 2 在 `visibility.rs:155` 已 `continue`，`contains` 未被调用；红观测也是 `Ok(NotVisible(...))` 而非 `Err`。 |
| 实现或夹具在此期间被改动 | 排除 | `src/pit/visibility.rs` 自 `07374f8` 未变；`tests/repro_audit001.rs` 在 `0820462..HEAD` 无 diff；`7813b6f`、`115ea53` 只改 `///` 注释。同一份代码在两次运行里不可能既是绿又是红。 |

## 5. repro 作为 WP-01 回归门，以及对 t5 修复设计的影响

1. 确定性：**是**。同一夹具在 1.88 与 1.99、10 次连发全部得到同一红（`NotVisible(OutsideSystemCut)`），无顺序敏感路径。可作为 WP-01 的回归门：修复到位即转绿，且测试 1/3/4 提供三侧护栏（不得误判可见、不得把他人证据当作不可见证明、不得放宽已证明前沿之后的 `OUTSIDE_SYSTEM_CUT`）。
2. 原因码自由度：测试 2 只断言 `Indeterminate(_)`。t5 无论新增 `CUT_COVERAGE_UNPROVEN` 还是复用 `SYSTEM_TIME_UNPROVEN`/`RECEIPT_UNPROVEN`，都不会破坏该门。
3. 落点判断（回答「三态 `contains` 是否仍是正确落点」）：**三态化不是必需，但边界归属必须在 `contains` 之前分流**。两条可行形态：
   - 形态 A（兄弟树 WIP 的形态）：`contains` 保持 `Result<bool, PitError>`，在 `visibility.rs` 调用点用 `cut.partitions()` 是否声明该 `partition` 区分「已声明且越出前沿」（保留 `NotVisible(OutsideSystemCut)`）与「覆盖未声明」（`Indeterminate`）。优点：改动小、`OUTSIDE_SYSTEM_CUT` 语义收窄为「前沿已证明越出」。缺点：分区声明判断散落在调用点，`contains` 仍会对跨边界事件返回 `Err`，语义上把「归属不同」与「切片不可用」混在同一个错误码里。
   - 形态 B：`contains` 返回三态（已包含 / 已排除 / 未知），跨边界或跨 `log_epoch` 返回「未知」而不是 `Err(UnprovenSystemCut)`。优点：把「成员关系未知」显式化，`UnprovenSystemCut` 回到「上下文缺少切片证明」的原意。缺点：改动面更大，需同步 `src/pit/selection.rs` 的三处 `system_evidence()` 消费点（`:391`、`:579`、`:815`）。
   - 两个形态都必须处理 `visibility.rs:154` 的跨 `log_epoch` 跳过：那也是「覆盖未声明」而非「已排除」，不得继续产出确定结论。
4. 连带修订（t5 必须一并处理，否则 Gate C 会因既有断言转红）：`tests/acceptance_align.rs` 有三处把缺陷推理固定为确定 `NotVisible`，任何满足 AUD-001 的修复都会使它们失败 —— `tp_t074_boundary_isolation_no_shared_available_at`（`:692-695` 断言 `NotVisible(_)`）、`tp_t075_log_epoch_isolation_no_cross_epoch_ordinal_compare`（`:729-733` 断言 `NotVisible(DecisionReason::OutsideSystemCut)`，其行内注释「log2 事件被跳过 → 无可见事件 → NotVisible」正是 AUD-001 的推理）、`tp_t076_multi_partition_cut_needs_completeness_proof`（`:764-769` 断言 `NotVisible(_)`）。兄弟树的 WIP 已把这三处改为 `Indeterminate(CutCoverageUnproven)` 并新增两个 `other_consumer` 测试，可作参考实现，但其未提交状态不构成正本，t5 须在 goal 树上重新落地并提交。
5. 改判定语义须按工作区规则同步正本：`spec.md` §13.1 与 §22 的原因码表若新增码，须同时更新仓内契约文档，避免机器检查与正本漂移。

## 6. 附带观察（不阻塞 t4 裁定）

| 编号 | 严重度 | 观察 | 建议 |
| --- | --- | --- | --- |
| OBS-1 | 中 | 已登记的「repro 4/4 绿 @ `4f719d7` / @ `0820462`」来自被污染的构建（§3），与 115ea53 的红不构成矛盾 | 在 `docs/evidence/DECISIONS-20261108.md` 把该两行标注为作废并写明成因；Gate C 只采用 115ea53 之后的记录 |
| OBS-2 | 中 | 门禁脚本未记录被测工作树是否干净、也未记录测试计数，导致假绿无法当场识破 | gate 在记录结果前先跑 `git status --porcelain`（`src/`、`tests/` 必须为空），日志头写入 `git rev-parse HEAD`、worktree 绝对路径、每个测试二进制 `running N tests` 的 N |
| OBS-3 | 中 | 两个 `temporal` 隔离树共用 `/home/workspace/bytechainx/.cargo/wt/temporal`，cargo 按 mtime 判新鲜度且单元哈希不含 worktree 路径，可跨树复用二进制 | 每棵树使用独立 target 目录（例如 `.cargo/wt/temporal-<slug>`），或 gate 前 `cargo clean`；按 `docs/rules/worktree.md` §4 复核该约定并在门禁脚本固定下来 |
| OBS-4 | 低 | repro 只钉住「证据属于其他 consumer」这一种形状，未覆盖「cut 未声明该 partition」与「跨 `log_epoch`」两种同样会被误判成确定 `NotVisible` 的形状，也未覆盖 `src/pit/selection.rs` 的候选侧跨 consumer 路径（`:391`、`:579`、`:815`） | WP-01 落地时按 R1 裁决补齐这两类断言与候选侧断言；不必改动本 repro 文件本身 |
| OBS-5 | 低 | 测试 3 与测试 2 输入完全相同、断言更弱（`!Visible` 是 `Indeterminate` 的必要条件），判别力被测试 2 完全覆盖 | 保留可作护栏；若要加强，建议把它改为候选侧断言（跨 consumer 证据不得进入可见集合），与 OBS-4 合流 |
| OBS-6 | 低 | `tp_t073_future_commit_with_old_wall_clock_excluded`（`tests/acceptance_align.rs:605-637`，断言 `NotVisible(OutsideSystemCut)`）与 repro 测试 4 同形，是 `OUTSIDE_SYSTEM_CUT` 的合法用途 | WP-01 的任何形态都必须让它保持绿；它是「前沿已证明越出」的现存护栏，无需新增 |

## 7. 门禁操作要求（防止再次假绿）

1. 记录 Gate C 之前必须确认：`git -C <worktree> status --porcelain` 对 `src/` 与 `tests/` 为空，且日志头写入 HEAD 与 worktree 路径。
2. gate 与任何手工 cargo 运行都不得跨 worktree 共享 target 目录（OBS-3）。已在共享目录里产生的产物应先失效（`cargo clean` 或改用独立目录）。
3. 每次 gate 运行后核对「日志里的每个 `running N tests`」与该文件在提交里的 `#[test]` 计数；不一致即说明被测源码不等于提交源码，本轮结果不得登记。
4. 兄弟隔离树 `fix-v0.2.0-pit-consumer-isolation` 的未提交改动不得被当作 WP-01 已完成；它是参考 WIP，其落地由 t5 在 goal 树重新提交。

## 8. 三问裁定汇总

1. 夹具是否真把跨 consumer 证据挂到 `version.system_evidence` 并触达 `:153-159` → `:217-221`：**是**（§1 第 1-9 步，动态红观测逐字印证）。
2. 绿→红翻转机制：**跨 worktree 构建产物复用**。兄弟树未提交的 WP-01 补丁在 16:2x 编译进共享 target 目录，17:21 与 17:28 的 goal 树运行按 mtime 判定复用其 `libtemporal` 与 `acceptance_align` 二进制（37 个 acceptance 测试、含两个任何提交里都不存在的测试名）；18:16 的 1.88 全新编译才量到真实行为。哈希随机迭代、早退分支、其它 `Indeterminate` 路径、夹具缺陷、代码漂移均已逐项排除（§4）。
3. repro 是否为确定性红测试、能否当 WP-01 回归门：**是**，10 连发一致（1.88 与 1.99），且测试 1/3/4 在未修复代码上全绿，构成修复边界护栏。

裁定：**pass**。夹具无需修正；需要修正的是证据登记与门禁操作（OBS-1..OBS-3），以及 t5 必须连带修订的三处既有断言（§5 第 4 点）。



