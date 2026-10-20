# Agent Note：固定名壳部署

Status: implemented

[English](2026-10-20-stable-dll-shell-deployment.md) | 中文

## Problem

TSF 文本服务 DLL 原按内容指纹命名部署（`zhu_ye_ime_<sha8>.dll` /
`zhu-ye-ime-<sha8>.dll`），每次升级都把注册表指针切到另一个文件。由此产生
两个问题。

**注册表指针每次升级都变。** TSF 注册（`CLSID\<tip>\InprocServer32` 与
语言档 `IconFile`）由宿主在激活输入法时读取。升级后新开的宿主用上了新 DLL，
但**已经打开的宿主在整个生命周期内继续用旧实现**。于是升级只有一半可见：
用户在新开的窗口看到新行为，在手头正看着的那个窗口看到旧行为。

**部署目录累积所有版本。** 升级路径没有清理环节，目录因此无界增长。开发机
实测：49 次部署后目录里 50 个 DLL、合计 106.2 MB，其中 48 个已不可达。

搜狗也是这个行为，它自己的二进制里写着（`SGTool.exe`，搜狗 15.4.0.1862）：

- 「以下程序使用了Sogou输入法,要避免重启,请先关闭这些程序.」
- 「本设置重启应用程序后生效」
- 「上一次的操作需要重启才生效」
- 「立即重启」/「稍后重启」
- 「重启后才能生效」

所以搜狗同样不热替换运行中的代码。它保留一个稳定壳
（`C:\WINDOWS\system32\SogouTSF.ime`）与版本化实现
（`D:\tools\SogouInput\<版本>\`），并告诉用户哪些程序还占着旧版。

## Decision

宿主只加载一个**文件名永不变**的文件，注册表指针也不再变化。

```
C:\Program Files\zhu-ye-ime\tsf\
├── zhu_ye_ime.dll                    ← 稳定壳，宿主唯一加载的文件
└── versions\
    └── <sha8>\
        ├── zhu_ye_ime.dll            ← 该版本的实现归档
        └── .zhu_ye_ime.dll.replaced  ← 本次升级被替换掉的实现
```

升级把新构建复制进 `versions\<sha8>\`，然后用 `ReplaceFileW` 替换稳定文件。
注册表**不动**——指针本来就是 `zhu_ye_ime.dll`，此后一直如此。

`ReplaceFileW` 正是为这件事设计的 Windows API——替换正在被使用的文件。它为
已打开该文件的进程保留旧的段映射，同时把新内容放到磁盘上，供此后打开该
文件的进程使用。本机实测：

- 在 chrome 与 msedge 已映射 `zhu_ye_ime_937632A1.dll` 的情况下替换成功；
  两个进程保持旧映射，磁盘上文件变为新构建（SHA-256 `9D8188AC…`）。
- 新进程加载稳定文件成功并映射 `zhu_ye_ime.dll`（句柄
  `140712931753984`，Win32 0）。

有一处平台细节决定了实现方式：`ReplaceFileW` 的 `lpBackupFileName` 必须给
**真实路径**——传 `NULL` 或空串在本机返回 `ERROR_PATH_NOT_FOUND`（Win32 3），
这与 `scripts/ime-identity.ps1` 里已记录的 `MoveFileEx` 属同一类平台限制。
因此备份路径指向版本归档目录，顺带把被替换掉的实现留了下来。

## Alternatives considered

- **保留版本化文件名，只补清理**：能解决目录膨胀，但解决不了指针漂移，用户
  仍要自己猜哪些窗口在跑旧代码。而且部署目录的清单会变成构建历史，而不是
  「当前加载的是什么」的陈述。
- **写转发壳，运行时再加载版本化实现**（搜狗的结构）：壳可按进程解析版本
  指针，但要多构建、多签名、多调试一个 DLL，且所有 TSF 入口都要穿一层转发。
  `ReplaceFileW` 能达到同样的用户可见结果——新开的宿主跑新代码——而无需这层
  机制，多出来的东西换不到任何可观察的收益。
- **每次升级后要求用户关闭所有使用输入法的程序**（搜狗的做法）：体验比现有
  布局更差，而且在本方案下并无必要。剩下的唯一缺口是已打开的窗口仍跑旧代码，
  这是运行中进程的固有属性，搜狗也一样，并且它用提示来让用户知晓。
- **用 `CopyFile` 原地覆盖稳定文件**：实测否决。文件被运行中的宿主映射，
  写入被拒绝（`Access to the path … is denied`）。
- **先把稳定文件改名让路，再把新文件复制进空出来的名字**：可行——重命名被
  映射的 DLL 在本机成功（实测把 `zhu_ye_ime_937632A1.dll` 从 chrome 与 msedge
  底下移走）——但它会留下一段时间「没有稳定文件」的窗口，且比 `ReplaceFileW`
  更脆弱，后者是原子的。

## Consequences

- 注册表指针现在恒定不变。升级不再改写 `CLSID\<tip>\InprocServer32` 与语言档
  的 `IconFile`。
- 新开的宿主总是运行当前构建，无需重启外壳或输入法框架。
- 已打开的宿主保持它启动时的实现。这是运行中进程的性质，与部署方式无关；
  搜狗同样是这个性质，只是用提示把它暴露出来。
- 部署目录只保留一个稳定文件 + 有界的版本归档。`Remove-TsfOldVersionDlls`
  保留最近 3 份归档并清扫其余。
- 仍被运行中宿主持有的文件无法删除，只能改名。沿用既有的
  `Add-TsfDelayedCleanup` 路径：改名为 `*.zy-del`，再用
  `MoveFileEx(MOVEFILE_DELAY_UNTIL_REBOOT)` 登记重启后清理。开发机上剩余的
  5 个文件正处于该状态——它们被 chrome、msedge、explorer、Weixin、Feishu、
  notepad++、Snipaste、WindowsTerminal 与本工具自身持有。

## Verification

- 被映射文件上的 `ReplaceFileW`：给真实备份路径成功（`Win32=203`），传
  `NULL` 失败（`Win32=3`）。
- 被映射文件的重命名：成功，且持有该文件的进程保持旧映射。
- `install.ps1` 演练到临时目录：首次安装产出 `zhu_ye_ime.dll` 与
  `versions\9d8188ac\`；第二次用不同内容安装时替换了稳定文件并归档新版本。
- 稳定文件被另一进程持有时的演练：替换仍然成功。
- 替换后文件的 `verify-tsf-dll.ps1`：PE 头校验通过，三个导出
  （`DllGetClassObject`、`DllCanUnloadNow`、`dll_probe`）全部解析成功。
- 迁移后的开发机：注册表指针为
  `C:\Program Files\zhu-ye-ime\tsf\zhu_ye_ime.dll`，该文件 SHA-256 与 release
  构建一致（`9D8188AC…`）；新进程映射 `zhu_ye_ime.dll`。

## Related

- [版本化 DLL 部署与词典定位](../architecture/2026-09-23-versioned-dll-deployment-and-dictionary-location.md)
- [个人化候选排序](../feature/2026-10-20-personalized-candidate-ranking.md)
