# 竹叶输入法官网（site/）

面向公众的静态官网。设计见 [docs/官网设计.md](../docs/官网设计.md)，需求见
[需求规格说明书 §19](../docs/需求规格说明书.md)，验收见 [验收标准 §15](../docs/验收标准.md)。

## 基本事实

- **零构建**：纯 HTML + 一份 CSS + 手绘 SVG，**无任何 JS 文件**（唯一 `<script>` 是首页
  JSON-LD 数据声明）。无第三方脚本、无统计、无外部字体/图标 CDN（NFR-012）。
- **相对链接契约**：站内链接一律相对路径（`../docs/`、`./assets/...`），禁止站内绝对 URL；
  这样 `file://` 直开、以及 GitHub Pages 以 `/repo/` 子路径发布都可用。
- **共享布局为复制式维护**（S-14）：header/footer 在 6 个 HTML 中各自一份，改布局要 6 处一起
  改；一致性由验收标准 15.1 的"每页必需元素"核对兜底。

## 目录

```text
site/
  index.html           首页（hero / 特性 / 候选窗示例 / 路线图 / JSON-LD）
  docs/index.html      文档页（安装与使用提炼，非仓库 docs/ 开发文档）
  download/index.html  下载页（发布状态如实呈现）
  privacy/index.html   隐私页（零网络 / 本机处理）
  about/index.html     关于页（简介 / 状态 / 许可 / 借鉴记录）
  404.html             未找到（吉祥物探头姿态）
  assets/
    css/site.css           全站样式（唯一 CSS，令牌见官网设计.md §4）
    logo.svg               logo 主标（图形 + 字标 + 拉丁名）
    logo-mark.svg          logo 图标变体（header 与 favicon 用）
    favicon.svg            favicon（深绿圆角底 + 竹叶）
    mascot-hero.svg        吉祥物·站姿（候选窗示例内嵌）——见 mascot 说明
    mascot-point.svg       吉祥物·引导姿态（备用于文档/下载页）
    mascot-404.svg         吉祥物·探头姿态（404 页）
    cand-window.svg        候选窗示例插图（真实 UI 快照 + 吉祥物 + 气泡）
```

## 本地预览

双击 `site/index.html` 即可（相对链接保证 file:// 可用）。也可起任意静态服务器：

```powershell
# PowerShell 简易 http 服务（root 需为站点目录）
# 其他选择：python -m http.server 8080 -d site
```

## 内链自检

```powershell
.\scripts\check-site-links.ps1
```

解析所有 HTML 的 `href`/`src`，站内相对引用必须命中真实文件（以 `/` 结尾的链接要求目录下有
`index.html`）；站外链接只列出不校验。任一失效 exit 1。**修改任何页面后、提交前必须跑一次。**

## 页面如何增删

- 新增页面：复制既有页面的 header/footer 块，改导航的 `aria-current="page"`，按相对链接契约
  写资源引用；保持与 5 页（+404）一致。
- 删除/改名页面：同步所有页面的导航与页脚链接，然后跑内链自检确认无残留。
- 首页 JSON-LD 只在 `site/index.html`，新页面一般不需要。

## 品牌资产如何替换

- 全部为手写 SVG（D-65：不引入第三方素材、不用 AI 生成，避免许可证登记义务）。
- logo 与吉祥物共用"叶片曲线家族"；要改风格时四个 mascot（含 cand-window 内嵌的吉祥物与
  气泡）一起改。
- **候选窗示例插图（cand-window.svg）是产品真实 UI 快照**（S-15）：配色锁定验收值
  （白底、蓝框/蓝字、选中行浅蓝块红字、灰色译文、页码）。产品候选窗配色或布局变更的 PR，
  必须同步更新此图，否则验收标准 15.1 不通过。
- favicon 用深绿底 + 浅叶，16/32px 下仍需可辨。

## 内容真实性守则（改文案前必读）

- 只宣称**已实现并经测试/验收**的能力；能力词表与禁止词见 [官网设计.md §7](../docs/官网设计.md)。
- 规划中的能力（如 M13 五项）只能出现在首页"路线图"区块并带 `<span class="tag">规划中</span>`。
- 版本状态行（footer/下载页/关于页共用的同一句话）与当前里程碑口径必须一致（S-17）；
  发布流程中，CHANGELOG 发布条目需同步更新该行。
- 下载页不得出现指向不存在产物的下载按钮；发布状态以 GitHub Releases 实际展示为准。

## GitHub Pages 部署

仓库已含 `.github/workflows/pages.yml`（将 `site/` 发布为 Pages 制品）。启用步骤：

1. GitHub 仓库 → Settings → Pages → Source 选择 **GitHub Actions**。
2. 推送 `site/` 相关改动到 `main`（或手动触发 workflow_dispatch）后，Pages 自动部署。
3. 上线前检查（验收标准 15.4）：`check-site-links.ps1` 全绿、内容核对表全过、版本状态行与
   当前发布口径一致。

之后如需自定义域名：在仓库 Settings → Pages 配置并同步 CNAME 文件。

## 本项目零 JS 决策（S-16）

目前没有 `site/assets/js/`，也不打算加。如果未来确实需要交互，先按 S-16 评审单一增强脚本，
同时保证 `noscript` 下页面完整可读。

## 相关链接

- 设计：`docs/官网设计.md`（视觉令牌、页面规格、S-13~S-17、风险）
- 决策：Agent Note `.agents/notes/implemented/architecture/2026-10-03-official-website-and-brand-assets.md`
- 验收：`docs/验收标准.md` 第 15 节（含 15.5 实测回填表格）
