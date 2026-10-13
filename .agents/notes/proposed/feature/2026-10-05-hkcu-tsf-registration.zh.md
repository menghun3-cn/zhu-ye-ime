# Agent Note：HKCU per-user TSF 注册——免 admin 的验收通道（批三第 5 项）

Status: proposed

[English](2026-10-05-hkcu-tsf-registration.md) | 中文

## 问题

语言栏"中/英"图标目视验收项（T-100/T-046）受阻：构建主机无管理员权限
（`admin=False`），`install.ps1` 的 HKLM TSF 注册（`HKLM\SOFTWARE\Microsoft\CTF\TIP`
+ `HKLM\SOFTWARE\Classes\CLSID`）无法执行；而验收 VM（Server 2019）没有桌面语言栏
（T-046 结论）。需要在本机 Win11 26200 上做一次用户批准、完全可回滚的试验，要么完成
目视验收，要么把确切的阻塞条件固化下来。

## 方案

TSF 文本输入处理器同样支持 per-user 注册：把 HKLM 布局镜像到 HKCU（无需提权）。

- DLL 与词典放用户目录：`%LOCALAPPDATA%\zhu-ye-ime\tsf\`（版本化
  `zhu-ye-ime-<sha8>.dll` + `dictionary.zyct`）；基础包目录解析
  （`tsf.rs resolve_base_dir`）会回退到 `%APPDATA%\zhu-ye-ime`，故 `en.zyen`
  等包放那里。
- `HKCU\SOFTWARE\Microsoft\CTF\TIP\{TipClsid}` 与 HKLM 同构子树：
  `Category\Category\{KeyboardCategoryGuid}\{TipClsid}`（空键）、
  `Category\Item\{TipClsid}\{KeyboardCategoryGuid}\Description`、
  `LanguageProfile\{0x00000804}\{ProfileGuid}`（`Description`/`Display Description`/
  `Enable`(DWORD 1)/`IconFile`=DLL/`IconIndex`(DWORD 0)）。
- `HKCU\SOFTWARE\Classes\CLSID\{TipClsid}`（默认值=显示名）+ `\InprocServer32`
  （默认值=DLL 路径，`ThreadingModel`=`Apartment`）。
- 免 admin 取证：`%APPDATA%\zhu-ye-ime\config.json` 设 `log_level: "debug"` 开启
  产品轨日志（`%LOCALAPPDATA%\zhu-ye-ime\logs\ime.log`）——`C:\zhu-ye-test`
  哨兵路径需要 admin 可写根目录。
- 不经窗口焦点、直接证明系统接受了注册：`msctf.dll!TF_CreateInputProcessorProfiles`
  （导出函数，无需 COM 类 GUID）拿到 profiles 对象，按 `ITfInputProcessorProfiles`
  QI——权威 IID 取自本地 windows-0.61.3 crate（`Windows.Win32.UI.TextServices`）：
  **IID_ITfInputProcessorProfiles = {1F02B6C5-7842-4EE6-8A0B-9A24183A95CA}**——然后调
  `EnableLanguageProfile(clsid, 0x0804, profile, 1)` 与 `ActivateLanguageProfile`。
  方法顺序（IUnknown 之后 vtable 序号）同源 crate：Register、Unregister、
  AddLanguageProfile、RemoveLanguageProfile、EnumInputProcessorInfo、
  GetDefaultLanguageProfile、SetDefaultLanguageProfile、**ActivateLanguageProfile**、
  GetActiveLanguageProfile、GetLanguageProfileDescription、GetCurrentLanguage、
  ChangeCurrentLanguage、GetLanguageList、EnumLanguageProfiles、EnableLanguageProfile、
  IsEnabledLanguageProfile。
- 回滚即逆操作脚本：删两个 HKCU 树、还原默认输入法 override、删除
  config.json/用户目录、重启 ctfmon。

## 结果（本机 Win11 26200，会话 1，无 admin）

- `EnableLanguageProfile` → S_OK、`GetCurrentLanguage` → 0x0804：系统接受了 HKCU
  注册的 TIP。
- `ctfmon` 成功加载用户 DLL 并实例化引擎（`ime.log` 出现多线程 dict 初始化行）；
  DLL 导出校验通过。
- 本机已有的两个第三方 HKCU TIP（`{81D4E9C9…}`、`{8613E14C…}`）佐证该通道在系统
  上确被活跃使用。
- 清场后主机完全复原：HKCU TIP 恢复为原两个键、`Get-WinDefaultInputMethodOverride`
  恢复为未设置、ctfmon 重启为单实例。

## 目视验收仍未达成（两个独立原因）

1. **本会话没有交互输入焦点。** `GetForegroundWindow` 恒为残留值或 0；
   `SetForegroundWindow` 被拒绝；ALT 模拟与 `SPI_SETFOREGROUNDLOCKTIMEOUT=0` 均无法
   解锁。三种宿主尝试全部失败（cmd/conhost 窗口 0x0 尺寸；自建 WinForms STA
   runspace 窗口能创建但永远拿不到前台）——无法注入真实按键，候选窗截图不可得。
2. **Windows 11 移除了经典桌面语言栏。** 注册
   `HKCU\Software\Microsoft\CTF\LangBar\ShowStatus` 不会产生语言栏窗口（与 Server
   2019 观察一致）。因此 T-046/T-100 的语言栏按钮（`ITfLangBarItemButton`）在 Win11
   上没有可见 UI；只有系统托盘输入指示，而它不承载第三方语言栏项目。

## 备选方案

- **提权后走 HKLM 安装器**：否决——构建主机无提权路径；仅为了验收向用户要管理员
  权限违背对用户主机的"最小触碰"原则。
- **等有权限的 VM（Server 2019 验收 VM）做目视**：否决——该 VM 无桌面语言栏，且
  无真实键入所需的交互焦点（前几轮已证明 `ShowStatus` 在那里无效）。
- **把二进制送到另一台 Win10/11 客户端**：当时没有可用机器；本机 Win11 是唯一
  客户端级机器，因此在其上做可回滚试验信息量最大，卸载脚本与试验同目录保存。
- **无窗口的低层输入注入伪造按键**：不可行——TSF 需要前台文本服务客户端线程；
  没有会话输入焦点就没有任何可用的注入路径。

## 验收标准

- 仅在 HKCU 注册 TIP 时，`ITfInputProcessorProfiles::EnableLanguageProfile(clsid,
  0x0804, profile, 1)` 返回 S_OK（系统接受 per-user 通道）。
- 调用序列后 `GetCurrentLanguage()` 返回 0x0804。
- `log_level: "debug"` 配置下 `ctfmon` 加载用户目录 DLL 且引擎初始化成功
  （产品日志出现多线程 dict 初始化行）。
- 部署副本通过 DLL 导出校验。
- 清场后主机完全复原：HKCU TIP 树仅余原来的键、
  `Get-WinDefaultInputMethodOverride` 恢复为未设置、`config.json`/用户目录删除、
  `ctfmon` 单实例运行。
- 受阻的目视项以确切阻塞原因记录（无输入焦点；Win11 无经典语言栏），不静默丢弃。

## 风险

- 不要在无法回滚的机器上跑 HKCU 试验；卸载脚本与试验同目录保存。注册表写入均为
  用户作用域，随配置文件消失。
- 重启 `ctfmon` 会短暂中断输入会话；空闲桌面无影响。
- 未来若变更 TIP CLSID/profile GUID，HKLM 安装器与本通道必须同步更新
  （单一来源：scripts/ime-identity.ps1）。
