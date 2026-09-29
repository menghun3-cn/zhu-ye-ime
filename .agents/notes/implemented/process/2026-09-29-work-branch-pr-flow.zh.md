# Agent Note: 工作分支 PR 合入 develop 流程

Status: implemented

[English](2026-09-29-work-branch-pr-flow.md) | 中文

## Problem

分支模型（T-019）只规定了发布段：`develop` → `release/vX.Y.Z` → `main` → tag，日常开发怎样进入 `develop` 没有规定。实际做法是功能、修复、文档直接提交在本地 `develop` 上，未经评审的提交越积越多（截至 2026-09-29 本地已有 9 个未推送提交）。这样集成前没有任何评审环节，而这些提交若要推送，又会违反"受保护分支禁止直推"。

## Decision

所有变更（新功能、bug 修复、优化、重构、文档、流程）一律从最新 `develop` 切工作分支开发，只能经 PR 进入 `develop`（`AGENTS.md` 5.2）：

- 分支命名：`<type>/T-xxx-<英文短描述>`，type 与提交类型一致（`feat/T-045-slang-pack`、`fix/T-043-candidate-border`）。
- 一个分支对应一个任务或一个里程碑批次。分支内可以有多个提交，每个提交都要能单独通过第 5 节门禁。
- 开发完成后：通过门禁 → 推送分支 → 创建目标为 `develop` 的 PR（标题沿用提交格式，正文列出改动、验证证据与关联 T 编号）→ 合入 `develop`。
- 合入之后才走发布段，发布段本身不变：`develop` → `release/vX.Y.Z` → PR 合入 `main` → 打 tag → 同步回 `develop`（git-publish）。工作分支不得直接向 `main` 提 PR。
- 合并后删除远程与本地工作分支，切回 `develop` 并更新。

## Alternatives considered

**继续在 `develop` 上提交并直推。** 否决：这样会绕过评审，也与 `AGENTS.md` 5.1 已有的受保护分支规则矛盾。

**工作分支直接向 `main` 提 PR。** 否决：`main` 是生产分支，只接收发布 PR；`develop` 必须保持唯一集成点，改动在这里汇合后才进入发布。

**现在就用本地 pre-push 钩子强制执行。** 暂缓：在服务端为 `develop` 配置保护规则效果更好，可以之后再启用；在此之前由 `AGENTS.md` 约束，agent 按规执行。

## Consequences

- 每个改动在集成前都有 PR 评审点和 CI 运行；`develop` 历史按任务分组。
- 成本：每个任务多出切分支、推送、建 PR 几步，而且这些步骤依赖网络和已登录的 `gh`。
- 本笔记扩展 [仓库本地 AI 开发工作流适配](2026-09-18-project-ai-workflow-adaptation.zh.md)，不取代它；发布段模型仍以那份笔记为准。任务在 [todos-list](../../../../docs/todos-list.md) 中登记为 T-047。
