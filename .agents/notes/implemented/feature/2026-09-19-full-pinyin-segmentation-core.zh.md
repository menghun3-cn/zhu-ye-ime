# Agent Note: Full-pinyin segmentation core for M2

Status: implemented

[English](2026-09-19-full-pinyin-segmentation-core.md) | 中文

## 问题

M1 脚手架只带了一份手工挑选的音节表和线性递归切分。该表缺失 `cha`、`duo`、`guan` 等常见音节，任意全拼无法可靠切分；递归遍历也不符合方案设计中的动态规划计划；双拼没有扩展点。

## 决策

`zhu-ye-core::pinyin` 现在提供 M2 的全拼核心。

`STANDARD_SYLLABLES` 收录 410 个标准普通话无调全拼音节，以小写 ASCII 保存（`ü` 按惯例写作 `v`），并按声母分组便于人工核对。音节是语言事实而非实现版权；来源已登记为 `docs/licenses.md` 的 D-003，T-006 将用权威数据重新生成并校验该表。

`SyllableTable::standard()` 将音节保存在排序后的 `Vec<String>` 中。`is_complete_syllable` 使用二分查找；前缀查询使用 `partition_point` 加 `starts_with` 扫描，而不是遍历整张表。旧的 `basic()` 构造器已移除，输入引擎与 CLI 都改用 `standard()`。

`segment_all` 使用自底向上动态规划构建切分：`dp[end]` 保存前 `end` 个字节的全部切分，最终顺序按结束位置与起始位置递增确定。非 ASCII 输入和无法切分的 ASCII 串返回空列表。

`PinyinScheme` 是预留的输入方案接口。`FullPinyinScheme` 将按键归一化为小写；双拼实现后续接入，切分核心无需改动。

## 曾考虑的替代方案

**保留线性递归遍历。** 落选：方案设计明确要求动态规划切分，排序数组也让前缀查询更快，并为将来 mmap 索引确定了清晰方向。

**引入现成拼音/rime 引擎。** 落选：项目承诺核心算法自研可控；引入第三方引擎只是绕过问题，而不是解决问题。

**现在一次性实现双拼。** 延期：FR-001 明确排除非全拼方案；trait 只预留接口，不扩大 M2 范围。

**现在就构建完整 Trie。** 本里程碑落选：410 个短音节用排序数组已经足够紧凑；T-006 定义 mmap 词典格式后再决定是否引入 Trie 或紧凑索引。

## 后果

标准表全部音节都能切分为自身，`xian` 仍同时得到 `xi-an` 与 `xian`，`xiange` 得到全部四种拼音合法切分。CLI 自检显示 410 个音节；`demo cha` 能在旧脚手架失败的地方正确切分。workspace 测试增至 38 项全部通过，clippy 以 `-D warnings` 通过。

音节表已用 CC-CEDICT 真实导入校验：318,015 个音节中清洗丢弃 894 个非标准音节。M2 验收通过（真实词典复验 `cha`、`duo`、`guan` 可切分，`xian` 保留 `xian` 与 `xi-an` 两种合法切分），T-007 已完成。当前切分顺序稳定但不计分；候选排序由 T-008 负责。
