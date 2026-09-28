# Agent Note: 候选框字体改为宋体（SimSun）

Status: implemented

[English](2026-09-25-candidate-window-font-simsun.md) | 中文

## 问题

候选窗原用系统 UI 无衬线字体 `Microsoft YaHei UI` 绘制（`candidate_window::create_font`）。
用户指令：候选框字体改为宋体。

## 决策

- `create_font` 改用英文字面名 `SimSun`（每套中文 Windows 都有的标准宋体衬线
  字体）。字号、字重、字符集、ClearType 抗锯齿、DPI 缩放（`font_height` 96 DPI
  下 `dp(16.0)`）与 `DEFAULT_GUI_FONT` 回退均不变。
- `candidate_ui` 的宽度估算（`estimate_text_width`：ASCII 0.55×字号、CJK 1.0×字号）
  维持原样：它们是字体无关的近似，`fit_text`/`DT_END_ELLIPSIS` 仍兜底截断溢出。
- 实现过程中暴露既有缺陷：创建的 HFONT 从未被 `SelectObject` 进绘制 DC，
  字面名此前对渲染毫无效果（见
  [缺陷笔记](../bug-fix/2026-09-25-candidate-window-font-never-selected.zh.md)）。
  `CandidateWindowState::paint` 现会在绘制开头选择字体；宋体改动也正因该修复才可见。

## 曾考虑的替代方案

**用中文名 `宋体` 而非 `SimSun`。**
否决：英文字面名与区域设置无关，在通用 `LOGFONTW` 构造中无歧义；`SimSun`
正是 Windows 对"宋体"的解析结果。

**保留雅黑，只调字重/字号。**
否决——用户要的是衬线字体本身，不是对无衬线的微调。

## 后果

字体此前未在任何文档中固化，仅验收标准 FR-009 新增字体验收行（T-034）。
GDI 渲染笔记中泛称的"字体"表述仍然成立。VM v0.1.6 验证（test11，真实 TSF 管线）：

- 宋体版与黑体 A/B 臂对比：整个面板不同——页 0 5964 像素、页 1 5327 像素，
  纵向范围 y 24..377——所选字面名已真正到达 DC。本 VM 默认 GUI 字体即黑体，
  故黑体臂与旧的"未选字体"构建逐像素相同（这也解释了早期"雅黑版"截图
  为何始终不像雅黑）。
- 安装后的 DLL 内嵌 `SimSun` 字符串；截图留档 shots11-simsun（终版）与
  shots11-hei（A/B 臂）；门禁全绿。
