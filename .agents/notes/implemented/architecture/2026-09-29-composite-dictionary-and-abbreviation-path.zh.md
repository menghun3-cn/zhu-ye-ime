# Agent Note: 复合词典与缩写输入路径

Status: implemented

[English](2026-09-29-composite-dictionary-and-abbreviation-path.md) | 中文

## 问题

M6-P 交付了四个独立的词典包（`base`、`it`、`med`、`slang`），但运行时仍然只加载
一个文件：DLL 同目录的 `dictionary.zyct`，回退 `%APPDATA%`，测试覆盖用
`ZHU_YE_DICT_PATH`。包所承诺的一切——FR-015（启用领域包后其词可达）、
FR-016/FR-017（网络语词与字母串缩写）、FR-022（配置驱动的包管理）——对真实用户
都不可达。T-048 的 VM 验收只能靠改环境变量再重启来切换包，那是测试夹具，不是产品。

两个彼此独立的缺口：

1. **没有组合能力。** `InputEngine` 只接受单个 `Arc<dyn Dictionary>` 与单个
   `Arc<dyn BigramModel>`。既没有办法把多个 `DictionaryFile` 当作一个来查询，
   也没有配置文件说明该加载哪些。
2. **没有缩写路径。** `yyds` 这类网络语键存在词条的拼音字段里，但常规管线把组合串
   送进 `generate_candidates`，后者按音节切分。`yyds` 无法切分，因此永远产不出网络语
   候选。T-049 修好了相邻的"数字进入组合串"问题，但那只让这个键**打得出来**——
   查询路径本身仍然缺失。

## 决策

### 组合

`zhu_ye_core::CompositeDictionary` 持有 `Vec<Arc<DictionaryFile>>`，实现
`Dictionary`、`BigramModel` 与 `Translator`。合并规则遵循设计文档：同
`(拼音, 词)` 去重且 `frequency` 取 **max**（S-6），bigram 取 max，译文取首个非空。

等价性是回归底线：只有单个包时，复合结果必须与 `DictionaryFile` 完全一致。合并
保持首次出现顺序、再以**稳定**排序按词频降序排列，这与 `DictionaryFile::lookup`
自身的顺序相同。

`from_paths` 不会整体失败：打不开的包被跳过并记入诊断列表。NFR-009 禁止半加载，
且单个损坏的包不得让输入法不可用。

**热路径快速通道。** `collect_merging` 在只有一个非空结果时直接返回，不分配哈希表。
最常见的配置——只启用基础包——因此零额外成本。用 `zhu-ye-cli bench` 实测：加入该
通道后多包查询从每次 9.69 µs 降到 2.30 µs，而单包为 2.26 µs。

### 配置

`zhu_ye_core::pack_config` 解析 `%APPDATA%\zhu-ye-ime\config.json`：

```json
{ "version": 1, "enabled_packs": ["it","med","slang"],
  "online_update": false, "last_check": null }
```

`online_update` 默认 `false`，这正是把"默认关闭"保证（P-03）变成配置层可测事实、
而不只是更新器内部行为的地方。未知包 id 被过滤进 `unknown` 列表并记日志；损坏或
不可读的文件回退默认（仅基础包）并返回诊断。`plan_packs` 解析
`<packs_dir>/<id>.zyct`，并把"配置了但文件缺失"与"未知 id"分开报告，使两种失败
在日志中可区分。

### 缩写路径

`is_abbreviation_input` 以三个条件共同把关：

1. 长度 ≥ `ABBREVIATION_MIN_LEN`（2，按 S-2）——单字母不得泛滥；
2. 仅由 ASCII 小写字母或数字组成；
3. `segment_all` 返回**空**——该串不是合法拼音组合。

第三个条件是设计 11.4 的防污染规则：`wo` 与 `emo` 能切成音节，因此绝不进入缩写
路径。这与构建管线拒绝此类键所用的判据相同，两侧保持一致。

`abbreviation_candidates` 查询精确键与其前缀补全，标注 `CandidateSource::Slang`；
`append_abbreviation_group` 把它们追加在常规候选之后，使其不参与默认排序竞争。

`CandidateSource::Slang` 经 `candidate_ui::display_main_text` 驱动 `[网络]` 标注。
标注追加到主文本而非独占一列，因此 `row_split` 的宽度估算自动把它算进去，不会与
译文列重叠。

### 装配

`create_engine` 读配置、规划包、构建复合词典，并通过 `InputEngine::with_slang`
单独挂载网络语包。只有在启用列表里出现 `slang` 时才挂载，因此该路径默认关闭。
每一步都有日志（`config-warn`、`config-unknown-pack`、`config-missing-pack`、
`pack-skipped`、`pack-ok`、`composite-ok`、`slang-path enabled`），使 VM 上仅凭
TSF 日志就能诊断。

`resolve_base_dir` 取代原先的单文件 `resolve_dictionary_path`。它保留 T-022 的优先级
（环境变量覆盖 > DLL 目录 > `%APPDATA%`），但产出的是**目录**，因为组合需要目录。

## 曾考虑的替代方案

**在安装期把多个包合并成单个磁盘包。**
否决：这会让包的启用/停用需要重新构建或重新下载，破坏"用户从不启用的包按需 mmap"，
并与 FR-020 所依赖的包独立版本化相矛盾。

**启动时把词条拼接进一个内存词典。**
否决：这放弃 mmap，会把每个启用包的全部文本载入内存，破坏"常驻内存 ≤ 100MB"验收行。

**始终执行合并，不做单来源快速通道。**
依实测否决：它让"仅基础包"配置比单个 `DictionaryFile` 慢四倍却毫无收益——单来源
合并在定义上就是空操作。

**把缩写查询放进 `generate_candidates` 内部。**
否决：该函数的契约是拼音切分，所有调用方（CLI `rank`、host-e2e、测试）都会悄悄
获得网络语行为。保持为独立函数使触发条件显式且可单独测试。

**让缩写组参与常规排序竞争。**
按设计 11.4 否决：构建期词频被抬高的网络语词会压过同一输入串的普通拼音候选，这
正是防污染规则要阻止的污染。

**让主组赢得去重。**
多包回归抓到后被否决：网络语包启用时也参与常规拼音路径，因此 `yyds` 在缩写组运行
之前就产出了一个 `Static` 候选。去重随即丢掉网络语版本，`[网络]` 标注消失。现在
由缩写组胜出，标注与排尾同时成立，且候选仍只出现一次。

## 后果

- 包通过配置文件变为用户可控；启用集合在启动时生效，重启后生效（P-12，不热切换）。
- 成本：包解析现在会在引擎创建时读一个配置文件。这发生在每次激活一次，与已经在做
  的"打开并校验词典"同一路径，不触及逐键路径。
- 成本：单来源快速通道是一个分支，其正确性依赖"单来源无需合并"这一论断。等价性
  测试针对真实 `DictionaryFile` 断言这一点，而不是信任论证。
- `DictionaryFile` 新增 `path()` 与 `entry_count()`。`path` 仅用于诊断，内存构造的
  实例为 `None`。
- `CandidateSource` 新增 `Slang` 变体。`display_main_text` 是把它变成可见标注的
  唯一位置。
- 测试：core 新增 12 项复合、13 项配置、7 项缩写测试；`zhu-ye-ime` 新增引擎层网络语
  测试与三项标注测试；`host-e2e` 新增 `--multi-pack` 模式，其七项检查覆盖等价性底线、
  不漂移、去重与防污染；`zhu-ye-cli bench` 新增 `ZYDT_PACKS` 多包场景。工作区共 288
  项测试；完整门禁（fmt、clippy `-D warnings`、工作区测试、`git diff --check`）全绿，
  host-e2e 种子 19/19 与多包 7/7 均通过。
