# Agent Note: 多音缺读补丁——谁 shui、熟 shou 与反查越界修复

Status: implemented

[English](2026-09-30-polyphone-gap-patch.md) | 中文

## 问题

用户主诉：输入 `shui` 候选框没有"谁"（`rank data/artifacts/real.zyct shui` 只有
说 413852 / 睡 17644 / 水 10119 / 税 414…，无谁；而 `dict … shei` 显示谁词频
127180）。根因：CC-CEDICT 行 `誰 谁 [shei2] /who/also pr. [shui2]/` 的拼音字段只标
`shei`，导入管线从未见到 `shui`，真实词典缺 (谁, shui) 词条，"谁"在 `shui` 下不可达。

复现时牵出第二个潜在缺陷：`dict real.zyct -r 谁`（中文键 UTF-8 字节大于全部英文反查键）
在 `dict_loader::find_en_to_zh` 触发切片越界 panic——二分可能停在 `low == count`，
取记录前未做边界检查。

## 决策

**构建期补丁、零引擎特判、只增不改**（与 M7 约束一致：确定性、离线、自研、不污染
既有排序路径）：

1. **读音权威源——kTGHZ2013**（`data/cache/kTGHZ2013.txt`，D-014，已哈希锁定）：
   官方《通用规范汉字表》读音，`U+8C01: shéi,shuí  # 谁` 格式（预组调符、字符不带调、
   一行一字）。由新模块 `zhu-ye-dict::polyphone` 解析：`load_standard_readings` 按
   `#`/`,` 拆解，归一化口径与 CEDICT 侧一致（预组调符→基础字母、`ü`/`nǚ`→`v`、
   组合变音符 U+0300–U+030F 删除），保证 `strip_tone_letters("shuí") ==
   normalize_pinyin("shui2")`。
2. **`polyphone_gaps` 审计**：CEDICT 单字读音集（与构建同一切分/归一化管线）对照
   规范读音表。全量：CEDICT 单字 10,789 个 vs 规范表 8,105 字 → 缺读 144 条；
   按词频 ≥50,000 过滤后仅剩 2 条：`嗯→ng`（鼻音，`SyllableTable` 无此音节，
   设计上拒收）与 `谁→shui`。其余为低频又音（熟→shou 词频 1619 真实且收录；
   嘘→shi、臂→bei、巷→hang、杉→sha… 记入审计报告，留待后续甄选）。
3. **补丁表 `data/patches/polyphone.tsv`**——人工把关 TSV，`字<TAB>读音<TAB>备注`，
   `#` 注释；本轮仅 谁→shui、熟→shou。`load_patch_table` 解析期拒绝非标准音节并
   给出逐行原因（`ng` 行直接加载失败，是有意的护栏）。
4. **`import --polyphone`**（缺省存在仓库补丁表即自动加载）：truncate 之后逐条应用——
   音节必须过 `SyllableTable::standard().is_complete_syllable`；字必须已存在于词表
   （拒绝引入表外字形）；(字, 读音) 已存在则跳过；补丁表内重复跳过；词频继承该字
   现有最高值；既有词条永不修改（补弱不污染）。
5. **`audit-polyphone <CC-CEDICT> <kTGHZ> [--freq] [--min-freq]` CLI 命令**：随时可
   重跑的全字对比（后续任何补丁批次的可复现证据链）。

反查修复为一行边界检查（`low >= count` 时返回 `None`）加回归测试
（中文键与超界键返回 `None` 不 panic）。

## 否决的备选方案

- **引擎特判 `shui`→谁。** 否决：只治一个症状，绕过排序管线的确定性并污染热路径；
  后续任何缺读都要各自打补丁。
- **维护 CC-CEDICT 私改变体补齐读音。** 否决：D-001 已哈希锁定（NFR-007/008）；
  私改版本会分叉权威源、破坏可复现导入，且把改动埋进大文件而失去可审查差异。
- **按规范表自动全量补缺。** 否决：144 条差异含音节表外的鼻音（嗯→ng）、低频又音与
  语域差异；自动补会注入未经审查的字形/读音，并与引擎可切分集合静默失步。
  人工把关 TSV + audit 报告让每条补丁可枚举、可复现。
- **补丁读音带调存储。** 否决：全词典键空间为无调全拼（与 CEDICT 同归一化口径），
  带调键无法匹配引擎查询。

## 后果

- real.zyct 重建：120,028 → 120,030 词条（`多音补丁: 读取 2 条，应用 2`），
  内容 SHA-256 变化；`rank shui` → 说 413852 居首、**谁 127180 第 2 位（首屏）**、
  睡/水随后；`rank shei` 仍出谁（原读音不受影响）；`rank shou` 出熟（第 4 位）。
  "谁"的两个读音共享同一继承词频，排序保持确定性。
- 补丁词条无译文；TSF 候选窗本就不显示译文，仅 CLI `rank` 展示（开发视图），可接受。
- 嗯→ng 刻意不补：标准音节表无 `ng`，引擎无法切分，补丁会静默失步；记入审计输出。
- 本机制在导入管线"保留首个读音"规则（见 [真实词典导入](../../architecture/2026-09-21-real-dictionary-import.zh.md)）之上**只增不改**地补读音，
  从不替换或重排既有词条，两条规则共存不冲突。
- host-e2e `--m7` 增 4 项断言（shui→谁、shei 仍→谁、shou→熟、反查中文键不 panic）：
  26/26。工作区 363 测试全绿（core +1、dict +9）；fmt/clippy/diff-check 干净。
- 验收 VM（真实 TSF 栈，全新 `tsf-m8` 目录规避词典 mmap 锁）验证：shui 候选窗含"谁"
  且数字键 2 上屏；shei 仍出"谁"；shou 出"熟"。证据留档 `target/t056-accept-deploy`
  （不入库）。

- **T-129 扩展（2026-10-08）**：同一补丁表现在经共享 \pply_patch_entries\ 一并应用到产品链 base（build_base），表已扩至 31 条。见 [多音补丁接入产品链 base](2026-10-08-polyphone-patch-into-base-pack.zh.md)。
