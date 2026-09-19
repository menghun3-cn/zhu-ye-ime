# 竹叶输入法 任务清单（todos-list）

## 维护规则

- 状态三态：`待办` / `进行中` / `已完成`
- 编号规则：`T-xxx` 全局唯一，增量递增；当前已用至 `T-020`
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
| T-006 | 待办 | 词典数据管线与二进制格式 | FR-002、FR-007 | M2 | 数据来源与许可证记录 |
| T-007 | 进行中 | 全拼音节切分核心 | FR-001 | M2 | 标准全拼表/动态规划切分/双拼接口已实现；待 T-006 数据管线校验音节表完整性；见 [Agent Note](../.agents/notes/implemented/feature/2026-09-19-full-pinyin-segmentation-core.md) |
| T-008 | 进行中 | 候选排序静态模型（unigram+bigram） | FR-002 | M2 | RankingModel/StaticRankingModel/内存 bigram 与排序接入已完成；待 T-006 真实 bigram 数据；见 [Agent Note](../.agents/notes/implemented/feature/2026-09-19-candidate-ranking-static-model.md) |
| T-009 | 进行中 | 用户词学习与持久化 | FR-003 | M3 | UserDictStore 版本化 JSON/原子写/损坏备份恢复、选择即记忆、删除与重置、TSF 接入真实路径已完成；待 VM 端到端复核；见 [Agent Note](../.agents/notes/implemented/feature/2026-09-19-user-dict-persistence.md) |
| T-010 | 进行中 | TSF 服务注册与卸载闭环 | FR-010、FR-013 | M1 | DLL 导出/COM 生命周期/安装卸载脚本/便携测试包已完成；见 [Agent Note](../.agents/notes/implemented/architecture/2026-09-18-tsf-registration-and-lifetime.md)；待其他机器安装验收 |
| T-011 | 进行中 | TSF 上屏闭环 | FR-001、FR-004 | M1 | 按键/组合/上屏已实现（20 项单测、clippy、DLL 导出校验通过）；待 VM 安装后记事本验收；见 [Agent Note](../.agents/notes/implemented/feature/2026-09-18-tsf-composition-and-key-events.md) |
| T-012 | 待办 | 候选窗渲染（主题/DPI/高对比度） | FR-009、FR-011 | M3 | Win32 自绘 |
| T-013 | 待办 | 键位交互（Shift/Tab/翻页/选择） | FR-004、FR-006、FR-013 | M3 | 与 T-011 联动 |
| T-014 | 待办 | 双语翻译层与译文切换 | FR-005、FR-006、FR-007 | M4 | 本地词典译文物化 |
| T-015 | 待办 | 性能基准与验收脚本 | FR-011、FR-014 | M4 | bench/self-check |
| T-016 | 待办 | README、安装文档、许可证清单 | FR-010、FR-012 | M5 | 发布就绪 |
| T-017 | 进行中 | 周期任务：使用体验与稳定性观察 | FR-008、FR-011、FR-012 | 观察期 | 每两周一轮 |
| T-018 | 已完成 | 适配 Agent Notes 与开发/提交/推送流程 | FR-014 | M0-流程 | 规则写入 AGENTS.md；见 [Agent Note](../../.agents/notes/implemented/process/2026-09-18-project-ai-workflow-adaptation.md) |
| T-019 | 已完成 | 确定 git-publish 分支模型并回写配置 | FR-014 | M0-流程 | develop 集成 / main 生产 / release/vX.Y.Z / tag vX.Y.Z；已回写 AGENTS.md 与 git-publish 配置 |
| T-020 | 已完成 | 技能与脚本适配 Rust/Cargo/PowerShell 工作流 | FR-014 | M0-流程 | 移除 pnpm/Vitest/Node 依赖；新增两个 PowerShell 门禁脚本，见 [Agent Note](../../.agents/notes/implemented/process/2026-09-18-project-ai-workflow-adaptation.md) |

## 周期任务登记

| ID | 周期 | 首轮周期 | 处理规则 | 最近结论 |
| --- | --- | --- | --- | --- |
| T-017 | 每 14 天 | 2026-09-18 至 2026-10-02 | 执行日常输入、性能、稳定性检查；结论追加 todos-done；发现问题创建新任务 | 等待首轮到期 |
