# Agent Note: 设置窗口描述文字下半截被文本矩形高度裁剪（T-152）

Status: implemented

[English](2026-10-13-settings-text-bottom-clip.md) | 中文

## Problem

用户反馈（T-152）：设置窗口里多条描述文字"下方被遮挡"，例如常用设置页
「自定义主题」条目下的"从 themes\*.json 加载配色…"这一行，字形下半截被
齐平切掉，其他页面的描述也有同样现象。根因：文本布局矩形按字体的 em 高
（`lfHeight.abs()`）布置，而 GDI 实际按 `tmHeight + tmExternalLeading`
排版（系统 UI 字体 `tmHeight≈lfHeight` 但另含 2-3px 外部行距；中文 fallback
字形在 em 框上下还有视觉余量）。`DrawTextW` 缺省把文字裁剪到矩形内，字形
底部 1-2px 被切平；用户机 125% 缩放下差异更明显。

## Decision

在 `crates/zhu-ye-settings/src/gdi.rs` 做两处互补修改：

- **实测行高**：`Fonts::ensure` 改用 `GetTextMetricsW` 实测每支字体，
  以 `tmHeight + tmExternalLeading` 作为 `body_height` / `small_height`
  （度量失败回退 `lfHeight.abs()`）。所有用这两个高度算出的文本矩形
  （条目标题/描述块、chips 行、子视图说明）自动扩大到足以容纳真实行框。
- **单行防护**：`draw_text` 对调用方传 `DT_SINGLELINE` 的文本强制附加
  `DT_NOCLIP`，残余 1-2px 垂直溢出永远不会被裁。单行文本安全：溢出落在
  相邻留白里（行块间留 ≥3px 间隙）；多行 `DT_WORDBREAK` 保留裁剪防止
  文字溢出面板。标志变换收敛在纯函数 `singleline_noclip`，配 2 个单测。

## Alternatives considered

**逐个调用点加高文本矩形。**
否决：设置窗口与工具箱面板的单行调用点多达几十处，容易漏改与漂移；
修两处共享源头（字体高度语义 + draw_text）一次覆盖两个窗口。

**对全部文本无条件加 `DT_NOCLIP`。**
否决：展开面板里的多行说明（`DT_WORDBREAK`）在矩形不足时会画出面板；
应当放大矩形而非整体放开裁剪。

**行高只用 `tmHeight`（不含 leading）。**
否决：`tmHeight` 重新等于 em 框，外部行距正是吸收 fallback 溢出的余量。

## Consequences

- 设置窗口与工具箱面板的所有文本矩形增大外部行距量（常见缩放下 2-3px）；
  条目块在固定 76px 行内重新居中，整行位移 ≤1-2px。修复前后帧对比只见
  这些预期的垂直微调，且每条描述带下方有 ≥3px 纯背景（无底部裁剪）。
- `panel_window.rs` 复用的同一 `Fonts`，自动受益。多行行为不变。
- 给后续 GDI 工作的第一性提醒：文本矩形高度绝不单靠 `lfHeight`，
  必须 `tmHeight + tmExternalLeading` 实测，单行绘制保留 `DT_NOCLIP`。

## Related

- 设置窗口视觉设计（字体角色、16/13/24 体系、行度量）：
  [docs/设置窗口视觉设计.md](../../../docs/设置窗口视觉设计.md) §5
- 验收：[docs/验收标准.md](../../../docs/验收标准.md) §20（T-152）
- T-125 排版规范化的上下文：同上设计文档 §5 / §6.3
