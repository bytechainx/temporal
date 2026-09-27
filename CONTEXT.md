# temporal

时间基础值、时钟与 PIT 纯规则的领域词汇。不是行为合同；目标与行为合同分别见 [goal.md](goal.md) 和 [spec.md](spec.md)。

## 术语

**UnixTimeNs**:
UTC POSIX epoch 纳秒的唯一绝对时刻 primitive。
**避免**：无单位的 Timestamp/i64、把 epoch=0 当未知。

**TimeField**:
一个时间角色上的三态：Known / Unknown / NotApplicable。
**避免**：用 Option、缺字段、null 或默认 now 混同三态。

**event_time**:
源声明的事件发生时刻。
**避免**：把接收时刻、本地捕获时刻或墙钟 now 当事件时刻。

**observation_time**:
事实所描述的观察时点或窗口，不是抓取时刻。
**避免**：把抓取或轮询时刻当观测时刻。

**publication_time**:
源公开该版本的时刻（须有发布证据才可当 Visible 依据）。
**避免**：把接收时刻当官方发布时间。

**received_time**:
系统在明确接收边界捕获输入的时刻。
**避免**：把源事件时刻或重放执行时刻当接收时刻。

**revision_time**:
源声明的修订时刻。
**避免**：把本地重映射或规范化修复时刻当源修订时刻。

**effective_time**:
规则或元数据开始生效的时刻；允许晚于知识截止点。
**避免**：把首次发现日期当生效时刻。

**SourcePublishedAsOf**:
按来源公开知识截止点选版的查询口径。
**避免**：默认 latest 或未声明知识口径的 as-of。

**SystemAsKnown**:
按本系统当时可消费证据选版的查询口径。
**避免**：把来源公开重建宣称为系统真实历史。

**Visible / NotVisible / Indeterminate**:
可见性三态。结构合法不等于 Visible。
**避免**：把可见性缩成布尔值或隐式 latest。

**AbsoluteTimeRange**:
半开绝对窗口 `[start, end)`，端点为 UnixTimeNs。
**避免**：闭区间、把民用期间直接当 UTC 时刻。

**CivilDate**:
民用日期标签，不偷偷升成精确 UTC 时刻。
**避免**：把无时区日期当精确时刻。

**SampledObservation**:
无证明源时刻的采样观察；保留捕获证据，不伪造 event_time。
**避免**：用伪造源时刻创建 EventRecord。

**FactVersionRef**:
一次不可变修订的身份与证据引用；库不解引用载荷。
**避免**：可变覆盖、只有日期而无内容身份的键。

**authority_scope**:
事实版本与修订顺序的权威作用域；与完整 FactKey、RevisionId 共同确定版本身份。同名 FactKey 不跨作用域合并。
**避免**：用 source_scope、consumer_boundary 或全局修订顺序代替权威作用域。

**SystemKnowledgeCut**:
系统视图的边界、epoch、截止点与一致视图绑定。
**避免**：只有墙钟时刻的 cut、未绑定的 now。

**mapping_ref**:
消费方端点映射版本；规范化错误改 mapping，不改来源 revision_id。
**避免**：无痕重算时间单位。

## 邻域

| 邻域 | 关系 |
|------|------|
| `kernel` | 现行仍拥有 `UnixTimeNs`；迁移完成前本叶不替代仓内合同 |
| `market_data` 时间标准 | 同向六角色问题陈述；`MD-PROP-*` 未生效 |
| `decimalx` / `domainx` | 合同仍在 `specs/kernel/` |
| `binancex` 类型层 | 完成定义仍在 `specs/binancex/` |
