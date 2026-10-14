# Agent Note: 候选窗定位不遮挡宿主输入行（下 → 右上角 → 上）

Status: implemented

[English](2026-10-15-candidate-placement-avoid-overlap.md) | 中文

## Problem

候选窗不得遮挡宿主输入行。旧 `place_at` 策略恒优先候选窗下方空间
（`anchor.y + gap`）：输入行贴近显示器工作区底部时，候选窗在下方放不下，于是被
向上压回（`top = work.bottom - height`），整块盖在输入行上，用户看不到正在输入的
内容。用户改为要求三级策略：优先下方，下方放不下转输入行右侧，再不行才转上方。
真机验收时用户进一步收紧右侧档语义：下方放不下时，候选框的**最下方也要在宿主输入框
文字的上方**（相当于候选框落在**光标的右上角**）。

## Decision

定位由 `candidate_window.rs` 中的纯函数 `place_in_work_area(work, anchor_x,
anchor_y, anchor_top, anchor_right, width, height, gap)` 决策：

1. **下方** —— top = `anchor.y + gap`；垂直方向在工作区内放得下即用（水平方向仍
   收进工作区）。
2. **右上角**（用户验收收紧：下方放不下时候选框落在光标右上角）—— left =
   `anchor_right + gap`，且候选窗整体置于输入行文字上方：top = `anchor_top − gap −
   height`（底边 = 文字顶 − gap，最下方不越过文字顶边）；水平与垂直都放得下才采用。
3. **上方** —— 同一垂直约束（整体在文字上方），水平对齐输入行左缘并收进工作区；
   用于右侧没有水平空间的场景。
4. **兜底** —— 压进工作区（底部对齐）保证可见，用于候选窗比整个工作区还高的
   极端情形。

`CandidateWindowPlacement` 携带输入行底部左缘（`anchor`）、顶部（`anchor_top`）与
右缘（`anchor_right`），由 `selection_placement` 从既有 `ITfContextView::GetTextExt`
矩形填入。这是对
[TSF 集成笔记](../../implemented/feature/2026-09-20-candidate-window-tsf-integration.md)
所述定位机制的扩展：锚定到光标仍来自 `GetTextExt`，只有展开策略是新增的。间距
常量 `PLACEMENT_GAP = 4`（与旧行为一致，不变）。

## Alternatives considered

- **恒放上方**：不少输入法默认如此，但用户明确要求"优先下方"，仅把上方作为最后
  兜底；翻转默认会改变每个正常输入会话的习以为常位置。否决。
- **保持向上压回工作区（现状）**：这正是遮挡输入行的行为本身；作为投诉根因否决。
- **放左侧而非右侧**：镜像的兜底方案；用户原话点名右侧，且左侧布局脱离阅读顺序
  观感。否决。
- **缩小/重排候选窗**：破坏 T-122/T-124 打磨的布局，且工作区比候选窗小的问题
  依然无解。否决。
- **把候选窗挪到另一台显示器**：对宿主桌面侵入过强。否决；候选窗仍限制在最近
  显示器的工作区内。

## Consequences

- 输入行贴近屏幕底部时，候选窗出现在输入行右侧且整体置于输入行文字上方（底边 =
  文字顶 − gap）——光标右上角，既不遮文字、也不遮输入行下方内容。
- 位于屏幕右下角时，候选窗转到输入行上方（同一垂直约束、水平对齐输入行左缘收进
  工作区）。
- 仅当候选窗高于整个工作区时兜底才压进工作区，此时覆盖输入行不可避免。
- `anchor_top`/`anchor_right` 来自既有锚定 API（`GetTextExt`），不新增 COM 面；
  引擎与按键路由零改动，改动纯属呈现层。

## Verification

- `place_in_work_area` 的 6 个单元测试：下方优先；下方放不下转光标右上角且底边
  不过文字顶；右下角转上方（同一垂直约束、左对齐）；三档全失败时兜底压进工作区；
  右上角档垂直空间不足时兜底、不夹在文字带里；水平方向收进工作区。
- `cargo test -p zhu-ye-ime`：lib 244（candidate_window 13）、tsf 44、
  host_e2e 167 全绿；`cargo fmt --check` 与
  `cargo clippy --all-targets -- -D warnings` 干净。
- 需求规格说明书 FR-009、验收标准 FR-009「定位策略（T-162）」行、方案设计 §5.1
  定位策略节、CHANGELOG [Unreleased]、todos T-162 均同步为右上角语义。
- 2026-10-15 两轮部署均经用户机提权 worker `copy-dll-ver`：首轮
  `zhu_ye_ime_878D26F8.dll`、右上角轮 `zhu_ye_ime_8DA8D8FB.dll`（均 2,244,096 B），
  每次都切换 CLSID InprocServer32 + 语言档 IconFile 指针并重启 ctfmon；release
  二进制 `verify-tsf-dll`（导出 3/3）与 `verify-tsf-identity`（7/7）通过。新宿主
  进程（如新建记事本）将加载新 DLL；右上角档真机目视验收待用户确认。
