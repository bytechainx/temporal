# BRIEF-PROTOCOL — temporal v0.2.0 PIT hardening 成员交接正本

> captain 2026-11-09 交接：全部成员为全新会话，无历史上下文。先读本文件，再按 captain 的逐人消息执行。
> 工作树：/home/workspace/bytechainx/.worktrees/temporal/goal-v0.2.0-pit-hardening（分支 goal/v0.2.0-pit-hardening）。

## 1. 当前状态（captain git 核验 @ 2026-11-09）

- HEAD `4db4ba4`，树干净（开工前自验 `git status --short`；见他人脏树记环境等待，只阻塞受影响验证）。
- 门禁全绿 @4db4ba4：fmt/clippy/doc EXIT 0；test 全绿（acceptance_align 35/35、repro_audit001 4/4、22 suites ok）。
  日志：/home/workspace/bytechainx/.scratch/temporal-gates/gates-4db4ba4.log、gates-4db4ba4-test.log。
- 已落地关键提交（新→旧）：`4db4ba4`→`c95568a`→`5283053`→`6f58422`→`6e2a284`（ADR-0001 修订 ×5，现 211 行）
  → `cec3cc6`（t6 tp_t074/075/076 NotVisible→Indeterminate 改写）→ `e9f39a0`（DECISIONS 第四批）
  → `883c1a9`…`19c1796`（ADR-0004 链）→ `97b8f85`（BRIEF-T5）→ `1cbb5e2`/`94092ef`（WP-01 修复）
  → `ba8975e`（门禁卫生）→ `b71aa17`（基线文档）→ `7cef81d`（origin/main）。
- 任务图：completed t1–t5、t8、t13、t14、t20；进行中 t6、t16、t21；未开始 t7、t9、t10、t11、t12、t15、t17、t18、t19、t22、t23。
- Gate B 现状：t16 第 1 轮 verdict=NEEDS_REVISION，10 findings（F-1..F-10）见 docs/reviews/REVIEW-GATE-B.md；
  修订已落 ADR-0001（见上 5 笔提交）；复审链 = t16 重跑 / t22（第 3 轮，视重跑结果可能取消去重）。

## 2. 纪律（违反即报告不予入账）

1. 不跑 cargo（门禁一律 captain 代跑，CARGO_TARGET_DIR=/home/workspace/bytechainx/.cargo/wt/temporal-goal）；未跑项报告写 `gate_status: NOT_RUN` 并引用 captain 门禁日志，不得写 PASS。
2. 不调用 compress 工具。
3. 单文件读取 ≤400 行（用 offset/limit 分页）。
4. 提交必带 pathspec（`git commit -m "…" <path> [path…]`）；禁 `git add -A` / `git add .`、禁切分支、禁 stash、禁 push/tag/merge。
5. 报告中每个 SHA 必须取自当回合命令输出原文（`git log --oneline` 等）；不得凭记忆或转写。
6. 写盘单次 ≤80 行；接近容量上限立即交报告，余量交后续 attempt。
7. 报告 ≤80 行，结构：task_id / commit（SHA + 路径）/ changed_paths / 要点 / gate_status。

## 3. 入账规则

captain 对每份完成报告先 git 核验（`git cat-file -t <sha>`、`git show --name-only <sha>`、`wc -l`）再入账；
SHA 核验不通过 = 报告不予入账，真实 git 历史是唯一权威。曾有伪造 SHA 事件（18 个假 SHA），勿再犯。

## 4. 范围与冲突

- 只写本任务 in-scope 路径；任务描述与本文件冲突时，以本文件 + 当前 git 状态为准。
- 不触碰兄弟树 /home/workspace/bytechainx/.worktrees/temporal/fix-v0.2.0-pit-consumer-isolation（他人脏树，环境等待）。
