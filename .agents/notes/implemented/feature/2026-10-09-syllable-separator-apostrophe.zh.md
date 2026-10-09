# Agent Note: 音节分隔符 `'`（自动边界 + 手动歧义覆盖）

Status: implemented

[English](2026-10-09-syllable-separator-apostrophe.md) | 中文

## Problem

输入拼音串（如 `nihao`）原样显示在候选框上方，无法表达音节边界：
歧义键如 `xian`（整读 `xian` 或分读 `xi an`）用户无法手动区分，长串
（`youmeiyoushenmeren`）也难读——候选窗页眉把一切拼音挤成不可分隔的一行。
T-021 有意对整词命中的键不做多音节组合候选，并把"显式 `xi an` 分隔符"
留作未来增强——本 Note 即该增强的落地。

用户决策（2026-10-09，四项澄清问答）：

1. **分隔符进输入串**（`sep_in_composing`，推荐项）：页眉显示 `xi'an`；
   退格先删分隔符再删字母。
2. **去噪+边界约束组合优先**（`query_contract`，推荐项）：所有查询
   视 `ni'hao ≡ nihao ≡ ni hao`（分隔符对词典键是噪声）**且**分隔符
   约束候选生成的音节边界。
3. **自动分隔作用于全部音节边界**（`auto_scope`，用户选择非推荐项）：
   首选切分方案的每个边界都显示 `'`（如 `ni'hao`），因此必须确定
   "首选切分"规则。
4. **空闲态 `'` 放行宿主**（`idle_apostrophe`，推荐项）：无组合时
   半角 `'` 正常键入。

## Decision

- **查询键保持纯拼音**：`InputEngine::composing` 永不包含 `'`。手动分隔
  存入新字段 `manual_seps: Vec<usize>`（纯键上的字符边界下标，升序）。
  `insert_separator()` 在当前缓冲末尾追加一个分隔符（引擎无光标模型），
  同位置重复插入被拒绝。退格**先删串尾手动分隔符**再弹出字母；每次
  插入/删除后自动边界实时重算。
- **显示串**：`composing_display()` 在「手动分隔符 ∪ 自动边界」位置插入
  `'`。自动边界来自 `segment_all` 首方案（首选切分，最长匹配优先），
  由输入引擎内 `preferred_segment_boundaries` 计算（全部边界都显示）；
  串尾手动 `'` 单独补画。自动插入仅在中文模式生效（英文组合原样显示）。
  `candidate_ui_view().composition` 改用显示串，候选窗页眉由此展示
  `ni'hao`/`xi'an`。
- **核心约束切分**：`pinyin::segment_constrained(table, input, hard)`
  过滤 `segment_all` 方案，仅保留音节终点覆盖全部硬边界者（边界 `0`
  恒成立；任一硬边界无法满足则结果为空）。
- **约束候选组**：`candidate::constrained_segment_candidates` 取满足硬边界
  的**第一个**方案，按与 `generate_candidates` 相同的规则（每音节取
  词典最高频词）拼成一个组合候选。只取首方案，多方案噪声（`nihao`
  还可切 `ni,ha,o`）不会泄漏进来。输入引擎在全部追加组组装完成后用
  既有 `prepend_group` 把该组前置——手动分隔表达强意图（`xi'an` →
  「西安」顶置），且无手动分隔时常规管线零介入（T-050 基线：无手动
  `'` 时逐位不变）。
- **键路**：`classify_key` 把 `VK_OEM_7` 无 Shift 归为 `KeyAction::Separator`
  （Shift+`'` 的 `"` 放行宿主）；`plan_action` 仅在中文模式且有活动组合时
  接收；空闲态放行宿主直出半角 `'`；`sync_engine` 路由到
  `insert_separator`。
- **查询归一化是结构性的**：`'` 从不进入 `composing`，`ni'hao` 与
  `nihao` 共用同一词典键；分隔符只约束切分与显示。

## Alternatives considered

- **把分隔符存进组合串**（显示串=查询键）：所有词典查询与候选路径都要
  剥 `'`，退格/光标语义需要字符分类遍历。落选：纯键+旁路字段让现有
  查询全部不动。
- **约束组按全部方案生成**：用户打 `ni'hao` 会连 `ni,ha,o` 组合
  （「你哈哦」）一起产出。落选：首方案规则与显示边界一致且无噪声。
- **分隔符同时抑制整词命中**（`xi'an` 绝不显示「先」）：T-021 保持
  精确键整词优先；在分隔符后移除整词会不可预期地改变候选页组成。
  落选：改为约束组顶置，用户仍先看到「西安」且常规候选保留。
- **光标模型 + 串中插入分隔符**：TSF 目前没有组合光标；用户确认的范围
  （末尾追加 + 尾部分隔优先退格）无需光标即可成立。延后。

## Consequences

- 歧义键现在可见且可功能控制：`xian` 原样显示（首选 `[xian]`）；
  打 `xi` + `'` + `an` 显示 `xi'an` 且「西安」置顶。
- 页眉宽度略增（`ni'hao` 比 `nihao` 宽一个字形）；T-124 页眉自适应宽度
  已覆盖（不截断）。
- `Enter`/原样提交仍输出纯拼音（`xian`），绝不输出 `'`——分隔符是
  显示/查询装置，不是上屏字符。
- 首选切分优先 ⇒ 自动 `'` 跟随最长匹配：`zhuan` 保持 `zhuan`（不显示
  `zhu'an`），除非用户手动插入 `'`。

## Testing

- `segment_constrained`：硬边界 `[2]` 对 `xian` 只留 `[xi,an]`；不可满足/
  越界边界返回空；`0`/`len` 边界恒成立等价断言；边界 `0` 无约束等价。
- `constrained_segment_candidates`：`xi'an` → 「西安」单候选；多方案输入
  只取首选（`nihao` → 「你好」，无「你哈哦」）；单音节/不可切分返回空。
- 引擎：显示串（`ni'hao`、`xian`、`xi'huan`）、尾 `'` 优先删除、退格后
  自动分隔重算、跨删除保留手动分隔、约束组置首的候选次序、视图组合串
  （`xi'an`）、无手动分隔时基线一致。
- TSF：`classify_key`（`VK_OEM_7` → Separator、Shift → None）与
  `plan_action`（组合态进入、空闲/英文态放行）。

## 分隔符显示字形修订（T-134，用户反馈"分隔符看起来是个逗号在上面"）

用户 2026-10-09 反馈：显示器上的分隔符"看起来像字顶的逗号"，应当像搜狗
那样"只是个撇号"——即**英文风格撇号**。根因：T-128 显示用键盘直撇
U+0027，而页眉组合串由**宋体（SimSun）粗体**渲染
（candidate_window `create_font(…, "SimSun")`，FW_SEMIBOLD）；宋体 U+0027
字形是"顶部带钩的竖线"，粗体下读作漂浮的逗号。搜狗/微软拼音用英文弯撇。

修复（T-134）：input.rs 新增 `pub const SYLLABLE_SEP_DISPLAY: char = '\u{2019}'`
（U+2019 RIGHT SINGLE QUOTATION MARK，英文排版正式撇号）；`composing_display`
两处插入改用它。纯显示层——查询键 `composing` 依旧无分隔符、`manual_seps`
依旧存字符下标、键语义不变。附带修复：`preview_after_backspace` 原来按字节
切片 `[..len-1]` 去掉尾分隔符，遇 3 字节 U+2019 会切出半个字符（潜在
panic），改为按字符 `pop()`。测试断言全部用 `\u{2019}` 转义。候选行拼音
（Segoe UI 渲染、词典数据为儿化 ASCII `'`，如 `nǎ'er`）不动——其字形观感正常。

## 页眉分隔符改画成直撇（T-139，用户"看起来是个逗号在上面"第二轮）

T-134 把显示字形换成 U+2019，但页眉字体是宋体，U+2019 在宋体下仍然是
"粗圆头逗钩"。96dpi 像素证据：宋体 U+2019 字形 4×5px、带圆头
（12 个深像素）；Segoe UI 的 U+0027 直撇是 2px 宽的细竖条（6 个像素）。
用户要的是"只是撇号"的观感。

修复（T-139）：页眉渲染改为逐段绘制。`draw_header_mixed` 按
`SYLLABLE_SEP_DISPLAY`（U+2019）切开显示串，各段在公共基线上用各自字体
绘制：主段继续用宋体半粗（`main_font`）；每个分隔符段改用英文键盘直撇
`'`（新常量 `SYLLABLE_SEP_APOSTROPHE: char = '\u{0027}'`）、以 Segoe UI
常规字体（`sep_font`，字号与页眉一致）绘制。纯显示层——`composing` /
`manual_seps` / 键语义不变。辅助函数：`draw_seg`（先 SelectObject 段字体，
再对段矩形 DrawTextW：段顶 = 基线 − 该字体 ascent、段高 = tmHeight）与
`text_extent`（对已选字体 GetTextExtentPoint32W 取布进宽）。分隔符段矩形
右缘加 8px 缓冲：「布进宽」含字形左侧空白（`'` 笔画偏右），若按布进宽
精确裁右缘，字形主体会被裁成 1–3px 残边（调试中曾一度误判为"没画出来"，
实为直撇本来就细）。demo 验证：`xi'an` 渲染为 宋体 `x` · Segoe UI 直撇
（位于 x-height 上方）· 宋体 `a n`，间距不变；浅/深主题均正常。候选行拼音
（`nǎ'er`）不受影响。

## Related

- [全拼切分核心（T-007）](../../implemented/feature/2026-09-19-full-pinyin-segmentation-core.zh.md) —— 本设计依赖的 `segment_all` 与首方案规则。
- [候选排序静态模型 / T-021 噪声清理](../../implemented/feature/2026-09-19-candidate-ranking-static-model.zh.md) —— 本 Note 实现的「显式 `xi an` 分隔符」增强；整词优先保持不变。
- [候选窗页眉内容宽度（T-124）](../../implemented/feature/2026-10-08-candidate-window-header-content-width.zh.md) —— 页眉渲染显示串。
