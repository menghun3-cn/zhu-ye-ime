# Agent Note: 相邻换位容错纠正与候选拼音显示

Status: implemented

## Problem

2026-10-06 用户报障：输入快时相邻字符常交替——`xiangzhe` 打成 `xiagnzhe`、
`zhegnq` 应识别为 `zhengq`、`shegnc` 应识别为 `shengc`、`zhagnh` 意图
`zhangh`（均为 `gn`↔`ng`、`agn`↔`ang` 类换位），输入法仍应识别为正确表达，
并在候选旁显示**纠正后的正确拼音**（如候选"生"标 `sheng`、"正确"标
`zhengque`、"账"标 `zhang`）。

既有模糊纠错（`corrected_candidates`，M7/FR-024）仅处理**可完整切分**串；
`zhegnq` 这类换位串**根本不可切分**，从未进入任何纠错路径。

## Decision

`zhu_ye_core::corrected_candidates` 现同时处理**不可完整切分**输入（FR-069）：
输入无整词命中且不可完整切分时，枚举**一次相邻字符换位**（`transposition_variants`，
仅 ASCII 小写、去重），每个变体依次走三路：

1. **整词命中**（`lookup` 非空）→ 直接产出；
2. **可完整切分**变体 → 常规候选管线（`generate_candidates`；
   `xiagnzhe`→`xiangzhe`→想着）；
3. **前缀路径** → `generate_prefix_candidates`（完成组 + 补全组；
   `zhegnq`→`zhengq`→`zhengque`→正确、`shegnc`→`shengc`→`shengcheng`→生成、
   `zhagnh`→`zhangh`→`zhanghu`→账户）。

产出候选全部标注 `CandidateSource::Corrected`（既有纠错组，随
`enable_fuzzy` 开关，O-05/T-103），`pinyin` = **纠正后正确拼音**；按文本去重、
受 `CORRECTION_VARIANT_CAP`（24）约束。

候选窗展示（FR-069 后半）：`CandidateUiItem` 新增 `pinyin` 字段（取自
`Candidate.pinyin`），`display_main_text` 将中文候选渲染为 `中文（拼音）`
（`生（sheng）`、`正确（zhengque）`）。适用范围 = **所有带拼音的候选**，不限于
纠错候选；Slang 保持 `[网络]` 标注不加拼音；无拼音候选（emoji、英文原形）
原样展示；译文层仍以译文为主文本。demo 数据（`candidate-demo`）与 e2e 断言
同步扩展。

## Alternatives considered

- **多次换位迭代 / 全量编辑距离。** 否决：一次换位已覆盖报障的
  `gn↔ng`/`agn↔ang` 类；越深编辑搜索空间爆炸、误报率上升。机制刻意有界，
  整词命中由词典数据长期解决。
- **仅扩既有音节模糊替换路径。** 否决：触及不到不可切分串；本需求正是
  "切分失败"场景。
- **拼音独立对齐列。** 暂缓：`（拼音）` 内联形式与用户示例一致（
  `1 生（sheng）`），且在 T-116 词性对齐布局落地前不再增加一个布局轴；
  内联形式融入既有行宽估算，零布局改动。
- **仅纠错候选显示拼音。** 否决：用户要求候选**普遍**显示正确拼音
  （`显示正确的拼音`），整词/前缀候选同样受益。

## Related

- [前缀候选与不完整切分](../../../implemented/feature/2026-09-24-prefix-candidates-incomplete-segmentation.md)——路 3 复用 `generate_prefix_candidates`（完成组 + 补全组）。
- [全拼音切分核心](../../../implemented/feature/2026-09-19-full-pinyin-segmentation-core.md)——`segment_all`，路 2/路 3 的可切分判定门。

## Consequences

- 纠错组现覆盖不可切分输入，代价 = 每次刷新至多 `len-1` 次词典查询（有界、
  离线、确定）。
- 候选行视觉变宽（`中文（拼音）`）；T-037 动态分栏自动吸收，demo/e2e 数据
  断言新主文本。
- 用户可见行为变更，规格化 FR-069（§22，D-84）；与 M7 模糊纠错共用
  `enable_fuzzy` 开关，可整体关闭。
