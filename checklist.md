# temporal 实施与验收检查表

**版本：0.1.0｜整理日期：2026-09-27｜初始状态：全部行为验收 NOT_RUN**

本表由 [goal.md](goal.md) 和 [spec.md](spec.md) 的条目生成，不替代正文。工作区原始文档的结构检查不能勾选下列实现、迁移或产品验收。

## 1. 证据登记规则

每项证据必须含：Owner、仓库SHA、工作树状态、工具链/target、测试函数或命令、输入snapshot/seed/policy/schema、退出码、日志引用/摘要、执行状态与评审引用。

允许状态：PASS、FAIL、NOT_RUN、BLOCKED、NOT_APPLICABLE。NOT_APPLICABLE必须附经过审核的适用性说明，不能用于跳过必需测试；多个Owner的子证据全部满足才能勾选汇总项。未知Owner、缺日志或只有文档不允许写PASS。

## 2. 目标完成核对

- [ ] **G-001：唯一、明确且可检查的时间基础**。状态：NOT_RUN；Owner与证据：待登记。
- [ ] **G-002：墙钟、单调钟和测试推进解耦**。状态：NOT_RUN；Owner与证据：待登记。
- [ ] **G-003：统一六种时间角色**。状态：NOT_RUN；Owner与证据：待登记。
- [ ] **G-004：点、期间和生效区间不混同**。状态：NOT_RUN；Owner与证据：待登记。
- [ ] **G-005：精度损失可见、可拒绝、可追溯**。状态：NOT_RUN；Owner与证据：待登记。
- [ ] **G-006：历史初值和修订不可被覆盖**。状态：NOT_RUN；Owner与证据：待登记。
- [ ] **G-007：明确两种知识查询口径**。状态：NOT_RUN；Owner与证据：待登记。
- [ ] **G-008：PIT 防前视成为核心合同**。状态：NOT_RUN；Owner与证据：待登记。
- [ ] **G-009：系统可消费证据不由墙钟字段冒充**。状态：NOT_RUN；Owner与证据：待登记。
- [ ] **G-010：确定性、可解释与可复现**。状态：NOT_RUN；Owner与证据：待登记。
- [ ] **G-011：派生数据继承知识边界**。状态：NOT_RUN；Owner与证据：待登记。
- [ ] **G-012：纯库适合高并发消费，不制造运行时负担**。状态：NOT_RUN；Owner与证据：待登记。
- [ ] **G-013：错误属于模块，处理策略属于消费方**。状态：NOT_RUN；Owner与证据：待登记。
- [ ] **G-014：完成 kernel 时间所有权迁移**。状态：NOT_RUN；Owner与证据：待登记。
- [ ] **G-015：生产就绪以真实证据判定**。状态：NOT_RUN；Owner与证据：待登记。
- [ ] **G-016：行情事件、采样观察和聚合窗口不能混同**。状态：NOT_RUN；Owner与证据：待登记。
- [ ] **G-017：补齐与重放保留真实历史，不伪造覆盖**。状态：NOT_RUN；Owner与证据：待登记。
- [ ] **G-018：每次时间解释均可追溯**。状态：NOT_RUN；Owner与证据：待登记。
- [ ] **G-019：跨语言与存储不会破坏时间合同**。状态：NOT_RUN；Owner与证据：待登记。
- [ ] **G-020：独立接线与测试不重新制造时钟墙**。状态：NOT_RUN；Owner与证据：待登记。

## 3. 180项验收场景

以下每项的输入、断言和不变量详见SPEC对应测试表。测试名不等于实现证据。前120项中涉及平台/后端/产品的部分须按SPEC §32.1拆分Owner，不能仅用本仓fixture标记真实集成通过。

- [ ] **TP-T001 — 单仓冷启动**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T002 — 生产依赖检查**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T003 — 唯一类型身份**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T004 — 禁止无单位构造**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T005 — 未知时间无默认值**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T006 — 序列化边界**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T007 — 私有字段边界**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T008 — 内部模块不可依赖**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T009 — 非穷尽枚举消费**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T010 — 根导出名册一致**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T011 — epoch 与任意纳秒**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T012 — 单位正负构造**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T013 — 单位乘法溢出**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T014 — 普通加减可逆**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T015 — 极值加减**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T016 — 全 i64 跨度**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T017 — 反向与相等差**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T018 — 最大 Duration**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T019 — 负子秒标准化**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T020 — 子秒输入校验**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T021 — SystemTime 正负 epoch**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T022 — SystemTime 极负值**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T023 — SystemTime 平台精度**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T024 — SystemTime 超范围**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T025 — 墙钟允许回退**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T026 — 单调测试推进**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T027 — 组合接口真实成立**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T028 — 单类时钟不冒充组合**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T029 — PIT 不读取 now**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T030 — 虚拟推进不冒充等待**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T031 — 日期合法性**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T032 — 期间半开与边界**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T033 — 粒度不是固定秒数**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T034 — 日历身份**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T035 — EventRecord 必需项**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T036 — PublishedObservation 粗时间**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T037 — Announcement 提前公告**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T038 — DerivedRecord 语义**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T039 — Unknown 与 NotApplicable**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T040 — 六角色不是排序链**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T041 — 默认拒绝精度损失**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T042 — 向零正负对称**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T043 — floor 负边界**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T044 — 无损整单位**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T045 — 存储乘法溢出**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T046 — floor 还原下溢**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T047 — ns 单位恒等**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T048 — 未对齐投影**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T049 — 投影值不一致**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T050 — 能力声明边界**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T051 — 同版同内容重复**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T052 — 同版载荷冲突**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T053 — 同版不同接收时间**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T054 — 相同标识跨事实或权威作用域**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T055 — 同纳秒多事件**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T056 — 旧版晚到**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T057 — 同序号冲突**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T058 — 来源代次重置**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T059 — 非法前驱关系**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T060 — 分支修订**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T061 — CPI 发布前**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T062 — CPI 初值窗口**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T063 — CPI 修订之后**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T064 — 精确边界**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T065 — 日内不确定区间**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T066 — 保守上界与精确模式**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T067 — 计划日期不能充证据**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T068 — 旧版内容缺失**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T069 — 来源补采口径**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T070 — 证据作用域或转换绑定不匹配**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T071 — 接收不等于消费**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T072 — 同纳秒不同序号**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T073 — 未来提交旧墙钟**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T074 — 边界隔离**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T075 — 日志代次隔离**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T076 — 多分区切片**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T077 — 事务起点假提交**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T078 — 流式边界先于持久化**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T079 — 缺历史 cut**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T080 — 截断候选/证据分页**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T081 — 先可见再选修订**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T082 — 撤回不能先过滤**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T083 — 撤回后恢复**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T084 — 不确定更高修订**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T085 — 空可见集合**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T086 — 合法输入乱序**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T087 — 合法重复输入**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T088 — 链关系不能偷看未来**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T089 — 部分失败不能伪装全绿**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T090 — Facts 不是 LatestObservation**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T091 — 公告与生效分离**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T092 — 区间起点包含**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T093 — 区间终点排除**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T094 — 非法有效区间**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T095 — 未来状态已公告**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T096 — 多个有效断言**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T097 — 未来替换保留当前**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T098 — 未来公告不截断历史**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T099 — 有效状态撤回**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T100 — 预测观测晚于截止**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T101 — 派生下界**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T102 — 完成时间下界**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T103 — 今日重算历史**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T104 — 反事实模拟**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T105 — PIT join**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T106 — 模型实际可用性**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T107 — 固定快照重放**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T108 — 合法未来追加**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T109 — 来源历史补全例外**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T110 — 跨快照混页**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T111 — 未来信息隔离**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T112 — 三种空值语义**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T113 — 错误机器合同**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T114 — 上下文错误适配**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T115 — 缓存隔离**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T116 — 资源上限**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T117 — 链/集合规模**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T118 — 零分配基础路径**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T119 — 后端等价对照**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T120 — 交付证据分级**。Owner：temporal；涉及实际边界时另需对应消费方证据（SPEC §32.1）。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T121 — 绝对窗口半开边界**。Owner：temporal。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T122 — 窗口交集与邻接**。Owner：temporal。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T123 — 窗口全域跨度**。Owner：temporal。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T124 — 绝对窗口与民用期间**。Owner：temporal / Adapter。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T125 — 非法窗口**。Owner：temporal。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T126 — 无源时刻采样**。Owner：temporal。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T127 — 本地采样不冒充源事件**。Owner：Adapter / Product。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T128 — K 线临时版与最终版**。Owner：Product / temporal。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T129 — 来源 close 时间与可用性**。Owner：Adapter / Product。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T130 — 订单簿连续性**。Owner：Product / Runtime。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T131 — 端点单位分别固定**。Owner：Adapter。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T132 — 禁止位数猜单位**。Owner：Adapter。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T133 — 原始无符号范围溢出**。Owner：Adapter / temporal。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T134 — 零值缺失约定**。Owner：Adapter / Product。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T135 — 未知时区与重复民用时间**。Owner：Adapter。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T136 — 不支持的时间尺度**。Owner：Adapter。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T137 — 映射可复现**。Owner：Adapter。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T138 — 事后时钟校准**。Owner：Platform。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T139 — 规范化器修复**。Owner：Adapter / Product。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T140 — transaction_time 含义**。Owner：Adapter / Product。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T141 — 补齐终点固定**。Owner：Runtime。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T142 — 闭区间原生网格换算**。Owner：Adapter / temporal。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T143 — 同时间戳跨页记录**。Owner：Adapter / Runtime。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T144 — 分页重叠和重传**。Owner：Adapter / Product。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T145 — 当前快照不能补历史**。Owner：Platform。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T146 — 空响应的完整性**。Owner：Adapter / Platform。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T147 — raw成功规范化失败**。Owner：Runtime / Platform。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T148 — 归档重放双时间**。Owner：Runtime / Platform。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T149 — 重建日志代次**。Owner：temporal / Runtime。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T150 — 水位线之后更正**。Owner：Product / Runtime。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T151 — Protobuf缺失与零**。Owner：Adapter。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T152 — 二进制纳秒往返**。Owner：Adapter。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T153 — JSON整数精确性**。Owner：Adapter。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T154 — 三态wire校验**。Owner：Adapter / temporal。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T155 — 未知协议变体**。Owner：Adapter。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T156 — Protobuf宽时间范围**。Owner：Adapter / temporal。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T157 — 后端有损时间投影**。Owner：Adapter / temporal。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T158 — 下推选版与撤回**。Owner：Adapter / temporal。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T159 — 跨页缓存绑定**。Owner：Adapter / Product。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T160 — 存储能力与实测区分**。Owner：Adapter。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T161 — 查询中配置热更新**。Owner：Product。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T162 — 本地错误消费适配**。Owner：Product / temporal。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T163 — gRPC上下游可见边界**。Owner：Product / Runtime。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T164 — 纯逻辑不真实等待**。Owner：temporal。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T165 — 虚拟时间控制范围**。Owner：Runtime / kernel。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T166 — 错误与管理诊断侧信道**。Owner：Platform。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T167 — 模式不得静默降级**。Owner：Product / temporal。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T168 — 多个消费者的版本绑定**。Owner：Product / Adapter。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T169 — 跨时钟表观负延迟**。Owner：Runtime / Product。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T170 — 关停中提交与checkpoint**。Owner：Runtime / kernel。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T171 — 无源序号的本地采样**。Owner：Product / temporal。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T172 — 今日重解析与历史实际**。Owner：Platform。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T173 — 未来窗口终点的临时观察**。Owner：Product / temporal。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T174 — 派生与模型全链**。Owner：Platform。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T175 — 不同完整性作用域**。Owner：Platform / temporal。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T176 — 未来追加性质扩展**。Owner：temporal / Platform。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T177 — 独立外部消费**。Owner：temporal。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T178 — 兼容候选回滚**。Owner：Governance / Platform。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T179 — 冻结参数与Owner缺失**。Owner：Governance。状态：NOT_RUN；证据：待登记。
- [ ] **TP-T180 — 文档检查不冒充实现**。Owner：Governance。状态：NOT_RUN；证据：待登记。

## 4. 独立模块与多仓收口

- [ ] M0：真实仓库/Owner、kernel基线差异、消费者名册、规范审批齐全。
- [ ] M1：单仓checkout、锁文件、MSRV与三平台、外部消费、公开API门禁实际通过。
- [ ] M2：时间角色/期间/窗口/profile与证据模型合同实际通过。
- [ ] M3：可见性三态、严格选版、撤回、有效状态、资源与性质测试实际通过。
- [ ] M4：各产品mapping/wire/storage/boundary/cut/coverage与派生链差分实际通过。
- [ ] M5：kernel时间公开项和错误映射清理；已登记消费者迁移；无永久依赖。
- [ ] 回滚组合、数据保留/恢复、例外截止与移除证据已审核。
- [ ] 将模块完成、迁移完成、产品PIT完成分别记录，不互相推断。

## 5. 禁止的完成替代

Markdown语法检查、编号校验、Python算术参考、人工阅读、cargo package、单仓CI或模拟时钟推进，均不能单独替代所有Rust行为、真实后端、运行时、长时稳定性与跨仓消费验收。
