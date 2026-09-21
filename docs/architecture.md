# 竹叶输入法 架构文档

## 文档信息

- 版本：v0.1.0
- 状态：已评审
- 日期：2026-09-18
- 依据：需求规格说明书、方案设计

## 1. 架构目标

1. 核心算法与 Windows 适配彻底分离，算法可独立测试、跨平台复用
2. 热路径简单、确定、快：按键到候选不经过网络、不等待外部服务
3. 扩展能力（AI、新输入方案、主题）以接口/数据包方式演进，不破坏内核
4. 整个项目可在开发者机器上从源码构建、安装、卸载、自检

## 2. 系统上下文

```text
+---------------------------+
|        用户与应用          |
| 记事本、浏览器、IDE 等    |
+------------+--------------+
             | TSF 协议（进程内 COM）
             v
+---------------------------+
|     竹叶输入法 DLL        |
| zhu-ye-ime               |
+------------+--------------+
             | 纯 Rust API
             v
+---------------------------+
| zhu-ye-core（算法核心）   |
+------------+--------------+
             | mmap
             v
+---------------------------+
| 词典数据包（版本化二进制）|
+---------------------------+
```

## 3. 模块架构

```text
workspace: zhu-ye-ime
             |              +--> zhu-ye-core（无 Windows 依赖）
             |              |      pinyin / dict / candidate / translate
             |              |      user_dict / ai
             |              |
             +--------------+--> zhu-ye-dict（数据管线构建器）
                            |
                            +--> zhu-ye-cli（自检/基准/演示）

zhu-ye-ime
  ├── tsf       TSF COM 输入处理器、生命周期
  ├── ui        候选窗 Win32 自绘、主题、DPI
  └── bridge    TSF 事件 <-> zhu-ye-core 调用映射
```

依赖规则（禁止反向）：

- `zhu-ye-core` 不得依赖 `windows` crate 或任何平台 API
- `zhu-ye-ime` 只做适配，不承载算法
- `zhu-ye-dict` 只构建数据，不依赖运行时平台
- `zhu-ye-cli` 可依赖全部 crate，仅用于开发调试

## 4. 关键数据流

### 4.1 中文输入

```text
按键事件 -> TSF 组合区更新 -> pinyin 串
  -> zhu-ye-core: 音节切分 + 词典查询 + 排序
  -> Candidate[] -> 候选窗渲染
  -> 用户选择 -> 上屏文本 -> 用户词记录
```

### 4.2 译文层

```text
Tab 切换模式 -> Candidate.translation 集合
  -> 候选窗切换为译文列表
  -> 用户选择 -> 上屏译文
```

### 4.3 TSF 注册与 COM 生命周期（M1）

- TIP CLSID 固定为 `{E54D6682-8650-40E7-A9EE-6FD1137849AE}`；zh-CN Profile 固定为 `{6315FE74-92C3-439B-8CDF-FDB6E43EDAF1}`
- DLL 只导出 `DllGetClassObject`、`DllCanUnloadNow` 与开发探针 `dll_probe`
- 注册表只由 `scripts/install.ps1` / `scripts/uninstall.ps1` 管理；使用 `InProcServer32` + `ThreadingModel=Apartment`
- `DllCanUnloadNow` 以活动对象数与 `LockServer` 计数双为零为卸载条件；不实现聚合
- 完整注册契约、备选方案与后果见 [Agent Note](../.agents/notes/implemented/architecture/2026-09-18-tsf-registration-and-lifetime.md)（T-010）

### 4.4 TSF 组合与按键闭环（M1）

```text
ITfKeyEventSink 收到按键 -> KeyAction 分类
  -> OnTestKeyDown 决定是否吃键
  -> OnKeyDown 请求 TF_ES_SYNC | TF_ES_READWRITE 编辑会话
  -> ITfInsertAtSelection + ITfContextComposition 启动/更新组合
  -> 空格/回车/数字/Esc 结束组合或取消
  -> ITfRange::SetText 上屏 -> 同步 InputEngine 状态
```

- `TextService` 实现 `ITfTextInputProcessorEx`、`ITfKeyEventSink`、`ITfCompositionSink`；按键 sink 在 `Activate` 注册、`Deactivate` 注销
- 组合串与候选由 `crates/zhu-ye-ime/src/input.rs` 的纯 Rust `InputEngine` 维护，TSF 层先写宿主再同步引擎
- 共享状态使用 `Rc<Mutex<EngineState>>`，适配 TSF apartment 单线程回调
- 完整设计、备选方案与后果见 [Agent Note](../.agents/notes/implemented/feature/2026-09-18-tsf-composition-and-key-events.md)（T-011）

### 4.5 AI 扩展（后续）

```text
输入空闲/显式触发 -> 后台任务调用 AiService
  -> 结果带来源标记 -> 附加区域展示
  -> 用户选用后才进入上屏路径
```

关键约束：AI 永远不在普通按键热路径上同步阻塞。

## 5. 词典格式设计（v2 正式）

二进制词典文件使用魔数 `ZYDT`，当前版本号 `DICT_VERSION = 2`，所有多字节字段小端。
格式实现位于 `zhu-ye-core::dict_format`，构建器与 mmap 加载器共用同一字节定义。

```text
+------------------------+  128 字节固定头部
| Header                 |  魔数 ZYDT、版本 2、索引/词条/bigram/翻译/反查数量与区偏移、SHA-256
+------------------------+
| Pinyin Index           |  24 字节/条：拼音文本偏移与长度、词条起始/数量、命中总数
+------------------------+
| Entry Table            |  24 字节/条：拼音索引、词频、译文偏移与长度
+------------------------+
| Bigram Table           |  16 字节/条：前词/后词索引、共现频次
+------------------------+
| Word Translation Index |  24 字节/条：中文词文本偏移/长度、译文偏移/长度
+------------------------+
| Reverse Translation    |  24 字节/条：归一化英文文本偏移/长度、中文词文本偏移/长度
| Index                  |
+------------------------+
| Text Pool              |  UTF-8 变长文本区，偏移从 1 开始，0 表示无译文
+------------------------+
| Content SHA-256        |  写入头部 80..112 字节，覆盖头部之后全部内容
+------------------------+
```

加载策略：

- `DictionaryFile` 对文件只读 mmap，不整体加载进堆内存；加载时校验魔数、版本、长度、SHA-256、UTF-8、偏移分区与排序
- 拼音索引按拼音排序，支持前缀查询与多音节按索引列表取词；bigram 按索引引用，直接接入 `BigramModel`
- 中文到译文索引按中文词排序，英文反查索引按归一化英文键排序；`DictionaryFile` 实现 `Translator`，`zh_to_en` 与 `en_to_zh` 均走二分查询
- 构建产物确定：相同输入与版本生成相同字节序列；`zhu-ye-dict` 提供 `build`、`inspect`、`verify` 三个命令，译文与反查会同步校验
- 当前 `data/artifacts/seed.zyct` 由自建演示种子构建（20 词条 + 10 bigram + 19 译文），真实数据源引入前经 `licenses.md` 登记
## 6. 拼音切分设计

- 音节表：`STANDARD_SYLLABLES` 收录 410 个标准无调全拼音节（T-007）；T-006 数据管线改为公开权威数据生成并校验唯一性
- 匹配：排序数组二分前缀查询 + 自底向上动态规划，输出全部可行切分；顺序确定、非 ASCII 输入返回空
- 输入方案：`PinyinScheme` 接口预留双拼接入，当前 `FullPinyinScheme` 只做小写归一化
- 切分算法决策见 [Agent Note](../.agents/notes/implemented/feature/2026-09-19-full-pinyin-segmentation-core.md)（T-007）

## 7. 候选排序模型

排序职责由 `zhu-ye-core::candidate::RankingModel` 承担；当前实现为
`StaticRankingModel`，权重通过 `RankingConfig` 配置，全部使用整数饱和运算，
跨平台结果确定。

```text
score = static_score(word) × unigram_weight
      + min(bigram_freq(prev_word, word), bigram_cap) × bigram_weight
      + min(user_freq(word), user_cap) × user_weight
```

默认权重：`unigram_weight=1`、`bigram_weight=16`、`bigram_frequency_cap=100_000`、`user_weight=48`、`user_frequency_cap=10_000`。

- 输入引擎维护 `previous_word`：空格/数字提交后更新，回车清空，Esc 不变
- 用户词命中时把候选 `source` 标记为 `User`；词频按文本跨拼音汇总
- bigram 通过 `BigramModel` trait 注入；T-006 提供 mmap/压缩实现时只替换数据源，不改排序逻辑
- 静态分仍可作为确定性基础排序，最终排序以 `RankingConfig` 加权结果为准

### 用户词持久化（T-009）

- 选择即记忆：`InputEngine` 在空格/数字选择真实候选时记录词与拼音并同步落盘；回车与拼音原文回退不学习
- 存储：IME 使用 `%APPDATA%\ai-zhu-ye-ime\user_words.json`；CLI 与测试注入独立路径，核心库不依赖 Windows API
- 文件格式：版本化 JSON `{ "version": 1, "entries": [...] }`，条目含词、拼音、词频与最近选择时间；`words_sorted` 确定性排序保证写入稳定
- 写入：`UserDictStore::save` 先写同目录临时文件并 `sync_all`，再原子替换目标；失败时清理临时文件、不破坏原文件
- 损坏恢复：`load` 遇到解析失败或低版本时先备份为 `.bak` 再重建空库；高于当前版本时拒绝打开并保留原文件，避免软件降级破坏用户数据
- 删除/重置：`InputEngine::delete_user_word`、`reset_user_words` 与 CLI `user` 子命令同步更新内存与磁盘

## 8. 进程与线程模型

- 输入法作为 DLL 加载到宿主进程（TSF 协议要求）
- TSF 相关状态在 STA/UI 线程维护，不跨线程直接访问 COM 对象
- 纯算法调用短小（`<= 30ms` 预算），无需线程池
- 未来 AI/联网调用必须转入后台任务，完成后再派发回 UI 线程
- 卸载：停止回调、撤销事件接收器、释放引用计数，最后释放注册

## 9. 数据管线

```text
raw 数据（CC-CEDICT/ECDICT 等）
   -> 清洗/校验
   -> 合并去重/拼音注音
   -> 词频统计（开放语料）
   -> 编译二进制
   -> hash + 清单（docs/数据清单）
   -> 词典包（版本化）
```

原始大文件不进入 git；数据处理脚本、版本、来源与许可证全部入库。

## 10. 目录规范

```text
/
├── AGENTS.md                  # 项目级 AI 协作与维护指令
├── README.md
├── Cargo.toml                 # workspace
├── docs/
│   ├── 需求规格说明书.md
│   ├── 方案设计.md
│   ├── 验收标准.md
│   ├── architecture.md
│   ├── todos-list.md
│   ├── todos-done.md
│   └── licenses.md
├── crates/
│   ├── zhu-ye-core/
│   ├── zhu-ye-ime/
│   ├── zhu-ye-dict/
│   └── zhu-ye-cli/
├── scripts/                   # 安装/卸载/数据抓取脚本
└── data/                      # 数据清单与构建产物（大文件忽略）
```

## 11. 构建与验证规范

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo build -p zhu-ye-cli --release
```

提交前必须依次通过上述检查，并同步 todos 状态。

## 12. 演进预留

| 方向 | 预留方式 |
| --- | --- |
| AI 翻译/润色/联想/预判/建议 | `AiService` trait + 来源标记 + 后台任务框架 |
| 双拼/五笔 | 码表配置接口 |
| 主题 | 自绘渲染参数数据化 |
| 云同步 | 明确不在 v1 范围，架构不为其妥协 |
| 在线翻译 | 本地 `Translator` 与 AI 服务并列，由配置选择 |
