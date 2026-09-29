# Agent Note：候选选中块贴面板边框与序号内边距（T-043）

Status: implemented

English | [中文](2026-09-28-candidate-window-highlight-flush-borders.zh.md) | 中英镜像

## Problem

用户反馈：候选词选中态的高亮背景块（Pill）左右内边距过小——高亮块外边缘
距离候选词面板的左右边框太远，而左侧序号又贴高亮块左缘太近，视觉上局促、
比例失调。

旧实现把选中块直接画在 `row_rect`（左右各内缩 `padding_x` = 12dp）上：
96dpi 下高亮块距面板左右边框 12/13px，而序号列起点与高亮块左缘重合、
数字字面距左缘仅约 2px。

## Decision

选中块改为**贴面板左右边框的整行矩形**（搜狗风整行选中块），行内容布局
不变：

- `CandidateMetrics` 新增 `highlight_inset_x` = 1（固定 1 物理像素，**不随
  DPI 缩放**：面板边框线由 GDI 1px 笔画绘制，任何 DPI 下都恰占最外圈一
  像素；内缩 1px 使高亮块贴边同时保留边框线不被覆盖）。
- 新增 `highlight_rect(index)`：上下与 `row_rect` 一致，左右从
  `highlight_inset_x` 延伸到 `panel_width - highlight_inset_x`。
- `paint()` 中选中块绘制改用 `highlight_rect`；序号/主文本/译文仍按
  `row_rect` 布局（`padding_x` = 12dp 不变），序号因此相对高亮块左缘保有
  约 12dp 的内边距。

效果（96dpi 像素取证）：高亮块左/右缘距面板边框 12/13px → 1/2px（贴边）；
序号字面距高亮块左缘 2px → 13px。192dpi 深色主题同验（1/2px），确认内缩
量为物理像素而非 DPI 缩放值。

## Alternatives considered

**高亮块与边框完全齐平（inset = 0）。** 否决：高亮块会覆盖最外圈 1px 蓝色
边框线，选中行处边框看起来断裂；1px 内缩在观感上无差别且保住边框。

**保留 12dp 内缩、只把序号列右移。** 否决：只解决"序号贴边"，"高亮块离
边框太远"依旧；且打乱 T-037 收窄序号列让候选词贴近序号的既定布局。

**高亮块收缩成只裹内容的胶囊（宽度随文本）。** 否决：与整行选中块的搜狗
经典风不符；内容短时胶囊外边缘离面板边框更远，与用户诉求相反。

## Consequences

选中块现为贴面板左右边框的整行浅蓝块，序号不再贴其左缘；边框线完整保留。
编号列宽、主文本/译文分栏（T-037）与页脚（T-040）均未受影响。单测锁定
几何不变量（左右贴边、上下包住整行、序号列起点距高亮块左缘 ≥ 10px、
`highlight_inset_x` 不随 DPI 缩放）。交叉参考：
[2026-09-25-candidate-window-row-layout-dynamic.md](2026-09-25-candidate-window-row-layout-dynamic.md)
（行布局指标）与
[2026-09-25-candidate-window-page-footer-indicator.md](2026-09-25-candidate-window-page-footer-indicator.md)
（面板高度指标）。
