# Agent Note: 候选窗默认浅色与空候选仍显示

Status: implemented

[English](2026-09-25-candidate-window-light-default-and-empty-panel.md) | 中文

## 问题

v0.1.1 VM 演练后收到两条用户反馈：

1. **黑底。** VM 系统处于深色模式（`AppsUseLightTheme = 0`），候选窗 `Auto`
   主题跟随系统渲染成深色配色（背景 `#202020`）。用户期望搜狗经典风浅色为
   默认，"黑底不是白底"看起来就是违反需求——尽管"深色跟随系统"曾是 T-028
   拍板的行为。
2. **候选框消失。** 组合串无法产生候选时（如 `z`，或词典缺词），候选窗整体
   隐藏（`visible_items().is_empty() → hide`）。用户期望组合串非空时候选框
   始终可见，即使零个候选行。

## 决策

- **T-030（默认浅色）：** `ThemePreference::Auto` 不再读取
  `AppsUseLightTheme`。非系统高对比度时一律解析为 `UiThemeKind::Light`；
  系统高对比度仍解析为 `HighContrast`（系统色）。`UiThemeKind::Dark` 仅经
  显式 `ThemePreference::Dark` 可达（`candidate-demo --theme dark` 与未来任何
  用户可见设置）。删除 `apps_use_light_theme` 及其 Registry 依赖。
- **T-031（空候选仍显示）：** 显隐契约从"有可见候选"改为"组合串非空"。
  `candidate_window::update` 仅当 `view.composition.is_empty()` 时隐藏窗口；
  否则按需创建并显示，包括 `visible_items()` 为空的情形。空候选面板渲染为
  页眉条：背景、圆角边框、组合串与拼音提示，零个候选行。
  `CandidateMetrics::panel_size(0)` 现在是真正的零行布局（页眉 + 上下内边距，
  96 DPI 下 58 像素），不再钳制为一行；算式改写后 `rows = 0` 不会下溢或产生
  负高度。TSF 的 `refresh_candidate_window` 在上屏/取消时记录
  `cand-hide (no composition)`，其余情况记录 `cand-show items=N`（N 可为 0）。
- 引擎与候选生成逻辑不动：`z`/`zh` 仍不产生候选词（T-029 语义），只改窗口
  呈现。

## 曾考虑的替代方案

**保持 `Auto` 跟随系统。**
按用户决定否决：浅色搜狗风必须是默认，与系统深浅色无关；跟随系统正是黑底
意外的来源。

**无候选时显示满尺寸空框。**
按用户决定否决：页眉条与搜狗/QQ 输入法一致，面板视觉更轻。

**只在 TSF 层区分"无候选"与"无组合"。**
否决：显隐决策应属于窗口控制器，让 TSF 刷新、演示程序与未来接入方共享同一
契约；在各调用方重复实现容易漂移。

## 后果

- 候选窗在深色系统上也是白底（`#FFF`，搜狗经典风）；深色 VM 演练截图是最
  直接的验证产物（shots8）。
- 输入 `z`、`zh` 或任何空候选输入都有可见的页眉条，同时也给用户"输入法处于
  组合中"的视觉确认。
- `panel_size(0)` 成为有测试的布局；既有 9 行几何不变（回归断言锁定）。
- FR-009 验收标准更新："深浅色（T-030）"与"皮肤色板（T-028/T-030）"明确
  浅色默认，FR-001 新增"无候选页眉条（T-031）"一行。
- 部分取代旧笔记：GDI 渲染笔记的主题跟随事实与前缀候选笔记的隐藏后果已就地
  改写并交叉链接到本笔记。

验证：`panel_size` 零行页眉条单测；fmt、clippy `-D warnings`、全 workspace
测试通过；host-e2e 种子（19/19）与真实词典 smoke（6/6）通过。VM v0.1.2
演练（系统保持深色模式）确认两项行为：`z`/`zh`/`w` 记录 `cand-show items=0`
且窗口可见——页眉条实测 58 像素（360×58 矩形）；`nihao`/`zhidao`/`wo`
渲染完整面板，提交序列为"你好知道我"（GBK `C4 E3 BA C3 D6 AA B5 C0 CE D2`）；
提交与 Esc 均记录 `cand-hide (no composition)`。截图（shots8）像素直方图与
浅色板一致：背景 `FFFFFF`、选中块 `E6F2FE`、次要 `999999`、选中文字
`0B57D0`——全部发生在深色系统下，证明默认浅色生效。
