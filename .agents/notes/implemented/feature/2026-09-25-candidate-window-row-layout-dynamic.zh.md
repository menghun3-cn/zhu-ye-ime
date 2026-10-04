# Agent Note：候选行布局——序号紧凑与动态译文分栏（T-037）

Status: implemented

English | [中文](2026-09-25-candidate-window-row-layout-dynamic.zh.md) | 中英镜像

## Problem

候选框的用户反馈（T-037）：候选词离序号太远；英文译文被钉死在一列的
右侧 1/3 固定列里，长英文词被省略号截断，译文看起来与候选词脱节。
固定几何在两侧都浪费空间：40 px 序号列只装一个约 10 px 的数字，右侧列
又无法随内容变宽。

## Decision

序号收窄，行改为按内容驱动的两段式分栏。

- `CandidateMetrics::marker_width` 由 40 改为 26 dp，`translation_gap`
  由 16 改为 8 dp——候选词更贴近序号。
- 固定的 `text_rect`/`translation_rect` 分栏被单一的
  `CandidateMetrics::row_split(row, main, translation)` 取代：
  - 主文本宽度用 `estimate_text_width` 估算（ASCII 0.55×em、CJK 1×em，
    略保守，不会重叠）；
  - 有译文时主文本封顶，保证译文区（含间距）至少占可用宽 1/3；无译文时
    主文本可占满整行；
  - 译文列紧跟主文本之后（间隔 `translation_gap`）一直延伸到行尾——
    左侧锚定，宽度约为原来的两倍，长英文词不再轻易被省略号截断。
- `candidate_window` 的 `paint()` 通过 `row_split` 绘制主文本与译文；
  选中/未选中配色不变。

## Alternatives considered

**保留固定分栏只收窄序号列。** 否决：右侧 1/3 仍然局促且与候选词脱节；
问题在于摆放方式而不只是宽度。

**用 `GetTextExtentPoint32W` 在渲染层实测文本。** 本里程碑否决：
`estimate_text_width` 刻意保守，保持 `candidate_ui` 平台无关、可单测；
布局层仍是行分栏的唯一权威。

**译文为空时也恒留 1/3。** 否决：无译文的行右侧会留一片无意义的空白；
空译文时主文本现在可占满整行。

## Consequences

行 API 变更：`text_rect`/`translation_rect` 删除，`row_split` 成为唯一
布局入口；单测迁移到新语义，覆盖短主文本、超长主文本与无译文三种情况。
渲染几何已在 VM 验证（v0.1.7）：译文灰字最左 x 由 528 移到 333
（约 −195 px，现紧跟候选词）；行内主文本左缘由 271/273 移到 255
（约 −16 px，对应序号列收窄 14 px 加抗锯齿取整）。配套四角修复见
[2026-09-25-candidate-window-corner-black-pixels.md](2026-09-25-candidate-window-corner-black-pixels.md)，
渲染基础见
[2026-09-19-candidate-window-gdi-rendering.md](2026-09-19-candidate-window-gdi-rendering.md)。
