# Agent Note: 前缀组词展开（稀疏整词命中音节，T-090）

Status: implemented

English | [中文](2026-10-04-prefix-word-expansion.zh.md)

## 问题

FR-059（M14，D-06 后置项）解决"完整拼音整词命中稀疏时候选窗近乎空白"的问题。
T-060 实测：410 个音节中 80 个整词命中不足一页（<9 条；lue/nue/cei 零候选，
1-4 条 43 个，5-8 条 34 个）。引擎"完整拼音整词优先匹配"
（`generate_candidates` 按完整拼音键查）令这些音节首屏稀疏——例如 `shui`
只有 说/谁/睡/水，而词典里已有同前缀更深组词（水稻/水果/水平/睡觉…）。
决策 D-70/D-71 收口范围：仅在候选不足一页时展开，且绝不动完整拼音 eval 路径。

## 决策

整词命中组之后追加**独立的前缀组词展开组**，仅当
`direct_hit && main.len() < page_size` 时触发：

- **core** 复用 `Dictionary::lookup_prefix("shui")` 基础设施（FR-023 前缀补全
  与复合词典同源）。`prefix_expand_candidates(dictionary, pinyin, fill, already)`：
  1. `fill == 0 || pinyin.is_empty()` → 空；
  2. 经 `candidate_from_entry` + `with_source(CandidateSource::PrefixExpand)`
     映射 `lookup_prefix(pinyin)`；
  3. 对 `already`（整词命中组）与内部按候选文本去重；
  4. 分数降序 / 文本升序排序后截断到 `fill`。
- **引擎**（`refresh_candidates`）：整词命中组 `main` 排序后，
  `fill = page_size - main.len()`；展开组用同一排序上下文单独排序，再经既有
  `append_group`（主组优先同文本去重）合并——模型权重无法把展开词混入主组。
- **位次链**（定稿）：整词命中组 → **前缀组词展开组** → 领域提权（D-13）/
  联系人（D-21）→ emoji 队尾。展开组不参与领域提权（D-15 仅完整词）；
  UI 无新增标签（与 Static 同渲染）。
- **测试固化的交互**：不完整拼音（`shuip`）仅走 FR-023 前缀补全路径（零展开
  词混入）；常见拼音整词命中 ≥9 时零展开（T-050 基线逐位不变）；注入更小
  `page_size` 时 `fill` 随之缩放；合并后列表无重复文本。

## 备选方案

**维持现状（不展开）。** 拒绝：80 个音节首屏持续稀疏，正是 FR-059 要解决的
痛点。

**不补足首屏而展开到第二页。** 拒绝：D-70 口径是补足首屏至 9 条；第二页填充
改变翻页语义且无用户可见收益。

**改完整拼音整词匹配（选项 B，词典层）。** 拒绝：风险高、越界——会扰动受评测
的整词路径。

**展开词按最终分数并入主组。** 拒绝（D-71）：独立分组保证整词命中组逐位不变
（T-050），eval 零回退可被简单证明。

## 后果

- 80 个低频音节首屏补足到一页；常见拼音零影响（验证：T-050 既有 host-e2e
  m7~m13 全过 + 新 `--m14` 组全过；`de` 10 条整词命中零展开）。
- **eval 零回退（最强证据）**：真实词典复跑与 T-057 基准逐项一致——Top1
  84.7% / Top3 97.2% / 整句 21.0%，且 MISS 清单（385 行）与基线复跑
  （stash 工作树后运行）`data/eval/miss-base.tsv` 逐行一致。
- **领域包词被展开截获**：领域包中键为所输拼音的词（如 it 包 谁 `shui`）先被
  复合 `lookup_prefix` 取为 `PrefixExpand` 显示一次；随后领域提权
  `append_group` 同文本去重将其吸收。展示与 D-13 位次（基础候选之后）等价，
  用户无感知；由 `领域包词被展开截获不重复` 引擎测试固化。
- 展开词不因排序进入用户词学习：学习只发生在上屏 commit 时；
  `StaticRankingModel::rank` 仅当词已有用户频率时才把来源重标为 `User`
  （既有用户词语义不变）。
- 一处既有引擎测试断言随行为更新：`backspace从残缺回到完整音节重算候选`
  现在看到 `ni → 你/你好/尼好`（ni 前缀展开），而不再只有 你。
- `CandidateSource::PrefixExpand` 为新增公开变体；无 UI 标签变化（同 Static
  渲染）。候选总数在常见情形下仍封顶一页（fill ≤ `page_size − main.len()`），
  翻页长度不膨胀。

验证（2026-10-04）：core +5 单测（fill 截断、对 already 与内部去重、空输入/
零填充、确定性、来源标注）；ime +5 集成测试（补足一页、已满不展开、
`page_size` 注入、不完整拼音互斥、领域截获去重）；host-e2e `--m14` 6/6
（内存词表 + 真实词典机制一致性：shui 主组 8 条 → 展开 1 条）；workspace
全绿（core lib 274、属性 8、ime 175、其余不变）；fmt/clippy
`-D warnings`/`git diff --check` 干净；eval 如上。

相关 note：[前缀候选（FR-023，
2026-09-24-prefix-candidates-incomplete-segmentation.md）](../../implemented/feature/2026-09-24-prefix-candidates-incomplete-segmentation.md)
——保持排他的不完整拼音路径与 `lookup_prefix` 契约来源；[复合词典
（2026-09-29-composite-dictionary-and-abbreviation-path.md）](../../implemented/architecture/2026-09-29-composite-dictionary-and-abbreviation-path.md)
——领域词经词典分层出现在展开组的机制；[候选排序静态模型
（2026-09-19-candidate-ranking-static-model.md）](../../implemented/feature/2026-09-19-candidate-ranking-static-model.md)
——展开组复用的排序语义。
