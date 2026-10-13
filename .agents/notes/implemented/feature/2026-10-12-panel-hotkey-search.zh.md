# Agent Note: 工具箱面板全局热键、搜索态与位置记忆（T-151）

Status: implemented

## Problem

工具箱面板（emoji 面板 / 符号大全）自 T-074 起以非激活置顶弹窗形态工作：
从设置窗口工具箱页打开，点格即把字符 `SendInput` 进用户前台窗口而不抢焦
（`WS_EX_NOACTIVATE`、`WM_MOUSEACTIVATE→MA_NOACTIVATE`）。用户于
2026-10-10 反馈：弹窗式打开在真实打字流里碍手碍脚，希望有更好的交互模型。
确定的方案：**全局热键呼出 + 锚定定位**——`Ctrl+Alt+E` 在光标旁呼出面板、
面板内切 emoji/符号、搜索时临时激活并在选中后自动还焦、位置记忆且首次贴光标。

## Decision

- **热键宿主 = 常驻托盘进程**（`zhu-ye-tray`，HKCU Run 自启）。设置窗口
  关窗即退，热键随它注册等于随它死亡；托盘是唯一保证存活的进程。
  `config.json` 新增 `panel_hotkey: Option<String>`（默认 `ctrl+alt+e`，
  serde default，不升 `CONFIG_FORMAT_VERSION`）。解析在新建的
  `zhu_ye_core::hotkey`：纯数值常量（`MOD_*`、VK），不调用 Win32 API——
  与 `identity` 常量同一先例（S-10）。解析失败的配置回默认并留日志
  （托盘在注册失败/呼出失败时追加写 `%APPDATA%\zhu-ye-ime\
  panel-hotkey.log`，绝不拖垮托盘）。
- **全局单面板**：窗口类 `ZhuYeSettingsPanel` 会话内唯一。两个入口——设置
  窗口打开与托盘 `WM_HOTKEY`——都先 `FindWindowW(CLASS)` 检查（设置页还
  拦自己的 `state.panel`）。热键是 *toggle*：面板已开（任意进程）→
  `PostMessage(WM_CLOSE)` 关闭；未开 → 按 `remembered_kind()`（上次页签，
  无则 emoji）创建。
- **搜索态 = 窄化的临时激活例外**（S-32）：点搜索框移除
  `WS_EX_NOACTIVATE`、把 `GetForegroundWindow()` 记为还焦目标、并以
  `WM_MOUSEACTIVATE→MA_ACTIVATE` 应答，让这一次点击合法地把焦点给面板以
  接收键盘。`WM_CHAR` 累积查询（仅 ASCII——面板不接入 IME，拼音别名过滤
  是主用途；中文组名查询是纯逻辑且已单测覆盖，但窗口层不可达，记为已知
  限制）。`Backspace` 删字符、`Tab` 切页签（查询保留）、`Left`/`Right`
  按**过滤后**结果翻页、`Esc` 清查询并还原 `NOACTIVATE` 还焦目标。搜索态
  选中格子：先还焦再投递；选中后面板回到完全非激活，连续点格上屏不打断——
  这正是 S-11 讨论中否决的"每次点击腾挪焦点"方案的唯一一次例外。
- **位置记忆**（S-33）：`%APPDATA%\zhu-ye-ime\panel.json`（tmp+rename
  原子写，失败静默）在 `WM_DESTROY` 里记 `{kind, left, top, width,
  height}`；下次打开把保存矩形钳入工作区（`SPI_GETWORKAREA`）后复用。
  无记忆 → 贴光标右下 12px（`GetCursorPos`）。截图模式（`--shot-panel`）
  有意不读不写记忆，保证帧可比。
- **过滤是纯逻辑**（`panel::filter_entries`），绘制/命中测试/分页/悬停
  共用：label/group/text 不分大小写子串匹配，空查询返回全量；
  `panel::page_of` 对过滤结果分页（至少一页、越界钳末页）。

## Alternatives considered

- **设置窗口注册热键**：设置窗口不常驻（关窗即无），热键恰在用户打字时
  失效——否决（S-30）。
- **为热键另起常驻进程**：为一次 `RegisterHotKey` 重复托盘的必需常驻，
  托盘是唯一的自启进程——否决。
- **保留弹窗式、只做锚定**：用户明确否决弹窗式作为主路径；热键+锚定才是
  诉求——否决。
- **每次点击都临时激活面板**（每点一次腾挪焦点）：破坏浏览态"永不抢焦"
  属性，点格上屏的即时感全无；只有搜索需要键盘，例外收窄到搜索态——
  否决（呼应 S-11 的否决）。
- **位置记忆并入 `config.json`**：运行时 UI 几何不是用户配置，每次关闭都
  搅动配置文件及其版本规则；独立 sidecar 文件两不相扰——采用。

## Consequences

- `zhu-ye-settings.exe` 与 `zhu-ye-tray.exe` **两个二进制都变**：托盘新增
  对设置库的依赖（单向，settings→core 本就存在，无环）。
- 键盘操作只在搜索态存在；浏览态保持非激活，连续点格上屏焦点始终留在打字
  应用。
- 搜索输入仅 ASCII（面板保持 GDI 单色字形、无 IME 挂钩）——拼音别名是
  主用途；中文组名搜索有测试无窗口可达性。
- 面板页脚按钮由"返回设置"改名"关闭"（托盘宿主没有设置窗口可返回）；
  底部提示行显示从配置读到的生效热键。
- 热键注册失败（如被占用）降级为仅托盘菜单可用 + 一行日志，绝不崩溃。

## Related

- T-074 面板窗口设计：[settings-window-process-and-registration-ownership](../../proposed/architecture/2026-10-01-settings-window-process-and-registration-ownership.md)
- 决策记录 S-30..S-33 在 [docs/设置窗口视觉设计.md](../../../docs/设置窗口视觉设计.md) §14
