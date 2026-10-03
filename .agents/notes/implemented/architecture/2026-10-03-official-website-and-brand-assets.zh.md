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
静态站；GitHub Pages 配置就绪、真部署另行安排。同日细化（D-69）：下载页以安装包为主、
下载一律为 GitHub Releases 资产直链（exe 安装包为规划项，显式标注）；站点新增 AI 发现
入口文件（llms.txt / agents.txt / robots.txt / sitemap.xml），需求 FR-058。

## Decision

### 站点：`site/` 下零构建静态页

- `site/` 含 5 个页面（`index`、`docs`、`download`、`privacy`、`about`）+ `404.html`、
  一份共享样式表 `assets/css/site.css`、SVG 资产与维护说明 README。**全站零 JS 文件**
  （设计 S-16）：唯一 `<script>` 是首页 JSON-LD 数据声明；动效纯 CSS 且由
  `prefers-reduced-motion` 门控。
- 所有站内链接使用**相对路径**（强制约定：Pages 以 `/repo/` 前缀发布与 `file://` 直开
  时都可用）。页间链接一律**显式写 `index.html`**（如 `./docs/index.html`、`../index.html`）：
  Chromium 系浏览器在 `file://` 下对以 `/` 结尾的目录 URL 渲染目录列表而非该目录的
  index.html；`/docs/` 这种目录形式 URL 仅用于 sitemap.xml（部署环境语义）。
  该契约由 `scripts/check-site-links.ps1` 强制（PowerShell 5.1 兼容、
  ASCII 注释）。
- header/footer 在 5 个页面间复制式维护（设计 S-14）；一致性由验收核对兜底，不用模板引擎。
- **下载页以安装包为主**（D-69）：「发行包」表区分便携 zip（内测期产物）与规划中的
  exe 安装包（带"规划中"标签）；下载一律走 GitHub Releases 资产直链——站点不托管
  安装包、不出现指向不存在产物的下载按钮。
- **AI 发现入口文件**在站点根（FR-058，设计 §5.7/S-18）：`llms.txt` 遵循 llmstxt.org v2
  （H1 + `>` 摘要 + 分节文件列表）；`agents.txt` 因无统一国际标准而采用**站点自有结构**
  （H1 + 摘要 + 「项目事实/内容边界/可信路径/文件索引」四节）；`robots.txt` 全允许并指向
  `sitemap.xml`，其绝对 URL 暂钉默认 GitHub Pages 地址、自定义域名后必改。四个文件全部
  纳入 `check-site-links.ps1`（Markdown 链接与 `<loc>` 集合），并与页面共享内容真实性规则。
- 部署就绪（设计 §10）：`.github/workflows/pages.yml` 通过 `upload-pages-artifact` 将
  `site/` 发布到 GitHub Pages；真部署为后续任务（D-68）。`site/README.md` 记录维护与
  部署步骤。
- 设计令牌（S-13~S-18，`docs/官网设计.md` §13）：零构建为当前最优解并定义了演进触发条件
  （S-13）；布局复制式维护（S-14）；候选窗示例插图锁定验收配色（S-15）；零 JS（S-16）；
  版本状态行与里程碑口径联动（S-17）；AI 入口文件格式与链接契约（S-18）。

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
并列译词）。规划项（M13 五项、发布流程、英文站、exe 安装包）只出现在首页"路线图"区块或
显式标注的"规划中"格中。共享版本状态行（"Windows 内测版 · M1–M12 核心能力已实现（自动化验证
通过）· 格式候选与设置窗口的真实环境交互验收收尾中"）与 T-061/T-080 验收欠账口径一致。
llms.txt/agents.txt 亦属站点内容，同样受词表约束（S-18）。按用户要求，站点任何页面
不点名第三方/竞品（青简、搜狗等）；借鉴留痕仅存仓库文档（设计 §12 借鉴模式清单、
docs/竞品和参考.txt）。关于页也不单设「当前状态」「借鉴记录」节（用户要求精简）：
状态口径由共享页脚版本状态行与首页承担。

### 领域词汇

CONTEXT.md 新增"站点"分节：官网、文档页（区别于 `docs/`）、路线图区块、品牌资产、吉祥物，
以及 zhu-ye（英文名）、AI 发现入口、发行包形态。
需求 FR-051~FR-058、设计 S-13~S-18 与本笔记互相链接；交付由任务 T-083（文档批次，已完成）
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
  必须在 UI/里程碑事实变化时同步更新（S-15/S-17 联动义务）；AI 入口文件必须与页面内容
  保持同口径（S-18 义务），sitemap.xml 在配置自定义域名后必须更新绝对 URL；站点内容仅中文，
  英文站需另行立项。
