//! UTF-16 转换：Win32 宽字符 API 的统一入口。

/// 把 Rust 字符串转为以 `NUL` 结尾的 UTF-16 序列。
///
/// Win32 的 `W` 系列 API 要求以 `NUL` 结尾；非 BMP 字符（emoji 等）由 `encode_utf16`
/// 产生代理对，长度与 `chars().count()` 不同，调用方不得据此推算字符数。
#[must_use]
pub fn to_utf16(text: &str) -> Vec<u16> {
    let mut wide: Vec<u16> = text.encode_utf16().collect();
    wide.push(0);
    wide
}

#[cfg(test)]
mod tests {
    use super::to_utf16;

    #[test]
    fn 转换以空字符结尾() {
        assert_eq!(to_utf16("竹叶"), vec![0x7AF9, 0x53F6, 0]);
        assert_eq!(to_utf16(""), vec![0]);
    }

    #[test]
    fn 非bmp字符产生代理对() {
        // emoji 是补充平面字符：一个 char 对应两个 u16，加上结尾 NUL 共三个单元。
        let wide = to_utf16("😄");
        assert_eq!(wide.len(), 3);
        assert_eq!(wide[2], 0);
    }
}
