# Agent Note: 官网与品牌资产（M-SITE）

Status: implemented

[English](2026-10-03-official-website-and-brand-assets.md) | 中文

## Problem

项目此前没有任何面向公众的呈现层：`README.md` 面向开发者，`docs/` 是开发文档，潜在用户没有
一处能回答"这是什么输入法、有什么不同、怎么获取"。同时也没有品牌资产——没有 logo、没有
favicon、没有吉祥物。

参考对象青简输入法官网（qingjian.app，另一个 Rust 输入法项目）已从原始 HTML/CSS/响应头完成
取证调研：**SvelteKit（Svelte 5）+ Tailwind CSS v4 + Vite，全静态预渲染，托管于 Cloudflare
Pages**，5 页结构（首页/文档/下载/隐私/关于）、构建期 markdown 文档页、下载直链 GitHub
Releases。需要决策的是：照搬该技术栈还是自研，以及内容边界定在哪里。

已确认决策（2026-10-03，grilling 访谈 D-58~D-68，用户逐条确认）：站点主体为竹叶输入法；
5 页对齐参考对象的信息架构；一次性交付内容填实的 v0（非骨架占位）；文案只宣称已实现并经
验收的能力，规划项只在显式标注的"路线图"区块出现；视觉原创（竹叶主题）而非像素级照抄；
仅中文；logo 为竹叶意象、吉祥物为**拟人化竹叶**（D-64）；品牌资产全部手写 SVG；零构建
静态站；GitHub Pages 配置就绪、真部署另行安排。

## Decision

### 站点：`site/` 下零构建静态页

- `site/` 含 5 个页面（`index`、`docs`、`download`、`privacy`、`about`）+ `404.html`、
  一份共享样式表 `assets/css/site.css`、SVG 资产与维护说明 README。**全站零 JS 文件**
  （设计 S-16）：唯一 `<script>` 是首页 JSON-LD 数据声明；动效纯 CSS 且由
  `prefers-reduced-motion` 门控。
- 所有站内链接使用**相对路径**（强制约定：Pages 以 `/repo/` 前缀发布与 `file://` 直开
  时都可用）。该契约由 `scripts/check-site-links.ps1` 强制（PowerShell 5.1 兼容、
  ASCII 注释）。
- header/footer 在 5 个页面间复制式维护（设计 S-14）；一致性由验收核对兜底，不用模板引擎。
- 部署就绪（设计 §10）：`.github/workflows/pages.yml` 通过 `upload-pages-artifact` 将
  `site/` 发布到 GitHub Pages；真部署为后续任务（D-68）。`site/README.md` 记录维护与
  部署步骤。
- 设计令牌（S-13~S-17，`docs/官网设计.md` §13）：零构建为当前最优解并定义了演进触发条件
  （S-13）；布局复制式维护（S-14）；候选窗示例插图锁定验收配色（S-15）；零 JS（S-16）；
  版本状态行与里程碑口径联动（S-17）。

### 品牌资产：手写 SVG，版权自有

- `assets/logo.svg`：竹叶图形（两条对称贝塞尔曲线 + 叶脉）+「竹叶」字标 + 拉开字距的
  拉丁名 `ZHUYE`；`assets/logo-mark.svg` 为独立图形变体，用作 `favicon.svg`
  （粗轮廓保证 16/32px 可辨）。
- 吉祥物：与 logo 共用叶片曲线家族的拟人化竹叶——`mascot-hero.svg`（站姿，首页）、
  `mascot-point.svg`（单手指引，文档/下载页）、`mascot-404.svg`（探头，404 页）。
- `assets/cand-window.svg`：产品候选窗真实快照（白底、蓝边框/蓝字、选中行浅蓝块红字、
  灰字译文、页脚 `1/2`），颜色取验收锁定值（T-030/032/037/043）。
- 无第三方素材、无 AI 生成素材；资产随仓库许可证（`MIT OR Apache-2.0`）分发，无需新增
  `docs/licenses.md` 登记（D-65）。

### 内容真实性（NFR-011）

`docs/官网设计.md` §7 给出站点文案可宣称的**已验收能力词表**（全拼与歧义切分、用户词学习、
unigram+bigram 排序、双语候选与 Tab 译文层、简拼/模糊音/整句、上下文联想、
数字格式/v 模式/emoji、英文拼写/邮箱/网址补全、领域提权、通讯录、词典包体系与签名在线
更新、设置窗口、性能基准、默认离线）与**禁止词**（macOS、AI 云服务、背单词体系、多语言
并列译词）。规划项（M13 五项、发布流程、英文站）只出现在首页"路线图"区块并带显式
"规划中"标签。共享版本状态行（"Windows 内测版 · M6-M12 已实现 · 发布流程收尾中"）与
T-080 验收欠账口径一致。

### 领域词汇

CONTEXT.md 新增"站点"分节：官网、文档页（区别于 `docs/`）、路线图区块、品牌资产、吉祥物。
需求 FR-051~FR-057、设计 S-13~S-17 与本笔记互相链接；交付由任务 T-083（文档批次，已完成）
与 T-084（实现批次）跟踪。

## Alternatives considered

### SvelteKit + adapter-static + Tailwind v4（青简技术栈）

否决。可以复刻参考站的技术栈，但需要引入 Node/pnpm 工具链与构建步骤，与仓库
Rust/PowerShell/git 工作流冲突（AGENTS.md 第 8 节），也不符合第一性原理：5 个低频静态页
不需要框架。演进触发条件已写明（S-13：文档量产或页面 >10）。

### Rust 静态生成器（cargo、pulldown-cmark + 模板）

本期否决：当前页面数量与手写静态内容下无收益。已记录为 S-13 触发后的演进路径。

### mdbook / Docusaurus / VitePress

否决：文档工具气质不匹配产品官网，且三者都引入工具链（Node 或 Rust mdbook）而无需求。

### AI 生成或第三方 logo/吉祥物

否决：带来许可证登记义务（docs/licenses.md）、多姿态风格不一致，且违反自研可控原则
（AGENTS.md 第 1 节）。手写 SVG 保证全部资产自有、零许可负担。

### 熊猫（或其他动物）吉祥物

访谈中否决（D-64）：熊猫群众基础广，但与"竹叶"品牌绑定弱、且流于俗套；拟人化竹叶辨识度
高、直接源于品牌名。

## Consequences

- **收益**：站点任何位置零工具链、零第三方请求（NFR-012 最强形态——`file://` 全可开、
  网络面板 0 外部请求、0 JS 文件）；资产完全自有；诚实文案可按能力词表机械核对。
- **代价**：header/footer 复制式维护存在漂移风险（由验收核对兜底，规模上来触发 S-13
  重评估）；零 JS 的表达限制（移动导航换行收起、动效仅 CSS）；版本状态行与候选窗示例插图
  必须在 UI/里程碑事实变化时同步更新（S-15/S-17 联动义务）；站点内容仅中文，英文站需另行
  立项。
