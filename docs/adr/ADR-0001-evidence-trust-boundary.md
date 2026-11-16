# ADR-0001：Evidence 信任边界（XH-TEMPORAL-REMEDIATION-002 / WP-02 / AUD-002）

**状态**：Proposed / Revision R2 In Progress（ADR 侧 R2 修订已提交，待 Gate B 第 3 轮复审；不构成 Accepted，不产生合入权）
**日期**：2026-11-08
**仓库与基线**：`bytechainx/temporal`，`goal/v0.2.0-pit-hardening`，基线 SHA `7cef81d`
**关联**：`.omc/drafts/1.md` AUD-002 / WP-02；`spec.md` §10.1、§12.3、§16.2、§16.3、§20.2、§20.3、§21.4、§25.2、§25.5、§32.3、§33.1；`goal.md` G-007、G-009、G-010、G-015；`docs/reviews/REVIEW-GATE-B.md` §9、§10；`docs/adr/ADR-0002-chain-digest-contract.md`（chain digest 承诺对象）、`docs/adr/ADR-0003-derived-lineage.md` §3、§4（派生 lineage 与时间下界）、`docs/adr/ADR-0004-wire-precision.md` §3、§5（时间精度合同条款与消费者验收条款）
**不在本 ADR 范围**：chain digest 承诺对象（WP-03 / ADR-0002）、Derived lineage（WP-06）、Wire precision（WP-07）。Cut 侧四个引用的比较规则不在本 ADR 实施：本 ADR 只给出责任划分（§3.1、§4 第 11 条），实施与证据落 `AUD-005` 要求 4（审计 R5；WP 标签冲突见 §9），判定待 Owner 追认（§6 R2-4）。

## 1. 背景

`EvidenceProofRef::try_new` 只检查五个字段非空且不超过 `ResourceLimits` 字节上限（`src/evidence.rs:116-131`，`label()` 见 `src/evidence.rs:8-16`）。它不计算摘要、不解析 `raw_evidence_ref`、不验证签名，也不查询任何验收记录。rustdoc 与 getter 文档中若干「已证明」「受信」「证明」措辞会让调用方把构造成功读成证据可信。`spec.md` §25.5 明确禁止「声称证据结构确保来源真实」。

本 ADR 固定四层职责、写入者、以及哪些措辞必须改写。

## 2. 四层职责

| 层 | 英文术语 | 负责方 | 输入 | 输出 | 不得声称 |
| --- | --- | --- | --- | --- | --- |
| 1 | Structural Validation | `temporal` | 调用方传入的 `*Ref` 值 | `Result<_, PitError>`；规则一致性与选择结果 | 输入内容真实、可信、已验收 |
| 2 | Integrity Verification | Trusted Evidence Adapter | 原始证据字节、载荷、manifest | 摘要比对结论（算法版本 + 摘要字节） | —— |
| 3 | Authenticity Verification | Trusted Evidence Adapter | 来源链、签名、密钥、发布渠道 | 来源与签名有效性结论 | —— |
| 4 | Acceptance | Trusted Evidence Adapter（消费方持有） | 2、3 的结论 + schema/extractor 判定 | 版本化验收记录，供 `acceptance_ref` 指向 | —— |

`temporal` 只拥有第 1 层：结构校验、引用绑定、时间边界、选择规则（`spec.md` §10.1、§13、§14）。第 2–4 层在第 1 层之前由消费侧适配器完成；`temporal` 不实现、不代理、不背书。

## 3. `temporal` 侧合同（基线 `7cef81d` 事实）

`temporal` 实际执行的检查：

| 检查 | 位置 |
| --- | --- |
| 标识与引用非空、字节上限 | `src/evidence.rs:8-16`；`src/revision.rs:12-25`；`src/pit/context.rs` `label` |
| `TimeBounds` 区间顺序与 `Exact` 上下界相等 | `src/evidence.rs:49-69` |
| 证据必须绑定同一 `FactVersionIdRef` | `src/revision.rs:314-338` |
| 同接收事件、同 `(consumer_boundary, log_epoch, event_id)` 与同 `(partition, cursor)` 的重复且内容不一致 → `EvidenceMismatch` | `src/revision.rs:324-350`；`src/pit/selection.rs:600-654` |
| 证据属于本次 cut 的边界、代次、分区前沿与 `known_at` | `src/evidence.rs:522-540`；`src/pit/selection.rs:577-587` |
| `proof` 各字段仅做长度检查 | `src/pit/selection.rs:408-426` |
| 覆盖声明与本次查询重新绑定 | `src/pit/context.rs` `validate_for`；`src/pit/selection.rs:127` |

`temporal` 不执行、且在 v0.2.0 之前不得被文档暗示其执行：

- 摘要计算或比对；`raw_evidence_ref`、`manifest_ref`、`payload_ref` 的解引用（`spec.md` §20.3）。
- 签名、密钥、来源渠道验证。
- 验收记录的存续、有效性与可复核性。
- 「该适配器确实受信」的判定：`coverage_adapter_ref` 只是调用方提供的字符串引用（`src/pit/selection.rs:82-83、260-262`）。

### 3.1 Cut 侧引用的合同边界（Gate B 第 2 轮 F-2，high）

`SystemKnowledgeCutRef` 另带四个引用：`manifest_ref`、`evidence_snapshot_ref`、`known_at_binding_ref`、`consistency_policy_ref`。语义已在 `spec.md`:588-592 定义，基线与当前实现的职责如下：

| 事实 | 位置 |
| --- | --- |
| 四个引用只有 getter，`temporal` 不解引用 | `src/evidence.rs:506`（`known_at_binding_ref`）、`:514`（`consistency_policy_ref`）、`:518`（`evidence_snapshot_ref`）及 `manifest_ref` getter |
| 四个引用只做标签与长度检查 | `src/pit/context.rs:287-290`；`src/pit/selection.rs:307-310` |
| `contains` 只比较 `boundary_id`、`known_at`、`consumer_boundary`、`log_epoch`、`partition`、`log_epoch` 与 `cursor <= inclusive_frontier`；边界不匹配返回 `UnprovenSystemCut` | `src/evidence.rs`:522-540 |
| 上述四个引用从未与实际消费的证据集合比较 | 全仓无比较点：`contains`（`src/evidence.rs:522-540`）与 `src/pit/selection.rs` 的绑定校验都不读取这四个引用；`docs/audit/AUDIT-VALIDATION.md:226` 独立判定「实现把这些字段当作不透明标签」 |

裁定（建议登记为 R2-4，待 Owner 追认）：v0.2.0 不在 `temporal` 内增加这四个引用与证据集合的比较。增加比较会新增拒绝路径，属行为变更，须按 `spec.md`:1426 走兼容性评审，与 `DECISIONS-20261108.md` R4-2（v0.2.0 不引入库内 chain manifest 引用解引用）同族。职责划分：

- `temporal`（第 1 层）：形状与绑定校验；覆盖未证明时不得给出确定可见性判定，已由 WP-01 落地（`VisibilityDecision::Indeterminate` + `DecisionReason::TargetCoverageUnproven`）。
- Trusted Evidence Adapter（第 2–4 层）：定义并执行「manifest ↔ 证据快照 ↔ `known_at` 绑定 ↔ 一致性策略」的一致性；不一致必须拒绝接收并留存证据与拒绝原因（§4 第 10 项）。

未决项（交 Owner，不阻塞本修订）：`DECISIONS-20261108.md` 把 R5 并入 R2，而 R2 的裁定只覆盖 `CandidateCoverageRef` 的边界/代次覆盖与三处旧断言改写；这四个引用的比较规则仍未裁定。
- schema/extractor 一致性判定：基线代码不把 `AsOfContext::evidence_schema_ref`（`src/pit/context.rs:228、343`）与每条 `proof.schema_version()`（`src/pit/selection.rs:416`）作等值比较，也不把 `interpretation_ref` 与 `extractor_version()` 作比较。该一致性是第 4 层义务，见 §4。

## 4. Trusted Evidence Adapter 合同（第 2–4 层）

`Trusted Evidence Adapter` 是消费方持有并版本化的接收/映射/存储组件，位于 `spec.md` §32.3 推荐调用边界内 `temporal` 纯函数调用之前。它是唯一有权把外部数据写成 `*EvidenceRef` 与 `CandidateCoverageRef` 的角色。

1. Integrity Verification：适配器必须解析 `raw_evidence_ref`、`manifest_ref`、`payload_ref` 指向的内容，按记录的摘要算法与编码版本重新计算并比对摘要。不一致即拒绝，不得写入 `temporal` 输入。
2. Authenticity Verification：适配器必须验证来源渠道与签名或等效来源证明，并留存验证结论与使用的密钥或渠道标识版本。`temporal` 不实现签名验证，也不引入相关依赖。
3. Acceptance：适配器必须为每条被接受的证据产生一条版本化验收记录，`acceptance_ref` 指向该记录。记录至少包含：`schema_version`、`extractor_version`、被验收的 `evidence_id`、摘要与算法版本、来源验证结论、验收时间、执行该验收的组件身份与版本、拒绝时给出的原因。
4. 版本化 identity：`schema_version` 标识证据结构的版本化 schema；`extractor_version` 标识生成该证据的 decoder / normalizer / mapping 版本；`acceptance_ref` 标识验收记录。三者与 `evidence_id`、`raw_evidence_ref` 一样是不透明引用，`temporal` 只比较其字节一致性，不解释其含义。
5. 一致性判定归属：同一查询内所有证据的 `schema_version` 必须等于 `AsOfContext::evidence_schema_ref`，所有证据的 `extractor_version` 必须落在 `interpretation_ref` 声明的解释版本内。该判定由第 4 层执行。是否在 `temporal` 侧增加等值校验属于 §6 待裁定项，不得在未评审前实现。
6. Coverage：`CandidateCoverage::Complete` 只能由适配器声明（`spec.md` §12.3）。适配器必须留存 `coverage_manifest_ref` 指向的枚举清单与 `coverage_acceptance_ref` 指向的验收记录。`temporal` 只重新绑定这些引用（`src/pit/context.rs` `validate_for`），不验证枚举是否真的完整。
7. 复现义务：消费者复现清单（`spec.md` §16.3）必须写入本次使用的 `schema_version`、`extractor_version`、`acceptance_ref`、`coverage_*_ref` 与适配器版本。缺任一项，该次查询不可复现。identity 的库侧口径（Gate B 第 2 轮 F-9）：`temporal` 只保留并透出这三条字段，可经 `FactOutcome::Selected { evidence }`（`src/pit/selection.rs:39`、`:470`）读出，不解释其值，也不写入新的 `PitManifestRef` 字段承载它们；把 identity 交给消费者从结果对象读取属本合同，不是库的复现清单写入者。
8. 授权边界独立于本 ADR：`source_scope` 不是授权票据（`spec.md` §21.4）。适配器与消费者必须在读取前完成授权与许可判定。
9. 多 Owner 不得互签：`spec.md` §33.1 要求 Adapter / Product / Runtime / Platform / Governance 分别出具各自证据。`temporal` 侧的结构测试通过不得替代适配器的验收证据。
10. 验收可追溯性（Gate B 第 2 轮 F-4）：适配器必须为每条验收记录留存执行时刻与被验收的日志或 manifest 状态引用，且该时刻可相对该次查询的 `known_at` 复核。`SystemAsKnown` 模式下不得为截止点后新增的证据虚构历史 receipt 或 cut，只能重建已有受信日志证明的历史事实（`spec.md`:606）。不可追溯的验收记录不得写入 `acceptance_ref`。派生数据的同类可追溯义务见 `docs/adr/ADR-0003-derived-lineage.md` §3、§4。
11. Cut 侧引用一致性（Gate B 第 2 轮 F-2）：适配器必须验证 `manifest_ref` 指向的 manifest 与 `evidence_snapshot_ref` 指向的证据集合一致，`known_at_binding_ref` 与 `consistency_policy_ref` 与实际消费的证据集合及其 `known_at` 匹配。不一致必须拒绝并留存证据。`temporal` 只提供这四个引用的形状与标签校验（§3.1），不代为判定。

## 5. 措辞清单与替换方案

规则：API 文档不得把 `try_new` 描述为真实性证明。构造成功只表示字段非空、长度受限、绑定一致。

禁用措辞（rustdoc 与 `docs/API.md` 不再使用，除非紧邻一句限定说明）：`已证明`、`证明`、`受信`、`可信`、`已验证`（指外部内容时）、`已验收`（指外部验收结论时，Gate B 第 2 轮 F-8）、`确保真实`、`verify`、`prove`、`guarantee`、`trusted`（指调用方输入时）。

计数口径（可复现，Gate B 第 2 轮 F-10）：本节主表 20 行，逐位置清点为 28 处（8 行各含 2 处以上位置）。全仓复扫口径与逐条处置见 §5.1。不使用「约 27 处」一类不可复现的计数。

| 位置 | 现行措辞 | 问题 | 替换 |
| --- | --- | --- | --- |
| `src/evidence.rs:115`（`EvidenceProofRef::try_new`） | 登记受信入口验收结果的稳定引用 | 「登记」暗示本库执行验收 | 构造一个指向外部验收记录的结构引用；不登记、不验收、不核验被指向的记录 |
| `src/evidence.rs:21-22` | 已证明为同一精确时刻 | 精确性由外部验收决定 | 调用方声明为同一精确时刻 |
| `src/evidence.rs:30-35` | 已证明不晚于截止点可用 / 已证明晚于截止点才可能可用 | 把规则判定说成事实证明 | 按已接受输入判定为不晚于截止点 / 判定为晚于截止点 |
| `src/evidence.rs:188` | 已证明的实际发布界限 | 同上；替换文本若写「已验收」会与本 ADR §2 第 1 层「不得声称」列和 §5 禁用措辞列冲突（Gate B F-8） | 调用方声明的实际发布界限；本库不核验该声明，也不声称它已验收 |
| `src/evidence.rs:192`、`:342` | 受信入口验收记录 | 「受信」暗示库背书 | 外部验收记录引用；本库不验证该记录 |
| `src/evidence.rs:420` | 历史决策边界固定的可读事件集合证明 | 切片是声明，不是证明 | 历史决策边界固定的可读事件集合声明 |
| `src/evidence.rs:435` | 检查切片各作用域、证明引用、分区唯一性及资源界限 | 「证明引用」读作已核验 | …引用字段、分区唯一性及资源界限 |
| `src/evidence.rs:505` | 截止点绑定证明引用 | 同上 | 截止点绑定的外部证明引用 |
| `src/revision.rs:131` | 必须提供与当前候选绑定的来源发布证明 | 库无法判断该证明是否成立 | …来源发布证据引用 |
| `src/revision.rs:150`、`:154` | 摘要固定权威链关系 / 固定完整权威链关系的摘要 | 暗示摘要内容被验证 | 调用方声明的链关系摘要；本库只比较字节，不验证摘要内容（语义边界见 ADR-0002） |
| `src/revision.rs:222` | 已验证时间资料的 profile | 「已验证」指外部真实时误导 | 已按该 profile 校验时间角色的 profile |
| `src/pit/selection.rs:16` | 选中修订的决策安全证据引用 | 「安全」易读作可信 | 仅含决策安全字段的证据引用 |
| `src/pit/selection.rs:34` | 已验证为新增或更新 | 操作类型来自调用方声明 | 调用方声明的操作类型 |
| `src/pit/selection.rs:38`、`:45` | 本次证明可见的证据 / 本次证明撤回可见的证据 | 同上 | 本次选择所引用的可见性证据引用 |
| `src/pit/selection.rs:57`、`:62` | 完整候选与覆盖证明 / 受信适配器的完整覆盖声明 | 完整性不是库的结论 | 完整候选与覆盖声明（由适配器声明） |
| `src/pit/selection.rs:71` | 决策安全的复现上下文与输入覆盖证明 | 同上 | …输入覆盖声明引用 |
| `src/pit/selection.rs:82` | 受信适配器身份和版本引用 | 库不判定谁受信 | 适配器身份与版本引用；本库不判定其是否受信 |
| `src/pit/context.rs:121` | 已证明可用时间的接受方式 | 同上 | 时间界限接受方式 |
| `src/pit/context.rs:355`、`:445` | 受信适配器声明并留存证明的完整覆盖 / 候选枚举证明清单 | 留存责任被误读为库的责任 | 适配器声明的完整覆盖；候选枚举清单引用 |
| `src/pit/visibility.rs:50`、`:52`、`:54` | 所有必要门禁均得到证明 / 已证明该版本在本次查询不可见 / 尚不能证明可见或排除 | 规则判定写成事实证明 | 全部规则门禁已被输入满足 / 按已接受输入判定为不可见 / 输入不足以判定可见或排除 |

保留为正面样例（措辞已正确，不改）：`src/evidence.rs:48`、`src/evidence.rs:104`、`src/evidence.rs:115` 后半句「任意字符串不自动构成可信证明」、`src/revision.rs:59`、`src/revision.rs:89`、`src/revision.rs:248`、`docs/API.md` 的四条现有免责声明。

本表由 WP-02 的代码 PR（rustdoc 改写，零语义变更）实施；改写后须按 `spec.md` §25.3 做兼容性审阅：rustdoc 文本变化不改变行为，不触发版本变更，但公开 API 名称或校验行为变化必须单独评审。

### 5.1 全仓复扫口径与残留项处置（Gate B 第 2 轮 F-3）

主表是首次起草时的清单，不是判定规则。落点改为可复现的扫描规则，命令与口径写入验收证据要求（§7）。

复现命令（仓库根执行；退出码 `0` 有命中、`1` 无命中、`2` 工具错误）：

```bash
grep -rn --include='*.rs' -E '^[[:space:]]*///.*(证明|受信|可信|已验证|确保真实|已验收)' src/
grep -n -E '证明|受信|可信|已验证|确保真实|已验收' docs/API.md
```

处置规则：命中不等于违规。(A) 该句已限定「本库不核验 / 由外部承载 / 否定式」→ 保留，见 §5.2。(B) 该句把校验、验收或真实性归给 `temporal`，或未加限定 → 改写。

HEAD `6e2a284` 的命中数：`src/` 23 行（10 保留 + 13 改写），`docs/API.md` 6 行（2 保留 + 4 改写）。口径与审查记录的换算：审查 `docs/reviews/REVIEW-GATE-B.md` §9 按「主表 28 处 / 已落地 25 处」计数，那是主表的落地进度口径；本节的 23 / 6 是按当前 HEAD 全仓扫描的命中口径。两者不冲突：主表已落地的位置在 HEAD 不再命中，未列进主表的历史遗漏在本节补齐。下游核对一律以本节两条命令的输出为准。下表是 HEAD 尚未处置的 17 处残留，逐条给出处置与替换。全部为 rustdoc 或文档文本，公开标识符与行为不变。

| 残留位置 | 现行措辞 | 处置 |
| --- | --- | --- |
| `src/model.rs:30` | 缺少可证明源时刻的本地采样观察 | 改写：缺少可用来源时刻声明的本地采样观察 |
| `src/revision.rs:128` | 来源发布证明的明确适用规则 | 改写：来源发布证据引用的明确适用规则 |
| `src/revision.rs:133` | 产品按版本化权威合同证明独立发布阶段不适用 | 改写：产品按版本化权威合同声明独立发布阶段不适用；依据由消费者留存 |
| `src/revision.rs:224` | 当前候选的来源发布证明适用规则 | 改写：当前候选的来源发布证据引用适用规则 |
| `src/revision.rs:366` | 独立来源发布证明的适用规则 | 改写：独立来源发布证据引用的适用规则 |
| `src/pit/visibility.rs:19`、`:21`、`:25`、`:27` | 来源时间缺乏证明 / 系统时间缺乏证明 / 接收链缺乏证明 / 必需来源释放条件缺乏证明 | 改写：来源时间证据不足，无法判定 / 系统时间证据不足，无法判定 / 接收链证据不足，无法判定 / 必需来源释放条件证据不足，无法判定（`DecisionReason` 变体名不变） |
| `src/evidence.rs:38` | 来源证明的最早与最晚可用时间 | 改写：来源证据声明的最早与最晚可用时间 |
| `src/evidence.rs:149` | 外部受信入口的验收记录引用 | 改写：外部验收记录引用；本库不核验该记录 |
| `src/evidence.rs:188` | 适配器声明并已验收的实际发布界限 | 改写（F-8 残留）：调用方声明的实际发布界限；其验收由外部记录承载，本库不核验。HEAD 仍是违规文本，须由 WP-02 改写任务按修订后的 §5 主表 `src/evidence.rs:188` 行同步 |
| `src/error.rs:142` | 系统历史切片缺乏证明 | 改写：系统历史切片的绑定证据不足，无法判定 |
| `docs/API.md:26` | 已验证时间 profile 与来源发布证据的必需或显式不适用合同 | 改写：已按该 profile 校验时间角色的 profile，以及来源发布证据的必需或显式不适用合同 |
| `docs/API.md:34` | …和一致性证明 | 改写：…和一致性策略引用 |
| `docs/API.md:42` | …绑定受信适配器的枚举声明 | 改写：…绑定适配器的枚举声明（同行后半句「不能独自证明数据库真的没有遗漏」保留） |
| `docs/API.md:52` | 本次证明可见的引用 | 改写：本次选择所引用的可见性证据引用 |

### 5.2 保留判断（不改写项与理由）

`src/model.rs:168`、`src/precision.rs:41`、`src/revision.rs:59`、`src/revision.rs:248`、`src/pit/selection.rs:82`、`src/pit/context.rs:449`、`src/evidence.rs:48`、`src/evidence.rs:104`、`src/evidence.rs:115` 后半句、`src/evidence.rs:198`、`docs/API.md:15`、`docs/API.md:32`。

理由：这些命中里的禁用语处于否定式或限定式——「不证明外部来源真实性」「声明本身不是读写证明」「摘要内容由外部受信入口核对」「本库不判定其是否受信」「不自动证明…」。「受信」用于指外部入口而非调用方输入，符合 §5 的括号限定。改写它们会删掉限定语，反而弱化边界。

## 6. Owner 裁定登记（已落定）与遗留待裁定项（Gate B 第 2 轮 F-1）

登记入口只有一个：`docs/evidence/DECISIONS-20261108.md`。本节只登记该文件已有行与本 ADR 的对应关系，不另立正本。

| 编号 | 事项 | 已落定内容 | 登记位置 |
| --- | --- | --- | --- |
| R1 | Indeterminate 原因枚举扩充 | 新增 `TARGET_COVERAGE_UNPROVEN`（`DecisionReason` 新变体 + `code()`），不损失可解释性。代码落 t5；`spec.md` §22、`docs/标准.md`、`docs/API.md` 正本同步落 t15 | `docs/evidence/DECISIONS-20261108.md`:9 |
| R2 | 目标消费者证据覆盖完整性落点 | 扩展既有 `CandidateCoverageRef` 的 boundary/epoch 证据覆盖校验，不新增独立类型；先改 `spec.md` §12.3/§13.3 正本（t15）；`tests/acceptance_align.rs:692-695`、`:730-733`、`:766-769` 三处 `NotVisible` 断言允许按新语义改写为 `Indeterminate`（测试落 t6） | `docs/evidence/DECISIONS-20261108.md`:10 |
| D-1 | `temporal` 是否在 v0.2.0 比较 `AsOfContext::evidence_schema_ref` 与各 `proof.schema_version()`、`interpretation_ref` 与各 `extractor_version()` | 不在 v0.2.0 实现。该比较新增拒绝路径，属行为变更，须先改 `spec.md` 正本并按 `spec.md:1426` 做兼容性评审。v0.2.0 由第 4 层合同（§4 第 5 条）覆盖；库侧只保留「identity 字段被保留且可经 `FactOutcome::Selected { evidence }` 读出」的断言 | `docs/evidence/DECISIONS-20261108.md`:26（R3）；本 ADR §4 第 5 条 |
| D-2 | `EvidenceProofRef` 是否更名 | v0.2.0 保留名称，只做零语义 rustdoc 改写；更名列为 v0.3 候选（breaking，须单独评审） | `docs/evidence/DECISIONS-20261108.md`:26（R3，落 t20）；`spec.md:1426` |
| D-3 | `docs/API.md` 是否增加四层表 | 采纳短指针，不复制正文；落 t15 | `docs/evidence/DECISIONS-20261108.md`:74（R7-3 同类口径），待 Owner 对 D-3 追认 |
| T-1 | TP-T070 两半归属 | 半边已实现：明确错配 → `EvidenceMismatch`（`src/revision.rs:314-338`、`src/pit/selection.rs:600-654`；断言见 `tests/evidence_contract.rs:49`、`:59`、`tests/pit_contract.rs:652`、`:842`、`:869`、`tests/acceptance_align.rs:584`）。半边未实现：缺绑定 → `Indeterminate`；`spec.md:1229` 已规定该期望，现行代码未实现，`docs/audit/AUDIT-VALIDATION.md:119` 记为缺陷 | `spec.md:1229`；`docs/audit/AUDIT-VALIDATION.md:298`（R3）；落点待 Owner 指定 |
| T-2 | 两个 R3 编号指不同事项 | `docs/audit/AUDIT-VALIDATION.md:298` 的 R3 = schema/快照版本绑定校验强度与错误码；`docs/evidence/DECISIONS-20261108.md`:26 的 R3 = AUD-002 零语义措辞改写落 t20。本 ADR 采后者的措辞口径，同时把前者的校验强度问题保留为待办 | 两处编号需 Owner 去重 |
| R2-4（新增，待追认） | `manifest_ref` / `evidence_snapshot_ref` / `known_at_binding_ref` / `consistency_policy_ref` 是否与实际消费的证据集合比较 | v0.2.0 不在库内比较，理由与责任划分见 §3.1 | `docs/evidence/DECISIONS-20261108.md`:12 宣布 R5 与 R2 同一裁决，但 R2 未给出该比较规则 → 待 Owner 追认 |

## 7. AUD-002 验收映射与 gate 状态

状态词汇只允许 `PASS / FAIL / NOT_RUN / BLOCKED / NOT_APPLICABLE`（`spec.md` §25.2）。本 ADR 是草案，全部实施状态初始为 `NOT_RUN`；本轮未运行任何 cargo 命令。

| 验收项 | 归属层 | Owner | 落点 | gate_status | 可登记的证据 |
| --- | --- | --- | --- | --- | --- |
| A：引用形状不声称真实性（构造成功只表示非空、长度受限、绑定一致） | 1 | rust-engineer / qa | `tests/evidence_contract.rs:49`、`:59` 既有断言 + §5 / §5.1 措辞改写 | NOT_RUN | `cargo test --test evidence_contract` 通过，且 §5.1 两条 grep 命令的残留改写项全部处置完毕 |
| B：证据与目标版本绑定不一致必须被拒 | 1 | rust-engineer / qa | `src/revision.rs:314-338`、`src/pit/selection.rs:600-654`；断言 `tests/pit_contract.rs:652`、`:842`、`:869`、`tests/acceptance_align.rs:584` | NOT_RUN | `cargo test --test revision_contract` 与 `cargo test --test pit_contract` 通过。注意：`spec.md:1229`（TP-T070）的「缺绑定 → `Indeterminate`」半边未实现（§6 T-1），该项在闭合前不得记 `PASS` |
| C：版本化 schema / extractor / acceptance identity 保留且可读出 | 1（判定在 4） | rust-engineer（保留字段）+ Adapter Owner（判定） | 新增断言：`proof.schema_version()`、`proof.extractor_version()`、`proof.acceptance_ref()` 可经 `SelectedEvidenceRef` 读出并随复现引用保留 | BLOCKED | 现无证据可登记：全 `tests/` 对 `schema_version` / `extractor_version` 命中数为 0，夹具恒为 `"schema1"` / `"extractor1"`（`tests/pit_contract.rs:106-107`、`tests/acceptance_align.rs:174-175`）。`cargo test --test pit_contract` 即使全绿也不产出本项证据（Gate B F-9） |
| D：消费者集成测试证明非法证据不能进入受信适配器输出 | 2 / 3 / 4 | Adapter / Product / Runtime Owner（`spec.md` §33.1） | 消费者仓集成夹具 | BLOCKED | 本机无消费者仓与适配器实现；需消费者仓检出后由 Owner 执行 |
| E：Cut 侧引用一致性（`manifest_ref` ↔ 证据快照 ↔ `known_at` 绑定 ↔ 一致性策略） | 2 / 3 / 4 | Adapter Owner；`temporal` 侧无判据 | §3.1 责任划分 + 适配器拒绝用例 | BLOCKED | 需 Owner 追认 §6 R2-4，并由适配器侧出具证据。属 `AUD-005` 要求 4 |
| F：验收证据相对 `known_at` 可追溯 | 4 | Adapter Owner | §4 第 10 条 + 适配器追溯用例 | BLOCKED | 库内无该判据；只能由适配器出具 |
| G：适配器合同可被消费者独立验证 | 4 | Owner + Adapter / Product / Runtime Owner | 规范正本落 `spec.md` §20.3 与 §12.3 的对应条目（t15）；本 ADR 只是设计记录 | BLOCKED | `spec.md:530` 明确 `temporal` 不实现也不验证受信证据适配器，因此本 ADR 不能充当规范正本（Gate B F-5）。正本条目出具前，该项无 Owner 可登记证据 |

D / E / F / G 不得记为 `PASS`，也不得由 `temporal` 侧的结构测试通过推断。`NOT_APPLICABLE` 不适用：这些项真实存在，只是本机环境或合同归属使本地无法执行。

验收证据要求（Gate B F-3）：A 项的措辞维度必须附 §5.1 两条 grep 命令在提交 HEAD 上的输出；命中若按 §5.2 保留，须逐条给出保留理由。只有命令输出为空、或全部命中都有保留理由，才允许把 A 项措辞维度登记为 `PASS`。

## 8. 后果与非目标

正面：职责与写入者明确；措辞与 `spec.md` §25.5 禁止的完成声明一致；WP-02 的文档改写与可能的行为变更分离，评审边界清楚。

负面：措辞改写的评审面变大——§5 主表逐位置 28 处，另有 §5.1 在 HEAD 复扫出的 17 处残留（13 处 `src/` + 4 处 `docs/API.md`）；schema/extractor 一致性按 §6 D-1 不在库内实施，只靠第 4 层外部合同，`temporal` 侧无法拦截不一致输入。

非目标与硬约束：`temporal` 不实现签名或摘要计算、不解引用外部引用、不存储验收记录、不判定适配器是否受信、不发放授权；不新增依赖（`.omc/drafts/1.md` §1.1 禁止 `serde`、`protobuf`、`tokio` 等）；本 ADR 不定义 chain digest 承诺对象（ADR-0002）、不定义 Derived lineage（WP-06）、不定义 Wire precision（WP-07）。

## 9. 修订循环与纠正登记（Gate B 第 2 轮 F-7 / F-8 / F-9 / F-10）

| 轮次 | Finding | 本 ADR 侧关闭方式与提交 | 仍未关闭 |
| --- | --- | --- | --- |
| attempt 1 | F-1（medium） | §6 改为 Owner 裁定登记，并给出 TP-T070 两半拆分口径与两个 R3 编号冲突 | TP-T070 缺绑定半边的实施落点待 Owner 指定 |
| attempt 1 | F-2（high） | §3.1 事实行 + 责任划分 + §4 第 11 条（`6e2a284`） | R2-4 待 Owner 追认；WP 标签冲突待 Owner 纠正（`docs/audit/AUDIT-VALIDATION.md:22`、`:300` 写作 WP-05，`.omc/drafts/1.md` §5 的 WP-04 = Cut 完整性） |
| attempt 1 | F-3（medium） | §5.1 可复现扫描口径 + 17 处残留逐条处置 + §5.2 保留判断 + §7 证据要求（`6f58422`） | 实施侧 17 处残留须由 WP-02 改写任务落地 |
| attempt 1 | F-4（medium） | §4 第 10 条验收可追溯性（`6e2a284`） | 适配器侧证据 |
| attempt 1 | F-5（medium） | §7 G 行写明规范正本是 `spec.md` §20.3 与 §12.3（落 t15），本 ADR 只是设计记录 | Owner 追认与正本条目出具 |
| attempt 1 | F-6（low） | §7 A / B 行写明既有断言位置与缺口 | — |
| attempt 1 | F-7（medium） | §5.1 给出逐位置残留清单，替代「按主表实施」 | 实施侧 `src/pit/visibility.rs` 4 处与 `docs/API.md` 4 处等残留 |
| attempt 2 | F-8（high） | §5 主表 `src/evidence.rs:188` 行替换改为「调用方声明的实际发布界限；本库不核验该声明，也不声称它已验收」（`6e2a284`），`已验收` 并入 §5 禁用列表（`6f58422`） | `src/evidence.rs:188` 在 HEAD 仍是违规文本「适配器声明并已验收的实际发布界限」，须由 WP-02 改写任务按修订后的 §5 同步 |
| attempt 2 | F-9（medium） | §7 C 行改 `BLOCKED` 并写明需要的断言；口径：per-evidence identity 由消费者从 `SelectedEvidenceRef` 读取并写入自身复现清单，`temporal` 不新增 manifest 字段承载（任何 `PitManifestRef` 字段新增须过 Gate C） | identity 保留断言测试尚不存在 |
| attempt 2 | F-10（low） | §5 与 §8 改为逐位置口径 28 处，并给出可复现命令 | — |

本节是修订记录，不构成 Accepted，不产生合入权（`spec.md:1424`）。ADR 侧修订不代替 Gate B 复审：复审须按 `docs/reviews/REVIEW-GATE-B.md` §7 的 7 条进入条件加 F-8、F-9、F-10 重新判定。
