# Agent Note：候选窗 emoji 彩色渲染（DirectWrite 旁路）（T-101，T-074 演进）

Status: proposed

English | [中文](2026-10-05-candidate-window-color-emoji.zh.md)

## Problem（问题）

候选窗全部文本走 GDI（`DrawTextW`、SimSun，决策见
2026-09-19-candidate-window-gdi-rendering）。GDI 无法光栅化彩色字体
（`COLR`/`CBDT` 图层），😀、🚀 等 emoji 只能呈现单色轮廓，带 VS16 的序列
（❤️）也失去彩色形态。用户明确点名"emoji 彩色渲染（DirectWrite）"（点名
清单批三第 1 项）。设置窗口工具箱 emoji 面板（T-074 note
2026-10-01-settings-window-process-and-registration-ownership）当时记录了这
一限制；本 note 仅在**候选窗**范围内部分超驰该段落——设置面板仍保持 GDI
单色。

## Proposal（方案）

1. **触发检测（`contains_color_glyph`）**：按 UTF-16 单元扫描；高代理
   `D83C..=D83F | D940` 后接低代理 `DC00..=DFFF`（补充平面 emoji），或 BMP
   落在 `2600..=27BF | 2B00..=2BFF`（杂项符号/装饰/箭头）与 `FE0F`
   （VS16）时触发。普通汉字与英文永不触发 → GDI 路径零回归。
2. **`draw_text` 单点旁路**：行内含彩色字形时改走
   `color_text::render_color_text`，结果经 `AlphaBlend(AC_SRC_ALPHA)` 合成；
   管线内任一步失败回退原 GDI 绘制（彩色是增强，不是正确性前提）。字号
   取自绘制 DC 当前字体（`GetCurrentObject(OBJ_FONT)` + `GetObjectW`，负
   `lfHeight` ≈ DWrite em 尺寸），保证彩色行与 GDI 行度量一致。
3. **渲染管线（纯软件，无 GPU 依赖）**：
   `DWriteCreateFactory(ISOLATED)` → `IDWriteFactory::CreateTextFormat`
   （"Segoe UI"，16.0，字体回退自动命中 Segoe UI Emoji）→
   `CreateTextLayout`（每次按行覆盖字号 `SetFontSize`）→
   `D3D11CreateDevice(D3D_DRIVER_TYPE_WARP, BGRA_SUPPORT)` → DXGI 设备 →
   `D2D1CreateFactory → CreateDevice → CreateDeviceContext`
   （`ID2D1DeviceContext::DrawTextLayout`——D2D 1.1 重载默认启用彩色字形）
   目标为 D3D11 渲染目标纹理上的 `B8G8R8A8` 预乘位图 → `CopyResource` 到
   staging 纹理 → `Map` 回读预乘 BGRA → 调用方
   `AlphaBlend(AC_SRC_ALPHA)` 合成。
4. **缓存**：DWrite 工厂、文本格式、D3D11 WARP 设备/上下文与 D2D1 设备上
   下文以进程级 `OnceLock<Option<SharedCtx>>` 单例缓存；布局/目标位图/画刷
   每次绘制重建。WARP（软件）驱动使 TSF 宿主无 GPU/交换链依赖，无头会话
   可用。
5. **windows 0.61 API 备忘**（后续扩展的审计线索）：`Win32_Graphics_Direct2D_Common`
   与 `Win32_Graphics_Dxgi_Common` 是独立 feature；`DWriteCreateFactory`/
   `D2D1CreateFactory` 是泛型 1/2 参函数返回 `Result<T>`；
   `CreateTextFormat` 7 参；`ID2D1DeviceContext::DrawTextLayout`（4 参重载）
   接收 `windows_numerics::Vector2`（依赖 `windows-numerics 0.2`，windows
   自身使用的同源类型）且返回 `()`；`EndDraw(None, None)` 返回
   `Result<()>`；`CreateDIBSection` 返回 `Result<HBITMAP>`；
   `Map`/`Unmap` 需先把资源 cast 成 `ID3D11Resource`。
6. **演示支持**：`candidate-demo --emoji` 注入一行 emoji 候选项便于截图；
   默认输出（基线截图）内容不变。

## Alternatives considered（备选方案）

- **`ID2D1RenderTarget::DrawTextLayout` + `D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT`**：
  经典 RT 重载的 flag 只是"尝试彩色"提示，对 `COLR` 字体实测仍输出单色；
  正确契约是 D2D 1.1 设备上下文重载（彩色默认开）→ 采用上下文重载。
- **`EndPath`/`GetPath` 轮廓提取**：轮廓无法恢复分层彩色数据 → 放弃。
- **D2D 渲到预乘位图后 `BitBlt`**：`BitBlt` 忽略 alpha（透明像素变黑）→
  `AlphaBlend(AC_SRC_ALPHA)` + 32bpp 预乘 DIB 才是正确合成器。
- **WIC（`IWICBitmap`）光栅化**：无 DirectWrite 文本布局 → 放弃。
- **每次绘制新建 `DWriteCreateFactory`**：对象创建昂贵且候选窗绘制路径
  高频 → 进程级单例缓存。

## Acceptance criteria（验收标准）

- `cargo test --workspace` 全绿（新测试：触发范围、混合文本、尽力而为的真
  实渲染自检——断言存在 alpha>0 的不透明像素）。
- `fmt --check`/`clippy --all-targets -- -D warnings`/`git diff --check` 干净。
- 宿主截图留档 `data/artifacts/t101-shots/`：浅/深 × 基线/emoji 四张；emoji
  组须与基线组出现明显不同配色（已验证：基线 5/6 色 vs emoji 42/43 色，
  且出现 emoji 专属的天蓝/粉色）。
- demo `--emoji` 存在且默认（非 emoji）输出内容与 T-101 前基线一致（基线
  行无回归残留）。
- VM 交互验收（真实 TSF 栈、敲出含 emoji 的候选行）与其余批次项一起留在
  验收欠账。

## Risks（风险）

- **回退正确性**：任何失败（缺 WARP、缺 Segoe UI Emoji、设备重建）返回
  `None` 走 GDI——候选行保持可读但单色；不 panic（全部结果经 `Option` 内
  `?` 展开）。
- **基线/度量漂移**：DWrite em 尺寸近似 GDI `-lfHeight`；通过读活字体的方
  式对齐度量，但逐像素对齐不保证——每行矩形独占区域，可接受。
- **文本溢出**：DWrite 路径不作 `DT_END_ELLIPSIS`；长彩色文本可能溢出行矩
  形而非省略。当前注入行短；候选行宽度由词典/引擎封顶，合成严格限于
  `rect` 内。
- **`AlphaBlend` 可用性**：`msimg32` 在全部受支持 Windows 上存在；调用
  结果忽略（fail-soft）。
- **单帧成本**：彩色行的布局 + WARP 纹理回读远超 GDI，但因只有 emoji 行走
  此路径且候选窗本就整帧双缓冲，可接受。
