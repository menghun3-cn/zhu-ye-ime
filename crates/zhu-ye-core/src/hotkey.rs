//! 工具箱面板呼出热键的解析与描述（第十四期 FR-070，T-151）。
//!
//! 纯逻辑、无 Win32 API 依赖（仅使用 Win32 的数值常量，与 `identity` 的 u128 字面量
//! 先例一致）：把 `config.json` 的 `panel_hotkey` 字符串（如 `ctrl+alt+e`）解析为
//! 修饰键位（`MOD_*` 数值）+ 虚拟键码（VK 数值），供常驻托盘进程 `RegisterHotKey`
//! 使用。解析失败/缺失时调用方回退 `DEFAULT_PANEL_HOTKEY` 并在日志记录。

/// 修饰键位（与 Win32 `MOD_*` 数值一致；此处仅作数值常量，不调用 API）。
pub const MOD_ALT: u32 = 0x0001;
pub const MOD_CONTROL: u32 = 0x0002;
pub const MOD_SHIFT: u32 = 0x0004;
pub const MOD_WIN: u32 = 0x0008;

/// 内建默认呼出热键（`Ctrl+Alt+E`）。
pub const DEFAULT_PANEL_HOTKEY: &str = "ctrl+alt+e";

/// 解析后的热键。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HotkeySpec {
    /// 修饰键位（`MOD_*` 按位或）。
    pub modifiers: u32,
    /// 虚拟键码（Win32 VK 数值；`'A'..='Z'`、`'0'..='9'`、`F1..=F24`）。
    pub vk: u16,
}

impl HotkeySpec {
    /// 人类可读描述（如 `Ctrl+Alt+E`）。
    #[must_use]
    pub fn display(self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if self.modifiers & MOD_CONTROL != 0 {
            parts.push("Ctrl".to_owned());
        }
        if self.modifiers & MOD_ALT != 0 {
            parts.push("Alt".to_owned());
        }
        if self.modifiers & MOD_SHIFT != 0 {
            parts.push("Shift".to_owned());
        }
        if self.modifiers & MOD_WIN != 0 {
            parts.push("Win".to_owned());
        }
        parts.push(vk_name(self.vk));
        parts.join("+")
    }
}

/// 把键名映射为 VK 数值；无法识别时返回 `None`。
fn vk_of(name: &str) -> Option<u16> {
    let upper = name.to_ascii_uppercase();
    if upper.len() == 1 {
        let byte = upper.as_bytes()[0];
        if byte.is_ascii_uppercase() {
            return Some(byte as u16); // 'A'..='Z' 的 VK 与 ASCII 码一致。
        }
        if byte.is_ascii_digit() {
            return Some(byte as u16); // '0'..='9' 的 VK 与 ASCII 码一致。
        }
        return None;
    }
    if let Some(n) = upper.strip_prefix("F") {
        if let Ok(value) = n.parse::<u16>() {
            if (1..=24).contains(&value) {
                return Some(0x70 + value - 1); // VK_F1..VK_F24。
            }
        }
    }
    None
}

/// VK 数值转键名（与 `vk_of` 互逆的子集）。
fn vk_name(vk: u16) -> String {
    if (0x41u16..=0x5Au16).contains(&vk) || (0x30u16..=0x39u16).contains(&vk) {
        // 'A'..='Z' 与 '0'..='9' 的单字符键。
        return char::from(vk as u8).to_string();
    }
    if let Some(id) = vk.checked_sub(0x70) {
        if id < 24 {
            return [
                "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12", "F13",
                "F14", "F15", "F16", "F17", "F18", "F19", "F20", "F21", "F22", "F23", "F24",
            ][id as usize]
                .to_owned();
        }
    }
    "?".to_owned()
}

/// 解析热键字符串：`ctrl+alt+e` 形式（修饰键任意顺序、小写；键单个字母/数字/F1..F24）。
///
/// 解析失败返回 `None`（调用方回退默认并记日志）。要求**至少一个修饰键**，避免
/// 裸键注册污染其他程序（S-30 防御口径）。
#[must_use]
pub fn parse_hotkey(spec: &str) -> Option<HotkeySpec> {
    let spec = spec.trim();
    if spec.is_empty() {
        return None;
    }
    let mut modifiers = 0u32;
    let mut key: Option<String> = None;
    for part in spec.split('+') {
        let part = part.trim().to_ascii_lowercase();
        match part.as_str() {
            "ctrl" | "control" => modifiers |= MOD_CONTROL,
            "alt" => modifiers |= MOD_ALT,
            "shift" => modifiers |= MOD_SHIFT,
            "win" | "super" => modifiers |= MOD_WIN,
            "" => return None,
            other => {
                if key.is_some() {
                    return None; // 出现多个键名。
                }
                key = Some(other.to_owned());
            }
        }
    }
    let vk = vk_of(key.as_deref()?)?;
    if modifiers == 0 {
        return None; // 裸键不注册（防御：避免吞掉系统/其他程序热键）。
    }
    Some(HotkeySpec { modifiers, vk })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 默认热键可解析且描述为ctrl_alt_e() {
        let spec = parse_hotkey(DEFAULT_PANEL_HOTKEY).expect("默认热键必须可解析");
        assert_eq!(spec.modifiers, MOD_CONTROL | MOD_ALT);
        assert_eq!(spec.vk, 0x45); // 'E'
        assert_eq!(spec.display(), "Ctrl+Alt+E");
    }

    #[test]
    fn 修饰键任意顺序与大小写均解析() {
        let a = parse_hotkey("alt+ctrl+E").expect("大小写与顺序不敏感");
        let b = parse_hotkey("e+ctrl+alt").expect("顺序任意");
        assert_eq!(a, b);
        let c = parse_hotkey("shift+f1").expect("shift+f1");
        assert_eq!(c.modifiers, MOD_SHIFT);
        assert_eq!(c.vk, 0x70);
        assert_eq!(c.display(), "Shift+F1");
        let d = parse_hotkey("win+9").expect("win+9");
        assert_eq!(d.modifiers, MOD_WIN);
        assert_eq!(d.vk, 0x39);
    }

    #[test]
    fn 非法输入全部拒绝() {
        assert!(parse_hotkey("").is_none(), "空字符串");
        assert!(parse_hotkey("   ").is_none(), "纯空白");
        assert!(parse_hotkey("e").is_none(), "裸键不注册");
        assert!(parse_hotkey("ctrl+").is_none(), "缺键");
        assert!(parse_hotkey("ctrl++e").is_none(), "空段");
        assert!(parse_hotkey("ctrl+alt+e+x").is_none(), "多键");
        assert!(parse_hotkey("ctrl+elephant").is_none(), "未知键");
        assert!(parse_hotkey("ctrl+f0").is_none(), "F0 越界");
        assert!(parse_hotkey("ctrl+f25").is_none(), "F25 越界");
        assert!(parse_hotkey("superdupont").is_none(), "一整个未知段");
    }

    #[test]
    fn f1_至_f24_与数字键_解析() {
        for n in 1..=24u16 {
            let spec = parse_hotkey(&format!("ctrl+f{n}")).expect("F 键");
            assert_eq!(spec.vk, 0x70 + n - 1, "F{n}");
        }
        for n in 0..=9u16 {
            let spec = parse_hotkey(&format!("ctrl+{n}")).expect("数字键");
            assert_eq!(spec.vk, 0x30 + n, "{n}");
        }
    }
}
