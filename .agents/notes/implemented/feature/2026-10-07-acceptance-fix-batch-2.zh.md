# Agent Note: 验收修复批二——顿号/错序容错/候选拼音/译文补齐/设置补装（T-112 后续）

Status: implemented

English | [中文](2026-10-07-acceptance-fix-batch-2.zh.md) | 中英镜像

## Problem

2026-10-07 用户在真机验收时报出四项缺陷，均发生在此前部署的
v0.1.2 批一 DLL 上：

1. **顿号无法输入**："、"号无论经由 Ctrl+Space 还是 Shift 切换中文
   模式后敲击均无法上屏——反斜杠键（`VK_OEM_5`）未被中文全角标点
   路径接管，被打成 `\` 或直接丢弃。
2. **快打错位无容错**：输入稍快时相邻字母颠倒（`xiagnzhe`、
   `zhegnq`、`shegnc`、`zhagnh`），候选一空。既有 M7 纠错
   （`corrected_candidates`，模糊替换/少字母补全，FR-024）只处理
   **可完整切分**的串；错位串本身不可切分，两条链路（主候选+纠错）
   全部落空。用户还要求候选词汇旁**显示正确的拼音**（输入 `shegnc`
   候选应显示"生（sheng）成（cheng）"，`zhagnh` 意图"账（zhang）号
   （hao）"）。
3. **常用词无译文**：`nihao` 译文出现 `hello; hi` 而期望只有
   `hello`；大量常用单字/组词（是、我、你、谢谢、再见…）译文为空。
   根因：CEDICT 译文此前只落入 jieba 扩充与 CEDICT 兜底两段，**骨架
   词（官方拼音表）在 `build_base` 第 b) 段命中 `merged` 后直接
   `continue`，从未补 CEDICT 译文**；且首义原样保留（`hello; hi`）。
4. **设置窗口不可达**：机器上 `Program Files\zhu-ye-ime\` 只有
   `tsf\` 目录与 DLL，无 `zhu-ye-settings.exe`/快捷方式；右键设置
   入口（T-114，ITfFnConfigure）指向的 exe 不存在。

## Decision

四项全部在同一验收修复批次落地（2026-10-07），并随 DLL 版本化升级
（见部署段）与词典重铸一并交付：

### 顿号全角化

`tsf.rs` 新增 `KeyAction::PunctDun`：`VK_OEM_5` 无 Shift 时
`classify_key` 判为顿号，`plan_action` 中文模式返回动作、英文模式
放行反斜杠；`apply_action` 与逗号/句号同分支——引擎活跃时先提交
候选再插「、」；`needs_edit_session`/`sync_engine` 一并扩展。三枚
中文标点（，。、）现在走同一门禁与提交路径。

### 错序容错（transposed_candidates）

`zhu-ye-core` 新增 `transposed_candidates(table, dict, pinyin, cap)`：
输入为纯 ASCII、**无整词命中且无法完整切分**时，枚举每对相邻字母
交换变体（`xiagnzhe` 七个交换位，含 `gn→ng` 的 `xiangzhe`）：

- 变体**可完整切分** → 走主链路 `generate_candidates`（整词优先、
  音节切分组合），`xiagnzhe→想着`；
- 变体**仍残缺**（`zhangh`、`zhengq`、`shengc`）→ 走前缀链路
  `generate_prefix_candidates`，`zhangh→账号`、`zhengq→正确`、
  `shengc→生成`。

候选标 `CandidateSource::Corrected` 且 `pinyin` 覆盖为变体（正确
拼音）。ime 侧在主候选**不足一页**且 `enable_fuzzy` 时把错序组挂在
纠错组之后、缩写组之前——正常命中输入零影响（候选带满不触发）。
与 FR-024 `corrected_candidates` 互补不取代：后者管可切分串的模糊
替换/少字母补全，前者管不可切分串的相邻交换。

### 候选拼音显示

`CandidateUiItem` 增 `pinyin: String` 字段（所有构造点补齐），
`candidate_ui_item` 从候选带出；候选窗渲染 `spell_pinyin` 按标准
音节表把主文本拼成"词（音节）"（`生成`→`生（sheng）成（cheng）`）。
音节数≠字数、无拼音或非 CJK 主文本原样返回，译文层（英文主文本）
不拼注。

### 译文补齐

`zhu-ye-dict` `m6.rs`：

- `clean_translation`：取 CEDICT 首义分号前段、去尾部 `!`/`.`/`…`
  并保留括注（语义信息）；
- `TRANSLATION_PATCHES`（约 45 条自撰精修）：是/你好/谢谢/再见/
  我/你/他/她/它/们系列/指示代词/疑问词/礼貌语等，覆盖 CEDICT
  首义不佳或缺失的常用词（`是`→`yes; to be`、`你好`→`hello`）；
- `build_base` 第 b) 段**不再跳过骨架词的译文**（`continue` 前把
  译文位留给补译段），新增第 d) 段：译文为 None 的词优先查
  精修表、再查 CEDICT 净化首义。

重铸结果：词条 287,201、**译文 119,267 条**（此前骨架段全空）。
构建时源校验拦截 CC-CEDICT 站点月度更新（本地旧档与 pin 均过期），
下载新版校验后**三次重锁** `data/pins/cedict.json`（SHA-256
`DFC286BB…`、125,238 行、内卷/躺平/元宇宙新增词条齐全），遵循
[源 pin 流程](../../implemented/process/2026-09-28-dictionary-source-pins-and-fetch-script.md)。

### 设置窗口补装

- worker（提权通道）新增白名单操作 `copy-exe`（source 锁定
  `target\release\*.exe`、target 锁定 `Program Files\zhu-ye-ime\
  bin\*.exe`），把 `zhu-ye-settings.exe` 与 `zhu-ye-updater.exe`
  部署到 `bin\`；
- 开始菜单快捷方式（用户级 `APPDATA\...\Start Menu\Programs\
  竹叶输入法设置.lnk`，图标指向 `tsf\zhu.ico`）；
- 设置入口本身已在 DLL（ITfFnConfigure，GetDisplayName
  "竹叶输入法 设置"），exe 落位即可达。
- worker 部署脚本修复：仓库侧无 BOM UTF-8 被 PowerShell 5.1 按
  ANSI 代码页解析，中文注释/字符串乱码致脚本解析失败（计划任务
  Last Result=1）；**部署副本必须带 UTF-8 BOM**。

### 部署（版本化升级）

新 DLL `zhu_ye_ime_2A85B1EF.dll`（2,171,392B）经 `copy-dll-ver`
落地并把 CLSID InprocServer32/IconFile 指针切到新文件，随后两地
`IconFile` 回写 `tsf\zhu.ico`（任务栏"竹"图标），ctfmon 重启；全新
`dictionary.zyct`（26,983,690B，base 重铸产物）覆盖部署至
`%APPDATA%\ai-zhu-ye-ime\`（旧档备份 `.bak_20261007`）；cli 实查：
`是→yes; to be`、`你好→hello`、`谢谢→thanks; thank you`、
`再见→goodbye`、`我→I; me`。

## Alternatives considered

**在 `corrected_candidates` 内部为不可切分串加交换变体。** 否决：
该函数的前置契约（可切分）与返回值语义（逐音节替换）都会被搅浑，
且调用点只有 ime 一处；独立函数职责单一、可独立单测
（core 43 项候选测试、ime 拼注测试均覆盖用户点名四例）。

**模糊替换熵扩（把不可切分串逐位替换成邻键字母）。** 否决：错位
是"交换"不是"替换"（用户四例全部为相邻两字母互换，`gn`/`ng`
手序颠倒），位替换会产生噪音变体且切分可能反而更远；交换枚举
恰好覆盖用户行为且每变体必有正确拼音可显示。

**译文对全部词条直接取 CEDICT 首义。** 否决：CEDICT 首义常为
词典学定义（`是` 首义 "to be (followed by substantives only)"），
直出会污染译文层；精修表优先、CEDICT 兜底的两级策略对用户点名的
高频词给出自然义，长尾仍有词典学定义兜底。

**设置窗口丢进 tsf DLL 内嵌。** 否决：设置窗是独立进程更稳（TSF
DLL 被多个宿主加载、进程内 UI 生命周期与宿主绑定）；ITfFnConfigure
本就是"拉起外部设置程序"的官方语义（微软拼音同样外拉）。

## Consequences

- 中英文标点（，。、）统一提交路径，`\` 键在英文模式语义不变；
- 错序组只在候选不足一页介入，正常输入逐位不变；错序候选带
  正确拼音，候选行整体加拼注后变宽，分栏（row_split）自适应，
  九页满页时拼音拼注直接可见；
- 词典重铸改变构建产物哈希与体积（24.2MB→26.9MB），T-050
  eval 基线的命中率/词频分布随之变化；CEDICT 月度更新触发 pin
  重锁已成为常规维护动作（记录在 pin note）；
- 部署面新增 `Program Files\zhu-ye-ime\bin\` 与开始菜单快捷方式，
  卸载脚本需覆盖；worker 副本带 BOM 规则须保持（源码侧保持无 BOM
  便于 git，仅部署副本重写 BOM）；
- 测试与门禁：fmt/clippy `-D warnings`/workspace 全测
  （203+29+137+102+7+2）全绿、`git diff --check` 干净、
  verify-agent-notes 76 份与 verify-translation-pairs 65 配对通过。

## Related

- 图标/名称与 T-114 设置入口：[2026-10-06-taskbar-icon-zhu.md](2026-10-06-taskbar-icon-zhu.md)
- CEDICT 真实词典导入与首义来源：[2026-09-21-real-dictionary-import.md](../../implemented/architecture/2026-09-21-real-dictionary-import.md)
- 源 pin 与重锁流程：[2026-09-28-dictionary-source-pins-and-fetch-script.md](../../implemented/process/2026-09-28-dictionary-source-pins-and-fetch-script.md)
- M7 纠错/整句原链路（本次补充而非取代）：[2026-09-19-candidate-ranking-static-model.md](../../implemented/feature/2026-09-19-candidate-ranking-static-model.md)
