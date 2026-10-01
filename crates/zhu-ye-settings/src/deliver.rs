//! 工具箱面板的字符投递：优先真上屏，失败回退剪贴板。
//!
//! 面板是**非激活**窗口（`WS_EX_NOACTIVATE`），点击它不会改变前台窗口，因此
//! `SendInput` 送出的按键落在用户原本的目标应用上——这是"选中即上屏"（FR-040）能成立的
//! 前提。
//!
//! `SendInput` 会被 UIPI 拒绝（前台窗口属于提权进程而本进程未提权）。此时回退剪贴板，
//! 让用户按 Ctrl+V 自行粘贴，而不是静默失败。

use std::mem::size_of;

use windows::Win32::Foundation::{GlobalFree, HANDLE};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE,
    VIRTUAL_KEY,
};

/// `CF_UNICODETEXT` 的数值。windows-rs 把该常量放在 `Win32::System::Ole` 下，
/// 为它单独打开 OLE/COM 依赖不值得，这里按剪贴板格式的稳定取值直接使用。
const CF_UNICODETEXT_FORMAT: u16 = 13;

/// 投递结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delivery {
    /// 已送进前台窗口（真上屏）。
    Typed,
    /// `SendInput` 被拒绝，字符已放入剪贴板。
    Copied,
    /// 两条路都失败。
    Failed,
}

impl Delivery {
    /// 供界面显示的说明；`Typed` 无提示。
    #[must_use]
    pub const fn hint(self) -> Option<&'static str> {
        match self {
            Self::Typed => None,
            Self::Copied => {
                Some("无法直接上屏（目标窗口权限更高），已复制到剪贴板，请按 Ctrl+V 粘贴")
            }
            Self::Failed => Some("上屏与复制都失败"),
        }
    }
}

/// 把文本投递给当前前台窗口。
#[must_use]
pub fn deliver_text(text: &str) -> Delivery {
    if send_unicode(text) {
        return Delivery::Typed;
    }
    if copy_to_clipboard(text) {
        return Delivery::Copied;
    }
    Delivery::Failed
}

/// 逐 UTF-16 单元发送；补充平面字符（emoji）按代理对分两次发送，这是
/// `KEYEVENTF_UNICODE` 的既定用法。
fn send_unicode(text: &str) -> bool {
    let inputs = key_sequence(text);
    if inputs.is_empty() {
        return false;
    }
    let expected = u32::try_from(inputs.len()).unwrap_or(0);
    let sent = unsafe { SendInput(&inputs, i32::try_from(size_of::<INPUT>()).unwrap_or(0)) };
    sent == expected
}

/// 一个字符对应的按下/抬起序列。
fn key_sequence(text: &str) -> Vec<INPUT> {
    let mut inputs = Vec::new();
    for unit in text.encode_utf16() {
        // 无对应虚拟键的字符用 `wVk = 0` + `KEYEVENTF_UNICODE` 以扫描码投递。
        for release in [false, true] {
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(0),
                        wScan: unit,
                        dwFlags: if release {
                            KEYEVENTF_UNICODE | KEYEVENTF_KEYUP
                        } else {
                            KEYEVENTF_UNICODE
                        },
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            });
        }
    }
    inputs
}

fn copy_to_clipboard(text: &str) -> bool {
    unsafe {
        if OpenClipboard(None).is_err() {
            return false;
        }
        let copied = write_clipboard(text);
        let _ = CloseClipboard();
        copied
    }
}

unsafe fn write_clipboard(text: &str) -> bool {
    unsafe {
        if EmptyClipboard().is_err() {
            return false;
        }
        let mut wide: Vec<u16> = text.encode_utf16().collect();
        wide.push(0);
        let bytes = wide.len() * size_of::<u16>();
        let Ok(handle) = GlobalAlloc(GMEM_MOVEABLE, bytes) else {
            return false;
        };
        let target = GlobalLock(handle);
        if target.is_null() {
            let _ = GlobalFree(Some(handle));
            return false;
        }
        std::ptr::copy_nonoverlapping(wide.as_ptr().cast::<u8>(), target.cast::<u8>(), bytes);
        let _ = GlobalUnlock(handle);
        // 成功后所有权转移给系统，不能再释放；失败时句柄仍归本进程。
        if SetClipboardData(u32::from(CF_UNICODETEXT_FORMAT), Some(HANDLE(handle.0))).is_err() {
            let _ = GlobalFree(Some(handle));
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::{key_sequence, Delivery};
    use windows::Win32::UI::Input::KeyboardAndMouse::{KEYEVENTF_KEYUP, KEYEVENTF_UNICODE};

    #[test]
    fn 每个utf16单元产生一次按下与抬起() {
        let inputs = key_sequence("ab");
        assert_eq!(inputs.len(), 4);
        let scan = |index: usize| unsafe { inputs[index].Anonymous.ki.wScan };
        assert_eq!(scan(0), u16::from(b'a'));
        assert_eq!(scan(1), u16::from(b'a'));
        assert_eq!(scan(2), u16::from(b'b'));
        assert_eq!(scan(3), u16::from(b'b'));
    }

    #[test]
    fn 同一单元按下在前抬起在后且标志正确() {
        let inputs = key_sequence("a");
        let flags = |index: usize| unsafe { inputs[index].Anonymous.ki.dwFlags };
        let down = flags(0);
        let up = flags(1);
        assert!(down.contains(KEYEVENTF_UNICODE));
        assert!(!down.contains(KEYEVENTF_KEYUP), "第一次是按下");
        assert!(up.contains(KEYEVENTF_UNICODE));
        assert!(up.contains(KEYEVENTF_KEYUP), "第二次是抬起");
        // 按下与抬起必须指向同一扫描码。
        let scans = unsafe { (inputs[0].Anonymous.ki.wScan, inputs[1].Anonymous.ki.wScan) };
        assert_eq!(scans.0, scans.1);
    }

    #[test]
    fn emoji代理对分两个单元发送() {
        // ❤️ = U+2764 + U+FE0F，两个 UTF-16 单元。
        let inputs = key_sequence("❤️");
        assert_eq!(inputs.len(), 4, "两个单元各一对按下抬起");
        let scans: Vec<u16> = (0..4)
            .map(|index| unsafe { inputs[index].Anonymous.ki.wScan })
            .collect();
        assert_eq!(scans, vec![0x2764, 0x2764, 0xFE0F, 0xFE0F]);
    }

    #[test]
    fn 空文本不产生输入() {
        assert!(key_sequence("").is_empty());
    }

    #[test]
    fn 投递结果的提示文案() {
        assert_eq!(Delivery::Typed.hint(), None, "上屏成功无需提示");
        assert!(Delivery::Copied.hint().unwrap().contains("Ctrl+V"));
        assert!(Delivery::Failed.hint().is_some());
    }
}
