# Agent Note：候选框四角黑点（T-037）

Status: implemented

English | [中文](2026-09-25-candidate-window-corner-black-pixels.zh.md) | 中英镜像

## Problem

候选框四角出现黑点。根因在 `paint_background`：它只画了一个
`RoundRect`，圆角矩形**外侧的四个角落区域从未被填充**。绘制走内存 DC
双缓冲（`CreateCompatibleBitmap`），其初始像素——未初始化、通常为黑/杂色
——在这些角落被 `BitBlt` 透出。

## Decision

`paint_background` 现在**先用背景色刷子填满整个客户区**，再在该背景上画
圆角矩形（边框）。圆角外侧的角落区域因此变为背景色，而不是未初始化像素。
其余绘制不变。

## Alternatives considered

**`WS_EX_LAYERED` 逐像素 alpha 实现真透明圆角。** 本里程碑否决：分层窗口
为了一片只有几像素宽的美化区域，会让定位、DPI 与主题处理复杂化。

**完全去掉边框笔。** 否决：FR-009 保留蓝色圆角边框（浅色主题
#1E88E5）；黑点并非边框引起。

**直角（半径 0）。** 否决：改变既定视觉风格而非修复缺陷。

## Consequences

VM（v0.1.7）上窗口内四角 3×3 区域全为纯白，旧构建同位置存在非白像素。
视觉上四角现为背景色；与行布局改动共同构成 T-037 候选框打磨。相关笔记：
[2026-09-25-candidate-window-row-layout-dynamic.md](2026-09-25-candidate-window-row-layout-dynamic.md)
与
[2026-09-19-candidate-window-gdi-rendering.md](2026-09-19-candidate-window-gdi-rendering.md)。
