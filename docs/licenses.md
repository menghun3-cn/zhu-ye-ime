# 数据来源与许可证清单

本文件记录项目使用到的所有第三方数据、词表、语料与工具，用于许可合规审查。新增数据源必须在此登记后才能进入数据管线。

现状：D-001 CC-CEDICT、D-004 FrequencyWords 中文词频与 D-006 OPUS GlobalVoices 已于 2026-09-21 引入数据管线，用于离线导入真实词库；原始文件与生成产物保存在 `data/` 且不进入 git，派生说明见 `docs/数据清单.md`。ECDICT（D-002）仍待引入。2026-09-28（M6）新增 D-007 至 D-015 九个外部源，均由 `data/pins/` 锁定并校验；D-016/D-017 为项目自有数据，随 git 版本管理。2026-09-29（M6-U）新增代码依赖登记一节，登记 `ed25519-dalek`（manifest 签名，BSD-3-Clause）。

## 数据源登记

| 编号 | 数据源 | 用途 | 许可证 | 版本/日期 | 状态 |
| --- | --- | --- | --- | --- | --- |
| D-001 | CC-CEDICT | 中英词条、拼音与译文（真实词库） | CC BY-SA 4.0 | 2026-09-21 | 已引入：`import` 命令 120,028 词条；清单见 [数据清单.md](./数据清单.md) |
| D-002 | ECDICT | 英中反查词条（计划） | 以仓库许可证声明为准 | 待登记 | 待引入 |
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

## 使用规则

- 原始大文件不进入 git；数据清单、处理脚本与版本 hash 必须入库
- 每个数据源引入前，必须核实许可证允许"离线生成派生作品并随开源项目分发"
- 派生数据包在文档中标注原始来源与许可证
- 对许可证不明确或冲突的数据，一律不引入
