# Agent Note: 仓库本地 AI 开发工作流适配

Status: implemented

[English](2026-09-18-project-ai-workflow-adaptation.md) | 中文

## 问题

仓库采用了来自 TypeScript/pnpm 风格仓库的 `.agents/notes` 与 `.agents/skills` 快照。这些约定假定 agent 文件被忽略、工具链基于 pnpm、提交使用 Conventional Commits，并以 Python 项目作为 git-publish 默认配置。本项目是使用 git 发布的 Rust/Cargo workspace，直接照搬这些假设会因为命令缺失和规则冲突而阻塞开发、提交与推送流程。工作流规则需要一份项目级统一适配。

## 决策

`.agents/` 随仓库纳入版本管理，Agent Notes 与技能随代码演进；`.gitignore` 不再忽略该目录。`AGENTS.md` 是具备约束力的适配层：它规定了 Agent Notes 与 todos 的分工、提交格式 `<type>(<scope>): <中文描述> (T-xxx)`、基于最小 diff 的推送前验证，以及以根 `Cargo.toml` 的 workspace 版本和中文 `CHANGELOG.md` 为锚点的发布规则。

dsh 技能中的每一项 pnpm、Vitest、`./invariant`、ACP/Loader、HMR 或 doc-sync 步骤都已改写为 Rust/Cargo/PowerShell/git 等价操作；本仓库没有对应概念的场景（文档站、Web 工具链）明确标注不再适用，改由本仓库自己的门禁脚本校验。`git-publish` 使用根 `Cargo.toml`、`CHANGELOG.md` 与已确定的分支模型：`develop` 集成、`main` 生产、`release/vX.Y.Z` 发布分支、`vX.Y.Z` 生产 tag（T-019）。

本记录以 `implemented/process/` 形式固化这次适配，todos 以 T-019 记录分支模型拍板结论，关联 [docs/todos-list.md](../../../../docs/todos-list.md)。

## 曾考虑的替代方案

**原样照搬技能默认配置。** 落选：本项目不存在 pnpm 命令与 Python 版本文件，且受保护分支默认值可能与此仓库实际分支冲突。

**继续忽略 `.agents/`。** 落选：Agent Notes 必须与其记录的决策一起纳入版本管理；忽略会隐藏工作流规则并阻碍评审。

**完全重写每个技能。** 落选：dsh 技能与 git-publish 仍提供评审、推送前证据与发布顺序的有用模式；适配比另起炉灶成本更低。

## 后果

开发、提交与推送环节现在共享 `AGENTS.md` 这一份中文、面向 Rust 的唯一权威规则，消除了互相矛盾的指令。Agent Notes 可以随代码一起评审，并按标准生命周期归档。

新增的 `scripts/verify-agent-notes.ps1` 与 `scripts/verify-translation-pairs.ps1` 已自动化笔记结构、归档封存与双语配对门禁；supersession 等语义判断仍需人工评审。git-publish 分支模型已固定为 `develop`（集成）→ `release/vX.Y.Z` → `main`（生产）→ tag `vX.Y.Z`，并写入 `AGENTS.md` 与技能配置（T-019）；发布按该模型执行。日常改动只能经工作分支 PR 进入 `develop`（T-047，见 [工作分支 PR 合入 develop 流程](2026-09-29-work-branch-pr-flow.zh.md)）。
