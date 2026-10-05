//! T-074：候选窗彩色 emoji 渲染（DirectWrite 彩色字形旁路）。
//!
//! 候选窗主体仍走 GDI（`candidate_window.rs` 的 `DrawTextW`），但对**含彩色
//! 字形（emoji）的文本行**切换到 DirectWrite 布局 + D2D 1.1 软件光栅化，
//! 否则 GDI 只能渲染单色轮廓（`COLR`/`CBDT` 图层需要 DirectWrite）。
//!
//! 彩色链路：`DWriteCreateFactory → CreateTextLayout`（字体回退自动命中
//! Segoe UI Emoji 等彩色字体）→ `D3D11CreateDevice(WARP)` 软件设备 →
//! `D2D1Factory1::CreateDevice → CreateDeviceContext →
//! CreateBitmapFromDxgiSurface`（B8G8R8A8 目标位图）→ `DrawTextLayout`
//! → `CopyResource` 到 staging 纹理后 `Map` 回读像素 → 预乘 alpha →
//! 调用方经 `AlphaBlend(AC_SRC_ALPHA)` 合成回候选窗内存 DC。
//!
//! 进程期单例只缓存工厂/设备/上下文/文本格式（线程无关的 COM 对象），
//! 每次绘制重建布局与目标位图。任何一步失败都回退 GDI（彩色渲染是
//! 增强，不是正确性前提）。

use windows::core::{Interface, PCWSTR};
use windows::Win32::Foundation::HMODULE;
use windows::Win32::Graphics::Direct2D::Common::{
    D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_COLOR_F, D2D1_PIXEL_FORMAT,
};
use windows::Win32::Graphics::Direct2D::{
    D2D1CreateFactory, ID2D1Bitmap1, ID2D1Brush, ID2D1Device, ID2D1DeviceContext, ID2D1Factory1,
    ID2D1Image, ID2D1SolidColorBrush, D2D1_BITMAP_OPTIONS_TARGET, D2D1_BITMAP_PROPERTIES1,
    D2D1_DEVICE_CONTEXT_OPTIONS_NONE, D2D1_DRAW_TEXT_OPTIONS, D2D1_FACTORY_OPTIONS,
    D2D1_FACTORY_TYPE_SINGLE_THREADED,
};
use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_WARP;
use windows::Win32::Graphics::Direct3D11::{
    D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, ID3D11Resource, ID3D11Texture2D,
    D3D11_BIND_RENDER_TARGET, D3D11_CPU_ACCESS_READ, D3D11_CREATE_DEVICE_BGRA_SUPPORT,
    D3D11_MAPPED_SUBRESOURCE, D3D11_MAP_READ, D3D11_SDK_VERSION, D3D11_TEXTURE2D_DESC,
    D3D11_USAGE_DEFAULT, D3D11_USAGE_STAGING,
};
use windows::Win32::Graphics::DirectWrite::{
    DWriteCreateFactory, IDWriteFactory, IDWriteTextFormat, IDWriteTextLayout,
    DWRITE_FACTORY_TYPE_ISOLATED, DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL,
    DWRITE_FONT_WEIGHT_NORMAL, DWRITE_PARAGRAPH_ALIGNMENT_CENTER, DWRITE_TEXT_ALIGNMENT_LEADING,
    DWRITE_TEXT_RANGE,
};
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC};
use windows::Win32::Graphics::Dxgi::{IDXGIDevice, IDXGISurface};
use windows_numerics::Vector2;

use crate::candidate_ui::UiColor;

/// 判断一行文本是否需要彩色字形渲染（emoji）。
///
/// 覆盖三类触发：
/// - 补充平面表情/符号区 U+1F000–U+1FAFF（`D83C..=D83F`/`D940` 高代理区）
/// - BMP 装饰与杂项符号 U+2600–U+27BF、U+2B00–U+2BFF（☀★➜ 等，多数系统
///   以彩色字体呈现）
/// - 变体选择符 VS16（U+FE0F）：文本符号加 VS16 要求彩色呈现（❤️）
///
/// 普通汉字/英文不触发（保持 GDI 路径零回归）。按 UTF-16 单元扫描，
/// emoji 的代理对与 VS16 都是专用/代理单元，直接命中范围即可。
pub fn contains_color_glyph(text: &str) -> bool {
    let mut units = text.encode_utf16();
    while let Some(u) = units.next() {
        match u {
            // 高代理区：U+1F000 起落在 D83C..=D83F（U+1F000–U+1FFFF 高半区）
            0xD83C..=0xD83F | 0xD940 => {
                if let Some(lo) = units.next() {
                    if (0xDC00..=0xDFFF).contains(&lo) {
                        return true;
                    }
                }
            }
            // BMP 彩色符号区与 VS16
            0x2600..=0x27BF | 0x2B00..=0x2BFF | 0xFE0F => return true,
            _ => {}
        }
    }
    false
}

/// 一次彩色文本渲染的输出：宽、高与预乘 alpha 的 BGRA 像素（自上而下、
/// 行距 = width * 4）。
pub struct ColorBitmap {
    pub width: i32,
    pub height: i32,
    /// 预乘 alpha BGRA（`AlphaBlend(AC_SRC_ALPHA)` 直接用）。
    pub pixels: Vec<u8>,
}

/// 进程期单例：DWrite 工厂 + D3D11 WARP 设备 + D2D1.1 设备/上下文 +
/// 默认文本格式。创建失败则彩色路径整体禁用（回退 GDI）。
fn shared_ctx() -> Option<&'static SharedCtx> {
    static CTX: std::sync::OnceLock<Option<SharedCtx>> = std::sync::OnceLock::new();
    CTX.get_or_init(build_ctx).as_ref()
}

struct SharedCtx {
    factory: IDWriteFactory,
    text_format: IDWriteTextFormat,
    context: ID2D1DeviceContext,
    d3d_device: ID3D11Device,
    d3d_context: ID3D11DeviceContext,
}

fn build_ctx() -> Option<SharedCtx> {
    unsafe {
        // 1. DirectWrite 工厂 + 默认格式（Segoe UI 字族，字体回退自动命中
        //    Segoe UI Emoji 等彩色字体）。字号按需由布局覆盖。
        let factory: IDWriteFactory =
            match DWriteCreateFactory::<IDWriteFactory>(DWRITE_FACTORY_TYPE_ISOLATED) {
                Ok(f) => f,
                Err(_) => return None,
            };
        let mut face = [0u16; 32];
        for (slot, unit) in face
            .iter_mut()
            .zip("Segoe UI".encode_utf16().chain(Some(0)))
        {
            *slot = unit;
        }
        let text_format = match factory.CreateTextFormat(
            PCWSTR(face.as_ptr()),
            None,
            DWRITE_FONT_WEIGHT_NORMAL,
            DWRITE_FONT_STYLE_NORMAL,
            DWRITE_FONT_STRETCH_NORMAL,
            16.0,
            PCWSTR::null(),
        ) {
            Ok(f) => f,
            Err(_) => return None,
        };
        let _ = text_format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING);
        let _ = text_format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);

        // 2. D3D11 软件（WARP）设备——TSF 宿主内不依赖 GPU/交换链。
        let mut d3d_device: Option<ID3D11Device> = None;
        let mut d3d_context: Option<ID3D11DeviceContext> = None;
        D3D11CreateDevice(
            None,
            D3D_DRIVER_TYPE_WARP,
            HMODULE::default(),
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            None,
            D3D11_SDK_VERSION,
            Some(&mut d3d_device),
            None,
            Some(&mut d3d_context),
        )
        .ok()?;
        let d3d_device = d3d_device?;
        let d3d_context = d3d_context?;

        // 3. DXGI 设备 → D2D1 设备 → 设备上下文。
        let dxgi: IDXGIDevice = d3d_device.cast().ok()?;
        let d2d_factory: ID2D1Factory1 = D2D1CreateFactory(
            D2D1_FACTORY_TYPE_SINGLE_THREADED,
            Some(&D2D1_FACTORY_OPTIONS::default()),
        )
        .ok()?;
        let device: ID2D1Device = d2d_factory.CreateDevice(&dxgi).ok()?;
        let context: ID2D1DeviceContext = device
            .CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)
            .ok()?;

        Some(SharedCtx {
            factory,
            text_format,
            context,
            d3d_device,
            d3d_context,
        })
    }
}

fn d2d_color(color: UiColor) -> D2D1_COLOR_F {
    let c = color.to_colorref();
    let r = (c & 0xFF) as f32 / 255.0;
    let g = ((c >> 8) & 0xFF) as f32 / 255.0;
    let b = ((c >> 16) & 0xFF) as f32 / 255.0;
    D2D1_COLOR_F { r, g, b, a: 1.0 }
}

/// 渲染含彩色字形的文本为预乘 BGRA 位图。失败返回 None（调用方回退 GDI）。
///
/// `font_size` 为逻辑字号（与候选行 GDI 字体一致）；`color` 与 GDI 文本色
/// 一致，仅作用于非彩色字形（彩色 emoji 用字体内置颜色）。
pub fn render_color_text(
    text: &str,
    width: i32,
    height: i32,
    font_size: f32,
    color: UiColor,
) -> Option<ColorBitmap> {
    let ctx = shared_ctx()?;
    if text.is_empty() || width <= 0 || height <= 0 {
        return None;
    }
    let w = width as u32;
    let h = height as u32;

    unsafe {
        let wide: Vec<u16> = text.encode_utf16().collect();
        let text_len = wide.len() as u32;
        if text_len == 0 {
            return None;
        }

        let layout: IDWriteTextLayout = ctx
            .factory
            .CreateTextLayout(&wide, &ctx.text_format, width as f32, height as f32)
            .ok()?;
        let _ = layout.SetFontSize(
            font_size,
            DWRITE_TEXT_RANGE {
                startPosition: 0,
                length: text_len,
            },
        );

        // 目标纹理（默认用法 + 渲染目标绑定），软件 WARP 可拷贝回读。
        let tex_desc = D3D11_TEXTURE2D_DESC {
            Width: w,
            Height: h,
            MipLevels: 1,
            ArraySize: 1,
            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_RENDER_TARGET.0 as u32,
            CPUAccessFlags: 0,
            MiscFlags: 0,
        };
        let mut texture: Option<ID3D11Texture2D> = None;
        ctx.d3d_device
            .CreateTexture2D(&tex_desc, None, Some(&mut texture))
            .ok()?;
        let texture = texture?;
        let surface: IDXGISurface = texture.cast().ok()?;

        let props = D2D1_BITMAP_PROPERTIES1 {
            pixelFormat: D2D1_PIXEL_FORMAT {
                format: DXGI_FORMAT_B8G8R8A8_UNORM,
                alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
            },
            dpiX: 0.0,
            dpiY: 0.0,
            bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET,
            colorContext: std::mem::ManuallyDrop::new(None),
        };
        let target: ID2D1Bitmap1 = ctx
            .context
            .CreateBitmapFromDxgiSurface(&surface, Some(&props))
            .ok()?;
        let target_image: ID2D1Image = target.cast().ok()?;
        ctx.context.SetTarget(&target_image);

        let brush: ID2D1SolidColorBrush = ctx
            .context
            .CreateSolidColorBrush(&d2d_color(color), None)
            .ok()?;
        let brush: ID2D1Brush = brush.cast().ok()?;

        ctx.context.BeginDraw();
        ctx.context.DrawTextLayout(
            Vector2 { X: 0.0, Y: 0.0 },
            &layout,
            &brush,
            D2D1_DRAW_TEXT_OPTIONS(0),
        );
        let end = ctx.context.EndDraw(None, None);
        ctx.context.SetTarget(None);
        end.ok()?;

        // staging 纹理拷贝回读像素。
        let stage_desc = D3D11_TEXTURE2D_DESC {
            Usage: D3D11_USAGE_STAGING,
            CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
            ..tex_desc
        };
        let mut staging: Option<ID3D11Texture2D> = None;
        ctx.d3d_device
            .CreateTexture2D(&stage_desc, None, Some(&mut staging))
            .ok()?;
        let staging = staging?;
        let staging_res: ID3D11Resource = staging.cast().ok()?;
        let texture_res: ID3D11Resource = texture.cast().ok()?;
        ctx.d3d_context.CopyResource(&staging_res, &texture_res);

        let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
        ctx.d3d_context
            .Map(Some(&staging_res), 0, D3D11_MAP_READ, 0, Some(&mut mapped))
            .ok()?;
        let stride = mapped.RowPitch as usize;
        let src = mapped.pData as *const u8;
        let row_bytes = (w as usize) * 4;
        let mut pixels = vec![0u8; (h as usize) * row_bytes];
        if stride == row_bytes {
            std::ptr::copy_nonoverlapping(src, pixels.as_mut_ptr(), pixels.len());
        } else {
            for y in 0..h as usize {
                std::ptr::copy_nonoverlapping(
                    src.add(y * stride),
                    pixels.as_mut_ptr().add(y * row_bytes),
                    row_bytes,
                );
            }
        }
        ctx.d3d_context.Unmap(Some(&staging_res), 0);

        Some(ColorBitmap {
            width,
            height,
            pixels,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{contains_color_glyph, render_color_text};
    use crate::candidate_ui::UiColor;

    #[test]
    fn 普通汉字与英文不触发彩色路径() {
        assert!(!contains_color_glyph("你好世界"));
        assert!(!contains_color_glyph("hello world"));
        assert!(!contains_color_glyph("竹叶输入法 abc 123"));
    }

    #[test]
    fn 补充平面表情触发() {
        assert!(contains_color_glyph("😀"));
        assert!(contains_color_glyph("👍🏼"));
        assert!(contains_color_glyph("🚀"));
        assert!(contains_color_glyph("🎉"));
        assert!(contains_color_glyph("🧑‍💻")); // ZWJ 序列（主字素为补充平面）
    }

    #[test]
    fn bmp装饰符号与vs16触发() {
        assert!(contains_color_glyph("☀"));
        assert!(contains_color_glyph("★"));
        assert!(contains_color_glyph("❤️")); // VS16
        assert!(contains_color_glyph("❌"));
        assert!(contains_color_glyph("➜"));
    }

    #[test]
    fn 混合文本触发() {
        assert!(contains_color_glyph("今天天气😀不错"));
        assert!(contains_color_glyph("完成 🚀 发布"));
    }

    /// 真实 D2D/WARP 链路自检：环境支持时渲染 emoji 并验证有非透明像素
    /// （预乘 alpha）。环境不支持（如无 WARP）时返回 None 视为跳过——彩色
    /// 渲染是增强，GDI 回退仍保证正确性。
    #[test]
    fn 彩色渲染链路自检() {
        if let Some(bmp) = render_color_text("😀", 64, 64, 22.0, UiColor::rgb(0x1E, 0x88, 0xE5)) {
            assert_eq!(bmp.width, 64);
            assert_eq!(bmp.height, 64);
            assert_eq!(bmp.pixels.len(), 64 * 64 * 4);
            assert!(
                bmp.pixels.iter().skip(3).step_by(4).any(|&a| a > 0),
                "渲染结果应为预乘 alpha 位图且含不透明字形像素"
            );
        }
    }
}
