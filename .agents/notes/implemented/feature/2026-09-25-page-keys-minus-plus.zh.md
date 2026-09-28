# Agent Note: 翻页键改为 `-`/`=`（`+`）替代 `,`/`.`

Status: implemented

[English](2026-09-25-page-keys-minus-plus.md) | 中文

## 问题

候选窗翻页原先绑定 `,`（上翻）与 `.`（下翻），即 `classify_key` 中
`VK_OEM_COMMA`→`PageUp`、`VK_OEM_PERIOD`→`PageDown`。用户指令：候选弹出时
用 `-` `+` 键翻页，替换原来的 `,` `.` 键。

## 决策

- `VK_OEM_MINUS`（`-` 键）→ `PageUp`；`VK_OEM_PLUS`（`=` 键；按住 Shift
  时输出 `+`）→ `PageDown`。按虚拟键码映射天然同时覆盖无 Shift 的 `=`
  与 Shift+`=` 的 `+`（两者是同一个 VK）。
- `,` `.` 不再映射，与其它未映射键一样放行给宿主（S_FALSE/BOOL(0)）。
- 关键障碍是 Shift 本身：`+` = Shift+`=`，而 Shift 键按下会立即触发
  `ToggleMode`——组合进行中按 `+` 翻页会把中文模式悄悄切走、候选窗却仍
  挂着。修复：`plan_action` 仅在引擎无活动组合（`!engine.is_active()`）
  时接受 `ToggleMode`；组合中先落下的 Shift 只是普通修饰键，放行给宿主。
  引擎层 `toggle_mode` 的"组合保留"语义在非组合场景下不变。
- 边界说明：`classify_key` 保持"纯 VK→动作"映射；组合感知的门控放在
  `plan_action`（本就负责字母/活动态规则），职责边界不变。

## 曾考虑的替代方案

**追踪"Shift 待定单击"（仅当按下与抬起之间没有其它按键时才切换）。**
本次否决：需跨 `OnKeyDown`/`OnKeyUp` 的状态机改造，且被吃下的键 TSF 是否
可靠送达 KeyUp 不值得为一个键位改动押注；按活动组合门控两行就达到 `+`
翻页的用户可见目标，且既有"Shift 单击切换"验收不受影响（组合外单击 Shift
与原来完全一致）。

**只映射带 Shift 的 `+`，`=` 不绑定。**
否决：物理键只有一个 VK，分类时区分 Shift 需引入 `GetKeyState`；且 `=`
不绑定会违背常见输入法 `-`/`=` 翻页习惯。`=` 与 `+` 均下翻。

## 后果

需求/方案设计/安装与使用/验收标准（FR-001 翻页行）键位表已更新；
TSF 集成笔记中关于逗号句号的表述被取代（见
2026-09-20-candidate-window-tsf-integration.zh.md）。新增纯函数测试覆盖
VK_MINUS/VK_PLUS 映射、逗号句号解除绑定、组合中 Shift 门控、英文模式
放行新翻页键。VM v0.1.4 演练（test10）断言 TSF 日志：`key 0xBB state
action PageDown`、`key 0xBD action PageUp`、按 `+` 后无 ToggleMode 动作、
`,` `.` 不再产生翻页动作。
