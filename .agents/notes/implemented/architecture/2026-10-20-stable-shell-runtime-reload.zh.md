# Agent Note: Stable shell with runtime reload of versioned implementation

Status: implemented

English | [中文](2026-10-20-stable-shell-runtime-reload.zh.md)

## Problem

T-175 的固定名部署让**文件名**与**注册表指针**恒定，但宿主加载进自己地址空间的
仍然是**实现本身**。因此已经打开的宿主会一直持有它映射的那份实现，直到它被关闭。

这个限制有实测代价。`user_words.json` 是所有宿主共享的单一文件，T-174 把它的格式
升到 v2（增 `transitions` 段），但 9 个常驻进程仍持有 **v1** DLL，每次上屏都用全量
覆盖写盘，把 `version` 写回 1 并整体丢弃 `transitions` 段——转移表因此永远无法积累。
根因就是那些 v1 DLL，而**没有任何部署技巧能把它们从已运行的进程里移除**。

要让改动送达已打开的宿主，必须替换宿主**内部**的实现，而不是宿主加载的那个文件。

两条观察约束了设计，二者都取自 `C:\zhu-ye-test\tsf-debug.log`。

**宿主失焦再聚焦时不会重新取类工厂。** `pid=35348` 的日志记录了 1 次取类工厂对应
3 次 `ActivateEx`，跨越 3 个完整焦点周期，其中包含一次真实的 `nihao` →「你好」上屏。
所以搜狗那种 **DLL 级壳**（宿主加载一个薄加载器，再由它加载版本化 DLL）在这里行不通：
点走再点回不会触发任何重新解析。

**外部 TSF 档案 API 无效。** `DeactivateProfile`、`ActivateProfile` 与带
`TF_RIP_FLAG_FREEUNUSEDLIBRARIES` 的 `ReleaseInputProcessor` 全部返回 `S_OK` 且毫无
变化：输入法仍处于 ACTIVE，同样的 9 个进程仍映射着旧 DLL。

确实有一个 API 会重新取类工厂：**切换输入法档案**。`pid=33100` 的日志记录了
`call#1`/`call#2`/`call#3`，第三次在 `DeactivateProfile` 之后。这是一个有效的重载
触发点，但它需要**用户主动操作**——而这正是本方案要消除的东西。

## Decision

宿主加载的那个稳定文件里**不含任何输入法逻辑**。它是一个**壳**，壳持有一个代理
COM 对象，代理内部的实现指针可替换。实现住在版本目录里，由一个指针文件选中。

```
C:\Program Files\zhu-ye-ime\tsf\
├── zhu-ye-ime.dll              ← 壳：宿主唯一加载的文件
└── versions\
    ├── current.txt             ← 指针：当前版本的 8 位十六进制 id
    └── 6fa6af97\
        └── zhu-ye-impl.dll     ← 该版本的实现
```

COM 本身就是稳定的跨模块 ABI，所以代理**不需要自造 ABI**。它转发宿主所查询的
那一组接口，共 24 个方法：`ITfTextInputProcessorEx`（8）、`ITfKeyEventSink`（6）、
`ITfCompositionSink`（1）、`ITfFunction`（1）、`ITfFnConfigure`（1），外加
`ITfTextInputProcessor` 的 `Activate` 与 `Deactivate`。

**共享粒度：模块共享，实现对象**不**共享。** TSF 的文本服务是**每实例独立状态**
的对象：一个进程里通常同时有多个（每个线程 / UI 上下文一个），各自持有编辑会话与
各自注册进 `ITfKeystrokeMgr::AdviseKeyEventSink` 的线程 id。因此壳只共享**实现
模块**（进程级资源——这样换版本时模块只加载一次），而每个代理实例持有**自己的**
实现对象（由该模块建出）。旧模块由 `Arc` 引用计数管理：直到最后一个仍持有它所
创建对象的代理实例换掉，才被 `FreeLibrary` 卸载。

把**对象**也一起共享——壳最初的实现方式，是一个回归——见下节「共享对象回归」。

**发布 = 写 `current.txt`。** 指针先写进 `.staging` 再改名，宿主因此绝不会读到半截
内容。写指针就是发布；不涉及任何注册表写入——`InprocServer32` 与语言档的
`IconFile` 永远指向那个固定名。

**切换发生在 `Activate`。** 每次激活（即宿主每次获得输入焦点），壳读指针并与当前
正在代理的实现比对。若不同，壳加载新实现、构造新的文本服务对象并换入，随后释放旧的
。此时旧实现已经 `Deactivate`、不持有任何编辑会话，因此替换是安全的。指针未变时，
这次检查只花一次很小的文件读取；按键热路径上**不做任何文件 I/O**。

**失败降级而非失效。** 若新实现加载失败——文件缺失、导出不对、模块装不进来——壳
**保留旧实现继续工作**并记录错误。保持可用优先于保持最新。

**品牌图标在壳里。** 固定名正是 `IconFile` 解析到的目标，因此 `build.rs` 把「竹」
图标资源注入到壳 crate 的产物中。

现在有两个 crate：`zhu-ye-ime-shell` 产出壳，`zhu-ye-ime` 产出实现。`zhu-ye-ime`
是无人依赖的叶子 `cdylib`，因此拆分是安全的——壳并不链接实现，而是运行时按路径加载它。

## Alternatives considered

**A DLL-level shell that loads the versioned implementation and re-exports
it.** 这是搜狗的结构，也是最初的方案。它被 `pid=35348` 的证据推翻：宿主在焦点变化
时不会重新请求类工厂，于是壳已加载的模块继续驻留，任何新版本都不会被查阅。之所以
仍把它记为已否决的候选，是因为它是最显然的首选设计，而**正是实测把它排除掉的**。

**Reloading via the external TSF profile APIs.** `DeactivateProfile`、
`ActivateProfile`、`ReleaseInputProcessor` 全部返回 `S_OK` 且毫无作用。这是**实测**
结论，不是假设。

**Making the user switch the input method profile to trigger a reload.** 这确实会
重新取类工厂（`pid=33100`），但需要用户动作，那正是要解决的问题本身。作为主要机制
被否决；不过用户无论如何都会切档案，它仍是一条兜底路径。

**Copying the new implementation over the stable name and hoping the host
re-reads it.** 宿主已经映射了该文件；改磁盘内容不会改变已映射的内容。T-175 已实测：
`ReplaceFileW` 成功、磁盘内容变新版，但已打开的宿主仍持有旧句柄。

**A custom ABI between shell and implementation.** 没有必要：COM 已经是稳定的
跨模块 ABI，而代理本来就要实现宿主自己的那组接口。自造 ABI 只会平白多出一层版本
协商面。

**Swapping the implementation on every keystroke instead of on `Activate`.**
按键热路径不允许文件 I/O 与锁竞争。`Activate` 是宿主释放编辑会话的最早时刻，因此
既廉价又安全。

## Consequences

**已打开的宿主无需重开任何窗口即用上新版。** 版本指针在每次激活时都被查阅，因此
用户下一次点进输入框时，宿主就已经代理到新实现。部署不再需要请用户重启什么——
这正是与搜狗的差别：搜狗自己的 `SGTool.exe` 里带着「立即重启」/「稍后重启」提示，
正因为它的壳做不到这件事。

**共享 `user_words.json` 的冲突不会仅凭本次改动消失。** 那 9 个进程里的 v1 DLL 在
它们重启并加载壳之前，仍会继续写 v1。冲突只有在**所有宿主都跑起壳之后**才消失——
那是计划中的系统重启，不是本任务的事。本任务移除的是让冲突**永久化**的机制；重启
移除的是冲突的**实例**。

**发行包从 1 个 DLL 变成 2 个。** 发行包含 `zhu-ye-ime.dll`（壳）与
`zhu-ye-impl.dll`（实现）。部署脚本、包布局与载荷守卫都已相应更新。

**壳是第二个可能出错的环节。** 畸形指针、缺失实现或损坏模块都不该让输入法瘫掉。
集成测试显式覆盖了畸形指针与缺失实现两条路径，失败路径保留旧实现继续工作。

**`install` 里的释放顺序有讲究。** 必须**先释放 COM 对象，再卸载它的模块**。若先
赋值模块句柄，旧实现的代码会在其 COM 对象仍存活时被卸载，而该对象之后的 `Release`
会落在未映射内存里——访问冲突（`0xC0000005`）。开发中复现过，现已修正并在调用点
注明为何顺序不可反。

## Testing

- `cargo clippy -p zhu-ye-ime-shell --all-targets -- -D warnings` 全绿。
- 壳 crate 的 6 个单测：指针缺失、指针往返、非法版本号被拒且不写盘、盘上垃圾被拒、
  实现路径落在版本子目录下、壳目录可解析。
- 4 个集成测试（`crates/zhu-ye-ime-shell/tests/shell_integration.rs`）：壳能加载
  版本化实现并返回代理；代理能被查询出宿主使用的每一个接口；**在同一进程内换版本
  指针即换实现**（断言旧模块已卸载、新模块已加载）；畸形指针不让壳崩溃；指向不存在
  的实现不让壳崩溃。
- 集成测试已对**部署位真实文件**复测并通过（`C:\Program Files\zhu-ye-ime\tsf\zhu-ye-ime.dll`
  282,112 字节，`versions\6fa6af97\zhu-ye-impl.dll` 2,321,920 字节）。
- 开发机上壳已被真实 TSF 宿主加载，宿主日志记下 `zhu-ye-shell: 已换用实现 6fa6af97`，
  证明跨 DLL 链路在真实宿主里成立，而不只是在测试夹具里成立。
- **真机交互式验收（重启后，在真实宿主上）**。重启释放了仍持有 v1 DLL 的 9 个常驻
  进程。指针停在 `6fa6af97` 时，实测 `explorer` 同时映射了壳与
  `versions\6fa6af97\zhu-ye-impl.dll`。随后**不重启、不关闭任何进程**，发布了一个
  可区分的新实现（指针改为 `73b86903`）。壳日志在任一宿主重新获得焦点时即记下
  `zhu-ye-shell: 已换用实现 73b86903`；此后所有已重新获得焦点的进程实测均映射
  `versions\73b86903\zhu-ye-impl.dll`：

  ```
  explorer (PID 13528) → 73b86903
  explorer (PID 33040) → 73b86903
  msedgewebview2 (PID 25456) → 73b86903
  ```

  较老的那个 `explorer` 实例起初仍持有 `6fa6af97`，在它自己下一次焦点变化时换成了
  `73b86903`——这正是「每进程各自持有实现槽位」的设计在起作用。

## 共享对象回归

壳最初的实现把实现**对象**也放进进程级共享槽位，进程内所有代理实例都转发到
同一个对象。这是错的，并造成了用户可见的缺陷。

**症状**：部署壳后，展开任务栏的隐藏图标箭头，任务栏输入法指示器从「竹叶输入法」
变成「搜狗拼音输入法」。

**机理**：实现把 `AdviseKeyEventSink` 注册用的线程 id 存在**自己的状态**里
（`state.tid`，`Activate` 时写入、`Deactivate` 时据此注销）。当同一进程内多个文本
服务实例共用一个实现对象时，后一个实例的 `Activate` 覆盖了先前实例的 `tid`，
先前实例的按键事件注册便对错了线程注销，TSF 报
`EVENT_E_ALL_SUBSCRIBERS_FAILED`（`0x80040201`「事件无法调用任何订户」）。TSF
认为一个送不出按键事件的文本服务不可用，于是**回退到别的输入法**——这就是指示器
变成搜狗的原因。

**证据**。

- 实现日志：`pid=17512 tid=28728` 连续两次 `Activate key-sink FAILED Error { code:
  HRESULT(0x80040201), message: "事件无法调用任何订户" }`。
- 同一份日志呈现出共享对象造成的激活/失活失衡：线程 `pid=33100 tid=37572` 记录了
  **1 次 Activate 却有 3 次 Deactivate**——每个实例都在失活那一个共享对象。
- 修复后，同样的操作得到 `Activate tid=22 key-sink ok`；连续 5 轮展开隐藏图标箭头，
  指示器始终停在「竹叶输入法」，`key-sink FAILED` 记录为 0。

**修法**。模块通过 `Arc` 共享；每个代理实例持有自己的实现对象，外加创建它的模块
的 `Arc` 引用。版本变化时共享槽位指向新模块，各代理在下一次激活时重建**自己**的
对象。`Mine::drop` 先释放 COM 对象再放模块引用，因此模块不会在仍有其对象存活时
被卸载。

回归测试（`多个文本服务实例持有各自独立的实现对象`）断言实现的每次 `CreateInstance`
给出的是**不同**对象。

**同一缺陷还导致 explorer 崩溃。** Windows 错误报告记录了 `Explorer.EXE` 在
`MSCTF.dll` 中发生 `0xc0000005` 访问冲突，时间 2026-10-21 09:11:11——正是在复现
上述症状时。共享对象让实现注销了**错误线程**的按键事件注册，TSF 自身的状态因此
失配，下一次回调进 TSF 就发生访问冲突。这解释了症状为何表现为「输入法被切走」：
explorer 崩溃 → 通知区重建 → 任务栏重新选择了一个输入法。修复后同一个 explorer
实例在整个验证过程中保持存活，没有新的崩溃记录。

**另一个更早的失败模式**。同一份日志里还有 2026-10-20 10:38 与 11:15 的
`Activate key-sink FAILED Error { code: HRESULT(0x80070057), message: "参数错误。" }`
——那是在壳存在**之前**，属于另一种情况（`ActivateEx` 带来的 `tid=0`，而
`AdviseKeyEventSink(0, …)` 拒绝该参数）。它不是本回归，此处未做改动；记录在此是
为了读日志时不要把两者混为一谈。

## 本次工作中发现的部署缺陷

**清理过滤器漏掉文件名中间带标记的文件。** 过滤器用的是 `-like 'zhu_ye_ime_*.dll'`
与 `-like 'zhu_ye_ime_*.dll.zy-del*'`，两者都**锚定在文件名结尾**。被改名探针在
**中间**打过标记的文件（开发机实测为
`zhu_ye_ime_937632A1.dll.renamed.zy-del.zy-del.zy-del.zy-del`）四个模式全部落空，
于是每轮清理都跳过它、永久留在盘上。判定改为**前缀**（两种历史版本化命名任一）加上
**文件名中任意位置含 `.dll`**，中间叠加任何标记都能覆盖。固定名壳 `zhu-ye-ime.dll`、
版本化实现 `zhu-ye-impl.dll` 与旧下划线固定名 `zhu_ye_ime.dll` 仍**不命中**该判定，
因此不会被误删。

**单元素返回值让清理「报了失败」。** worker 对清理函数的返回值取 `$stale.Count`。
当该函数恰好返回一个路径时，PowerShell 会把单元素数组拆包成标量，对得到的 `String`
取 `.Count` 会抛「在此对象上找不到属性"Count"」——即**清理其实成功了却报成失败**。
worker 现已用 `@()` 包裹该调用。

## Related

- [Stable DLL shell deployment](2026-10-20-stable-dll-shell-deployment.md)
  （T-175）——本方案所基于的恒定文件名与注册表指针。其 `ReplaceFileW` 替换、
  `versions\<sha8>\` 归档与延迟清理机制继续有效；变化之处在于归档内容现在是实现，
  而固定名现在是壳。
- [Versioned DLL deployment and dictionary location](2026-09-23-versioned-dll-deployment-and-dictionary-location.md)
  （T-026）——被 T-175 部分取代，其引入的文件命名在此再次被取代。
- [Taskbar icon 「竹」](2026-10-06-taskbar-icon-zhu.md)——图标资源现在嵌在壳里，
  因为固定名解析到的是壳。
