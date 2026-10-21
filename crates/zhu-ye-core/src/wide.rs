//! NUL 终止的 UTF-16 字符串（Win32 宽字符 API 的统一入口）。
//!
//! # 为什么需要这个模块
//!
//! Win32 的 `W` 系列 API（`FindWindowW`、`CreateMutexW`、`RegisterClassW`、
//! `LoadLibraryW` …）都要求传入**以 NUL 结尾**的 UTF-16 序列。直接用
//! `str::encode_utf16()` 得到的序列**没有**尾部 NUL，传给 Win32 会让它继续
//! 往后读内存直到碰到一个 0 —— 于是：
//!
//! - `FindWindowW` **找不到**本该找到的窗口；
//! - `RegisterClassW` 注册出**带内存垃圾**的类名。
//!
//! 这两类缺陷在本项目已多次发生（同一个陷阱踩了三次），共同特征是
//! 「**能编译、能测试、真机却不工作**」—— 单元测试测的是纯函数，
//! 不会发现传给 Win32 的字符串没终止。
//!
//! # 用法
//!
//! ```ignore
//! let name = zhu_ye_core::wide::WideString::new("ZhuYeTrayHostWindow");
//! let class = WNDCLASSW { lpszClassName: PCWSTR(name.as_ptr()), .. };
//! ```
//!
//! `WideString` **只能由 `&str` 构造**，因此「忘了补 NUL」在类型层面
//! 不可能发生。确需不含 NUL 的序列（如 GDI 绘制的文本缓冲）时用
//! [`WideString::units`]，但那不能喂给需要终止符的 API。

/// NUL 终止的 UTF-16 字符串。
///
/// 构造时自动追加尾部 NUL；`as_ptr()` 可直接作为 `PCWSTR` 传给 Win32。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WideString {
    /// 以 NUL 结尾的 UTF-16 序列。
    units: Vec<u16>,
}

impl WideString {
    /// 由 Rust 字符串构造，追加尾部 NUL。
    #[must_use]
    pub fn new(text: &str) -> Self {
        let mut units: Vec<u16> = text.encode_utf16().collect();
        units.push(0);
        Self { units }
    }

    /// NUL 终止序列的切片（以 `PCWSTR` 传给 Win32 时用 `as_ptr()`）。
    #[must_use]
    pub fn units(&self) -> &[u16] {
        &self.units
    }

    /// 序列指针，可直接作为 `PCWSTR` 的来源。
    #[must_use]
    pub fn as_ptr(&self) -> *const u16 {
        self.units.as_ptr()
    }

    /// 原始文本（不含尾部 NUL）。
    #[must_use]
    pub fn as_str(&self) -> String {
        // 去掉尾部 NUL 再解码。
        let end = self.units.len().saturating_sub(1);
        String::from_utf16_lossy(&self.units[..end])
    }
}

#[cfg(test)]
mod tests {
    use super::WideString;

    /// 构造出的序列一定以 NUL 结尾（Win32 的硬性要求）。
    #[test]
    fn 序列以空字符结尾() {
        let wide = WideString::new("ZhuYeTrayHostWindow");
        assert_eq!(wide.units().last(), Some(&0));
        assert_eq!(wide.units().len(), "ZhuYeTrayHostWindow".len() + 1);
    }

    /// 空串也要有一个 NUL（否则指针指向空内存）。
    #[test]
    fn 空串也有终止符() {
        let wide = WideString::new("");
        assert_eq!(wide.units(), &[0]);
    }

    /// 往返：原始文本可还原。
    #[test]
    fn 往返保留原文() {
        let text = "竹叶输入法";
        let wide = WideString::new(text);
        assert_eq!(wide.as_str(), text);
    }

    /// 非 BMP 字符（emoji）按代理对处理，长度与字符数不同。
    #[test]
    fn 非bmp字符产生代理对() {
        let wide = WideString::new("😄");
        // 代理对 2 个单元 + 终止 NUL。
        assert_eq!(wide.units().len(), 3);
        assert_eq!(wide.units()[2], 0);
    }
}
