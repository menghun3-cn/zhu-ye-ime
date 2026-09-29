# 项目级指令（AGENTS.md）

本文件对在本仓库工作的 AI 与开发者同样有效，优先于通用模板中的所有冲突条款。

## 1. 项目原则

- 遵守第一性原理：每个决策先回到"该功能本质要解决什么"，再选择实现方式
- 遵守对抗性审查：实现前质疑假设，代码审查优先找缺陷与风险，不流于形式
- 所有交流、注释、文档默认使用中文
- 代码与数据尽量自研可控；借鉴成熟模式时必须在文档记录
- 不引入任何运行时广告、统计、捆绑行为

## 2. 任务执行顺序

1. 需求不明确时，先调用 grill-me 技能澄清，再写需求规格说明书
2. 按顺序产出：需求规格说明书 -> 方案设计 -> 验收标准 -> todos-list -> 代码实现与测试 -> 部署/维护/使用文档
3. 每完成一个里程碑，归档 todos-done

## 3. todos 自动维护规则

### 3.1 关键节点必须更新 docs/todos-list.md

以下节点自动同步任务状态：

- 文档或需求变更完成
- 任务从待办进入开发
- 任务通过测试验收
- 每个里程碑结束
- git 提交前
- 周期任务到期时

### 3.2 编号与状态规则

- 编号：`T-xxx`，全局递增；新任务编号紧跟最新已用编号（当前已用至 `T-047`），不得重复或复用
- 状态：仅允许 `待办` / `进行中` / `已完成`，不得自定义第四态
- 每项任务必须关联需求编号（FR 关联）
- 未通过验收的任务不得标记为已完成

### 3.3 周期任务处理

- T-017 为每两周一次的观察任务，登记在"周期任务登记"区
- 首轮周期：2026-09-18 至 2026-10-02
- 到期后必须执行验收标准第 6 节规定的检查
- 结论追加到 docs/todos-done.md；任务本体保留在 todos-list，状态回到"进行中"开始下一轮
- 禁止删除周期任务或将其标记为一次性完成

### 3.4 Agent Notes 与 todos 分工

- todos 只负责任务状态、周期与验收进度，不承担决策记录职责
- Agent Notes 存放于 `.agents/notes/`，负责记录决策、备选方案、后果与放弃项
- 以下变更必须新增或更新 Agent Note：行为、架构、跨模块约定、流程、测试策略、磁盘/协议格式、配置格式以及其他值得事后回顾的决策
- 新的 Agent Note 必须按 `.agents/notes/README.zh.md` 的格式与生命周期规则创建，并执行 supersession 检查
- 新增流程或机制时，todos 与 Agent Note 都要更新，两者使用相对链接互相引用

## 4. 编码规范

- Rust 代码必须通过 `cargo fmt` 与 `cargo clippy --all-targets -- -D warnings`
- 新增代码必须携带必要测试；算法变更必须补充单元测试
- `zhu-ye-core` 禁止依赖 Windows 专有 API
- 错误处理必须显式，禁止静默丢弃关键错误
- 热路径禁止网络、锁竞争、全局日志等不可控行为
- 大文件与构建产物不得进入 git，数据来源必须记录

## 5. 提交代码规范

- 提交信息格式：`<type>(<scope>): <中文描述> (T-xxx)`
  - type 使用 `feat` / `fix` / `refactor` / `perf` / `docs` / `chore` / `test` / `ci`
  - 描述简洁、说明改动目的，不使用无意义描述
- 提交前运行：`cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test --workspace`、`git diff --check`
- 检查 `git status`，确认无意外文件、无大文件、无用户本地数据
- 同步本次改动对应的 todos-list 状态与 Agent Note
- 不要提交 `target/`、词典原始数据、注册表备份、日志等产物

### 5.1 推送规范

- 推送前先确认当前分支、`git status --short` 与本次改动范围（`git diff --stat`）
- 按改动范围选择最小且相关的验证，不反复重复整套检查；跨模块或跨 crate 改动时保留整套回归
- 禁止使用裸 `--force` 覆盖远程分支
- 重写历史使用 `--force-with-lease=<branch>:<observed-oid>`，推送后核对远端 HEAD
- 推送后 CI 为 pending 时如实报告 pending，不得声称通过
- 受保护分支（main、develop）不允许直推，发布走 release 分支 + PR（见 git-publish 技能）

### 5.2 分支与合并流程（T-047）

- 所有新功能、bug 修复、优化、重构、文档与流程变更，一律先从最新 `develop` 切工作分支开发，禁止直接在 `develop` / `main` 上提交
- 工作分支命名：`<type>/T-xxx-<英文短描述>`，type 与提交信息一致（如 `feat/T-045-slang-pack`、`fix/T-043-candidate-border`）
- 开发完成后：通过第 5 节提交前门禁 → 推送工作分支 → 创建目标为 `develop` 的 PR（标题沿用提交格式，正文列出改动、验证证据与关联 T 编号）→ 合并入 `develop`
- 合入 `develop` 后才进入发布流程：`develop` → `release/vX.Y.Z` → PR 合入 `main` → 打 tag → 同步回 `develop`（见第 6 节与 git-publish 技能）；工作分支不得直接向 `main` 提 PR
- PR 合并后删除远程与本地工作分支，本地切回并更新 `develop`
- 一个工作分支对应一个任务（或一个里程碑批次）；分支内可多次提交，保持每个提交可独立通过门禁

## 6. 发布代码规范

- 发布前执行验收标准"发布检查清单"
- 核对 docs/licenses.md：新增数据源必须带来源与许可证
- 确认在线功能默认关闭时零网络请求
- 确认安装/卸载脚本可完整回滚
- 版本号以根 `Cargo.toml` 的 `workspace.package.version` 为唯一主版本源，发布时所有 crate 版本随 workspace 同步
- CHANGELOG 使用 Keep a Changelog 中文格式维护在 `CHANGELOG.md`；发布流程执行 `.agents/skills/git-publish/SKILL.md`
- 发布前在 Windows 上运行 `.agents/skills/git-publish/scripts/pre-publish-check.ps1` 校验版本与 CHANGELOG
- 分支模型：工作分支 → PR 合入 develop（集成分支，见 5.2）→ release/vX.Y.Z → PR 合入 main（生产分支）后打 vX.Y.Z tag，再同步回 develop（T-019、T-047）

## 7. 文档规范

- docs 按分类维护：需求、设计、验收、架构、任务清单、任务归档、许可证
- 文档变更后同步更新关联文档与 todos
- 用户指南、API 文档、技术文档均使用中文
- Agent Notes 使用 `README.zh.md` 规定的中英双语与伴随记录格式

## 8. 技能适配规则

本仓库内的 `.agents/skills/` 已按 Rust/Cargo/PowerShell/git 工作流适配，不再依赖 pnpm、Vitest、Node 或 TypeScript。使用技能时若再遇到此类命令，一律按等价规则换算，不运行不存在的 pnpm 脚本。

- Agent Notes 与双语配对门禁由 `.\scripts\verify-agent-notes.ps1`（含 `-ArchiveWrite` 追加封存）与 `.\scripts\verify-translation-pairs.ps1`（含 `-Write` 重写一致性记录）提供，替代 `pnpm run seal-archived`、`verify-archived-agent-notes`、`doc-sync`
- `dsh-pre-push-checks` 的核心理念适用：推送前按 diff 选择最小证据；具体命令以本文件第 5/5.1 节为准
- `git-publish` 以本仓库根 `Cargo.toml`、`CHANGELOG.md` 与已确定分支模型为准：集成分支 `develop`、生产分支 `main`、release 分支 `release/vX.Y.Z`、tag `vX.Y.Z`
- 新添加技能若含 Node/web 工具命令，必须同步适配为本仓库命令，或明确标注“仅在其他适用仓库使用”
