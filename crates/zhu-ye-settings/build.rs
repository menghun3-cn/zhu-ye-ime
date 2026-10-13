//! T-142：把品牌"竹"字图标（zhu-ye-ime/assets/zhu.ico）嵌入设置窗口 EXE 资源
//! （resource id 1）。窗口类注册时 `LoadIconW(hInstance, MAKEINTRESOURCE(1))`
//! 取用——标题栏左侧与任务栏显示竹叶 LOGO。与 TSF DLL（build.rs 同款
//! winresource 注入）共用同一份图标资产（跨 crate 相对引用，避免双份拷贝漂移）。
//! 仅 Windows 目标需要；设置 crate 本身仅 Windows，非 Windows 直接跳过。

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let icon = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("zhu-ye-ime")
        .join("assets")
        .join("zhu.ico");
    if !icon.exists() {
        eprintln!("[build.rs] 设置窗口图标资产缺失: {icon:?}");
        std::process::exit(1);
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon(icon.to_string_lossy().as_ref());
    if let Err(e) = res.compile() {
        eprintln!("[build.rs] winresource 嵌入设置窗口品牌图标失败: {e}");
        std::process::exit(1);
    }
}
