# Agent Note：验收修复批四——候选拼音横截断修复与声调显示、任务栏"未激活状态"双语言档案（T-112 后续）

Status: implemented

[English](2026-10-07-acceptance-fix-batch-4.md) | 中文 | 中英镜像

## 问题

2026-10-07 批三部署后，用户在真机验收时再报两项：

1. **候选拼音显示不全，一直有"…"**：拼音带（候选词上方小号拼注）
   总是被截断成省略号；用户同时要求拼音**显示音调**。
2. **任务栏无"输入法禁用/未激活状态图标"（IME Disabled Icon）**：
   用户在未激活/英文状态下看不到斜杠圆圈指示图标，要求让该图标
   出现。

## 决策

两项在同一验收修复批次落地（2026-10-07）。

### 1. 拼音横截断根因与修复 + 声调旁挂表

**截断根因**：`candidate_window.rs` 中拼音明细行（pin）的矩形
`pin_rect` 右界被框在主文本列右界 `main_col.right`（列宽按 CJK
主文本宽度估算），而拼音是 ASCII，实际像素宽度超出列宽 →
`DrawTextW(DT_END_ELLIPSIS)` 恒补"…"。

**修复**：`pin_rect.right` 放开到整行右界 `row_ui.right`，拼音带
不再随主文本列宽框死（带注释"拼音带不随主文本列宽截断"）。

**声调显示方案（旁挂带调拼音表，方案 U）**：运行时输入流只有无调
拼注（sheng 是「生成/声称/花生…」的共码），**无法反推声调**（sheng
三读），因此从构建期带调数据直接发货：

- **构建**（`zhu-ye-dict/src/m6.rs`）：新增数字调→符号调转换器
  `tone_digits_to_marks`（a>e>o>i/u/ü 并列取后者 → `ui`→`uǐ`、
  `iu`→`iū`；`u:`→`ü`；声调 0/5 不加标；**含非拼音字符（如 `zhong!`）
  返回 None**——对 CEDICT 尾部数字调号与元音按位替换，先分离数字再
  校验剩余字符全为 `[a-z]`/`ü`）。**字级常用音优先**：
  `CHAR_PREFERRED_PINYIN` 改为带调值（着→`zhe` 轻声无调、说→`shuō`、
  还→`hái`），无调拼注用 `strip_tone_marks` 从带调值派生（zyct 内容
  不变）。词级带调来自 CEDICT 数字调转换（词命中优先标注），字级
  来自 kTGHZ2013 符号调（逐字兜底拼合）。
- **产物**：`build_base` 新导出 `data/artifacts/base.zyct.tones`
  （`#word` 节 118,842 条 + `#char` 节 8,102 条，共 126,944 行，
  2,677,520B；排序输出保证稳定）。**zyct v2 格式与内容零变化**
  （本轮重铸 SHA-256 前 16 位 `d669f834b2bfbd80` 与已部署 base
  完全相同，无需重部署 zyct）。
- **解析**（`zhu-ye-core/src/tone.rs`，新模块）：`ToneMap`
  `#word/#char` 两节解析（坏行跳过、未知节忽略）、
  `word_tone_spaced`（词命中优先，缺字返回 None）、5 项单测。
- **装配**（`zhu-ye-ime`）：`CandidateUiItem::pinyin_tone` 字段；
  `InputEngine::with_tone_map/tone_spaced/ui_item_for`；候选窗取词时
  `pinyin_tone` 非空直接显示，否则回退无调 `spell_pinyin`（译文模式
  不显示拼音带）；`tsf.rs` `tone_engine` 在引擎装配链尾部挂载
  （路径=`<plan.base>.tones` 即
  `%APPDATA%\zhu-ye-ime\dictionary.zyct.tones`，**整体读入内存**，
  不用 mmap → 覆盖部署不受旧进程文件锁影响）；读取失败/缺失只记
  log（`tone-map-missing`）回退无调，不阻断输入。
- 分词独有词（如「输入框」无 CEDICT 词条）走字级兜底 → 逐字带调拼合
  （`shū rù kuāng`）；用户新造词无带调数据 → 回退无调（如实降级）。

### 2. 任务栏"未激活状态图标"：双语言档案（方向 a 的忠实子集）

> **已被批五回退**（[2026-10-07-single-profile-revert.zh.md](2026-10-07-single-profile-revert.zh.md)）：
> 用户明确要求语言列表保持单一条目，双档案全链（注册/引擎装配/
> ying.ico/worker copy-icon）已移除；本段仅作决策记录保留。

**平台事实（先查明）**：

- Win11 任务栏的**斜杠圆圈"输入法禁用"图标**是由系统在
  **没有可用输入法启用**时显示的全局状态，第三方 TSF 没有任何接口
  可以自行注册它；
- 用户粘贴的四条方向逐一核过：**b**（`ITfInputProcessorProfileMgr`）
  是系统侧的档案管理器接口，供系统/管理工具查询与设置档案，不是
  输入法可调用以实现"显示禁用图标"的接口；**c**（开启桌面语言栏）
  T-046 已裁定不可行（Win11 无第三方语言栏项目渲染）；**d**
  （`Shell_NotifyIcon`）需要常驻进程，本输入法无驻留进程（纯 TSF
  DLL 宿主加载），不可实现；
- **a**（`ITfInputProcessorProfileMgr::GetCurrentLanguage`）拆开看，
  其可落实的忠实子集是**第二个语言档案**：微软拼音正是用"中/英"两个
  档案让 Win+Space 切换后任务栏图标随之改变。

**落地**：同一个 TIP 树、同一 `0x00000804` 段注册第二个语言档案
`{EF42481A-233D-4035-A80A-7F416BE0E6CA}`（英文态档案）：

- Description/Display Description=`竹叶输入法（英文）`、Enable=1、
  IconIndex=0；IconFile=独立图标文件 `ying.ico`（32px 橙底白字"英"，
  与 `zhu.ico` 同款 766B，`crates/zhu-ye-ime/assets/ying.ico`）；
- 用户用 **Win+Space** 在"竹叶中文 ⇄ 竹叶英文"两个档案间切换，
  任务栏指示器随档案显示竹/英图标；
- **引擎装配**（`tsf.rs`）：`active_language_profile` 经
  `CLSID_TF_InputProcessorProfiles`（CoCreateInstance）+
  `ITfInputProcessorProfiles::GetActiveLanguageProfile` 查询当前激活
  档案；`start_mode_for_profile` 纯函数：激活档案=英文档案 →
  会话起始模式 English，否则（中文档案/查询失败）回退配置默认；
  `create_text_service` 用该结果装配（含单测）；
- **身份契约**：`zhu_ye_core::identity` 增 `PROFILE_GUID_ZHU_YE_EN`；
  `ime-identity.ps1` 的 `New-TsfRegistration` 幂等重建双档案 + `Test`
  校验英文档案（Enable=1 且 IconFile 存在）；`verify-tsf-identity.ps1`
  交叉比对扩为 8 项；提权 worker `copy-icon` 白名单扩到 ying.ico
  （dst 固定 `tsf\ying.ico`，大小 766 校验）。

**如实告知的边界**（验收需向用户讲清）：Shift 单击切换是本会话内
状态，系统只跟踪 Win+Space 的档案级选择；因此 **Shift 切换到英文态
不会让任务栏立即显示斜杠/英图标**——该行为与原生中文输入法一致
（微软拼音在 Win11 的 Shift 切换同样不改变任务栏显示，且其"未激活"
斜杠仅在系统级禁用输入法时出现）。

## 部署

- 新 DLL `zhu_ye_ime_80DE7C22.dll`（2,191,872B）：worker
  `copy-dll-ver`（CLSID InprocServer32 + 中文档案 IconFile 指针切换）；
- `ying.ico`（766B）→ `C:\Program Files\zhu-ye-ime\tsf\ying.ico`
  （worker `copy-icon`）；
- 英文档案 5 项注册（Enable/Description/Display Description/IconFile/
  IconIndex）经 worker `registry-text` 写入 HKLM，`dump-ref` 双视图
  核对（中文档 `6315FE74…` 与新 `EF42481A…` 并存）；
- `dictionary.zyct.tones`（2,677,520B）→ `%APPDATA%\zhu-ye-ime\`
  （zyct 本体不重部署：SHA 与已部署完全一致）；
- ctfmon 重启；`verify-tsf-dll.ps1` 导出校验通过
  （DllGetClassObject/DllCanUnloadNow/dll_probe）；
- 门禁：fmt/clippy `-D warnings`/workspace 全测
  （322+8+60+210+29+143+102+7+2，共 883 用例）全绿、
  `git diff --check` 干净、verify-tsf-identity 8 项 PASS。

## 备选方案

**方案 A：zyct v3 附录带调拼音 + DICT_VERSION 升级。** 否决：改磁盘
格式要动加载器/版本协商/全部消费方，工作量约等于本方案两倍；旁挂
文件让旧客户端/新客户端共享同一 zyct，回退路径干净。

**运行时反推声调。** 否决：拼注到词的多对一映射（sheng → 生成/
声称/花生…）且候选窗同时列多候选，无唯一解；sheng 三读靠数据侧
出厂。

**用 `ITfInputProcessorProfileMgr` 把系统全局输入法切到"无输入法"
以触发斜杠圆圈。** 否决：等于把系统禁输入法，属于系统管理行为，
会清掉用户语言栏所有输入法且不可预期恢复，不是"显示图标"的正当
实现。

**`Shell_NotifyIcon` 常驻托盘图标。** 否决：无常驻进程（纯 TSF
DLL），且与任务栏指示器是两套 UI，用户要求的是指示器本身。

## 后果

- 拼音带不再被列宽带死；带调显示覆盖 126,944 词/字（构建期全量
  CEDICT + kTGHZ），分词独有词逐字带调，用户词无带调数据回退无调；
- 旁挂 `.tones` 是**排版期数据**（词 → 空格分隔带调拼音串），与
  zyct 加载路径完全解耦；缺失/损坏只降级不阻断；
- 双档案是**系统正规机制**（微软拼音同款）：用户获得 Win+Space
  中英文档切换 + 任务栏竹/英图标；Shift 切换不改变任务栏（与原生
  行为一致）——验收交底的措辞已固化在本 note 与 todos；
- 英文档 IconFile 指向独立 .ico 文件（非 DLL 内嵌），部署/重装依赖
  worker 或 `New-TsfRegistration`（幂等）同步分发 `ying.ico`；
  发行包资源清单（assemble-release）需后续补 ying.ico（本轮内网
  部署未走发行包）；
- DLL 版本化升级后旧宿主进程仍持旧 DLL，用户需**新开记事本**验收；
- 任务栏"未激活状态图标"（斜杠圆圈）最终仍由系统语义决定：它在
  **系统级别关闭全部输入法**时出现，任何第三方 IME 都无法自行点亮
  （如实记录，避免后续再接到同类请求）。

## 关联

- 批三（拼音上置小号字体的位置/字号机制延续，本批只改内容与矩形）：
  [2026-10-07-acceptance-fix-batch-3.zh.md](2026-10-07-acceptance-fix-batch-3.zh.md)
- 图标/名称与 T-114 设置入口（IconFile 机制 + DLL 内嵌竹图标，本批
  扩展出独立 ying.ico 的档案级 IconFile）：
  [2026-10-06-taskbar-icon-zhu.zh.md](2026-10-06-taskbar-icon-zhu.zh.md)
- 语言栏结论（c 方向不可行的依据）：
  [2026-09-28-language-bar-mode-icon.zh.md](2026-09-28-language-bar-mode-icon.zh.md)
- CEDICT 真实词典导入与译文来源：
  [2026-09-21-real-dictionary-import.zh.md](../../implemented/architecture/2026-09-21-real-dictionary-import.zh.md)
- 批五（本批 #2 双档案的回退记录）：
  [2026-10-07-single-profile-revert.zh.md](2026-10-07-single-profile-revert.zh.md)
