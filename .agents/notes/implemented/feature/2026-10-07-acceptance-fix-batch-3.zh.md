# Agent Note: 验收修复批三——设置入口注册/错序容错补全/拼音上置/译文全面补齐/英文译文/词性标注（T-112 后续）

Status: implemented

English | [中文](2026-10-07-acceptance-fix-batch-3.zh.md) | 中英镜像

## Problem

2026-10-07 批二部署后，用户在真机验收时再报六项：

1. **状态栏没有设置 GUI 图标**：切换竹叶输入法后任务栏只出现输入法
   状态栏图标，没有可点击的设置图标，无法点开 GUI 改皮肤。
2. **错序容错仍不完整**：`zhegnq` 打不出「正确」（用户期望识别为
   `zhengq` 意图）、`shegnc` 打不出「生成」、`zhagnh` 打不出
   「账号」；并要求候选显示正确拼音（如「账（zhang）号（hao）」）。
3. **候选拼音形式**：从括号内联（`生（sheng）成（cheng）`）改为
   词汇**上方、小一号字体**。
4. **译文仍大量缺失**：许多词打出来无译文；用户要求按**日志里已上屏
   的测试文字**逐词排查。
5. **英文输入应出中文译文**：输入 `hello`，候选应显示
   1 hello 你好。
6. **拼音候选的英文译文前应显示传统词性标注**。

## Decision

六项全部在同一验收修复批次落地（2026-10-07），随 DLL 版本化升级与
词典重铸一并交付。

### 1. 设置入口（EnableConfiguration 注册）

先做机制查明：Win11 任务栏输入法指示器是**系统固有 UI**，第三方
IME 无法在其旁自绘齿轮按钮（与 T-046 语言栏结论一致，注册表
`IconFile` 只能决定指示器图标本身）。系统对经典 TSF 输入法唯一的
"设置"入口是 **设置 → 时间和语言 → 语言和区域 → 中文(简体) →
键盘选项 → 竹叶输入法**，点击经 `ITfFnConfigure::Show`
（T-114 已实现，GetDisplayName "竹叶输入法 设置"）拉起
`zhu-ye-settings.exe`。

要令经典 TSF 输入法在该页出现可点击设置链，需要在 TIP 注册键写
`EnableConfiguration = 1`（DWord）。改动：

- `scripts/ime-identity.ps1`：`New-TsfRegistration` 写
  `EnableConfiguration`；`Test-TsfRegistration` 增加该校验；
- 生产机器经提权 worker `registry-text`（kind=dword）写入
  HKLM TIP 键；
- 补充可达路径：开始菜单"竹叶输入法设置"快捷方式（批二已装）。

### 2. 错序容错前缀分支补全（append_transposed_group）

根因：错序串通常带有**非空前缀完成**（`zhegnq` 前缀 `zhe` 出
「这/者」），输入流因此走**前缀分支**并在填完前缀候选后提前
`return`，从未挂载批二的错序组（`transposed_candidates` 只在主链路
分支调用）。

修复（`crates/zhu-ye-ime/src/input.rs`）：抽公共入口
`append_transposed_group(&self, main, &composing)`——统一条件：
`enable_fuzzy` 开启、主候选不足一页（`main.len() < page_size`）、
输入纯 ASCII、不可完整切分、无直接命中时，挂
`transposed_candidates` 结果（`CandidateSource::Corrected` 且
`pinyin` 覆盖为正确变体）。**主链路与前缀路径共用此入口**，正常输入
（可切分/带满候选/模糊关闭）逐位不触发，行为不变。

### 3. 候选拼音上置小号字体

- `spell_pinyin` 输出格式从括号内联改为空格连接音节：
  `生（sheng）成（cheng）` → `sheng cheng`；
- `candidate_ui.rs` 计量：`pin_font_height = dp(11.0)`、
  `pin_line_gap = dp(3.0)`；
- `candidate_window.rs` 绘制重构：`show_pin` 行（拼音带非空且与
  主文本不同）在**词上方**画拼音带（独立 `pin_font`、主题次级色），
  主文本/译文区整体下移；行高 36px、面板固定宽 360px 不变，拼音带
  不截断；`pin_font` 随 DPI 重建并随窗口释放。

### 4. 译文全面补齐（本批最大头）

**部署错目录发现**：批二把新 base 部署到了
`%APPDATA%\ai-zhu-ye-ime\`，而运行中的 ime 实际加载
`%APPDATA%\zhu-ye-ime\dictionary.zyct`（10/5 旧 base，译文仅
75,063 条）——**批二的译文修复从未生效**。本批把重铸 base 部署到
正确目录（见部署段），并修掉一批用户上屏词的缺译/缺词：

- 译文精修表 `TRANSLATION_PATCHES` +19（更多/哪位/救来/泥湿/逆事/
  逆施/升成/四书五经/书/输入/也/和/上/后/词/帐号/水…）；
- **kTGHZ 单字首音生僻**：kTGHZ2013 拼音列按新华字典序，个别常用
  字首音非常用（着→`zhāo`、说→`shuì`），无 CEDICT 词级注音的多字
  词逐字注音因此落错（想着→`xiangzhao`、说来→`shuilai`）。
  新增 `CHAR_PREFERRED_PINYIN` 常用音覆盖表（着→zhe、说→shuo），
  想着/说来恢复；"有什么"无 jieba/CEDICT 词条，属短语由分词覆盖；
- **wordfreq 低估现代常用词**：输入框 zipf≈1.5（≈1510）低于 jieba
  扩充门槛（min_score 2000）被卡。新增 `WORD_FREQ_PATCHES` 保底
  词频表（输入框→2500）。未采用 `max(wordfreq, jieba_score)`
  （会让 jieba 高分生僻词盖过骨架词，见 Alternatives）；
- **同音位次**：生成（4440）被声称（4480）压为 shengcheng 次位，
  与用户点名预期相反 → 生成提频 4600 抢回首候选。

重铸结果：词条 287,202（+1 输入框）、译文 **119,275**（+8 净增
精修）、拼音索引 221,878、词性标注 71,967。

### 5. 英文候选带中文译文

`zhu_ye_core::dict::Dictionary` 特征新增默认方法
`translate_en_to_zh(&self, word) -> Option<String>`（默认 None），
`DictionaryFile` 覆盖实现 → 复用既有 EnToZh 反查索引
（`find_en_to_zh`）。`input.rs` 英文候选块对无译文候选挂译文
（hello→你好）。zyct/en.zyen 格式零变化，英文候选来源与排序不受扰。

### 6. 译文前传统词性标注

- `load_jieba_pos`：解析 jieba 第三列（`词 频次 词性`）；
- `pos_label`：jieba 细类归并为传统缩写——n./v./adj./adv./pron./
  prep./conj./int./num./cls./aux.，未映射类（生僻）不标注；
- `build_base` 新 e) 段：译文前拼 `"{label} "`（生成 →
  `v. to generate`）；`WORD_POS_PATCHES` 修 jieba 误标（谢谢
  `nr`→int.、想法 `v`→n.、总是 `c`→adv.、可以 `c`→aux.、
  用 `p`→v.）；
- **反查键纯净**：core 新增 `POS_PREFIXES` 白名单与
  `strip_pos_prefix`（字节级 `eq_ignore_ascii_case` 前缀匹配，
  ASCII 前缀切片安全），`dict_builder` 反向键剥前缀，英→中反查
  不受词性前缀污染（`-r "to generate"` → 生成 ✓）。

## Deployment

- 新 DLL `zhu_ye_ime_6F61B01F.dll`（2,176,000B）：`copy-dll-ver`
  落地并切 CLSID 指针，`IconFile` 回写 `tsf\zhu.ico`（HKLM/HKCU），
  `EnableConfiguration=1` 写入 HKLM TIP 键，ctfmon 重启（worker 五
  步全部 ok）；
- `dictionary.zyct`（28,098,872B）部署至**正确目录**
  `%APPDATA%\zhu-ye-ime\`：旧档占位（24,192,872B）映射在多个宿主
  进程中（浏览器/记事本等 TSF 宿主），直接覆盖报"用户映射区域"错；
  用**先写 `dictionary.zyct.new` 再改名交换**的方式绕开（旧映射句柄
  不受影响），旧档备份 `.bak_20261007b`；
- cli 验收：正确→`adj. correct`、生成→`v. to generate`（首位）、
  账号→`n. account number`、想着（`xiangzhe` 首位）、
  `-r hello`→你好、`-r "to generate"`→生成；
- S-1 `audit-coverage` 抽检通过；fmt/clippy `-D warnings`/
  workspace 全测 873 用例全绿、`git diff --check` 干净。

## Alternatives considered

**在任务栏指示器旁自绘齿轮按钮。** 否决：指示器（"中/英"框）是
系统 UI，无第三方扩展位；注册表只控制语言档图标。系统键盘选项入口
是唯一官方路径，配合开始菜单快捷方式覆盖可达性。

**对 jieba 扩充直接取 `max(wordfreq, jieba_score)` 提词频。** 否决：
jieba 语料频次对生僻词膨胀（垸 jieba 5690 vs wordfreq 2450，会盖过
同音骨架词「元」5600）的既有问题会复发；白名单保底表只维修用户
证据词，不改变其余排序。

**重排 kTGHZ 多音列或全部改取第二音。** 否决：新华字典序对绝大多数
字就是常用音，全局改序会让大批多音字词误注；白名单覆盖表
（着/说两个证据字）零风险。

**词性直接写入 zyct 新字段。** 否决：改磁盘格式牵动加载器/词典
版本；前缀拼入显示串 + 反查剥前缀零格式变更，旧客户端也能读。

## Consequences

- **词典部署目录教训固化**：ime 数据目录是 `%APPDATA%\zhu-ye-ime`
  （代码统一），验收固定用 `dict <部署位文件>` 复核，不再出现"部署
  到位但没生效"；
- 词典文件被 TSF 宿主 mmap 后不可覆盖/删除，升级用改名交换；旧宿主
  进程内存仍是旧词典，**用户需新开记事本**（新进程读新词典）；
- 词性标注覆盖 71,967 条（约词条 25%），长尾词无词性前缀；jieba
  tag 表与 `pos_label`/`WORD_POS_PATCHES` 形成维护耦合；
- 构建产物体积 26.9MB→28.1MB；词性前缀使译文显示变长，候选窗译文
  列自适应截断，反查键经 POS_PREFIXES 白名单保持纯净；
- 任务栏指示器不显示齿轮是系统限制（非缺陷），设置入口在系统键盘
  选项页 + 开始菜单快捷方式；用户如有困惑需在交付说明中讲清。

## Related

- 批二（含误部署记录与既有 4 项修复）：[2026-10-07-acceptance-fix-batch-2.md](2026-10-07-acceptance-fix-batch-2.md)
- 图标/名称与 T-114 设置入口：[2026-10-06-taskbar-icon-zhu.md](2026-10-06-taskbar-icon-zhu.md)
- CEDICT 真实词典导入与译文来源：[2026-09-21-real-dictionary-import.md](../../implemented/architecture/2026-09-21-real-dictionary-import.md)
- M7 纠错/候选排序（错序容错补充而非取代）：[2026-09-19-candidate-ranking-static-model.md](../../implemented/feature/2026-09-19-candidate-ranking-static-model.md)
