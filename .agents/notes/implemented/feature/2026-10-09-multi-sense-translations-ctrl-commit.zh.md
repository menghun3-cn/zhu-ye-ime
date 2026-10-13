# Agent Note: 多义译文（按义拆行、上屏剥词性、Ctrl+数字直上屏译文）（T-131）

Status: implemented

## Problem

用户 2026-10-09 三项需求一并提出：

1. Tab 选择上屏英文时，词性不需要一起上屏（如 `v. suspicious` 应上屏 `suspicious` 而非 `v. suspicious`）。
2. 「你好」应同时显示 `hello` 与 `hi` 两个候选。现状译文层只显示一条：m6 构建把每条译文在首个 `;` 处截断（`clean_translation`），精修表又把 你好 的 `hello; hi` 覆盖成单义 `hello`。
3. Ctrl+数字可直接上屏译文（无需先按 Tab 切换）。

已确认的产品决策：**全部**多义译文（`a; b`）拆成独立行（不只 你好）；Ctrl+数字在**任意层**都生效——中文层取第 N 个候选译文的**首义**、译文层取第 N 行；越界/无译文时放行宿主。

## Decision

v2 词典格式**不变**：译文仍是单一字符串字段，多义以一条 `a; b` 全串存储（m6 停止截断）。拆分发生在展示/上屏时，而非数据层。

- **m6 构建**（`crates/zhu-ye-dict/src/m6.rs`）：`clean_translation` 保留整段多义（`hello; hi`），仅按义清尾部标点（经 core `split_translations`）；你好补丁改为 `hello; hi`；词性前缀改为**按义**前置（`pos_label_each`）：`hello; hi` + `int.` → `int. hello; int. hi`。
- **反查索引**（`crates/zhu-ye-core/src/dict_builder.rs`）：按键按义生成（每义 `split_translations` → `strip_pos_prefix`），配按源的 `seen_keys` 守卫；加载器的 (key, word) 严格递增排序要求不受影响（一词多键可正常排序）。
- **IME 译文层**（`crates/zhu-ye-ime/src/input.rs`）：`build_translation_candidates` 把每个带译文的候选按义展开为多行（text=原词、translation=单义、拼音/来源透传、行序号微降 score 保序）；中文层候选窗副文本只显示**首义**（避免 `int. hello; int. hi` 溢出）。
- **上屏剥词性**：显示保留前缀（`v. suspicious` 照常展示），凡离开发动机的提交文本一律剥离——`commit_candidate`（译文层选择 + 空格预览经 `display_text`）与新直上屏路径。
- **Ctrl+数字**（`crates/zhu-ye-ime/src/tsf.rs`）：新 `KeyAction::CommitTranslation(usize)`。`plan_action` 的修饰键分支调 `plan_modifier_key(wparam, lparam, shift, ctrl_held, engine)`：仅当 **Ctrl 单独按下**（排除 Alt）、组合态且 `engine.can_translate_by_index(index)`（中文层：可见候选译文拆义非空；译文层：拆分行存在）时产出 `CommitTranslation`；其余 Ctrl/Alt 组合一律返回 `None` 放行宿主。`commit_text` 经 `engine.preview_translation_by_index` 预取文本，`sync_engine` 经 `engine.commit_translation_by_index` 推进状态（与 commit_candidate 同款收尾：记用户词+拼音、清组合、置前词、刷联想）。

先拆后剥的顺序不可颠倒：对整串 `int. hello; int. hi` 剥前缀会残留 `int. hi`——拆义必须先于剥词性。

## Alternatives considered

**数据层拆分多义词条（每义一条独立词典记录）。** 否决：候选生成按候选文本去重，同词多义条目会被合并回一行；且会改动 v2 格式、加载器排序键与正反查语义，无用户可见收益。

**上屏整串、最后统一剥前缀。** 否决：译文层提交候选携带整串 `int. hello; int. hi`，对整串剥前缀会给第二义残留 `int. `；反正要拆义（已实施）。

**Ctrl+数字复用 `commit_candidate`（伪造选择）。** 否决：`commit_candidate` 按当前层决定取 text 还是 translation，无法表达中文层"取首义当作文本上屏且层不动"的语义；专用 `commit_translation_by_index` 使两层语义显式。

**Ctrl+数字无条件吃键。** 否决：按用户决策，无译文/越界必须放行宿主（如英文应用里的 Ctrl+1）。

**Alt+数字或 Ctrl+字母也走修饰键分支。** 否决：仅 Ctrl+数字且候选可译才吃键；Alt 组合（及其它 Ctrl 快捷键）本来就有"放行宿主"的既定行为，保持不变。

## Consequences

- 译文层行数=义数（你好 → 两行 `hello`/`hi`）；长尾多义词行数变多。行内 score 微降保证同词多义顺序稳定。
- 词典格式未动，加载器、正查/反查（`zh_to_en`/`en_to_zh`）照常工作；反查键按义增多，词典体积小幅上升。
- 词性前缀保持用户可见（译文层显示），延续 T-115 决策；仅上屏文本剥前缀。`previous_word` 存剥后的文本（bigram 上下文不受影响）。
- `base.zyct` 内容变化（译文变长、按义词性），整条构建链需与 IME DLL 一起重建部署。
- Ctrl+数字仅在"组合态 + 候选可译"时对输入法有意义，其余情况按键仍到达宿主。

## Related

- [验收修复批三——译文词性标注（T-112 后续）](2026-10-07-acceptance-fix-batch-3.zh.md)：m6 词性前缀决策的源头；本 note 将其细化到按义应用，标签映射与显示格式仍由批三定义。
- [候选窗 TSF 集成](2026-09-20-candidate-window-tsf-integration.zh.md)：译文层机制（`CandidateLayer`、Tab 切换、提交回中文层）。
- [多音缺读补丁接入产品链 base（T-129）](../process/2026-10-08-polyphone-patch-into-base-pack.zh.md)：`base.zyct` 随附的构建/部署链。
