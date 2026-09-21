# 竹叶输入法 任务清单（todos-list）

## 维护规则

- 状态三态：`待办` / `进行中` / `已完成`
- 编号规则：`T-xxx` 全局唯一，增量递增；当前已用至 `T-021`
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
| T-007 | 进行中 | 全拼音节切分核心 | FR-001 | M2 | 标准全拼表/动态规划切分/双拼接口已实现；T-006 v2 词典已能按拼音前缀加载查询；音节表已用 CC-CEDICT 真实拼音校验（318,015 音节，894 个非标准音节被清洗丢弃）；见 [Agent Note](../.agents/notes/implemented/feature/2026-09-19-full-pinyin-segmentation-core.md) |
| T-008 | 已完成 | 候选排序静态模型（unigram+bigram） | FR-002 | M2 | RankingModel/StaticRankingModel/内存 bigram 与排序接入已完成；T-006 mmap bigram 模型已可用，真实统计语料已接入（820,368 词对）；候选生成迁入 zhu-ye-core，CLI 新增 `rank` 子命令；真实字典验证：`rank de` 的 3,957,141 居首，`rank de 我们` 的 3,979,557 仍居首、得 158,049 明显提升，两次运行结果一致；见 [Agent Note](../.agents/notes/implemented/feature/2026-09-19-candidate-ranking-static-model.md) |
| T-009 | 进行中 | 用户词学习与持久化 | FR-003 | M3 | UserDictStore 版本化 JSON/原子写/损坏备份恢复、选择即记忆、删除与重置、TSF 接入真实路径已完成；待 VM 端到端复核；见 [Agent Note](../.agents/notes/implemented/feature/2026-09-19-user-dict-persistence.md) |
| T-010 | 进行中 | TSF 服务注册与卸载闭环 | FR-010、FR-013 | M1 | DLL 导出/COM 生命周期/安装卸载脚本/便携测试包已完成；见 [Agent Note](../.agents/notes/implemented/architecture/2026-09-18-tsf-registration-and-lifetime.md)；待其他机器安装验收 |
| T-011 | 进行中 | TSF 上屏闭环 | FR-001、FR-004 | M1 | 按键/组合/上屏已实现（20 项单测、clippy、DLL 导出校验通过）；待 VM 安装后记事本验收；见 [Agent Note](../.agents/notes/implemented/feature/2026-09-18-tsf-composition-and-key-events.md) |
| T-012 | 进行中 | 候选窗渲染（主题/DPI/高对比度） | FR-009、FR-011 | M3 | 独立 GDI 双缓冲自绘、主题跟随系统深浅色/高对比度、DPI 适配与演示截图已验证；TSF 联动与 VM 验收待 T-013；见 [Agent Note](../.agents/notes/implemented/feature/2026-09-19-candidate-window-gdi-rendering.md) |
| T-013 | 进行中 | 键位交互（Shift/Tab/翻页/选择） | FR-004、FR-006、FR-013 | M3 | 引擎分页/译文层、Shift 中英切换、Tab/逗号/句号与数字键接入、TSF 候选窗生命周期与定位已完成（49 项 lib 单测、clippy 通过）；待 VM 记事本验收；见 [Agent Note](../.agents/notes/implemented/feature/2026-09-20-candidate-window-tsf-integration.md) |
| T-014 | 已完成 | 双语翻译层与译文切换 | FR-005、FR-006、FR-007 | M4 | 词典格式升级 v2：中文词→译文索引、归一化英文→中文反查、DictionaryFile 实现 Translator、输入引擎译文层驱动、zhu-ye-cli dict -r 反查；见 [Agent Note](../.agents/notes/implemented/architecture/2026-09-21-dictionary-v2-translation-index.md) |
| T-015 | 已完成 | 性能基准与验收脚本 | FR-011、FR-014 | M4 | zhu-ye-cli bench 覆盖切分/候选查找/bigram/正反翻译并输出指标行；self-check 输出快速基线；scripts/bench.ps1 按阈值验收；见 [Agent Note](../.agents/notes/implemented/testing/2026-09-21-cli-benchmark-metrics-and-acceptance.md) |
| T-016 | 已完成 | README、安装文档、许可证清单 | FR-010、FR-012 | M5 | 新增 docs/安装与使用.md、CHANGELOG.md；README 更新安装/使用入口与许可证；许可证清单补充现状说明 |
| T-017 | 进行中 | 周期任务：使用体验与稳定性观察 | FR-008、FR-011、FR-012 | 观察期 | 每两周一轮 |
| T-018 | 已完成 | 适配 Agent Notes 与开发/提交/推送流程 | FR-014 | M0-流程 | 规则写入 AGENTS.md；见 [Agent Note](../../.agents/notes/implemented/process/2026-09-18-project-ai-workflow-adaptation.md) |
| T-019 | 已完成 | 确定 git-publish 分支模型并回写配置 | FR-014 | M0-流程 | develop 集成 / main 生产 / release/vX.Y.Z / tag vX.Y.Z；已回写 AGENTS.md 与 git-publish 配置 |
| T-020 | 已完成 | 技能与脚本适配 Rust/Cargo/PowerShell 工作流 | FR-014 | M0-流程 | 移除 pnpm/Vitest/Node 依赖；新增两个 PowerShell 门禁脚本，见 [Agent Note](../../.agents/notes/implemented/process/2026-09-18-project-ai-workflow-adaptation.md) |
| T-021 | 待办 | 清洗多音节切分噪声候选 | FR-001、FR-002 | M2 | 真实字典排序验证时发现噪声候选（如 给哦/夹哦/生和哦/发站）会挤占真实候选；需在切分或候选生成阶段收紧多音节词边界后复验；关联 [Agent Note](../.agents/notes/implemented/feature/2026-09-19-candidate-ranking-static-model.md) |

## 周期任务登记

| ID | 周期 | 首轮周期 | 处理规则 | 最近结论 |
| --- | --- | --- | --- | --- |
| T-017 | 每 14 天 | 2026-09-18 至 2026-10-02 | 执行日常输入、性能、稳定性检查；结论追加 todos-done；发现问题创建新任务 | 等待首轮到期 |
