# Agent Note：TSF 诊断日志产品化（T-091）

状态：已实现

[English](2026-10-04-productized-file-logging.md) | 中文

## 问题

FR-060 要求产品级诊断日志。验收期文件日志（`C:\zhu-ye-test\tsf-debug.log`，
哨兵 `C:\zhu-ye-test\tsf-debug.enable` 存在时经 `debug_log` 无条件全量写）
在用户机器上不存在，因此错误只能走调试器输出通道（`OutputDebugStringW`），
现场完全不可见；任何只记日志的失败路径在验收后都会变成完全无法诊断。
决策 D-72/D-73 锁定了形态：默认 `warn` 级别、热路径零文件写（D-72），
以及双轨哨兵让验收路径与 vm-accept-sop 断言的字节级行为完全一致（D-73）。

## 决策

三层结构：

- **core（纯 std，零 Windows API）：**
  - `zhu_ye_core::log_level::LogLevel` — `Error < Warn < Info < Debug`
    （`Ord` 支撑级别门），`parse()` 大小写不敏感、未知/空回退 `Warn`，
    serde 宽松反序列化（缺失/非字符串/未知值 → `Warn`，与 `theme`
    放宽同款），序列化写小写 `as_str()` 文本。
  - `zhu_ye_core::file_log::FileLogger` — 无内部锁（TSF 侧用 `Mutex`
    包裹）：`new(path, level)`、`with_size_limit`（测试注入）、
    `write(level, tid, line)` 应用级别门、每次写前检查
    `len > size_limit` 并轮转（`ime.log` → `ime.1.log`，先删旧文件），
    然后 `ensure_dir()`（幂等 `create_dir_all`）再追加。全部尽力而为
    （任一失败静默跳过）。`format_line(level, tid, message)` 渲染
    `[{unix_now}] pid={pid} tid={tid} <LEVEL> {message}`；`tid` 由调用方
    传入（core 不得依赖 Windows 线程号）——相对设计草稿的两参数
    `format_line` 是签名偏移。
- **ime（tsf.rs）：** `product_log(level, message)` — 双轨：
  1. 哨兵 `C:\zhu-ye-test\tsf-debug.enable` 存在 → 全量写
     `C:\zhu-ye-test\tsf-debug.log`，行格式与之前逐字节一致（哨兵检查仍
     走 `FILE_LOG_ENABLED`/`FILE_LOG_CHECKED` 静态缓存；`debug_log`
     保持旧签名并映射为 `LogLevel::Debug`）；
  2. 否则 → 对 `config.log_level` 走级别门，再经
     `static PRODUCT_LOGGER: Mutex<Option<FileLogger>>` 惰性装配的
     `FileLogger` 写 `%LOCALAPPDATA%\ai-zhu-ye-ime\logs\ime.log`
     （1 MiB 轮转）。`LOCALAPPDATA` 缺失 → 跳过文件写，仅保留
     `OutputDebugStringW`。53 个调用点已分级（8 error / 7 warn / 14
     info / 24 debug；见《诊断产品化设计》§4.1 表）；error/warn/info
     调用点直接调 `product_log(zhu_ye_core::LogLevel::X, …)`，debug
     调用点保留 `debug_log`。`zhu-ye: ` 消息前缀与 `OutputDebugStringW`
     保留（VM 取证依赖）。配置级别进程生命周期内缓存一次（`OnceLock`；
     P-12 不做热切换）。
- **热路径短路：** 每键必跑的 debug 调用点（TestKeyDown / key /
  v-consume / replace-last / compose-text / commit-text / cand-show /
  cand-hide / comp-update / commit）外包
  `if should_log(LogLevel::Debug)`，配置级别为 `Warn` 时按键路径上不构造
  `format!` 字符串。
- **settings（window/shell/model/config）：** `acceptance_log_dir` 变为
  `product_log_dir() -> Option<PathBuf>` = `%LOCALAPPDATA%\ai-zhu-ye-ime\logs`
  （与 `data_dir` 同机制）；诊断行与「打开日志目录」动作改为产品文案，
  打开动作先幂等创建目录（设置进程可能在 TSF 写出首行日志之前就打开它）。

`config.json` 增加 `log_level`
（`#[serde(default, deserialize_with = deserialize_log_level)]`）；
`CONFIG_FORMAT_VERSION` 保持 1（T-073 宽松模式：未知字段绝不破坏既有配置）。

## 备选方案

**全部写进用户数据 `%APPDATA%`、与 `config.json` 同目录。** 拒绝：
日志是一次性诊断产物而非用户数据；分离也让 TSF 热路径写入不落在配置
文件路径上。

**引入宏/日志 crate 做分级日志。** 拒绝：53 个带显式级别参数的调用点
转换零成本，避免新依赖与 format 宏改造成本；两个热路径守卫覆盖了真正
的性能关切。

**消息一律先构造再过滤。** 对 11 个热路径调用点拒绝（守卫只有一行）；
对其余 debug 调用点接受（即使被过滤每条事件仍 `format!` 一次）——此处
如实记录取舍：文件 IO 与目录系统调用是主要成本且保持为零，仅 `format!`
是亚微秒级。

**FileLogger 内部加锁。** 拒绝：core 没有线程模型；单一 TSF 消费方用
静态 `Mutex` 包裹（跨线程回调可能发生）。

## 后果

- 默认安装（无 `log_level` 键）：除非触发 `Warn`/`Error`，零文件写、
  零目录系统调用；每条 `Error`/`Warn` 落盘
  `%LOCALAPPDATA%\ai-zhu-ye-ime\logs\ime.log`，1 MiB 轮转。
- 哨兵文件存在时按构造不可能出验收回归：该分支复用旧格式与旧路径
  原样。
- `debug` 消息在任何级别都继续流向 `OutputDebugStringW`（对调试器辅助
  排障零行为变化），仅当 `log_level = "debug"` 时落盘。
- 设置窗口定位的目录与 TSF 写入目录一致，按需创建——不再弹系统
  「找不到文件」对话框。
- 隐私：只记录引擎状态与诊断信息，绝不记录候选词或已上屏的输入文本。

验证（2026-10-04）：core 新增 6（log_level）+ 6（file_log）+ 3
（pack_config 宽松 `log_level`）个测试；ime `log_tests` 2 个
（`product_log_path` 的 LOCALAPPDATA 注入含缺失变量情形、哨兵存在性）；
settings 1 个（`product_log_dir`）。工作区各套件全绿（core lib 289、
ime 177、settings 102，其余 crate 不变绿色）；fmt/clippy
`-D warnings`/`git diff --check` 干净；验收 §16.2 FR-060 行已回填；
host-e2e 重跑作为回归证据。

关联 notes：[前缀组词展开（T-090，
2026-10-04-prefix-word-expansion.md）](../../implemented/feature/2026-10-04-prefix-word-expansion.md)
— 验收期 note；其展开组使 host-e2e 一条断言语义适配：「多包基础顺序不漂移」
由前缀匹配收紧为子序列检查（网络语包词如 你好安怡 可依 D-71 词频序合法进入
展开组并排在某些基础前缀词之前）。[候选排序静态模型
（2026-09-19-candidate-ranking-static-model.md）](../../implemented/feature/2026-09-19-candidate-ranking-static-model.md)
— 排序层不受分级影响；[领域提权
（2026-10-02-domain-boost-scenario8.md）](../../implemented/feature/2026-10-02-domain-boost-scenario8.md)、
[联系人（2026-10-02-contacts-scenario9.md）](../../implemented/feature/2026-10-02-contacts-scenario9.md)、
[英文词表（2026-10-04-en-wordbook-zyen-v1.md）](../../implemented/feature/2026-10-04-en-wordbook-zyen-v1.md)
— 其 `debug_log` 诊断现已按 §4.1 表分级。
