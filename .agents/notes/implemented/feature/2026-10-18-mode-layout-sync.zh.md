# Agent Note: 中英切换同步系统键盘布局（T-168）

Status: implemented

[English](2026-10-18-mode-layout-sync.md) | 中文

## Problem

用户观察到搜狗切英文时任务栏会出现系统灰色"A/"指示，竹叶从不出现。根因：
灰色"A/"是**系统层**的「英文键盘布局激活」指示；搜狗英文模式会真的把系统
键盘布局切到英文（00000409），而竹叶英文模式只是引擎内部状态——系统布局
仍是中文（00000804），指示自然不出现。代码里英文模式的键映射本就按英文
布局语义设计（T-066：Shift+2=`@`、`;`=`:`、`/`=`?`），却从未切换布局，
语义一直未落地。

## Decision

在 `tsf.rs` 把模式切换与系统键盘布局挂钩（T-168，对齐搜狗）：

1. `system_layout_for(mode)`——纯函数：英文 → `0x0409_0409`（US 英文）、
   中文 → `0x0804_0804`（中文-美式键盘）。
2. `sync_system_layout(mode)`——以该句柄调用 `ActivateKeyboardLayout`；
   只影响调用线程（Shift/Ctrl+Space 切换路径即宿主 UI 线程），失败静默
   （布局不可用也不影响输入闭环）。
3. 挂接两处切换入口：`sync_engine` 的 `ToggleMode` 分支（Shift 弹起/
   Ctrl+Space/设置回调全部汇聚于此）与语言栏点击处理器（Win10 路径；若
   回调线程非宿主输入线程，布局切换静默无效、无害）。

英文模式因此激活真实英文布局：任务栏出现系统灰色"A/"指示（与搜狗一致），
且 T-066 上档符号映射天然成立。

## Alternatives considered

- **保持引擎内部状态（现状）**：无系统指示，正是用户投诉点；否决。
- **只在键盘路径切布局、语言栏点击不切**：否决——统一走同一辅助函数，
  惰性场景（ctfmon RPC 线程）无害。
- **记忆/还原宿主原布局**：过度设计；搜狗也不做，中文模式还原 00000804
  是双方共识。

## Consequences

- 中文模式 → 系统布局 00000804、无灰色"A/"；英文模式 → 00000409、出现
  灰色"A/"——任务栏可见性与搜狗拉平。
- `ActivateKeyboardLayout` 线程级生效；可靠路径为 TSF 宿主 UI 线程的切换
  （Shift 弹起、Ctrl+Space）；失败静默使输入免疫布局问题。
- 英文布局键映射（T-066）与真实布局真正一致。

## Verification

- 单测「模式对应系统键盘布局」：两常量与映射断言（英文→0x04090409、
  中文→0x08040804）。
- 全部门禁绿：`cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、
  `cargo test --workspace`（ime lib 249、tsf 44、core 352、host_e2e 170）、
  `git diff --check`。
- 文档同步：需求规格说明书 FR-004、验收标准 FR-004（系统英文指示/英文
  布局符号两行）、方案设计 §5.3、CHANGELOG [Unreleased]、todos T-168。
- 已部署（2026-10-18，PR #211 → develop f74619e）：`zhu_ye_ime_A4277E6B.dll`
  （2,260,480 B）落地、指针切换、ctfmon 重启（PID 18940），verify-tsf-dll/
  identity 通过。真机验证（英文模式出现灰色"A/"、切回中文消失、英文模式
  `@`/`:`/`?` 输入正确）待用户确认。
