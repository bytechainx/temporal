# goal/v0.2.0-pit-hardening 审计档案

本目录归档 `goal/v0.2.0-pit-hardening` 分支（2026-11-08 至 2026-11-17）产出的
PIT 加固审计文档。内容为**时点记录**：文中对代码行为的描述以各文档标注的
基线提交为准，不随 main 演进更新；与现行代码冲突时以 `src/` 与 `spec.md`
为准。

| 目录 | 内容 |
| --- | --- |
| `adr/` | ADR-0001 证据信任边界、ADR-0002 链摘要合同、ADR-0003 派生 lineage、ADR-0004 wire 精度 |
| `audit/` | AUD-001..AUD-007 逐条核实报告（AUDIT-VALIDATION） |
| `evidence/` | 基线、执行简报、Owner 裁决登记（DECISIONS-20261108） |
| `reviews/` | Gate B 复审、AUD-001 repro 复核 |

分支上与 main 重复的代码修复已通过独立 PR 落地（如 #12、#14）；
本目录只保留文档资产。
