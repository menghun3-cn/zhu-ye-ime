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

## Decision

宿主只加载一个**文件名永不变**的文件，注册表指针也不再变化。

```
C:\Program Files\zhu-ye-ime\tsf\
├── zhu_ye_ime.dll                    ← 稳定文件，宿主唯一加载的文件
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

## 决策范围

本决策只覆盖**稳定文件身份与升级机制**，**不**让已打开的宿主运行新代码，
也并非以此为目标。详见下节「保证的边界」。

## 保证的边界

已经加载该文件的宿主，会继续执行加载那一刻映射进去的字节。这不是实现上的
缺陷，而是 Windows 加载器的既定行为。

迁移后的开发机实测：仍有 9 个进程映射着旧 DLL（notepad++ →
`zhu_ye_ime_9ACBCEF1.dll`、Weixin → `zhu_ye_ime_A4277E6B.dll` 等），而
**这些进程映射的文件在磁盘上已经不存在**——被清理路径删掉了。进程仍然从
Windows 为其保留的内存页里执行旧代码，直到进程退出。

进程内重载在技术上可达：受控实验显示，同一进程内 `FreeLibrary` 后再
`LoadLibraryW`，会拿到已被替换的新内容。但在真实宿主里做到这一点，需要宿主
释放它持有的全部 text service COM 引用并重新激活输入法——那是宿主的决定，
不由我们决定。TSF 侧的对应入口是
`ITfInputProcessorProfileMgr::ReleaseInputProcessor`（`msctf.h`）。

## 搜狗的结构（实测）

因为用户反馈「搜狗输入法自动更新后貌似不要求用户重新打开宿主文件」，本机
实测了搜狗 15.4.0.1862。测出的结构是三层，而最上面那层恰好是本决策**没有**
的那一层：

| 层 | 文件 | 由谁加载 | 升级后 |
| --- | --- | --- | --- |
| 稳定壳 | `C:\WINDOWS\system32\SogouTSF.ime`（4,672,832 B） | 所有宿主，经 `CLSID {E7EA138E-…}` | 永不变化 |
| 输入法实现 | `D:\tools\SogouInput\15.4.0.1862\SogouTSF.dll`（96 KB）+ `ImeFunc.dll`（2,644 KB） | **壳在运行时加载** | 可替换 |
| 资源与辅助 | `Resource.dll`、`SearchCand.dll` 等 | 部分宿主直接映射 | 部分变旧 |

壳自带加载器——`UserEnv::t_envContainer::t_loader::Load`、`GetEnv`、
`CopyFrom`、`DumpFrom` 等字符串就在壳里，同时有 `LoadLibraryA`、
`LoadLibraryW`、`LoadLibraryExW`、`GetProcAddress`——并经
`Software\SogouInput` 定位版本目录。壳引用的是**不带路径的** `SogouTSF.dll`，
而该 DLL 只存在于版本目录里，所以壳是在加载时解析版本目录并从那里加载实现。
组件目录也按组件各自版本化（`Components\IChat\` 下并存 `1.0.2.3040`、
`1.0.2.3099`、`1.0.2.3127`），这正是「按进程切换版本」得以成立的前提。

关键区别在于：**宿主依赖的是壳，而加载实现的是壳。** 像本项目这样让宿主直接
加载实现，宿主就会在加载那一刻把实现冻结在自己的地址空间里，无法原地升级。

搜狗自己的二进制同样暴露了它剩下的缺口：`SGTool.exe` 里有「以下程序使用了
Sogou输入法,要避免重启,请先关闭这些程序.」与「立即重启」/「稍后重启」，
所以壳的版本切换也不是覆盖所有进程。

## Alternatives considered

- **保留版本化文件名，只补清理**：能解决目录膨胀，但解决不了指针漂移，用户
  仍要自己猜哪些窗口在跑旧代码。而且部署目录的清单会变成构建历史，而不是
  「当前加载的是什么」的陈述。
- **写转发壳，运行时再加载版本化实现**（搜狗的结构）：这是唯一能让**已打开的
  宿主**也换到新实现的结构，也是本决策的自然后继。本轮未采用，原因有二：
  稳定文件身份与目录清理问题本身就值得单独解决；而且壳要真正兑现价值，还得
  先验证宿主侧的「释放 + 重新激活」路径可行。壳必须是独立产物，转发每一个 TSF
  入口，并且它解析的版本指针必须是**按进程读取**的，不能固化在壳里。
- **每次升级后要求用户关闭所有使用输入法的程序**（搜狗弹的那个提示）：体验比
  现有布局更差，而且在本方案下并无必要。
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
- 已打开的宿主保持它启动时的实现。
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
- 开发机第二次升级（新构建 `2A2AC376…`）：指针仍为 `zhu_ye_ime.dll`，磁盘
  文件已换成新构建。
- 进程内重载实验：持有该文件的进程执行 `FreeLibrary` + `LoadLibraryW` 后，
  观察到已被替换的新内容。

## Related

- [版本化 DLL 部署与词典定位](../architecture/2026-09-23-versioned-dll-deployment-and-dictionary-location.md)
- [个人化候选排序](../feature/2026-10-20-personalized-candidate-ranking.md)
