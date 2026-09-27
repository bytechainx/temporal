# 公开 API 清单

跨 crate 稳定入口只认 `temporal` 根路径；`src/` 子模块为私有实现。行为与失败语义以 [spec.md](../spec.md) 为准。本页登记本仓 `0.1.0` 的可编译输入形态。

## 基础值与模型

| 类型或入口 | 用途 |
| --- | --- |
| `UnixTimeNs` | POSIX epoch 纳秒；只允许具名单位构造与检查运算 |
| `WallClock` / `MonotonicClock` / `RuntimeClock` | 明确区分墙钟与进程内单调时间 |
| `CivilDate` / `ObservationPeriod` / `AbsoluteTimeRange` / `EffectiveInterval` | 公历日期、民用期间、绝对半开窗口与业务有效区间 |
| `TimeField<T>` / `TemporalDraft::validate(profile)` / `Temporal` | 显式三态、五种 profile 与不可变六角色视图；校验返回 `Result<Temporal, TemporalError>` |
| `TimePrecision` / `PrecisionLossPolicy` / `TimeStorageCapabilities` | 显式单位、截断或向下取整及存储能力声明 |

`UnixTimeNs` 不实现无单位 `From<i64>`、`Default` 或默认序列化。`TemporalDraft::validate` 只证明角色结构，不证明来源发布或历史消费可见。

## 历史版本与证据

`FactKeyRef`、`RevisionIdRef` 借用规范化字节。`DigestRef` 还带算法版本；`PayloadRef` 借用不可变载荷身份和版本。构造器检查非空和资源界限，不执行来源摘要计算或网络解引用。

`FactVersionParts<'a>` 明确包含：

| 字段 | 合同 |
| --- | --- |
| `identity` / `dataset_snapshot` | `(authority_scope, FactKey, RevisionId, semantic_digest)` 与固定输入快照 |
| `profile` / `source_release` | 已验证时间 profile 与来源发布证据的必需或显式不适用合同 |
| `input_provenance_ref` / `source_temporal` | 原始输入及转换绑定、五个固有时间角色 |
| `operation` / `payload_ref` / `revision_order` | Upsert 必带完整载荷，Withdraw 无载荷；顺序为权威序号或单前驱链 |
| `source_evidence` / `receipts` / `system_evidence` | 来源发布、独立接收和消费者可读事件引用；缺发布证据仍可表示结构合法的版本 |
| `effective_interval` | 与已知 `effective_time` 一致的可选业务有效区间 |

`FactVersionRef::try_new(parts, &limits)` 返回 `Result<FactVersionRef, PitError>`。同版内容不可变；不同接收事件通过证据切片追加，不修改版本身份。证据引用含 schema、提取器、原始证据及验收引用；字符串存在不自动证明外部事实真实。

`TimeBounds::try_new` 接受可缺失上下界、质量及解释引用；`ExactOnly` 只放行上下界相同的精确证据。`DatasetSnapshotRef` 固定研究输入；`SystemKnowledgeCutRef` 还固定边界、代次、分区前沿、`known_at` 和一致性证明。

## 查询输入与结果

`ResourceLimits::try_new` 要求调用方显式给出九项正值：身份字节、单版证据数、分区数、候选总数、事实总数、每事实版本数、链深、诊断条目数和工作内存字节。无生产默认值。

`QueryPolicy::try_new` 固定 profile、`EvidenceAcceptance`、`RevisionOrderPolicy`、`OutputScope`、策略引用和资源上限。`AsOfContextParts<'a>` 明确包含 `known_at`、`knowledge_basis`、`authority_scope`、`source_scope`、`consumer_boundary`、`dataset_snapshot`、`system_cut`、`policy`、三种 schema 引用和解释合同引用；`AsOfContext::try_new(parts)` 返回 `Result<AsOfContext, PitError>`。

`CandidateCoverageRef` 对 authority、完整事实键集合、snapshot、知识口径和截止点绑定受信适配器的枚举声明。`Complete` 标志及验收引用是外部合同输入，库可检查关联和一致性，不能独自证明数据库真的没有遗漏。

| 入口 | 确定性结果 |
| --- | --- |
| `evaluate_visibility(version, context)` | 单版本 `Visible` / `NotVisible` / `Indeterminate`；不宣称该事实的最新修订 |
| `select_fact_as_of(candidates, coverage, context)` | 单事实 `Selected` / `Withdrawn` / `NoVisibleVersion`，不确定性经 `PitError` 返回 |
| `select_facts_as_of(input, context)` | 同一 `authority_scope` 的完整事实集合，按完整键字节稳定排序 |
| `inspect_candidates(input, context)` | 独立的 `AdministrativeDiagnostics`；显式请求、有界，可能含未来版本身份，只供行政审计 |
| `select_effective_state_as_of(input, context, valid_at, state_policy)` | 先完成严格 PIT，再计算业务有效状态；不自动化解重叠冲突 |

默认事实结果仅返回已选版本及本次证明可见的引用，不携带未来版本数量、身份或时间。消费者需将固定的快照、cut、策略、schema、解释版本、实现 commit 与证据引用写入自己的复现清单。
