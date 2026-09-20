# Agent Note: Candidate window GDI rendering for T-012

Status: implemented

[English](2026-09-19-candidate-window-gdi-rendering.md) | 中文

## 问题

FR-009 要求输入法有可见的候选窗：展示候选词、译文、选中状态与页码指示。FR-011 还要求窗口跟随系统深浅色、高对比度和多显示器 DPI。此前仓库没有任何 UI 代码，在接入 TSF 之前无法验证布局模型或渲染路径。

## 决策

候选窗拆成平台无关的视图层与 Win32 GDI 呈现层。

`candidate_ui` 用纯 Rust 持有快照模型、布局尺寸与主题。`CandidateUiView` 携带输入串、拼音提示、页码、选中项、译文层与候选列表；`CandidateMetrics` 以 96 DPI 基准线性缩放计算全部几何；`UiThemeKind` 与 `CandidateUiTheme` 定义浅色、深色与系统高对比度配色。它还提供数字标记与文本宽度估算辅助，并且不依赖 Windows，便于单元测试。

`candidate_window` 实现 Win32 弹出窗口，采用 GDI 双缓冲绘制。窗口使用 `WS_POPUP` 并带 `WS_EX_NOACTIVATE`、`WS_EX_TOOLWINDOW`、`WS_EX_TOPMOST`，不会从输入目标抢焦点。`WM_PAINT` 先画到内存 DC，再 `BitBlt` 到窗口；背景擦除直接返回，避免闪烁。主题解析读取 `AppsUseLightTheme` 与 `SPI_GETHIGHCONTRAST`；`WM_THEMECHANGED` 与 `WM_SETTINGCHANGE` 刷新配色，`WM_DPICHANGED` 重建布局与字体。字体、位图、DC 与 boxed 状态在所有路径上都会被释放，包括窗口销毁路径。

演示程序 `candidate-demo` 渲染固定 9 项快照，支持 `--theme`、`--dpi`、`--seconds`、`--shot` 与 `--translation-mode`。`--shot` 通过 `GetDIBits` 把客户区捕获成自上而下的 32 位 BMP，用于可视化验证。TSF 接入、光标定位与按键处理明确留给 T-013；当前窗口只绘制静态快照。

## 测试

`cargo fmt`、`cargo clippy --all-targets -- -D warnings` 与 `cargo test --workspace` 均通过。覆盖分页截取、页内选中、DPI 缩放比例、行与页眉矩形不重叠、主题配色区分、UTF-16 转换与 BMP 头一致性。

演示程序在 96 与 192 DPI 下分别运行浅色与深色主题，生成可读 BMP 截图。像素直方图与项目配色一致：浅色背景 `FAFAFA`、深色背景 `202020`、选中行高亮色也吻合，确认窗口与文本表面确实被绘制。

## 曾考虑的替代方案

**Direct2D/DirectWrite。** 本里程碑落选：候选窗只有几行短文本，GDI 已经足够；DirectWrite 在 32 位与 64 位 TSF 宿主内会引入更大的 API 面与更细致的资源生命周期负担。若未来文本质量或复杂排版要求上升，`CandidateUiView` 边界允许 T-013 之后只换渲染器，不动布局与测试。

**浏览器内核覆盖层（WebView/HTML）。** 直接否决：在输入进程里嵌入 Web 运行时违背项目反臃肿原则，增加启动成本，并带来不必要的攻击面与签名面。

**直接单缓冲绘制窗口。** 落选：在窗口 DC 上擦除再重绘，会在每次主题或 DPI 变化时闪烁与撕裂；内存 DC 加 `BitBlt` 是成本最低且成熟的 GDI 模式。

**基于估算宽度自定义文本截断。** 落选：`DrawTextW` 的 `DT_END_ELLIPSIS` 已经正确且按区域规则截断；纯 Rust 估算函数只作为后续布局工作的测量辅助，不作为渲染唯一依据。

## 后果

M3 候选窗渲染基础已经落地，并且可以在 TSF 之外验证。Win32 调用只留在 `zhu-ye-ime`，`zhu-ye-core` 保持平台无关。快照与截图路径为 VM 复核提供了可视化基线，之后 T-013 再接入真实组合上下文。

仍有一些 UI 细节依赖 TSF 阶段：真实输入串、基于光标的定位、按键路由、翻页与窗口显隐。因此 T-012 保持“进行中”，直到 VM 端到端复核通过。若后续出现文本质量需求，DirectWrite 是在同一视图契约背后的局部替换，不需要重做整个候选窗。
