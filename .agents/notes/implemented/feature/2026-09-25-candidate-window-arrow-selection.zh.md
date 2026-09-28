# Agent Note：候选框上下键选择（T-039）

Status: implemented

English | [中文](2026-09-25-candidate-window-arrow-selection.zh.md) | 中英镜像

## Problem

用户反馈（T-039）：默认选中第一个候选后，上下方向键无法把选中行移到
第二个、第三个及后续候选。两层根因：引擎根本没有页内选中序号状态
（`candidate_ui_view` 硬编码 `selected: 0`，候选窗永远从第一行打开）；
TSF 键路由没有映射 `VK_UP`/`VK_DOWN`，按键落给宿主（移动宿主光标）而非
移动选中行。

## Decision

引入真实的页内选中状态，并把方向键路由到它。

- `InputEngine` 新增 `selected_on_page` 以及 `select_up()` / `select_down()`
  （按页内可见候选数封顶，选中行在页边界停住）。输入变化
  （`refresh_candidates`）与组合结束（`clear_composition`）时选中行重置 0；
  翻页与切层**保持行位**，并按新页候选数封顶（`clamp_page` 内含
  `clamp_selected`，`next_page`/`previous_page` 也调用）。
- `CandidateUiView.selected` 改取引擎页内序号，高亮随方向键移动。
- **空格改为上屏当前选中行候选**（`handle_space` / `preview_space` 按
  `selected_on_page` 索引 `visible_candidates`，不再用 `.first()`）。
  默认选中 0，旧行为完全不变。
- TSF 路由：`VK_UP` → `SelectUp`，`VK_DOWN` → `SelectDown`。这两个动作是
  纯状态推进（不需要编辑会话），与翻页同一处理路径；仅在组合活跃时吃下，
  Ctrl/Alt 修饰键参与时照常放行——没有候选窗时方向键保持宿主语义。

## Alternatives considered

**用 `ITfCandidateList`/TSF 候选 UI。** 否决：项目刻意使用自绘 GDI 候选窗；
伪候选集成只会重复状态而无收益。

**选中行在页边界自动翻页。** 本里程碑否决：把两个机制耦合在一起并改变
翻页行为；页内封顶可预测，且已有 `-`/`=` 翻页
（2026-09-25-page-keys-minus-plus.md）。

**翻页时选中行重置 0。** 否决：微软拼音/搜狗翻页都保持行位；保持行位更
自然且零成本。

## Consequences

方向键现在可在页内移动高亮，空格上屏高亮候选；数字选择、翻页与默认
第一行行为均不变。引擎层与路由层的单测全覆盖。交叉参考：
[2026-09-25-page-keys-minus-plus.md](2026-09-25-page-keys-minus-plus.md)
（翻页键）、
[2026-09-25-candidate-window-row-layout-dynamic.md](2026-09-25-candidate-window-row-layout-dynamic.md)
（行布局）与
[2026-09-18-tsf-composition-and-key-events.md](2026-09-18-tsf-composition-and-key-events.md)
（键路由基础）。
