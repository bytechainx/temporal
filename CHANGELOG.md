# 变更记录

## [Unreleased]

`v0.1.0` 之后合入 `main` 的变更（截至 `1186bdc`），尚未打 tag：

### 新增

- `select_facts_as_of_partial`：批量宽松入口，事实级失败进 `failures` 显式枚举（按键规范序），查询级错误仍整体 `Err`；`failures` 非空即结果不完整。对应 spec.md §14.5、§33.7 TP-T181 与 `docs/API.md`。
- GitLab CI（特性 034 中央模板 `governance-v1.0.1`）。

### 变更

- PIT 可见性修正：cut 内事件已证未来与分区未被 cut 声明的事件同时存在时，由 `NotVisible(SystemAvailableAfterCutoff)` 改为 `Indeterminate(TargetCoverageUnproven)`；无未覆盖事件时保持原 `NotVisible`。
- profile 角色组合校验收敛为单一来源（行为不变）。

### 说明

- 验收口径同步：独立验收分母 143（temporal_only 128 + mixed 15，含 TP-T181）；38 个纯消费方场景 Out of Scope 另立 Goal（2026-11-16 Owner 裁定，含 TP-T181 后增）。
- `spec.md` / `goal.md` 文档版本升至 0.1.1：并入工作区叶 v0.1.1 的验收口径（§22/§32.1/§33.1 分母与消费方证据排除、§24/§9/§25.1/G-014 迁移期条款移除），保留本仓 TP-T181 与 `select_facts_as_of_partial` 合同。
- `checklist.md` 场景状态同步至上述口径：143 PASS / 38 NOT_RUN（Out of Scope）。
- 性能：`spec_matrix` 于受控机器 `xhyperium`（rustc 1.99.0）复跑，最大 cell `select_ordered_100000` median 430,704,828ns，较 D6 基线（567,752,462ns）改善约 24%，未触发 10% 回归门禁。

## [0.1.0] - 2026-11-14

首个正式 tag：`v0.1.0`。发布范围为 temporal core 与工程门禁（Owner 裁定 D1=A）。U3 的 59 项未纳入本次发布范围。回滚基线 `7cef81d`（pre-release fallback；此前无正式发布）。

### 新增

- 独立时间值对象、六角色校验、精度转换及基于证据的历史版本选择合同。
- `DecisionReason::TargetCoverageUnproven`（稳定码 `TARGET_COVERAGE_UNPROVEN`）。`DecisionReason` 不是 `#[non_exhaustive]`，本项属于穷尽匹配可见的公共 API 增加。

### 变更

- 事件不在 cut 声明的分区覆盖内，或消费者边界、代次不符时，返回 `Indeterminate(TargetCoverageUnproven)`，不再返回 `NotVisible(OutsideSystemCut)`。

### 说明

- 本仓不发布到 crates.io；生产集成、真实来源与消费者验收按各消费方合同独立完成。
- 候选实现提交 `0005cc34ab5a6c15f4e59ac5a95ce2fa7a7240f0`；tag 指向本文件的合入提交。
