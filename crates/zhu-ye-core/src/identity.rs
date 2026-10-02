//! TSF 组件身份契约常量（Rust 侧单一主源）。
//!
//! 这些值会字面写进系统注册表（TIP/语言配置/类别树）与安装脚本，是"产品身份"：
//! 设置窗口的注册表修复、TSF DLL 的注册与安装脚本必须写同一个值。PowerShell 侧
//! 在 `scripts/ime-identity.ps1` 双份维护，由 `scripts/verify-tsf-identity.ps1`
//! 逐字交叉比对（D-42）。第八期 T-081 把主源从 `zhu_ye_ime::tsf` 移入本模块，
//! 使设置窗口脱离对 TSF 侧代码（`rlib`）的依赖后仍能引用身份值。
//!
//! GUID 以 `u128` 字面量表示（与 `windows::core::GUID::from_u128` 的输入一致，
//! 大端文本观感：`0xE54D6682_8650_40E7_A9EE_6FD1137849AE`），本 crate 不依赖
//! Windows 类型；由消费方按需转换（如 `windows::core::GUID::from_u128`）。

/// 输入法 TIP 的 CLSID，与 `scripts/ime-identity.ps1` 中的 `TipClsid` 保持一致。
pub const CLSID_ZHU_YE_TIP: u128 = 0xE54D6682_8650_40E7_A9EE_6FD1137849AE;

/// 安装/便携包随带的 v2 词典文件名，与 `scripts/ime-identity.ps1` 保持一致。
pub const DICTIONARY_FILE_NAME: &str = "dictionary.zyct";

/// 简体中文（zh-CN，LCID 0x0804）下的语言配置文件 GUID，
/// 与 `scripts/ime-identity.ps1` 中的 `ProfileGuid` 保持一致。
pub const PROFILE_GUID_ZHU_YE: u128 = 0x6315FE74_92C3_439B_8CDF_FDB6E43EDAF1;

/// 键盘输入处理器（TIP）类别的 TFCAT GUID，与 `scripts/ime-identity.ps1` 中的
/// `KeyboardCategoryGuid` 保持一致；二级修复重建 `Category\Category` 与
/// `Category\Item` 两棵子树时使用。
pub const TFCAT_ZHU_YE_KEYBOARD: u128 = 0x34745C63_B2F0_4784_8B67_5E12C8701A31;

/// TSF 语言配置文件注册路径中的语言段（简体中文 zh-CN），与
/// `scripts/ime-identity.ps1` 中的 `LanguageIdHex` 保持一致。
pub const TSF_LANGUAGE_ID_HEX: &str = "0x00000804";

/// 安装目录相对 `Program Files` 的路径段，与 `scripts/ime-identity.ps1` 的
/// `Get-TsfInstallDir` 保持一致；二级修复在读不到 `InProcServer32` 时用它在
/// 自身安装目录下推导 DLL 搜索位置。
pub const TSF_INSTALL_DIR_RELATIVE: &str = "ai-zhu-ye-ime\\tsf";

/// GUID 的标准大括号文本（与 `ime-identity.ps1` 常量格式一致，全大写）。
#[must_use]
pub fn guid_text(value: u128) -> String {
    format!(
        "{{{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}}}",
        (value >> 96) as u32,
        (value >> 80) as u16,
        (value >> 64) as u16,
        (value >> 56) as u8,
        (value >> 48) as u8,
        (value >> 40) as u8,
        (value >> 32) as u8,
        (value >> 24) as u8,
        (value >> 16) as u8,
        (value >> 8) as u8,
        value as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        guid_text, CLSID_ZHU_YE_TIP, DICTIONARY_FILE_NAME, PROFILE_GUID_ZHU_YE,
        TFCAT_ZHU_YE_KEYBOARD, TSF_INSTALL_DIR_RELATIVE, TSF_LANGUAGE_ID_HEX,
    };

    #[test]
    fn guid文本与安装脚本值逐字一致() {
        // 与 scripts/ime-identity.ps1 的 $script:TsfIdentity 一致（D-42 双份维护的
        // Rust 侧锚点，verify-tsf-identity.ps1 另行交叉比对）。
        assert_eq!(
            guid_text(CLSID_ZHU_YE_TIP),
            "{E54D6682-8650-40E7-A9EE-6FD1137849AE}"
        );
        assert_eq!(
            guid_text(PROFILE_GUID_ZHU_YE),
            "{6315FE74-92C3-439B-8CDF-FDB6E43EDAF1}"
        );
        assert_eq!(
            guid_text(TFCAT_ZHU_YE_KEYBOARD),
            "{34745C63-B2F0-4784-8B67-5E12C8701A31}"
        );
        // u128 字面量的大端观感与 `windows::core::GUID::from_u128` 的输入一致：
        // 高位 32 位为 data1、次 16 位为 data2、再 16 位为 data3、低 64 位为 data4。
        assert_eq!((CLSID_ZHU_YE_TIP >> 96) as u32, 0xE54D_6682);
        assert_eq!((CLSID_ZHU_YE_TIP >> 80) as u16, 0x8650);
        assert_eq!((CLSID_ZHU_YE_TIP >> 64) as u16, 0x40E7);
        assert_eq!(CLSID_ZHU_YE_TIP as u64, 0xA9EE_6FD1_1378_49AE);
    }

    #[test]
    fn 字符串常量与安装脚本值一致() {
        assert_eq!(DICTIONARY_FILE_NAME, "dictionary.zyct");
        assert_eq!(TSF_LANGUAGE_ID_HEX, "0x00000804");
        assert_eq!(TSF_INSTALL_DIR_RELATIVE, "ai-zhu-ye-ime\\tsf");
    }
}
