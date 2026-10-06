//! T-112b：把品牌"竹"字图标（assets/zhu.ico）嵌入 TSF DLL 资源。
//!
//! Win11 任务栏输入法指示器按 TIP 语言档注册的 `IconFile` 加载图标（微软拼音
//! `ResourceDll.dll`"拼"、搜狗 `SogouTSF.ime`"S"同机制），本 DLL 用 `winresource`
//! 在链接产物中注入该图标资源，`IconIndex=0` 即被 `ExtractIconEx` 取到。
//! 仅 Windows 目标需要；其他平台（crate 本身仅 Windows）直接跳过。

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let icon = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("zhu.ico");
    let mut res = winresource::WindowsResource::new();
    res.set_icon(icon.to_string_lossy().as_ref());
    if let Err(e) = res.compile() {
        eprintln!("[build.rs] winresource 嵌入品牌图标失败: {e}");
        std::process::exit(1);
    }
}
