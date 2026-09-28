# Agent Note: 候选窗 HFONT 从未被选入绘制 DC

Status: implemented

[English](2026-09-25-candidate-window-font-never-selected.md) | 中文

## 问题

候选窗本应以 `create_font`（`LOGFONTW` + `CreateFontIndirectW`）生成的字体渲染，
但**绘制路径上从未对 HFONT 调用 `SelectObject`**，所有 `DrawTextW` 实际使用 DC
默认字体（`DEFAULT_GUI_FONT`）。`create_font` 的句柄被创建、保存、销毁，却从未
使用——源码里的字面名对渲染零效果。症状：改了字面名，屏幕上毫无变化。

长期未被发现的原因：用户桌面 Windows 的 DC 默认字体与当时想要的
"Microsoft YaHei UI" 观感接近；而 VM（Windows Server，中文环境）的默认 GUI
字体是黑体（SimHei）——因此"雅黑版"与"宋体版"候选框截图逐像素相同
（见"后果"节的 A/B 证据）。

## 决策

在绘制路径中真正选择字体，而不是创建后闲置：

- `CandidateWindowState::paint` 现在开头 `SelectObject(hdc, self.font)`，
  返回前恢复旧对象。
- `paint_window` 每帧新建兼容 DC，恢复属防御性卫生；字体句柄生命周期
  （构建时创建、DPI 变化时重建、drop 时删除）不变。
- 本修复属 T-034（字体名改 SimSun）；另见
  [字体名笔记](../feature/2026-09-25-candidate-window-font-simsun.zh.md)。

## 曾考虑的替代方案

**不选字体，依赖 `DEFAULT_GUI_FONT`。**
否决：`create_font` 的字面名成了死代码，观感随系统区域/调校漂移
（桌面中文雅黑、中文 Server 黑体），正是用户提出字体需求所要消除的不确定性。

**在 `draw_text` 内逐次选择字体。**
否决：每帧选择一次即可；逐次选择徒增噪音，帧内并无其他代码改 DC 字体。

## 后果

- 字面名现在真正决定渲染字形。VM v0.1.6 A/B（真实 TSF 管线）：黑体臂与旧的
  "未选字体"构建渲染完全一致（该机默认 GUI 字体即黑体），而宋体在整面不同——
  页 0 5964 / 页 1 5327 像素，纵向 y 24..377。
- 修复前的行为已记录为已修复缺陷；截图留档 shots11-simsun / shots11-hei；
  门禁全绿。
