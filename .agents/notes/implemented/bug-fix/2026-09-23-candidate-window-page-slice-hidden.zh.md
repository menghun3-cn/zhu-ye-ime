# Agent Note: 翻页后候选窗因双重分页切片而隐藏

Status: implemented

[English](2026-09-23-candidate-window-page-slice-hidden.md) | 中文

## Problem

T-013 的 VM 验收发现候选窗在翻页瞬间消失：输入 `yi` 显示第 1 页（9 项），按 `.`
后窗口隐藏（`tsf-debug.log` 记录 `cand-hide (no items)`），但按 `1` 仍提交了第 2
页首项（`依`）——引擎翻页正确而窗口空白。
`CandidateUiView::visible_items()` 会再次按 `page_size` 切片
（`start = page * page_size`），而 `InputEngine::candidate_ui_view` 把 `items`
填成了引擎的**当前页**候选（`visible_candidates()`），不是完整候选列表。翻页后
`page >= 1` 使第二次切片的起点 `>= items.len()`，可见区间为空：刷新逻辑据此得出
“无候选”并将窗口隐藏。

## Decision

`InputEngine::candidate_ui_view` 的 `items` 改由 `current_layer_candidates()`
填充——即当前层的**全部**候选（中文层为全部中文候选，译文层为过滤无译文后的
候选）——把分页切片完全交给 `CandidateUiView::visible_items()`，这正符合其
`items` 字段的注释（“全部候选；窗口只展示当前页”）。英文模式的提前返回仍产生
空 `items`，英文输入时窗口保持隐藏。

## Alternatives considered

**让 `CandidateUiView::visible_items()` 对越界页码做容错。**
否决：截断或返回空切片只是掩盖而非修复契约违例；视图继续收到“每页大小”的
列表，永远无法渲染第一页之后的页面。

**由 TSF 适配层直接把引擎当前页切片交给窗口，绕过 `visible_items()`。**
否决：两条切片路径需要手工保持同步；由 `CandidateUiView` 保持唯一权威切片，
才能延续现有渲染契约与单元测试。

## Consequences

翻页后候选窗保持可见且显示正确页面内容。回归测试
（`翻页后视图快照可见项跟随当前页`）断言 `candidate_ui_view().visible_items()`
在 `next_page` 后跟随引擎当前页；该测试对修复前代码失败。VM 重跑翻页阶段显示
按 `.` 后 `candidate-window visible=True`，日志在 `Select(0)` 提交前记录
`cand-show items=9 first=依`。T-013 连同任务清单中的其余 VM 验收项一并关闭。
