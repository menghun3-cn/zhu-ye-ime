# Agent Note：批四英文态档案回退——单语言档案回归（T-112 后续批五）

Status: implemented

[English](2026-10-07-single-profile-revert.md) | 中文 | 中英镜像

## 问题

批四（[2026-10-07-acceptance-fix-batch-4.zh.md](2026-10-07-acceptance-fix-batch-4.zh.md)）
为任务栏"输入法禁用/未激活状态图标"诉求落地了**双语言档案**（同 TIP
树、同 0x00000804 段第二档案 `{EF42481A-…}` + `ying.ico`）。部署后
用户在系统键盘布局中看到两个"竹叶输入法"选项（图标"竹"与"英"），
明确要求：

> 不应该出现双选项，而应该回退单档案；"中"、"英"应该在设置的
> 图标上面进行变化，而不是像现在这样出现两个输入法。

## 决策

**回退单语言档案**（英文态档案从注册表与引擎装配全链移除）——用户的
产品形态判断优先：语言列表中的输入法条目必须保持单一（与用户心智
模型一致），中/英模式是会话内状态，不得占用系统语言列表。

### 回退范围（代码）

- `zhu_ye_core::identity`：删除 `PROFILE_GUID_ZHU_YE_EN` 常量及其
  测试断言（guid 文本校验回 3 项）；
- `zhu_ye_ime::tsf`：删除 `en_profile_guid`、`active_language_profile`
  （`CLSID_TF_InputProcessorProfiles` + `GetActiveLanguageProfile`）、
  `start_mode_for_profile` 及装配测试；`create_text_service` 回退为
  直接 `configured_default_mode()`（新会话从配置默认模式开始，Shift
  会话内切换照旧）；
- `scripts/ime-identity.ps1`：`TsfIdentity` 移除 ProfileGuidEn/
  DisplayNameEn；`New-TsfRegistration` 不再注册英文档案；
  `Test-TsfRegistration` 不再校验；
- `scripts/verify-tsf-identity.ps1`：交叉比对回 7 项；
- `crates/zhu-ye-ime/assets/ying.ico`：从仓库删除（git rm）；
- `elevated-worker.ps1`：`copy-icon` 白名单回单对（只留 zhu.ico）；
  新增 `registry-delete-key` 操作（TIP 树 `LanguageProfile\0x…\{guid}`
  形态白名单，递归删整键），用于清掉线上英文档案键。

### 回退执行（部署）

- worker（新 op 版，20,021B，BOM UTF-8）重新部署后手动触发计划任务；
- `registry-delete-key` 删除 HKLM 英文档案键：`dump-ref` 前后快照
  核对，**两个视图均只剩中文档案** `{6315FE74-…}`（op[2] 对
  `WOW6432Node` 物理前缀报"键不存在"——CTF\TIP 树是**共享键**，
  32 位 dump 读到的就是 64 位物理副本，英文档案已无任何残留）；
- 新 DLL `zhu_ye_ime_9C951EF5.dll`（2,191,360B）经 `copy-dll-ver`
  部署并切换 CLSID InprocServer32 + 中文档案 IconFile 指针（新 DLL
  与回退代码对应，SHA8 9C951EF5）；ctfmon 重启；
- `verify-tsf-identity.ps1` 7 项 PASS、`verify-tsf-dll.ps1` 导出校验
  通过（DllGetClassObject/DllCanUnloadNow/dll_probe）；
- 门禁：fmt/clippy `-D warnings`/workspace 全测全绿（ime 209 项——
  装配测试随回退删除）。

## 备选方案

**保留双档案但改名"竹叶输入法（中）/（英）"。** 否决：用户明确
不接受"两个输入法"，改名不解决条目数问题。

**把英文档案做成不可见（不注册在 Win+Space 列表），仅保留
`ying.ico` 与激活查询装配。** 否决：Win+Space 列表 = 系统枚举已启用
的 LanguageProfile；不可见档案无法被用户选中，激活查询永远返回中文
档案，"英"装配成了死代码——不如干净回退。

**单档案 + 引擎内部维持"英态"起始模式开关。** 否决：用户此前对
"Shift 会话内切换无任务栏反馈"已知情接受；单档案下新会话从配置默认
模式起是唯一无歧义语义，维持原 FR-041 装配即可。

## 后果

- 系统键盘布局恢复**单一条目**"竹叶输入法"（图标"竹"），与用户
  心智模型一致；Win+Space 不再出现"竹叶英文"；
- 任务栏指示器恒显"竹"：中/英是会话内状态，与所有单档案第三方
  IME 行为一致（微软拼音"英"图标之所以能随 Shift 变，是微软私有的
  系统级 IME 模式跟踪，第三方 TSF 无对应 API）；斜杠圆圈"禁用"图标
  依旧只在系统级无可用输入法时出现；
- "中"/"英"的会话态反馈路径=设置窗口默认模式配置（FR-041）+ Shift
  切换的会话行为；这是第三方 TSF 能力边界内的全部（如实记录，
  避免再次收到同款"任务栏状态图标"请求时重新论证）；
- `registry-delete-key` 成为 worker 白名单常规 op（回退/清理复用）；
- `ying.ico` 不再是资产；英文档案 GUID `{EF42481A-…}` 已从代码与
  注册表移除（不再作为契约常量维护）。

## 关联

- 被回退的批四（拼音声调与横截断修复**仍生效**，本回退只覆盖其
  #2 双档案部分）：
  [2026-10-07-acceptance-fix-batch-4.zh.md](2026-10-07-acceptance-fix-batch-4.zh.md)
- 图标/名称与 IconFile 机制（中文档案 DLL 内嵌，未受影响）：
  [2026-10-06-taskbar-icon-zhu.zh.md](2026-10-06-taskbar-icon-zhu.zh.md)
- 语言栏结论（Win11 不渲染第三方语言栏项，任务栏图标能力边界）：
  [2026-09-28-language-bar-mode-icon.zh.md](2026-09-28-language-bar-mode-icon.zh.md)
