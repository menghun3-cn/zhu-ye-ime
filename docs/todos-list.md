# 竹叶输入法 任务清单（todos-list）

## 维护规则

- 状态三态：`待办` / `进行中` / `已完成`
- 编号规则：`T-xxx` 全局唯一，增量递增；当前已用至 `T-026`
- FR 关联：任务必须关联需求规格说明书中的需求编号
- 周期任务：到期后登记观察结论并追加到 todos-done，不迁移、不删除任务本体
- 提交前校验：任务状态与实现进度一致，未完成不得标记已完成

## 任务表格

| ID | 状态 | 任务 | FR 关联 | 里程碑 | 备注 |
| --- | --- | --- | --- | --- | --- |
| T-001 | 已完成 | 产出需求规格说明书.md | FR-001 至 FR-014 | M0-文档 | 经 grill-me 访谈确认 |
| T-002 | 已完成 | 产出方案设计.md | FR-001 至 FR-014 | M0-文档 | 含 UI/UX 设计 |
| T-003 | 已完成 | 产出验收标准.md | FR-001 至 FR-014 | M0-文档 | 功能与性能验收 |
| T-004 | 已完成 | 产出架构文档 architecture.md | FR-011、FR-014 | M0-文档 | 模块与数据架构 |
| T-005 | 已完成 | 搭建 Rust workspace 与 crate 骨架 | FR-011、FR-014 | M0-脚手架 | build/test/clippy 全部通过 |
| T-006 | 已完成 | 词典数据管线与二进制格式 | FR-002、FR-007 | M2 | v2 二进制格式（ZYDT/128 字节头部/拼音索引/bigram/文本池/翻译与反查索引/SHA-256）、自建演示种子、mmap 加载器与 build/inspect/verify/import CLI 已完成并验证；CC-CEDICT + FrequencyWords + OPUS GlobalVoices 已接入，真实导入 120,028 词条、23.44% 词频命中、820,368 个真实 bigram；真实词典候选生成已下沉 zhu-ye-core，`rank de 我们` 验证真实 bigram 生效（的 3,979,557 居首、得提升至 158,049）；见 [Agent Note](../.agents/notes/implemented/architecture/2026-09-19-dictionary-binary-format-v1.md)、[v2 翻译索引](../.agents/notes/implemented/architecture/2026-09-21-dictionary-v2-translation-index.md) 与 [真实词库导入](../.agents/notes/implemented/architecture/2026-09-21-real-dictionary-import.md) |
| T-007 | 已完成 | 全拼音节切分核心 | FR-001 | M2 | 标准全拼表/动态规划切分/双拼接口已实现；T-006 v2 词典已能按拼音前缀加载查询；音节表已用 CC-CEDICT 真实拼音校验（318,015 音节，894 个非标准音节被清洗丢弃）；真实词典复验 `cha`/`duo`/`guan` 可切分、`xian` 保留 `xian` 与 `xi-an` 两种歧义；M2 验收通过；见 [Agent Note](../.agents/notes/implemented/feature/2026-09-19-full-pinyin-segmentation-core.md) |
| T-008 | 已完成 | 候选排序静态模型（unigram+bigram） | FR-002 | M2 | RankingModel/StaticRankingModel/内存 bigram 与排序接入已完成；T-006 mmap bigram 模型已可用，真实统计语料已接入（820,368 词对）；候选生成迁入 zhu-ye-core，CLI 新增 `rank` 子命令；真实字典验证：`rank de` 的 3,957,141 居首，`rank de 我们` 的 3,979,557 仍居首、得 158,049 明显提升，两次运行结果一致；见 [Agent Note](../.agents/notes/implemented/feature/2026-09-19-candidate-ranking-static-model.md) |
| T-009 | 已完成 | 用户词学习与持久化 | FR-003 | M3 | UserDictStore 版本化 JSON/原子写/损坏备份恢复、选择即记忆、删除与重置、TSF 接入已完成；VM 端到端复核通过（选择记忆与频率提升逐键上屏验证：nihao/zhuye/wo/yi 均落盘且频次递增）；见 [Agent Note](../.agents/notes/implemented/feature/2026-09-19-user-dict-persistence.md) |
| T-010 | 已完成 | TSF 服务注册与卸载闭环 | FR-010、FR-013 | M1 | DLL 导出/COM 生命周期/安装卸载脚本/便携测试包已完成；VM 安装验收通过（v4→v5 版本化切换 + explorer 重启加载验证）；发布前卸载回滚演练已通过（VM 演练 v3 场景 4：注册清空 + DLL/词典全量清理；场景 5：Program Files 残留目录清理）；见 [Agent Note](../.agents/notes/implemented/architecture/2026-09-18-tsf-registration-and-lifetime.md)、[版本化部署](../.agents/notes/implemented/architecture/2026-09-23-versioned-dll-deployment-and-dictionary-location.md) |
| T-011 | 已完成 | TSF 上屏闭环 | FR-001、FR-004 | M1 | 按键/组合/上屏已实现并 VM 记事本验收：组合/候选/空格提交/Enter/Esc/Backspace 全过，零崩溃；见 [Agent Note](../.agents/notes/implemented/feature/2026-09-18-tsf-composition-and-key-events.md) 与 [写入路径崩溃修复](../.agents/notes/implemented/bug-fix/2026-09-23-tsf-insert-at-selection-write-path-crash.md) |
| T-012 | 已完成 | 候选窗渲染（主题/DPI/高对比度） | FR-009、FR-011 | M3 | 独立 GDI 双缓冲自绘、主题跟随系统深浅色/高对比度、DPI 适配与演示截图已验证；VM 上候选窗连通、定位（GetTextExt 锚定）与跨页可见性验证通过；深浅色/高对比度截图待人工审查（shots4/shots5）；见 [Agent Note](../.agents/notes/implemented/feature/2026-09-19-candidate-window-gdi-rendering.md) |
| T-013 | 已完成 | 键位交互（Shift/Tab/翻页/选择） | FR-004、FR-006、FR-013 | M3 | 引擎分页/译文层、Shift 中英切换、Tab/逗号/句号与数字键接入已完成并 VM 记事本验收：数字选词/Enter/Esc/翻页/Backspace/Shift 切换/Tab 译文层/译文上屏全部通过；验收中发现翻页后候选窗隐藏缺陷并修复（view.items 双重分页切片），见 [Agent Note](../.agents/notes/implemented/feature/2026-09-20-candidate-window-tsf-integration.md) 与 [修复笔记](../.agents/notes/implemented/bug-fix/2026-09-23-candidate-window-page-slice-hidden.md) |
| T-014 | 已完成 | 双语翻译层与译文切换 | FR-005、FR-006、FR-007 | M4 | 词典格式升级 v2：中文词→译文索引、归一化英文→中文反查、DictionaryFile 实现 Translator、输入引擎译文层驱动、zhu-ye-cli dict -r 反查；见 [Agent Note](../.agents/notes/implemented/architecture/2026-09-21-dictionary-v2-translation-index.md) |
| T-015 | 已完成 | 性能基准与验收脚本 | FR-011、FR-014 | M4 | zhu-ye-cli bench 覆盖切分/候选查找/bigram/正反翻译并输出指标行；self-check 输出快速基线；scripts/bench.ps1 按阈值验收；见 [Agent Note](../.agents/notes/implemented/testing/2026-09-21-cli-benchmark-metrics-and-acceptance.md) |
| T-016 | 已完成 | README、安装文档、许可证清单 | FR-010、FR-012 | M5 | 新增 docs/安装与使用.md、CHANGELOG.md；README 更新安装/使用入口与许可证；许可证清单补充现状说明 |
| T-017 | 进行中 | 周期任务：使用体验与稳定性观察 | FR-008、FR-011、FR-012 | 观察期 | 每两周一轮 |
| T-018 | 已完成 | 适配 Agent Notes 与开发/提交/推送流程 | FR-014 | M0-流程 | 规则写入 AGENTS.md；见 [Agent Note](../../.agents/notes/implemented/process/2026-09-18-project-ai-workflow-adaptation.md) |
| T-019 | 已完成 | 确定 git-publish 分支模型并回写配置 | FR-014 | M0-流程 | develop 集成 / main 生产 / release/vX.Y.Z / tag vX.Y.Z；已回写 AGENTS.md 与 git-publish 配置 |
| T-020 | 已完成 | 技能与脚本适配 Rust/Cargo/PowerShell 工作流 | FR-014 | M0-流程 | 移除 pnpm/Vitest/Node 依赖；新增两个 PowerShell 门禁脚本，见 [Agent Note](../../.agents/notes/implemented/process/2026-09-18-project-ai-workflow-adaptation.md) |
| T-021 | 已完成 | 清洗多音节切分噪声候选 | FR-001、FR-002 | M2 | 整词拼音存在直接词典条目时不再追加音节切分组合，无整词时保留回退；真实词典复验：`rank jiao` 叫 92,723 居首、`rank xian` 洗按消失、`rank fazhan` 发展 3,680 居首且无发站、`rank geio` 给哦 保留；新增 2 项 core 单测；见 [Agent Note](../.agents/notes/implemented/feature/2026-09-19-candidate-ranking-static-model.md) |
| T-022 | 已完成 | 安装与便携包随带词典数据 | FR-007、FR-010 | M5 | install/package 把 v2 词典复制为 DLL 同目录 `dictionary.zyct`；TSF 运行时支持 `ZHU_YE_DICT_PATH`、DLL 同目录与用户目录解析顺序；门禁与便携包产物已验证；见 [Agent Note](../.agents/notes/implemented/architecture/2026-09-21-installed-dictionary-deployment.md) |
| T-023 | 已完成 | AI 服务接口对齐方案设计 | FR-008 | M5 | `AiService.translate` 增加 `TranslationDirection` 参数；`OfflineAiService` 显式实现空建议/双向空翻译/空润色；`TranslationDirection` 从 core 公共导出；fmt/clippy/workspace 测试与 Agent Notes 双语文档门禁全部通过；见 [Agent Note](../.agents/notes/implemented/architecture/2026-09-21-ai-service-contract-and-offline-default.md) |
| T-024 | 已完成 | 主机侧端到端回归（核心输入闭环） | FR-001 至 FR-007、FR-014 | M5 | 新增 `zhu-ye-ime` 的 `host-e2e` 检查器与 `scripts/e2e.ps1` 验收入口；种子词典 17 项检查与真实词典 smoke 4 项已通过，fmt/clippy/workspace tests 与脚本门禁全部通过；覆盖切分/排序/数字选择/翻页交互/模式切换/用户词/译文层/正反查；见 [Agent Note](../.agents/notes/implemented/testing/2026-09-21-host-e2e-regression.md) |
| T-025 | 已完成 | 便携包脚本适配 Windows PowerShell 5.1 UTF-8 编码 | FR-010、FR-013、FR-014 | M1 | 全部 PowerShell 脚本加 UTF-8 BOM，打包脚本复写 BOM，`README-测试.txt` 同步；Windows PowerShell 5.1 下已验证解析与中文错误消息；见 [Agent Note](../.agents/notes/implemented/bug-fix/2026-09-22-portable-scripts-windows-powershell-5-1-encoding.md) |
| T-026 | 已完成 | 版本化 DLL 无锁升级机制固化 | FR-010、FR-013 | M5 | install/uninstall/ime-identity/verify-tsf-dll 已固化：版本化命名（`-Version` 显式或 SHA-256 前 8 位内容指纹）、复制→PE 头静态校验（MZ+PE 签名，实测 LoadLibraryEx 对文本文件不报错）+导出校验→词典复制（源==目标跳过）→注册表切换（切换前快照上一版）→失败回滚（Set-TsfRegistrationRollback 还原 InProcServer32/IconFile，无旧版则清空注册）→旧版延迟清理（Add-TsfDelayedCleanup：无占用立即删，占用则 DELAY rename 为 `*.zy-del`，因平台 MoveFileEx 删除操作返回 ERROR_PATH_NOT_FOUND）；VM 演练 v3 全场景通过（坏 DLL PE 拒绝且注册不动/坏副本删除/v6→v7 无锁切换/卸载全清/备份词典重装 v8）；见 [Agent Note](../.agents/notes/implemented/architecture/2026-09-23-versioned-dll-deployment-and-dictionary-location.md) |
| T-027 | 待办 | 恢复 VM 交互会话后补做 T-017 首轮 UI 级日常输入观察 | FR-008、FR-013 | 观察期 | T-017 首轮观察（2026-09-24）发现验收 VM 会话 1 处于 Disc 状态（输入桌面不可用，SendKeys/截屏被拒，explorer 无法重启），UI 级日常输入 smoke 无法执行；先以引擎级输入环代替；待会话恢复（RDP/控制台重启 Shell）后补做 vm_ui_test5 全场景 |

## 周期任务登记

| ID | 周期 | 首轮周期 | 处理规则 | 最近结论 |
| --- | --- | --- | --- | --- |
| T-017 | 每 14 天 | 2026-09-18 至 2026-10-02 | 执行日常输入、性能、稳定性检查；结论追加 todos-done；发现问题创建新任务 | 首轮观察执行中（2026-09-24 日常输入 smoke + 性能采样 + 2h 稳定性会话）；轮末 2026-10-02 到期后正式登记结论 |
