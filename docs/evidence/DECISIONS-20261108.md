# Owner 裁决与 captain 决定登记（2026-11-08）

依据：pit-researcher T1 报告 §9（commit 942fb11）与 architect ADR-0001 §6 / ADR-0002 §8 提出的待裁决项。R1、R2 为 Owner 在会话中的直接答复；其余为 captain 依据 Owner 已批准的执行指示 + AGENTS.md 反过度设计规则采纳的默认决定，全部供 Gate B/C/E 复核。

## Owner 直接裁决（用户答复，2026-11-08）

| 项 | 裁决 | 落点 |
| --- | --- | --- |
| R1 | 扩充 §22 Indeterminate 原因枚举：新增 `TARGET_COVERAGE_UNPROVEN`（`DecisionReason` 新变体 + `code()`），不损失可解释性 | 代码：t5；正本同步（spec.md §22、docs/标准.md、docs/API.md）：t15 |
| R2 | 「目标消费者证据覆盖完整性」落点 = 扩展既有 `CandidateCoverageRef`（boundary/epoch 证据覆盖校验），不新增独立类型；先改 §12.3/§13.3 正本（t15）；`tests/acceptance_align.rs:692-695`、`:730-733`、`:766-769` 三处 `NotVisible` 断言允许按新语义改写为 `Indeterminate` | 测试改写：t6；正本：t15 |

（R5 与 R2 为同一裁决：`cut_partition_missing` 类断言按 Indeterminate 新语义处理。）

## captain 默认决定（采纳 ADR 建议 + AGENTS.md 反过度设计）

| 项 | 决定 | 依据 |
| --- | --- | --- |
| R4-1 chain digest 承诺对象 | 采纳 ADR-0002 裁定 C2：同一连通链所有节点声明同一 `chain_digest` 与同一 `algorithm_version` | ADR-0002 §2 否决 C1（与 semantic_digest 重叠）与 C3（破坏纯函数、breaking） |
| R4-2 库内 `chain_manifest_ref` | v0.2.0 不新增该字段；chain manifest 本体留在消费侧 Trusted Evidence Adapter（ADR-0002 §5） | AGENTS.md 反过度设计四问；spec.md:1426 兼容性评审。t9 只实现 CH-1/CH-2 字节一致性 |
| R4-3 拒绝错误码 | 复用 `InvalidRevisionChain`，不新增 `PitError` 变体 | ADR-0002 §3 |
| R4-4 `chain_digest.algorithm_version` 与 `semantic_digest.algorithm_version` 等值 | 不强制；文档明示两者独立 | 最小改动；避免无消费者的耦合 |
| R4-5 §22 新增 digest 一致性 TP + `spec.md:496` 措辞收紧 | 由 t15 执行正本变更 | 超出 ADR 任务写范围 |
| ADR-0001 D-1 库内 schema/extractor 等值比较 | v0.2.0 不做；一致性属 Adapter 第 4 层义务（ADR-0001 §2/§4） | 草案 AUD-002 修复要求 2/3 的分层口径 |
| ADR-0001 D-2 `EvidenceProofRef` 更名 | v0.2.0 不改名（breaking） | spec.md:1426 兼容性评审 |
| ADR-0001 D-3 docs/API.md 落点 | 与 spec.md 一起由 t15 交付 | — |
| R3 AUD-002 措辞 | 按 ADR-0001 §5 清单做零语义变更 rustdoc 改写，单独任务 t20（rust-engineer），供 t16 审阅 | ADR-0001 §5；WP-02 交付物「spec + adapter contract」 |
| AUD-005 confirmed 缺陷（`src/evidence.rs:535-539` 合并 Ok(false)） | 纳入 t9 的 WP-04 范围：区分「cursor 越过前沿」与「Cut 缺该 partition」两种语义 | AUDIT-VALIDATION §6 |
| R6 AUD-006 lineage | t13（ADR-0003）先给合同方案；任何 `PitManifestRef` 字段新增须过 Gate C | 合同缺口在记录层，不在算法 |

## 版本口径（Owner 既定目标）

v0.2.0：CH-1/CH-2（链 digest 一致性收紧）、R2 覆盖合同扩展、R1 新原因码均为行为收紧/新增，属 v0.2.0 minor 语义变更，须 CHANGELOG + Gate C 兼容性评审（t11/t18）。本 Goal 不执行 tag/merge/push（Gate E 属 Owner）。

## 2026-11-08 第二批登记（captain）

| 事项 | 裁定 | 依据 |
| --- | --- | --- |
| Gate C：`PitManifestRef.lineage_manifest_ref` | v0.2.0 不受理；维持 t13 裁定 L-B（lineage = 消费者侧版本化不可变记录 + 复现清单绑定）；字段新增仅作为 ADR-0003 §6 item 4 的 Gate C 候选保留 | t13 L-B + R2/R6 + 反过度设计（库内无消费者） |
| git 事故 3924dc2 | 不改写历史；`docs/reviews/REVIEW-GATE-B.md`（133 行）内容归属 security-reviewer（t16 作品），由 t18 Evidence Manifest 登记；根因 = 共享 index + commit 无 pathspec；全队纪律：提交必须带 pathspec | architect 自报 + captain 处置 |
| repro 翻转观测 | 4f719d7/0820462：repro 4/4 绿（各 1 次）；115ea53：`cross_consumer_evidence_is_indeterminate` 连续 10 次 FAILED（panics tests/repro_audit001.rs:247，实际 NotVisible(OutsideSystemCut)；1.88 与 1.99 一致）。其间 src/ 仅 /// 注释变更（7813b6f、115ea53、selection.rs 未提交 diff 逐行核对）。红/绿机制未定论，交 t4 静态判定（重点：夹具/实现是否经 HashSet/HashMap 随机迭代等顺序敏感路径） | gates 日志 + 10 连发复跑 |
| MSRV 1.88 补测（HEAD 115ea53） | `cargo +1.88 build` EXIT 0；`cargo +1.88 doc --no-deps` EXIT 0；`cargo +1.88 test --all-targets` EXIT 101，唯一失败 = 上述已知红 repro（AUD-001 缺陷证据，预期红）；无 MSRV 不兼容编译错误 | .scratch/temporal-gates/msrv-1.88-115ea53.log |
| t13 test_commands | profile_contract 6 passed / acceptance_align 35 passed / api_compile 2 passed（1.99） | captain 代跑 |

## 2026-11-08 第三批登记（captain，19:5x）

| 项 | 裁定 | 依据 |
| --- | --- | --- |
| Gate E 版本落点 | 0.1.0→0.2.0 授权链 = Owner 草案 1.md（规定 v0.1.x→v0.2.0 的 MINOR 语义变更）；执行 = t18（release-reviewer）在 worktree 手改 `Cargo.toml version` + `CHANGELOG.md` 首标题 + rustdoc 元数据三处；`scripts/bump-patch.mjs` 不适用（只升补丁位）；检查器 `node scripts/check-version-consistency.mjs` 由主 Agent 代跑；不 publish/tag/merge/push | docs/code-standards/versioning.md §7.1/§8 |
| crates.lock.json | 元仓库正本、不在 temporal worktree 写范围；lock 钉的 temporal sha `a5b1d42` 落后基线 `7cef81d`（为其祖先）。登记为 t18 Manifest 的「Owner 侧后续项」，本轮不修改元仓库 | AGENTS.md main-first；Gate E = Owner |
| Gate E 消费方兼容项 | 工作区 30 crate 无 temporal 依赖 → 该项记 BLOCKED(no consumer in workspace)，待 Owner 豁免或外部消费方证据，不得记 PASS | release-reviewer 扫描 + 既往 BLOCKED 裁定 |
| t12 写范围 | `Cargo.toml` 仅新增 `[[bench]]` 段落归 performance-engineer 独占；三个性能候选改进只作 WP05 evidence finding，需要落地时另立 repair | captain 裁定 A/B |
| t6 测试设计 | 批准 qa 的 6 项断言映射 + 4 对 differential probes；Q1 主候选 = visibility 内部三态 `contains`（InCut/OutsideCut/Unproven），(a)(c) 备选随 t5 定稿；Q3 测试不锁原因码 | AUDIT-VALIDATION 建议 + t6 契约 |
| t16 角色偏差 | Gate B 审查实际由 release-reviewer 承接（attempt 2）；独立性成立（非 ADR-0001 作者）；与草案 §8 角色映射的偏差在此登记 | 调度认领 + captain 追认 |
| 成员更替 | security-reviewer 会话因模型端故障（PI_AI_ERROR 空回复、CONTEXT_WINDOW_EXCEEDED）退出并被移除；t4 由 security-reviewer-2 以全新会话接手（attempt 4）；t7/t11 已回共享池 | 故障记录 + 队长处置 |

## 2026-11-08 第四批登记（captain，21:0x）

| 项 | 裁定 | 依据 |
| --- | --- | --- |
| t4 verdict = pass | 夹具确实捕获 AUD-001：`tests/repro_audit001.rs:148-176` 事件确实进入 `system_evidence`，`visibility.rs:154-155 continue` → `:217-220 NotVisible(OutsideSystemCut)`（该码全文件仅此一处产生）；repro 为确定性红门（1.88/1.99 十连发一致），直接作 WP-01 回归门 | docs/reviews/REVIEW-REPRO-AUD001.md（f1f341a..2ca9de6） |
| 「repro 4/4 绿 @4f719d7/@0820462」作废 | 属假绿：两 temporal 树共享父级 `.cargo/config.toml` 的 CARGO_TARGET_DIR（`.cargo/wt/temporal`）+ 兄弟树 `fix-v0.2.0-pit-consumer-isolation` 有未提交 WP-01 形态补丁 + cargo 按 mtime 判新鲜 → goal 树复用了兄弟树编译产物（`gates-0820462.log` 中 37 个 acceptance 测试名在 goal 树任何提交都不存在）。18:16 MSRV 1.88 全新编译才量到真实行为（acceptance 35 通过、repro 3 通过 1 失败）。此前登记的「repro 翻转观测」行的绿侧结论不得用于 Gate C | t4 报告 + 门禁日志对证 |
| 门禁卫生（captain 强制） | ① gate 运行一律显式 `export CARGO_TARGET_DIR=/home/workspace/bytechainx/.cargo/wt/temporal-goal`，不依赖父级 config；② 记录前校验 `git status --porcelain` 对 src/ tests/ 干净，日志头写 HEAD 与 worktree 绝对路径；③ 日志保留每个二进制 `Running ...`/`running N tests` 行并与提交树测试数核对 | t4 建议 + captain 裁定 |
| 兄弟树边界 | `fix-v0.2.0-pit-consumer-isolation` 的未提交改动（spec.md/src/evidence.rs/src/pit/visibility.rs/tests/acceptance_align.rs/tests/evidence_contract.rs）不是 WP-01 完成物，不得引用或复制；t5 在 goal 树重新落地。该树继续记环境等待，不触碰 | AGENTS.md 他人脏树 |
| Gate B 处置 | t16 verdict NEEDS_REVISION（10 findings，F-2/F-8 high）登记成立。返工链：t21（ADR-0001 修订 R2）→ t23（src 措辞收敛，rust-engineer）→ t22（Gate B 第 3 轮复审）；t18/t19 依赖由 t16 改接 t22；AUD-002 identity 库侧测试另派 t24（qa） | REVIEW-GATE-B.md §9/§10（baea429）+ captain 处置 |

## 2026-11-08 第四批登记（captain，23:2x）

| 登记项 | 内容 |
| --- | --- |
| t5 WP-01 修复核实完成 | attempt 7（rust-engineer-3）逐条核对已落地 94092ef（+24/-2，src/pit/visibility.rs：TargetCoverageUnproven 三态判定 + saw_uncovered 优先 Indeterminate，:240-242 保留 NotVisible）与 1cbb5e2（rustdoc 与 ADR-0001:89 第 20 行逐字一致），结论无缺口、本 attempt 零补丁。captain 门禁 @97b8f85：fmt/clippy/doc=0；repro 全绿；唯一红 tp_t074/075/076（:692/:730/:766，R2 授权 t6 改写）。修复实际由 rust-engineer-2 崩溃前会话落地（时间戳早于 attempt 7 派工），登记为调度事实。 |
| t5 歧义裁定（saw_future） | visibility.rs:227-231 saw_future 与 :247-249 收尾 fallback 在同时存在未覆盖事件时仍返回确定 NotVisible——与 AUD-001 同类「以忽略未知求确定结论」模式。裁定：归 WP-04（t9）——与 AUD-005 拆分同处处理；t9 须含差分测试（未覆盖事件 + 已证 frontier 同存 → Indeterminate 优先）。不在 t5 追加。 |
| t14 WP-07 完成（git 核验通过） | ADR-0004-wire-precision.md 102 行/8 节；提交链 19c1796→17cac54→b2e6e84→de868fe→637c5a7→883c1a9，全部 pathspec。裁定采纳 W-B（库只保留解码后置合同，否决 DTO/TimeEnvelope）。前提修正成立：AUD-007 partially-confirmed（要求 3/4 库内已实现 refuted；1/2 NOT_APPLICABLE；5 拆半）——WP-07 零代码。captain 复跑 t14 指定测试 @883c1a9：p1a_numeric 10/10、p1a_clock 10/10、p1b_precision 10/10、precision_contract 6/6、api_compile 2/2 全 EXIT 0（t14-tests-883c1a9.log）。 |
| 证据真实性事件二（已更正） | architect 此前上报 1978415/710e13b/0588f9b 均不存在于任何 ref，已由 architect 本人作废并出示命令原文更正。常设规则生效：captain 对每份完成报告先经 git 核验再入账；SHA 必须来自当回合命令输出。t18 Manifest 须登记本事件与更正证据。 |
| R7 裁定 | R7-1：spec.md §22 不新增跨语言/存储往返 TP 条目、不升 SCHEMA_VERSION（temporal 无 wire 层，该判据属消费者侧验收；登记为 Owner 侧后续项）。R7-2：可选判据 wire_guard_negative_and_extreme_seconds_nanos_round_trip 采纳为库内纯数值判据，并入 t10 测试范围（QA 经消息派发）。R7-3：docs/API.md §31 指针采纳，落 t15。R7-4：V-1..V-6 以 t18 Manifest「Owner 侧后续项」登记，本轮不写 checklist.md。W-9 库内不可强制归 WP-02/R3（t21 路径），不在 WP-07 重复。 |
| 调度异常登记 | attempt 6（rust-engineer-2）与 attempt 8（rust-engineer-3）两次 claimed→in_progress 均死于模型端（attempt 6：CONTEXT_WINDOW prompt 136428>131072——成员会话注入团队全量任务图/成员列表，重派消息不可过长；attempt 8：PI_AI_ERROR 空回复，同会话连续死亡）。captain 处置：改派前用短消息 + 简报文件（BRIEF-T5）而非长描述消息；同会话连续模型端死亡即换成员。另：AgentTeams 运行时两次把已 completed 的 t14/t15 重新派发给 idle 成员（t15 派给 qa-engineer，qa 按依赖纪律未开工）；captain 处置：不重复开工，完成报告入账后即核验依赖闭合并停止再派。 |

## 2026-11-09 第五批登记（captain）

| 登记项 | 内容 |
| --- | --- |
| 路由全灭取证 | 四路由逐一定性：magpie/`qwen3.8-flash-next-iq3_xxs` 死（`UNKNOWN_MODEL`，且 magpie provider 本 Session disallowed）；deepseek-official/`deepseek-v4-pro` 死（`Insufficient Balance`/QUOTA，architect-2 连续 3 次唤醒不可恢复，t6/t16/t21 因此 3 任务 FAILED）；deepseek-account/`deepseek-v4-pro` 死（402 `ACCOUNT_QUOTA`，request_id e3c862a7…）；zenmux 限流（`bili_throttle_terminal`，retryable:false，bili 后台 ≤30s 自动探测、上游恢复后自动放行）。ccteam MCP 未配置（agent/status 工具不可用）。存活成员路由仅 `magpie/strata-colab/qwen3.8-27b-ud-q6_k_xl`（captain 本路由，实测可出报告）。 |
| 成员整组更换 | 移除 6 名死路由成员（architect-2/pit-researcher-2/rust-engineer-4/security-reviewer-3/performance-engineer-2/release-reviewer-2）；新增 6 名替补（architect-3/pit-researcher-3/rust-engineer-5/security-reviewer-4/performance-engineer-3/release-reviewer-3），全部挂 magpie/27b 路由；`reasoning_effort` 强制 `default`（27b 不支持 `high`，add_member 报 `UNSUPPORTED_REASONING_EFFORT` 后改）。27b 能力弱，简报必须短（长消息会触发成员上下文死亡，见第四批调度异常）。 |
| t6/t16/t21 自动派发 | requeue 后调度器自动认领 3 个 ready 任务：t6→qa-engineer-5、t16→architect-3、t21→pit-researcher-3（均新会话、无历史上下文）；captain 逐人发短延续简报，并统一交接正本 docs/evidence/BRIEF-PROTOCOL.md（状态 @4db4ba4 + 7 条纪律 + 入账规则）。 |
| t16 重跑独立性 | ADR-0001 修订（6e2a284→4db4ba4）由前代 architect 会话撰写；t16 重跑者 architect-3 为全新会话、未参与撰写，独立性成立。若 t16 重跑 verdict=pass，captain 直接取消 t22（第 3 轮复审，pending 未开始，去重）；若 needs_revision，按 findings 开修复任务后再走 t22。 |
