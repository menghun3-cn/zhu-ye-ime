# Agent Note: 抽取 zhu-ye-ui crate 并移除 rlib

Status: implemented

[English](2026-10-02-zhu-ye-ui-crate-and-rlib-removal.md) | 中文

## 问题

设置窗口要复用候选窗的纯逻辑主题与布局原语。T-073 选择的落地方式是给 `zhu-ye-ime`
加 `crate-type = ["cdylib", "rlib"]`，让 `zhu-ye-settings` 直接依赖整个 IME crate，
代价是设置窗口二进制在构建与链接层面连带 TSF 侧代码（`input`/`tsf`/词典加载）。
S-10 把抽取记为该收尾任务（T-081）。

抽取有两处难点：

- 原语本身（`UiColor`/`UiThemeKind`/`SystemColors`/`UiRect`/`BASE_DPI`/
  `estimate_text_width`/`fit_text`）夹在 `candidate_ui` 里，而该模块同时承载候选窗
  视图逻辑；移动必须不触碰任何既有调用路径与测试。
- TSF 身份常量（CLSID、Profile GUID、键盘 TFCAT、语言 ID 十六进制、词典文件名、
  安装目录段）由 `zhu-ye-settings` 从 `zhu_ye_ime::tsf` 引用。移除 `rlib` 依赖后
  这些常量在 IME crate 里就孤立了，Rust 侧单一主源必须迁到两个消费方都能到达的地方。

## 决策

### 1. 新 crate `crates/zhu-ye-ui`（纯 std，零依赖）

持有七项 UI 原语及其测试。`candidate_ui` 以 `pub use` 转发，保证既有调用路径
（`candidate_window`、设置窗口 import、165 项 IME 测试）零改动。`fit_text` **刻意不
转发**：它没有生产代码消费者（仅测试使用），转发会在 `#[path]` 独立编译的
demo/e2e 二进制里触发 `unused_imports` 警告；该 API 现在只存在于 `zhu_ye_ui::fit_text`。

### 2. `zhu_ye_core::identity` 成为 Rust 侧单一主源

身份常量迁入 `zhu-ye-core`（两个消费方共享的唯一 crate）的新纯模块。GUID 以
`u128` 字面量书写——文本观感为大端形式，恰是 `windows::core::GUID::from_u128`
的输入——因此 `zhu-ye-core` 保持零 Windows 依赖的契约。模块同时提供
`guid_text(u128)`，并用测试把值与 `scripts/ime-identity.ps1` 钉死。

消费方按需转换：

- `zhu_ye_ime::tsf` 用 `windows::core::GUID::from_u128(identity::...)`（const fn）
  重建自己的 `windows::core::GUID` 常量。
- `zhu-ye-settings::registry` 接收 `u128`，在本地 `guid_text` 里转换；其 GUID 相关
  测试改从 `zhu_ye_core::identity` 导入。
- 字符串常量由 `zhu_ye_ime::tsf` 以 `pub use` 转发。

`scripts/verify-tsf-identity.ps1` 的读源改为 `crates/zhu-ye-core/src/identity.rs`
（`Get-RustGuidConst` 正则适配了 `: u128 = 0x...` 形式）。

### 3. 设置窗口高对比度映射本地化

`settings_theme_from_system_colors` 原先委托 `candidate_ui::theme_from_system_colors`
做 BGR→RGB 字节序转换与高对比度映射。解耦后本地重新实现同一映射口径，渲染结果
不变，设置窗口不再伸手进 IME crate。

### 4. `zhu-ye-ime` 恢复仅 `cdylib`

`crate-type` 去掉 `rlib`；`zhu-ye-settings/Cargo.toml` 移除 `zhu-ye-ime` 依赖、
改依赖 `zhu-ye-ui`。`cargo tree` 证实 `zhu-ye-settings` 只依赖 `zhu-ye-core` +
`zhu-ye-ui`。`candidate-demo` 与 `host-e2e` 二进制不受影响：它们经 `#[path]`
独立编译候选窗模块，从不按包名引用 `zhu_ye_ime`。

## 曾考虑的替代方案

**身份常量留在 `tsf.rs`，在 settings 里复制一份。** 落选：复制会产生第三份漂移源，
`verify-tsf-identity.ps1` 还得维护两条 Rust 源路径。

**让 `zhu-ye-core` 依赖 `windows-core` 存真 GUID。** 落选：`windows::core::GUID`
虽是纯数据，但 core crate 的契约是"不依赖 Windows"；保持无依赖让 core 在任何环境
可构建可测试。`u128` 字面量零表示成本，转换在消费方各做一次。

**只本地化常量，`theme_from_system_colors` 经 `pub use` 继续从
`candidate_ui` 调。** 落选：那样设置窗口仍得依赖 IME crate，抽取失去意义；本地化
映射只有六行，由既有四项主题测试钉住。

**`fit_text` 也一并转发。** 落选：无生产消费者，转发在 `#[path]` 二进制造成
`unused_imports` 警告；其归宿就是 `zhu_ye_ui`。

**单独建一个 identity crate。** 落选：六个常量不值一个新 crate；`zhu-ye-core`
本就托管其他共享纯模块。

## 后果

- 依赖图：`zhu-ye-settings` → `zhu-ye-core` + `zhu-ye-ui`；`zhu-ye-ime` →
  `zhu-ye-core` + `zhu-ye-ui`。IME crate 不再产出 `rlib`。
- 改 `zhu-ye-ime` 不再触发设置窗口重编；TSF 符号离开设置窗口的链接面（exe 体积
  几乎不变，因为 LTO 早已剥掉——收益在依赖图与耦合，不在字节数）。
- 后续 UI 原语在 `zhu-ye-ui` 演进；身份常量在 `zhu_ye_core::identity` 演进，
  `verify-tsf-identity.ps1` 继续与 `ime-identity.ps1` 交叉比对（D-42）。
- 验证证据：workspace 测试全绿——core 222（含新增 identity 2）、zhu-ye-ui 7（新增）、
  ime 165（不变）、settings 97（不变）、dict 45、cli 117、updater 20、host-e2e 4、
  demo 2；fmt、clippy `-D warnings`、三道 verify 脚本与 `git diff --check` 干净；
  `Cargo.lock` 无 GUI 框架；宿主截图（候选窗深色、候选窗 192 DPI 浅色、设置窗口
  工具箱页、设置窗口更新页）渲染正常。

登记于 [docs/todos-list.md](../../../../docs/todos-list.md)（T-081），S-10 的
决策记录见 [docs/设置窗口设计.md](../../../../docs/设置窗口设计.md)。
