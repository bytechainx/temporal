# 变更记录

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
