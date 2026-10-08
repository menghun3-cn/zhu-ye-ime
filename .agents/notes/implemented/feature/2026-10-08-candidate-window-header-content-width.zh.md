# Agent Note: 候选窗页眉按内容自适应宽度（长拼音输入）

Status: implemented

## Problem

输入长拼音串（如 `youmeiyoushenmeren`）时，候选窗页眉的输入串被右侧省略号截断成
`youmeiyoushenmere...`。根因：`CandidateMetrics::panel_width` 固定为 360dp；
`header_hint_rect`（T-031）为带调提示预留了页眉 5/9 的固定宽度，输入串区只剩约
141dp（360 − 2×12 内边距 − 5/9 提示区 − 间距）。17 个 ASCII 拼音字符在 16dp 页眉
字号下按 `estimate_text_width`（ASCII 每字符 0.55×字高）估算约 150px，超出输入串区，
`draw_text` 的 `DT_END_ELLIPSIS` 便截断了它。候选词上方的拼音行不受影响——它本就用
整行宽度绘制。

## Decision

页眉（以及整个面板）由固定宽改为按内容自适应：

- `CandidateMetrics::header_widths(composition, hint)` 用 `estimate_text_width` 返回
  `(输入串宽, 提示分配宽)`。估算总宽仍能放进固定 360dp 面板时行为不变：面板保持
  360dp、提示按其估算宽完整分配。总宽超过固定宽时，面板扩为
  `max(360dp, 输入串宽 + 间距 + 提示分配宽 + 2×内边距)`，提示分配宽上限两者估算总和的
  2/3——用户输入串永远完整，仅有辅助性的提示在极端输入下才可能截断。
- `panel_size_for(rows, composition, hint)` 在 `panel_size` 之上加入该宽度；三处窗口
  尺寸计算（创建、demo、每次 update 缩放）都改用它。
- `CandidateMetrics::with_panel_width(w)` 返回按指定面板宽布局的副本；`paint` 每帧构造
  `metrics = self.metrics.with_panel_width(width)`，页眉输入串/提示、行、页脚所有矩形
  都按真实客户区宽度计算，而非固定 360dp 基准。
- 页眉绘制改用 `header_text_rect_with(hint_alloc)` / `header_hint_rect_with(hint_alloc)`；
  提示为空或与输入串相同时 `hint_alloc` 为 0，输入串占页眉整行。
- `candidate-demo` 新增 `--long` 参数渲染复现场景（输入串 `youmeiyoushenmeren`、
  提示 `yǒu méi yǒu shén me rén`）用于像素验收。

## Alternatives considered

- **全局加宽固定面板（如 480dp）**：短输入时白白加宽，且任何固定宽度对足够长的输入
  仍会截断；否决——问题本质是空间分配，不是基准宽本身。
- **缩小提示字号 / 缩写提示**：损害刚在 T-122 打磨的带调提示可读性，也没解决输入串
  自身空间不足；否决。
- **缩小输入串字号**：本末倒置——用户输入串是第一阅读对象，分配空间（窗口加宽）
  优于压缩字号；否决。
- **候选词上方拼音行改为换行/扩宽**：本次截断发生在页眉输入串，候选词拼音行本就
  横跨整行宽，无需修改。

## Consequences

- 长拼音 + 带调提示时面板从 360dp 扩到约 421dp（96dpi 基准实测），输入串完整渲染、
  无省略号。
- 短输入像素级不变（360dp、提示完整）。
- 极端输入（提示远长于输入串）时被截断的是*提示*——可接受的辅助信息；`place_window`
  仍把 x 限制在工作区内，超出屏幕的畸形宽面板只是物理上不可能，而非崩溃。

## Verification

- `candidate_ui.rs` 单元测试：长输入串+提示扩展面板且输入串矩形 ≥ 估算宽；提示分配
  上限 2/3 且输入串矩形完整、不重叠；短输入保持 360dp 且提示完整。
- `candidate-demo --long` 像素探针（96dpi ×1.25 系统缩放）：窗口 536px 对基线 450px；
  页眉输入串蓝色字形连续一段 x=16..194（17 字符无右侧省略号；若截断会停在约 178 再
  加省略号）；带调提示灰色字形 x=225..451。
- 门禁：`cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、
  `cargo test --workspace`、`git diff --check` 全部通过。
