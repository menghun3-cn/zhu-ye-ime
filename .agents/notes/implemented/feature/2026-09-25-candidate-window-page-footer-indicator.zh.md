# Agent Note：候选框页脚 m/n 翻页指示（T-040）

Status: implemented

English | [中文](2026-09-25-candidate-window-page-footer-indicator.zh.md) | 中英镜像

## Problem

用户需求（T-040 第二项）：翻页时，候选框底部页脚处显示 `m/n` 这样的翻页
指示。顺带核对确认数字选词已具备（见
2026-09-25-candidate-window-arrow-selection.md）；真正的新工作是页脚。

## Decision

在指标体系中新增固定高度页脚条，多页时在底部右端绘制灰色右对齐 `m/n`
标签。

- `CandidateMetrics.footer_height` = 20dp（与其他指标一样随 DPI 等比缩放）。
  `panel_size(rows)` 仅当 `rows > 0` 时叠加页脚——无候选面板保持纯页眉条
  （T-031 行为不变）。
- 新增纯函数 `page_footer_label(page, page_count) -> Option<String>`：
  `page_count <= 1` 返回 `None`；否则输出 1 基当前页/总页数 `m/n`，
  页码越界收敛到末页。
- `CandidateUiView` 新增 `page_count` 字段（引擎中文分支、英文模式分支与
  demo 二进制都填充），视图暴露 `footer_label()`。
- `paint()` 用新增右对齐辅助函数 `draw_text_right`（DT_RIGHT）以次要色在
  `footer_rect(rows)` 内绘制标签；页脚矩形位于最后一行与底部内边距之间，
  不与行重叠。
- 数字键选词：代码路径未动（引擎 `select_index` + `KeyAction::Select`），
  在 VM 上补验收证据而非重写。

## Alternatives considered

**只在"还有下一页"时显示页脚。** 否决：`m/n` 是通例（微软拼音/搜狗在有
翻页可能时恒定显示），位置忽隐忽现会让面板跳动；只要 `page_count > 1`
就恒显，包括末页。

**显示 `页 1/3` 或箭头而非 `m/n`。** 否决：用户明确要"类似 m/n"的指示；
保持最简且无歧义。

**页脚叠加在最后一行上。** 否决：独立页脚条更干净，成本仅 20dp；行几何
测试断言不重叠。

## Consequences

多页候选列表现在在右下角传达翻页状态；单页列表与无候选页眉条渲染与之前
完全一致。单测覆盖（标签格式/单页门控/越界收敛、页脚矩形与行区不重叠、
面板尺寸回归更新）与 VM v0.1.9 像素取证（`shi` 双页页脚灰色文字
x≈535-563 存在，单页 `nihao` 无）。交叉参考：
[2026-09-25-page-keys-minus-plus.md](2026-09-25-page-keys-minus-plus.md)
（翻页键）、
[2026-09-25-candidate-window-arrow-selection.md](2026-09-25-candidate-window-arrow-selection.md)
（上下键选择）与
[2026-09-25-candidate-window-row-layout-dynamic.md](2026-09-25-candidate-window-row-layout-dynamic.md)
（行布局指标）。
