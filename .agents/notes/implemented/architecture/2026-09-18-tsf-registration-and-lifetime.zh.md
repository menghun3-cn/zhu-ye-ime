# Agent Note: TSF registration and COM lifetime for M1

Status: implemented

[English](2026-09-18-tsf-registration-and-lifetime.md) | 中文

## 问题

TSF 文本输入处理器只有在 Windows 能发现其 CLSID、在宿主进程中加载 DLL，并在最后一个引用释放后卸载时才是可用的。注册契约横跨 Rust 代码、安装脚本与系统注册表；若身份不固定、生命周期未验证，安装或卸载输入法时会破坏其他文本服务，或留下陈旧引用。

## 决策

`zhu-ye-ime` 是名为 `zhu-ye-ime.dll` 的 Rust `cdylib`，安装目录中的 DLL 只导出两个 COM 入口 `DllGetClassObject` 与 `DllCanUnloadNow`，以及开发探针 `dll_probe`；全部使用 `#[no_mangle] extern "system"`。

TIP 身份固定：CLSID 为 `{E54D6682-8650-40E7-A9EE-6FD1137849AE}`，zh-CN 语言 Profile 为 `{6315FE74-92C3-439B-8CDF-FDB6E43EDAF1}`，键盘类别为 `{34745C63-B2F0-4784-8B67-5E12C8701A31}`。Rust 侧单一主源位于 `crates/zhu-ye-core/src/identity.rs`（T-081 迁入，见 [2026-10-02-zhu-ye-ui-crate-and-rlib-removal.zh.md](2026-10-02-zhu-ye-ui-crate-and-rlib-removal.zh.md)），`zhu_ye_ime::tsf` 转发；PowerShell 侧 `scripts/ime-identity.ps1` 与主源保持同步，由 `scripts/verify-tsf-identity.ps1` 交叉比对。

注册表由 `scripts/install.ps1` 与 `scripts/uninstall.ps1` 管理，DLL 本身永不写注册表。安装写入 HKLM TIP 键、键盘类别的 `Category` 与 `Item` 键、`LanguageProfile\0x00000804\{ProfileGuid}`（`Enable=1`），以及 `SOFTWARE\Classes\CLSID\{Clsid}\InProcServer32`（`ThreadingModel=Apartment`）。卸载删除两个顶层键，缺失时静默跳过。两条操作均幂等，安装会先清理旧注册再重建，保证不残留旧 DLL 路径。

COM 生命周期显式管理：类工厂实现 `IClassFactory`，通过 `CLASS_E_NOAGGREGATION` 拒绝聚合；仅当活动对象数与 `LockServer` 计数同时为零时，`DllCanUnloadNow` 才返回 `S_OK`。由于文本服务没有 control-unknown 语义，不提供聚合；空的 outer 指针按 COM 非聚合创建正常接受。

M1 仅面向 x64 的 HKLM 进程内 COM，DLL 部署到 `C:\Program Files\zhu-ye-ime\tsf\zhu-ye-ime.dll`。M1 不实现按键处理、组合、显示属性或候选窗；这些由 T-011、T-012 接入。

## 注册契约

```text
HKLM\SOFTWARE\Microsoft\CTF\TIP\{TipClsid}
  Category\Category\{34745C63-B2F0-4784-8B67-5E12C8701A31}\{TipClsid}
  Category\Item\{TipClsid}
    Description = 竹叶输入法
  Category\Item\{TipClsid}\{34745C63-B2F0-4784-8B67-5E12C8701A31}
  LanguageProfile\0x00000804\{ProfileGuid}
    Description = 竹叶输入法
    Display Description = 竹叶输入法
    Enable = 1 (DWORD)
    IconFile = <dll 路径>
    IconIndex = 0 (DWORD)
HKLM\SOFTWARE\Classes\CLSID\{TipClsid}
  (Default) = 竹叶输入法
  InProcServer32
    (Default) = <dll 路径>
    ThreadingModel = Apartment
```

该布局对照真实 Windows 11 HKLM TIP 树验证；`scripts/verify-tsf-dll.ps1` 负责导出符号校验，`Test-TsfRegistration` 负责安装后的 Profile 与 InProcServer32 校验。

## 便携测试打包

`scripts/package-portable.ps1` 在已忽略的 `target/portable/` 下生成 zip，内含 release DLL、四个安装/卸载/校验脚本与简短测试说明。测试机无需安装 Rust，解压后执行 `scripts/install.ps1 -SkipBuild`，即可在干净或带快照的 Windows 机器上先验证 TSF 注册与卸载，再决定是否改动开发主机。

## 曾考虑的替代方案

**像微软拼音一样使用 `LocalServer32` 注册。** 落选：进程外 TSF 服务会引入 IPC、生命周期与服务管理成本，M1 没有收益；项目架构本来就面向进程内 DLL。

**安装时由 Rust DLL 直接写注册表。** 落选：低层注册表写入属于安装器职责，PowerShell 更易回滚与排查，DLL 也无需提权即可被普通进程加载。

**每台机器或每次安装动态生成 GUID。** 落选：稳定的语言 Profile、更新兼容与可验证卸载都依赖固定身份。

**同时注册 32 位与 64 位宿主。** 落选：M1 只面向 x64；后续可补 Wow6432Node 注册，不改变 DLL 契约。

**手写 vtable 实现 COM 接口。** 落选：`windows` 0.61 生成绑定提供更安全的 `#[implement]` 包装、更小表面积与相同 ABI；直接依赖 `windows-core` 可解析宏展开，且不暴露实现细节。

**支持 COM 聚合。** 落选：TSF 文本服务没有 control-unknown 需求，拒绝聚合让生命周期与测试保持简单。

## 后果

安装与卸载现在构成可重复、幂等、带导出与注册校验的闭环，失败或陈旧部署可通过重跑卸载恢复。单元测试覆盖类工厂创建、拒绝聚合、探针导出与卸载状态，不触碰系统注册表。

脚本需要管理员权限；`verify-tsf-dll.ps1` 使用 `DONT_RESOLVE_DLL_REFERENCES` 映射 DLL，构建期校验不会执行 `DllMain`。真实宿主激活仍需一次 Windows 注册运行，由 T-010 跟踪；在完成那次安装且输入法出现在系统语言列表之前，T-010 保持"进行中"。T-011 接入首批按键输入与文本上屏路径。
