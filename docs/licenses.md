# 数据来源与许可证清单

本文件记录项目使用到的所有第三方数据、词表、语料与工具，用于许可合规审查。新增数据源必须在此登记后才能进入数据管线。

现状：D-001 CC-CEDICT、D-004 FrequencyWords 中文词频与 D-006 OPUS GlobalVoices 已于 2026-09-21 引入数据管线，用于离线导入真实词库；原始文件与生成产物保存在 `data/` 且不进入 git，派生说明见 `docs/数据清单.md`。ECDICT（D-002，CC BY-SA 4.0）已于 2026-10-02 实测下载并定稿为第九期英文词典扩容数据源（770,611 词条，SHA-256 锁定，见 数据清单.md）。2026-09-28（M6）新增 D-007 至 D-015 九个外部源，均由 `data/pins/` 锁定并校验；D-016/D-017 为项目自有数据，随 git 版本管理。2026-09-29（M6-U）新增代码依赖登记一节，登记 `ed25519-dalek`（manifest 签名，BSD-3-Clause）。2026-10-02（T-064，第五期）新增 D-018 FrequencyWords 英文词频（英文词表构建基座）与 D-019（英文大小写补丁，项目自有）。2026-10-04（T-087，第九期）新增 D-020 social-media-chinese-words（MIT，网络语扩充源）与 D-021 清洗子集（项目自有，源 MIT 派生）。

## 数据源登记

| 编号 | 数据源 | 用途 | 许可证 | 版本/日期 | 状态 |
| --- | --- | --- | --- | --- | --- |
| D-001 | CC-CEDICT | 中英词条、拼音与译文（真实词库） | CC BY-SA 4.0 | 2026-09-21 | 已引入：`import` 命令 120,028 词条；清单见 [数据清单.md](./数据清单.md) |
| D-002 | ECDICT | 英文词典扩容全量词表（第九期 FR-046）：word + 词频（frq/bnc）；**第九期英文候选主词表（替代/合并 D-018 FrequencyWords 基座）** | **CC BY-SA 4.0**（README 声明；CC-CEDICT 扩展同源。随源文件分发的 `word`/`translation` 等内容版权归原贡献者，由 CC BY-SA 4.0 授权） | 2026-10-02 实测下载：65,933,428 字节 / 770,611 词条，SHA-256 `1A6947E04785DB63613A92E14903CDAE7954F7E84860B10E68E5C7CBB3F9C3CF` | 已引入（第九期方案定稿；构建管线扩展见 [方案设计.md](./方案设计.md) §14.2，来源标注 NFR-008） |
| D-003 | 拼音音节表 | 全拼合法性校验与切分 | 语言事实，引自公开标准 | 2026-09-19 | 已引用：T-007 手写标准全拼常量；T-006 数据管线将进一步校验 |
| D-004 | FrequencyWords 中文词频 | unigram 词频映射（真实词库） | 内容 CC BY-SA 4.0；代码 MIT | 2026-09-21 | 已引入：导入命中 28,130 词条；bigram 语料由 D-006 提供；清单见 [数据清单.md](./数据清单.md) |
| D-005 | 自建演示种子 | T-006 演示词典（20 词条 + 10 bigram + 19 译文） | 自建数据 | 2026-09-19 | 已引入：`demo.rs` 种子与 `seed.zyct` 构建产物；不含第三方词表 |
| D-006 | OPUS GlobalVoices 简体中文分词语料 | bigram 共现统计（真实词库） | 内容 CC BY 3.0（官网声明：This site is licensed as Creative Commons Attribution 3.0） | 2026-09-21 | 已引入：真实 bigram 820,368 词对；清单见 [数据清单.md](./数据清单.md) |
| D-007 | jieba dict.txt | base 骨架扩充 + 第二词频源 | MIT | 2026-09-28 | 已引入（M6）：pins 锁定，清单见 [数据清单.md](./数据清单.md) |
| D-008 | THUOCL_IT（清华开放中文词库 IT 类） | IT 领域包 | MIT | 2026-09-28 | 已引入（M6）：pins 锁定，清单见 [数据清单.md](./数据清单.md) |
| D-009 | THUOCL_medical（医学类） | 医学领域包 | MIT | 2026-09-28 | 已引入（M6）：pins 锁定，清单见 [数据清单.md](./数据清单.md) |
| D-010 | 现代汉语常用词表（草案）2008（文本镜像，源出教育部官网扫描 PDF） | base 骨架 | 官方行政文件（依《著作权法》第 5 条不受著作权保护） | 2026-09-28 | 已引入（M6）：pins 锁定（官方扫描 PDF 不进入构建，改用转录文本镜像，哈希见 [数据清单.md](./数据清单.md)）；转录内容同为官方文件不受著作权保护 |
| D-011 | 通用规范汉字表（2013，维基文库页面文本） | 字集覆盖基线 | 官方文件（页面文本 CC BY-SA 4.0） | 2026-09-28 | 已引入（M6）：pins 锁定，清单见 [数据清单.md](./数据清单.md) |
| D-012 | wordfreq 3.1.1（PyPI wheel） | 主词频源（zipf） | 代码 Apache-2.0；**数据 CC BY-SA 4.0** | 2026-09-28 | 已引入（M6）：pins 锁定；派生词典数据文件须按 CC BY-SA 4.0 发布并署名，与 CC-CEDICT 处理一致 |
| D-013 | MDN zh-cn 术语表（glossary slug 快照） | IT 领域包补充 | **CC BY-SA 2.5+**（署名 Mozilla Contributors） | 2026-09-28 | 已引入（M6）：快照提交入库，清单见 [数据清单.md](./数据清单.md) |
| D-014 | Unihan kTGHZ2013 单字拼音（pinyin-data 仓库） | 领域/网络语包单字注音底表 | **MIT**（仓库 mozillazg/pinyin-data）；数据源自 Unicode Unihan kTGHZ2013（Unicode 许可） | 2026-09-28 | 已引入（M6-P）：`source-check` / `build-pack` 注音底表，清单见 [数据清单.md](./数据清单.md) |
| D-015 | MDN zh-cn 术语表正文（按 slug 打包） | IT 领域包补充（术语标题） | **CC BY-SA 2.5+**（署名 Mozilla Contributors） | 2026-09-28 | 已引入（M6-P）：锁定 commit 打包 + pins 哈希；派生的 it.zyct 须按 CC BY-SA 发布并署名 Mozilla Contributors |
| D-016 | 网络语种子表 | slang 包词源 | 项目自有 | 2026-09-28 | 已引入（M6-P）：提交入库，逐行带来源注释 |
| D-017 | 把关负面清单与抽查样例 | 构建期内容把关 | 项目自有 | 2026-09-28 | 已引入（M6-P）：仅构建期输入，**不进任何发行物** |
| D-018 | FrequencyWords 英文词频（en_full） | 英文词表构建基座（第五期 FR-030） | 内容 CC BY-SA 4.0；代码 MIT（与 D-004 同 repo 同许可） | 2026-10-02 | 已引入（T-064）：pins 锁定（data/pins/frequencywords-en.json）；派生 EN_WORDS 表须按 CC BY-SA 4.0 发布并署名，与 D-004 处理一致；清单见 [数据清单.md](./数据清单.md) |
| D-019 | 英文大小写补丁表（en-capitals.tsv） | 英文词表大小写原形标注（D-09） | 项目自有 | 2026-10-02 | 已引入（T-064）：提交入库（data/patches/en-capitals.tsv），随版本维护；取舍原则见文件头部注释 |
| D-020 | social-media-chinese-words（jilelab） | 网络语扩充源（第九期 FR-047/T-087，方案决策 D-52） | **MIT** | 2026-10-04（仓库 2021-05，107.78 万行，合并文件 15.4 MB，SHA-256 见 [数据清单.md](./数据清单.md) D-020） | 已引入（T-087）：7 分类 txt 由 `scripts/fetch-social-media.ps1` 下载合并，pins 锁定（data/pins/social-media-zh.json）；**派生 slang 词条按 MIT 随发行物分发**，产物头注释保留来源 |
| D-021 | 网络语清洗子集（social-words.tsv） | slang 包词源（T-087） | 项目自有数据整理；源数据许可 MIT（D-020） | 2026-10-04 | 已引入（T-087）：`zhu-ye-dict social-clean` 确定性生成并提交入库，许可随源（MIT） |

## 代码依赖登记

Rust 第三方 crate 依赖；新增依赖必须在此登记后才能进入构建。全部为宽松许可（MIT / Apache-2.0 / BSD），与项目 `MIT OR Apache-2.0` 兼容。

| crate | 用途 | 许可证 | 引入 |
| --- | --- | --- | --- |
| memmap2 | 词典只读 mmap 加载 | MIT OR Apache-2.0 | T-006 |
| serde / serde_json | 配置、用户词与 manifest 序列化 | MIT OR Apache-2.0 | T-006 |
| sha2 | 词典内容 SHA-256 校验 | MIT OR Apache-2.0 | T-006 |
| windows / windows-core | TSF 与 Win32 COM 绑定（仅 zhu-ye-ime） | MIT OR Apache-2.0 | T-010 |
| **ed25519-dalek 2.2** | manifest 签名与验签（M6-U 更新分发信任链） | **BSD-3-Clause** | T-051 |

说明：
- `ed25519-dalek` 传递依赖 `curve25519-dalek`（BSD-3-Clause）、`fiat-crypto`（MIT OR Apache-2.0）等，均为宽松许可；不在仓库内 vendor，由 Cargo 按 `Cargo.lock` 锁定版本获取。
- 发布私钥**不入库**：签名工具从环境变量 `ZHU_YE_RELEASE_SECRET_KEY` 读取，客户端只内置公钥。
- 更新器下载使用系统自带 `curl.exe`（Windows 10+），因此不引入 HTTP/TLS 依赖栈。

## 打包/构建工具登记

CI/本机构建链使用的外部打包与自动化工具；各自许可允许免费（含商业）使用，
不随发行物分发其本体（仅使用其编译产物），无捆绑依赖。

| 工具 | 用途 | 许可 | 引入 |
| --- | --- | --- | --- |
| Inno Setup 6（ISCC.exe） | exe 安装包编译（T-097，exe 第 7 资产；载荷与 zip 同 staging） | Inno Setup License——freeware：个人与商业免费使用，用其生成的安装器/卸载器可随发行包免费分发（Inno Setup 官方安装包内附许可证全文，官网 jrsoftware.org 亦提供；修改 Inno Setup 本体再分发须遵循其条款，本项目仅作编译器使用，不修改本体） | T-097 |

## 使用规则

- 原始大文件不进入 git；数据清单、处理脚本与版本 hash 必须入库
- 每个数据源引入前，必须核实许可证允许"离线生成派生作品并随开源项目分发"
- 派生数据包在文档中标注原始来源与许可证
- 对许可证不明确或冲突的数据，一律不引入
