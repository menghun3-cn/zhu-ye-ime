# Agent Note: 设置窗口视觉改版（FR-064，T-148/T-149）

Status: implemented

## Problem

设置窗口（`zhu-ye-settings.exe`）自 T-073 起一直是 Windows 原生中性风——白底、
淡蓝选中（`#E1EEFB`/`#1E88E5`），T-125 只规范了排版。窗口与[官网建立的竹叶
品牌](../../implemented/architecture/2026-10-03-official-website-and-brand-assets.md)
毫无关联，观感像"任何设置框"。经 grill-me 两轮（2026-10-09）用户锁定**只换
面容**的视觉改版；结构性[设置窗口设计](../../proposed/architecture/2026-10-01-settings-window-process-and-registration-ownership.md)
与其已验收的度量、交互机制全部不动（T-148 交付规格与视觉稿，2026-10-10 用户
目检通过）。

## Decision

窗口以**颜色 + 唯一品牌块**承载竹叶身份（设计文档 [§3–§9](../../../docs/设置窗口视觉设计.md)）；
所有视觉改动均在 GDI 可表达范围内（实色、1px 细线、圆角矩形、ClearType 文字；
无渐变/投影/半透明/动画，S-24）。

1. **范围——只换面容**：颜色、字重层级、控件形态改变；窗口 880×700、导航 200、
   行高 76、标题区 80、字号体系 16/13/24（T-125 验收值）、三页信息架构、
   chips 二选一/占位展开/子视图/浮层面板交互、候选窗已验收的蓝系全部不动。
2. **品牌块**位于导航列顶部、只在导航列内部（`LOGICAL_BRAND` 64 + 顶部间距 10；
   **对内容区度量零影响**）：叶片图形以 `DrawIconEx` 绘制 `zhu.ico`（T-142
   资源 ID 1，22px，`DI_NORMAL`），其后为字标「竹叶输入法」16/600 与 1px
   `bark` 细线。无吉祥物、无花纹、其余页面零图形。
3. **两套主题跟随系统**（D-30 不变；高对比仍走系统色，S-23）：浅=竹纸
   `#F7F9F4` / 墨 `#22302A` / 叶 `#3D7A4E`；深=墨绿黑 `#1F2822` / 亮叶
   `#80B98A`。
4. **统一的选中语言**（S-22）：导航与 chips 选中=淡竹叶底 + 深叶字；实底叶色
   只用于按钮（与面板悬停格）。深色实底按钮用**墨字** `#10241A` 于 `#80B98A`
   （7.15:1）——白字仅 2.3:1 被否决（S-20）。
5. **配置兼容**（S-25）：`SettingsTheme` 从 15 键扩到 **22 键**——新增 7 个
   可选键（`on_accent`/`chip_border`/`tag_bg`/`tag_text`/`sprout`/`ok_text`/
   `expanded_bg`）；`zhu-ye-core` 的 `SettingsPalette` 以 22 个 Option 字段
   对应。自定义 `themes\*.json` 缺任一新键即回退内建默认；不升
   `CONFIG_FORMAT_VERSION`；关于页状态行显示「应用 N/22 键」。`bark` 为品牌
   固定色常量，**永不被主题文件覆盖**。高对比由 `SystemColors` 全量映射 22 键
   （8 个新映射：`on_accent`→highlight 前景、`chip_border`/`expanded_bg`→
   系统边框/窗口色、标签对/sprout/ok_text→前景等），且不读主题文件。
6. **组件语言**（T-149）：实底按钮 `accent` + `on_accent`（`small_medium`
   500）；描边按钮 `control_background` + 1px `chip_border` 描边 + `item_text`
   （返回/恢复/打开系统输入法设置/二级修复/可点的应用更新）；禁用=同描边 +
   `secondary_text`；规划中标签=pill `tag_bg`/`tag_text` + 6px `sprout` 圆点，
   13px；占位展开=`expanded_bg` 底 + `line` 描边，13px 次要字；关于页结果区
   （新增容器）=同一展开面 + 空态提示；状态行用 `ok_text`/`warn_text`
   （管理页已注册/异常、关于页在线/离线）；chips 13px、选中 600 字重；面板格
   以鼠标悬停高亮（TrackMouseEvent `TME_LEAVE` 生命周期）表达选中格——非激活
   浮层面板无驻留选中（点击即上屏，S-11 不变）；面板外框 `chip_border`。
7. **字号/品牌锁定**：chips/按钮 13px、标签 13px，按 T-125 的 16/13/24 体系
   （草稿 14px 作废，S-28）；品牌图形 1:1 复用 `logo`/`zhu.ico`，零新资源
   （S-26）。

## Alternatives considered

- **处处铺满竹叶品牌**（吉祥物、叶片花纹、每页品牌条）：否决——设置窗口是日常
  高频工具；"颜色 + 唯一品牌块"才是与它匹配的克制表达，官网保留丰富表达。
- **保留中性风只做润色**：否决——用户简报与 grill 轮次要求可辨识的身份；该方案
  保留"任何设置框"的缺陷。
- **内容区顶部通栏品牌条**：否决——会占用正文字度量、削弱 T-125 验收断言；导航
  列内位置视觉权重相同且度量零影响。
- **渐变/投影/半透明**：否决——GDI 无法表达；为装饰引入 D2D 路径与窗口零框架
  架构相悖（S-24）。
- **品牌图形走 `PolyBezier` 控制点表 / 预渲染 DIB**（草稿候选）：实现时否决——
  曲线表维护成本高，DIB 违背"零新资源"；`DrawIconEx` 1:1 复用现有 `zhu.ico`
  （S-26）。
- **深色实底按钮白字**：否决——实测 2.3:1 不达 AA；墨字 `#10241A` 于
  `#80B98A` 达 7.15:1（S-20）。
- **浮层面板驻留选中态**：否决——非激活面板点击即上屏（S-11）；悬停高亮表达
  当前格而不改变机制（S-27）。
- **占位展开复用 `tag_bg`**（草稿）：实现时否决——独立 `expanded_bg` 键让主题
  文件可控该面（S-25）。

## Verification

- 单测：`设置窗视觉改版新键可选解析`（theme_file）、`视觉改版新字段随深浅预设且深浅互异`
  与 `主题文件新键覆盖与缺键回退且bark不可覆盖`（theme），连同既有套件全部通过
  （`cargo test --workspace`）。
- 门禁：`cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、
  `git diff --check` 全绿。
- 像素取证（宿主 `--shot`，125% DPI，帧 1100×875）：浅/深共 22 帧；逐帧令牌
  直方图与浅色（`#F7F9F4`/`#EFF3EA`/…）与深色（`#1F2822`/`#192019`/…）调色板
  完全吻合；品牌块竹节棕细线位于逻辑 y=63；导航 rail-sel 与 chips 面计数符合
  设计；首屏 ≥7 行断言不变；关于页结果区呈现展开面。完整表见 验收标准 §18.2。
- VM 交互目检（真机深色悬停、高对比实色）沿用设置窗口既有验收挂起口径
  （vm-accept-sop）。

## Consequences

- **代价**：主题面更大（22 键 + 保留常量），主题文件可多调七个面；面板新增
  一小段 TrackMouseEvent 悬停生命周期；关于页文字在其新结果容器内缩进一个
  `gap`。
- **收益**：窗口终于体现竹叶品牌；深色强调对比度按 AA 审计（7.15:1，取代被
  否决的 2.3:1 白字）；既有自定义主题照常解析并优雅回退；零新资源、零新依赖；
  候选窗蓝系与全部交互机制不动（S-15 不触发）。

## Was proposed

由 `proposed/feature/2026-10-10-settings-window-visual-redesign.{md,zh,i18n.yaml}`
（设计批次 Agent Note）在 T-148 用户目检通过、T-149 实现落地后移动至此。

## Related

- [官网与品牌资产](../../implemented/architecture/2026-10-03-official-website-and-brand-assets.md)：
  令牌家族、`logo`/`zhu.ico` 及 T-142 标题栏图标——品牌块复用的出处。
- [设置窗口流程与注册归属](../../proposed/architecture/2026-10-01-settings-window-process-and-registration-ownership.md)：
  本次视觉改造刻意不触碰的结构性设置窗口设计。
- 设计与验收文档：[设置窗口视觉设计](../../../docs/设置窗口视觉设计.md)
  （规格，§3–§9，S-19–S-29）与 [验收标准 §18](../../../docs/验收标准.md)
  （T-148/T-149 验收，18.2 像素取证表）。
