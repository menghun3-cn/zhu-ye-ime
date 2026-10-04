# Agent Note: GitHub Pages 首次启用与环境保护分支策略（v0.1.1-alpha 发布）

Status: implemented

[English](2026-10-04-github-pages-first-enable.md) | 中文

## Problem

v0.1.1-alpha 发布批次包含 `site/`（T-084 官网）——首次真正把官网部署上线。
`release/vX.Y.Z` PR 合入 `main` 后，`pages.yml` 自动触发但部署失败：第一次
`actions/configure-pages@v5` 报 "Get Pages site failed... repository has Pages
enabled and configured to build using GitHub Actions"（对应 GitHub REST Pages
API `404 Not Found`——仓库从未创建过 Pages 站点）；用
`gh api -X POST /repos/{o}/{r}/pages -f build_type=workflow` 创建站点后，后续
workflow run 全部 **completed|failure 且 job 无任何 step（2 秒内失败）**，日志
blob 缺失，`error: null`——job 在分配到 runner 之前就被拒绝。

## Decision

- 首次启用无 UI 操作的自动化路径：`gh api -X POST repos/{owner}/{repo}/pages
  -f build_type=workflow`（等效 Settings → Pages → Source: GitHub Actions）。
- POST 创建站点时会**自动生成 `github-pages` 环境**，并附带
  `branch_policy` 保护规则（`deployment_branch_policy.custom_branch_policies:
  true`）——默认只把 **develop** 加为唯一允许部署分支（POST 未传 source 时的
  默认值）。`pages.yml` 从 `main` 构建部署，被环境保护拒绝。
- 修复：把 `main` 加入环境部署分支策略：
  `gh api -X POST repos/{owner}/{repo}/environments/github-pages/deployment-branch-policies
  -f name=main -f type=branch`（或 Settings → Environments → github-pages → 添加部署分支）。
- 重跑：`gh workflow run pages.yml --ref main`（pages.yml 含 `workflow_dispatch`）
  或推送一次 `main`。

## Alternatives considered

- **改 `pages.yml` 从 develop 部署**：否决——Pages 只应由生产分支（main）产物
  供给；develop 上的站点内容未经发布门禁。
- **把环境保护改成 `protected_branches: true`**：可用但过宽（删除策略对默认分支
  外的任意保护分支开放）；按需加分支更精确。

## Consequences

- 排查特征备忘：workflow **completed|failure + job `steps: []` + 2 秒内完成 +
  日志缺失**（"log not found"）= runner 未启动 = 环境保护拒绝，优先查
  `GET /environments/github-pages/deployment-branch-policies`；若
  `configure-pages` 报 "Get Pages site failed" = Pages 未启用，先 POST 建站。
- `site/README.md` 部署节补充"首次启用"注意（[site/README.md](../../../../site/README.md)
  L101 起），把两步 API 固化。
- 环境 `github-pages` 的分支策略是仓库级状态，不在 git 中；重装/迁移仓库后需
  重新配置。`pages.yml` 本身不声明环境权限以外的策略，因此部署不因分支策略而
  在克隆副本上失败。
- 变更 `pages.yml`、更换自定义域或回滚时，环境策略仍是同一份配置。

验证（2026-10-04）：分支策略加入 `main` 后，`gh workflow run pages.yml --ref main`
成功（deploy-pages completed|success）；线上验证
`https://menghun3-cn.github.io/zhu-ye-ime/` 返回 200，首页 hero-note 与
download 页显示 v0.1.1-alpha，`llms.txt` 同口径；Release v0.1.1-alpha 含
`ai-zhu-ye-ime-0.1.1-alpha-test.zip`（18.56 MB）。

相关记录（交叉链接）：[官网与品牌资产
（architecture/2026-10-03-official-website-and-brand-assets.zh.md）](../../implemented/architecture/2026-10-03-official-website-and-brand-assets.zh.md)
——site/ 部署配置与 S-18 口径出处；[发布流程
（process/2026-09-29-work-branch-pr-flow.zh.md）](../../implemented/process/2026-09-29-work-branch-pr-flow.zh.md)
——develop→release→main→tag 的发布链。
