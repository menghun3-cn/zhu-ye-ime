# Agent Note：语言栏图标随实际中英模式显示"中"/"英"（T-046）

Status: implemented

English | [中文](2026-09-28-language-bar-mode-icon.zh.md) | 中英镜像

## Problem

用户要求任务栏语言栏图标（Language Bar Icon）反映输入法的**实际激活状态**：
中文模式显示"中"、英文模式显示"英"，随 Shift 切换实时变化。Windows 没有
纯注册表方案让文本服务的语言栏图标动态变化——TIP 的图标来自注册表
`IconFile` 或语言栏项目，只有**语言栏项目**能在运行期刷新。

## Decision

每激活线程注册一个**自定义 TSF 语言栏项目**——微软文档标注的语言栏动态
UI 正道机制：

- 项目：`ITfLangBarItemButton`（含 `ITfLangBarItem`）+ 自实现 `ITfSource`。
  `ITfTextInputProcessor::Activate` 经 `ITfThreadMgr → ITfLangBarItemMgr::
  AddItem` 注册，`Deactivate` 用 `RemoveItem` 注销。每个激活线程实例
  注册自己的项目，图标因此始终跟随该线程引擎的实际模式。
- 通知：MSDN 指出实现 `ITfLangBarItem` 的对象可暴露 `ITfSource`，语言栏
  管理器经它 advise `ITfLangBarItemSink`。引擎模式变化时（`tsf.rs`
  `sync_engine` 的 `ToggleMode` 分支），项目在**引擎锁外**调用已存 sink 的
  `OnUpdate(TF_LBI_ICON)`；语言栏随即重查 `GetIcon`/`GetInfo` 并重绘。
  `windows` 0.61 未绑定 `ITfSource`，故在 `lang_bar.rs` 用 `define_interface!`
  + 手写 Vtbl 补定义；IID `4EA48A35-60AE-446F-8FD6-E6A8D82459F7` 取自
  `msctf.h` 的 `MIDL_INTERFACE`，不是猜测值。
- 图标运行期 GDI 绘制，16×16：32bpp 自顶向下 DIB（`CreateDIBSection`，
  `biHeight = -16`，BI_RGB）整块填充底色，再用白字宋体粗体
  （`CreateFontIndirectW` + `DrawTextW`，`DT_CENTER | DT_VCENTER |
  DT_SINGLELINE | DT_NOPREFIX`）写字；掩码为**同尺寸** 1bpp 全 0 位图——
  `CreateIconIndirect` 要求掩码与颜色位图尺寸一致（不一致报 E_INVALIDARG），
  32bpp 图标的透明由颜色位图 alpha 通道决定。句柄进程期存于 `OnceLock`
  且**永不删除**：`CreateIconIndirect` 引用这两张位图，删除即释放后使用
  风险。
- 外观：中=品牌蓝 `#1E88E5`、英=中性灰 `#757575`，白字；项目样式
  `TF_LBI_STYLE_BTN_BUTTON | TF_LBI_STYLE_SHOWNINTRAY`（Win10 托盘/语言栏
  可见；Win11 默认任务栏指示器不渲染第三方项目，见 Consequences）。
- 注册失败非致命（仅记日志、不中断 `Activate`）：ctfmon/语言栏不可用时
  输入照常工作。
- **点击切换刻意延后**：`OnClick` 返回成功但不动作。引擎状态是
  `Rc<Mutex<EngineState>>`（非 `Send`），ctfmon 可能从任意 RPC 线程进入
  项目方法；做切换需先迁移到 `Arc<Mutex<… Send>>`。记为后续增强。

## Alternatives considered

**仅注册表 `IconFile`/类图标。** 否决：图标按 TIP 解析，不区分线程/模式；
没有受支持的运行期刷新路径，且 ctfmon 缓存期间改 HKCU 很脆弱。

**不带 `ITfSource`（不安装 sink）的项目。** 否决：没有 `ITfLangBarItemSink`
项目就无法通知语言栏重绘，语言栏会一直显示初始图标——正是本任务要消灭
的病。

**`OnClick` 直切引擎。** 延后（见 Decision）：`Rc` 引擎状态非 `Send`，
从 ctfmon 任意回调线程进引擎互斥锁违反现有线程模型安全假设。

## Consequences

- 图标只在语言栏真正渲染项目的场景跟随真实模式：桌面语言栏与 Win10 托盘
  可用；Win11 默认任务栏输入指示器不渲染第三方语言栏项目——安装与使用
  文档已指导开启"使用桌面语言栏"。
- `lang_bar.rs` 承载项目、图标、`ITfSource` 胶水与单测；`tsf.rs` 只接线
  Activate/Deactivate 与 `ToggleMode` 通知。
- 验证：新增 10 项单测不依赖 ctfmon 直接驱动项目（图标/文本随模式、
  模式切换发 `OnUpdate(TF_LBI_ICON)`、riid 过滤、重复 advise 替换、
  unadvise 后静默、`GetInfo` 身份、`GetIconInfo` 回读图标句柄、
  `OnceLock` 单例）；crate 全部 79 项 lib 测试与 host-e2e（种子 19 项 +
  真实词典 6 项）全绿；`clippy -D warnings` 与 rustfmt 干净。OS 级显示
  验收（VM 桌面语言栏截图）仍为人工验收步骤。
- 交叉参考：模式切换键接线——
  [2026-09-20-candidate-window-tsf-integration](2026-09-20-candidate-window-tsf-integration.zh.md)；
  组合外 Shift 门控——
  [2026-09-25-page-keys-minus-plus](2026-09-25-page-keys-minus-plus.zh.md)；
  MSDN：[ITfLangBarItemButton](https://learn.microsoft.com/zh-cn/windows/win32/api/ctfutb/nn-ctfutb-itflangbaritembutton)、
  [ITfLangBarItemSink::OnUpdate](https://learn.microsoft.com/zh-cn/windows/win32/api/ctfutb/nf-ctfutb-itflangbaritemsink-onupdate)、
  [ITfSource](https://learn.microsoft.com/zh-cn/windows/win32/api/msctf/nn-msctf-itfsource)。
