//! 把托盘状态图标内嵌进 `zhu-ye-tray.exe` 资源：
//! 101 = 中文模式（白"中"），102 = 英文模式（白"英"）。
//! `winresource` 的 `set_icon` 只支持单枚，这里用 `set_icon_with_id`
//! 追加两枚（编译出的 .rc 行为 `101 ICON …` / `102 ICON …`）。

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    let zh = manifest.join("tray-zh.ico");
    let en = manifest.join("tray-en.ico");
    if !zh.exists() || !en.exists() {
        eprintln!("[build.rs] 托盘图标资产缺失: {zh:?} / {en:?}");
        std::process::exit(1);
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon_with_id(zh.to_string_lossy().as_ref(), "101")
        .set_icon_with_id(en.to_string_lossy().as_ref(), "102");
    if let Err(e) = res.compile() {
        eprintln!("[build.rs] winresource 嵌入托盘图标失败: {e}");
        std::process::exit(1);
    }
}
