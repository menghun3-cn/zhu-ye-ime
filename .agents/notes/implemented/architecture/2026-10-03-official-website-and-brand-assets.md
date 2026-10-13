# Agent Note: Official website and brand assets (M-SITE)

Status: implemented

[中文](2026-10-03-official-website-and-brand-assets.zh.md) | English

## Problem

The project had no public-facing presence: `README.md` addresses developers and
`docs/` is development documentation, so a potential user had no place that
answers "what is this input method, what makes it different, how do I get it".
There were also no brand assets — no logo, no favicon, no mascot.

The reference site, qingjian.app (another Rust input-method project), was
studied forensically from raw HTML/CSS/response headers: **SvelteKit (Svelte 5)
+ Tailwind CSS v4 + Vite, fully static pre-rendered, hosted on Cloudflare
Pages**, with a 5-page structure (home / docs / download / privacy / about),
build-time markdown docs, and downloads direct-linked to GitHub Releases. The
question was whether to replicate that stack or build our own, and at what
content boundary.

Confirmed decisions (2026-10-03, grilling rounds D-58~D-68, user confirmed all):
the site belongs to zhu-ye; five pages mirroring the reference information
architecture; a fully content-filled v0 (not a placeholder skeleton); copy that
claims only implemented and accepted capabilities, with planned items confined
to an explicitly labeled roadmap block; original visual language (bamboo-leaf
theme) rather than a pixel copy; Chinese only; logo with a bamboo-leaf motif and
a **personified bamboo leaf** as mascot (D-64); all brand assets as
hand-written SVG; zero-build static site; GitHub Pages configuration ready,
actual deployment deferred. Same-day refinement (D-69): the download page is
installer-centric with all downloads as GitHub Releases asset direct links — the
exe installer was a planned item back then, shipped since v0.1.2 (T-097)
and the page was simplified on 2026-10-14 (T-158); the site also ships
AI-discovery entry files (llms.txt / agents.txt / robots.txt / sitemap.xml),
FR-058.

## Decision

### Site: zero-build static pages under `site/`

- `site/` holds five pages (`index`, `docs`, `download`, `privacy`, `about`) +
  `404.html`, one shared stylesheet `assets/css/site.css`, SVG assets, and a
  maintenance/README. **No JS files at all** (design S-16): the only `<script>`
  on the site is the JSON-LD data declaration on the home page; motion is pure
  CSS gated by `prefers-reduced-motion`.
- All internal links are **relative paths** (mandatory so the site also works
  when Pages serves it under a `/repo/` prefix and when opened via `file://`).
  Page-to-page links target **explicit `index.html` files** (`./docs/index.html`,
  `../index.html`) because Chromium's `file://` renders a directory listing for
  slash-terminated directory URLs instead of the directory's index.html; the
  pretty `/docs/` form is used only in sitemap.xml (deployment semantics).
  The contract is enforced by `scripts/check-site-links.ps1` (PowerShell 5.1
  compatible, ASCII-only comments).
- Header/footer are copy-maintained across the five pages (design S-14);
  consistency is checked by the acceptance pass, not by a template engine.
- Downloads are **GitHub Releases asset direct links only** (D-69) — the site
  never hosts installers and shows no button for a nonexistent asset. Current
  form (simplified 2026-10-14, T-158): the home-page primary CTA reads 下载 and
  points straight at the latest exe installer asset (no hop through the download
  page); the download page is a version line plus two download cards (exe
  installer recommended / portable zip) plus compact install, uninstall and
  checksum/源码 sections. Asset names embed the version
  (`ai-zhu-ye-ime-setup-<v>.exe`, `zhu-ye-ime-<v>.zip`), so every direct link
  must be updated on release (S-17 duty, checklist in 发布流程.md).
- **AI-discovery entry files** at the site root (FR-058, design §5.7/S-18):
  `llms.txt` follows llmstxt.org v2 (H1 + blockquote summary + sectioned file
  lists); `agents.txt` uses a site-owned structure (H1 + summary + 项目事实 /
  内容边界 / 可信路径 / 文件索引 sections) since no universal agents.txt
  standard exists; `robots.txt` allows all crawlers and points at
  `sitemap.xml`, whose absolute URLs are pinned to the default GitHub Pages
  address until a custom domain is configured. All four files are covered by
  `check-site-links.ps1` (markdown links and `<loc>` sets) and share the page
  content-honesty rules.
- Deployment readiness (design §10): `.github/workflows/pages.yml` publishes
  `site/` to GitHub Pages via `upload-pages-artifact`; actual deployment is a
  later task (D-68). `site/README.md` documents maintenance and deployment.
- Design tokens (S-13~S-18, `docs/官网设计.md §13`): zero-build is the current
  optimum with a defined revisit trigger (S-13); copy-maintained layout (S-14);
  candidate-window illustration locked to the verified UI palette (S-15); zero
  JS (S-16); version-status line kept in sync with milestones (S-17);
  AI-entry-file format/linkage contract (S-18).

### Brand assets: hand-written SVG, self-owned

- `assets/logo.svg`: bamboo-leaf mark (two symmetric Bézier curves + vein) with
  the 「竹叶」 wordmark and spaced `ZHUYE` latin; `assets/logo-mark.svg` is the
  standalone glyph used as `favicon.svg` (bold outline for 16/32 px legibility).
- Mascot: a personified bamboo leaf sharing the logo's leaf-curve family —
  `mascot-hero.svg` (standing, home), `mascot-point.svg` (pointing, docs and
  download pages), `mascot-404.svg` (peeking, 404 page).
- `assets/cand-window.svg`: faithful snapshot of the real candidate window
  (white panel, blue border/pinyin text, light-blue selected block with red
  selected text, gray translations, `1/2` page footer) per verified palette
  values (T-030/032/037/043).
- No third-party or AI-generated material; assets ship under the repository
  license (`MIT OR Apache-2.0`) with no new `docs/licenses.md` entries (D-65).

### Content honesty (NFR-011)

`docs/官网设计.md §7` lists the verified-capability vocabulary the site copy
may claim (full pinyin and ambiguous segmentation, user-word learning,
unigram+bigram ranking, bilingual candidates and Tab translation layer,
initial/fuzzy/sentence input, context suggestions, digit/v-mode/emoji, English
spelling/email/URL completion, domain boost, contacts, dictionary-pack system
with signed online updates, settings window, performance benchmarks, offline by
default) and forbidden claims (macOS, AI cloud services, memorization systems,
multi-language parallel translations). Planned items (roadmap block) are
currently the English site only; shipped facts (online update source, exe
installer, Pages) may appear normally. The
shared version-status line ("Windows 10/11 x64 · v0.1.3 正式版 · 更新源已开通 · exe
安装包已发布 · Pages 已上线") tracks the released version (S-17). The llms.txt/agents.txt
files are site content too and
obey the same vocabulary rules (S-18). Per user directive the site never names
third parties or competitors (青简, 搜狗, …) on any page; attribution records
live only in repo docs (设计 §12 借鉴模式清单, docs/竞品和参考.txt). The about
page carries no dedicated 状态/借鉴 sections by user request: status claims
live in the shared footer version-status line and the home page.

### Domain vocabulary

CONTEXT.md gained a 站点 (site) section: 官网, 文档页 (distinct from `docs/`),
路线图区块, 品牌资产, 吉祥物, plus zhu-ye (the English name), AI 发现入口, and
发行包形态. Requirement FR-051~FR-058, design S-13~S-18, and this note are
cross-linked; task T-083 (docs batch, done) and T-084 (implementation) track
delivery.

## Alternatives considered

### SvelteKit + adapter-static + Tailwind v4 (qingjian's stack)

Rejected. It reproduces the reference site's stack but requires a Node/pnpm
toolchain and a build step, which conflicts with the repository's
Rust/PowerShell/git workflow (AGENTS.md §8) and fails first principles: five
low-frequency static pages do not justify a framework. Revisit trigger is
explicit (S-13: docs volume production or >10 pages).

### Rust static generator (cargo, pulldown-cmark + templates)

Rejected for this phase: no benefit at the current page count and static,
hand-crafted content. Recorded as the evolution path when the trigger in S-13
fires.

### mdbook / Docusaurus / VitePress

Rejected: documentation-tooling character does not fit a product site, and all
three add a toolchain (Node, or Rust mdbook) without a need.

### AI-generated or third-party logo/mascot

Rejected: license-recording obligations (docs/licenses.md), style
inconsistency across poses, and violation of the self-reliance principle
(AGENTS.md §1). Hand-written SVG keeps every asset self-owned and license-free.

### Panda mascot (or other animals)

Rejected in the interview (D-64): a panda has broad appeal but weak binding to
the "竹叶" brand and reads as a cliché; a personified bamboo leaf is
distinctive and derives directly from the name.

## Consequences

- **Gains**: no toolchain or third-party requests anywhere in the site
  (NFR-012 in strongest form — `file://` opens everything, network panel shows
  0 external requests, 0 JS files); assets fully self-owned; honest copy is
  mechanically checkable against the capability vocabulary list.
- **Costs**: copy-maintained header/footer drift risk (mitigated by acceptance
  checks and, at scale, the S-13 revisit trigger); no JS interactivity (mobile
  nav is line-wrapped, motion is CSS-only); the version-status line and the
  candidate-window illustration must be updated whenever UI/milestone truth
  changes (S-15/S-17 duties); AI entry files must stay in lockstep with the
  pages (S-18) and sitemap.xml URLs must be updated on a custom-domain
  deployment; the site content is only Chinese until an English site task is
  raised.
