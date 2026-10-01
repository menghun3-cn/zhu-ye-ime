# Agent Note: 通讯录索引候选（场景 9，M11）

Status: implemented

[English](2026-10-02-contacts-scenario9.md) | 中文

## Problem

场景 9（通讯录，FR-036/FR-037/FR-038）需要让**本地、隐私最小化**的联系人姓名从拼音组合
可达：用户导出一个 vCard 3.0 文件、在 `config.json` 里列出路径，联系人姓名的全拼与简拼
就作为提权候选出现（FR-037）。已确认口径（2026-10-02，D-18~D-22）：只从 vCard 文件导入、
绝不读系统通讯录（D-18）；配置列 vcf 路径按需导入（D-19）；仅姓名建索引（D-20）；提权
与领域同层（D-21，基础候选之后、追加组之前，同层内联系人排在领域之后）；多音字全部读音
形态建键（D-22）。

排序基线（T-050）不得漂移：无联系人配置时，候选清单必须与场景 9 之前的构建逐位一致。

## Decision

### 核心层：`zhu-ye-core/src/vcard.rs`（T-071-1）

`parse_vcard(input: &str) -> Result<Vec<VCardContact>, VCardError>` 实现 vCard 3.0
子集：`BEGIN:VCARD` 段解析、`FN` 优先于 `N` 组件拼接、其余字段跳过、行折叠与转义处理、
损坏输入按行号报错。16 个单测。

### 核心层：运行时注音表 `zhu-ye-core/src/char_pinyin.rs`（T-071-2）

core 此前没有汉字→拼音表，而运行时必须**离线**为联系人姓名注音。沿用 `en_words` 模式
（生成脚本 + 生成文件入库 + 数据源 pin 哈希锁定）：`scripts/build-char-pinyin.ps1`
从锁定的 `kTGHZ2013.txt` 缓存（8,105 字，sha256 pin）生成 `CHAR_PINYIN`，归一化为无调
ASCII（`ü`→`v`），按字升序供二分查找。这是词典包构建期注音管线（见 dictionary-pack-build
note）的**运行时平行件**：dict crate 构建期注音；联系人姓名在索引构建期注音，绝不上热路径。

### 核心层：`zhu-ye-core/src/contacts.rs`（T-071-2/T-071-3）

- `annotate_name(name) -> Vec<String>`：逐字读音笛卡尔积（多音全形态，D-22），ASCII
  字母/数字小写原形，空格/符号跳过，键排序去重；`CONTACT_KEYS_CAP`（64）约束组合数。
- `abbreviation_key(key, table) -> Option<String>`：**索引构建期**从每个全拼键派生——按
  标准音节表切分（`segment_all` 首个切分）→ 每音节首字母。非拼音键（英文名、数字）与
  单音节键不派生。由此 FR-037 简拼可达且**不动 FR-023 词典缩写路径**：`zs → 张三` 直接
  命中索引。—— 与设计 §3.2「复用 FR-023」的表述有偏离：行为等价，机制改为索引原生。
- `build_contact_index(contacts: &[VCardContact]) -> ContactIndex`：BTreeMap 建键
  （全拼键 + 派生简拼键），同名去重，`CONTACT_INDEX_CAP`（10k）按出现顺序截断。
  —— 与设计 §4 草案签名 `build_contact_index(contacts, annotate: &dyn Fn(...))`
  有偏离：内置注音表替代了注音回调注入（理由同 `en_words`：自研可控、离线、确定性、
  无 trait object）。
- `contact_candidates(index, pinyin_prefix, cap) -> Vec<Candidate>`：
  `partition_point` 前缀区间（`upper = prefix + "\u{10FFFF}"`），
  `Candidate::new(name, 0).with_source(CandidateSource::Contact)`，姓名 seen 去重保序。
- `CandidateSource::Contact` 独立变体：与用户词学习分离（FR-003 语义——联系人选中
  不进学习流水线）。

### 配置（T-071-3）

`ConfigFile.contact_vcards: Vec<PathBuf>`（`#[serde(default)]`，缺失 = 空 = 无联系人
基线）持久化到 `config.json`（D-19；用户可手改，FR-038 清除 = 清空列表）。

### 引擎（`crates/zhu-ye-ime/src/input.rs`）

`InputEngine` 新增 `contacts: Option<ContactIndex>`，经 `with_contacts(index)`
（索引为空时置 None）挂载、`clear_contacts()` 清除。`refresh_candidates` 在领域提权块
之后、FR-030 追加组（英文/缩写/emoji）之前执行联系人步：组合串非空时查
`contact_candidates(contacts, &self.composing, 8)`，经 `append_group` 追加——与 D-13
同层，同命中时联系人排在领域之后、追加组之前（设计 §3.3 顺序）。无索引 / 空输入 ⇒ 跳过
（T-050）。

TSF 层（`crates/zhu-ye-ime/src/tsf.rs`）新增 `contact_engine(...)`：读
`config.contact_vcards`、逐个打开文件、`parse_vcard`、汇总姓名、`build_contact_index`、
经 `with_contacts` 挂载；缺失/不可读/解析失败记 `debug_log` 并跳过；全部无效则保持无
联系人基线。**TSF 交互代码零改动。**

### 验收：`host-e2e --m12 [<vcf 文件>]`

内存断言组（确定性基础词表 + 构建好的联系人索引）覆盖：全拼前缀可达与基础候选之后的位次、
`zs` 简拼、多音简拼（`cz`→曾子）、英文名原形键（`al`→Alice）、无命中逐位一致基线、
`clear_contacts` 恢复基线。提供真实 vcf 文件时追加完整导入链（解析 → 建索引 → 可达）
复核。结果 7/7（含真实导入）。

## Alternatives considered

- **读系统通讯录（WinRT `Windows.Contacts`）**：否决（D-18）——COM/WinRT FFI 违反
  core 层零 .NET 约束；隐私最小化倾向显式导出文件。
- **IME 注入注音回调**：否决——内置 kTGHZ2013 表自研可控、离线、确定性，与
  `en_words` 先例一致；省去 `dyn Fn` 接线。
- **简拼走 FR-023 词典缩写路径**：否决——FR-023 路径及其窄触发（O-02）保持不动；
  索引原生简拼键以零热路径/零词典改动达成相同 `zs` 语义。
- **热路径逐字回退注音**：否决——与 dictionary-pack-build note 同理：注音只在索引
  构建期做一次，绝不在每次刷新时做。
- **解析器支持 GBK 编码**：延期——vCard 按 UTF-8 接受；非 UTF-8 文件记录并跳过
  （设计 §8 风险清单记载）。

## Consequences

- 无联系人配置 / 全部文件无效 ⇒ 逐位基线（T-050），m12 用例 5 与 workspace 回归验证。
- 联系人候选标注 `source = Contact`；UI 不加标签。
- 确定性（FR-002）：同 vcf + 同输入 ⇒ 同清单；键 Ordinal 排序、同名多键去重。
- 联系人选中不进用户词学习流水线（FR-003 语义不变；独立来源变体）。
- 隐私：联系人内容只存在于用户导入文件与内存索引；绝不进词典产物/日志/提交（FR-038
  清除删索引；运行时文件处理不在本批次）。
- 新增验收命令 `host-e2e --m12 [<contacts.vcf>]`；`--m11`/`--m7`/`--m8`/`--m9`/`--m10`
  不变（本批次后 workspace 与 e2e 回归全绿）。

## Related notes

- [2026-10-02-domain-boost-scenario8](2026-10-02-domain-boost-scenario8.zh.md)：
  D-13 层；联系人与其共用插入点（D-21，排在领域之后）。
- [2026-10-02-ime-experience-optimization](2026-10-02-ime-experience-optimization.zh.md)：
  FR-023 词典缩写路径——不变；联系人简拼是索引原生机制。
- [2026-09-28-dictionary-pack-build-and-manifest](2026-09-28-dictionary-pack-build-and-manifest.zh.md)：
  构建期注音管线；本 note 的运行时表是联系人注音的平行件。
