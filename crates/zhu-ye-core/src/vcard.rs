//! vCard（.vcf）解析器（场景 9 通讯录，FR-036）。
//!
//! 本模块是**纯算法、确定、可测**的 vCard 3.0 子集解析器（RFC 2426），
//! 覆盖本项目数据契约所需的最小面：
//!
//! - 多段解析：`BEGIN:VCARD` … `END:VCARD`，段外内容忽略；
//! - 行折叠：`\r\n` 后跟空格或制表符视为续行（RFC 2426 §2.4.1）；
//! - 转义：`\\` `\,` `\;` `\n`（RFC 2426 §2.1.1）；
//! - 姓名提取：`FN` 全名优先，无 `FN` 时用 `N` 结构姓名字段拼接（中文顺序：姓+名）；
//! - 其余字段（`ORG`/`TEL`/`EMAIL`/`ADR` 等）忽略不报错（D-20 仅姓名建索引）；
//! - 未知属性、组前缀（`item1.FN`）、显式 `CHARSET` 参数按规则处理；
//! - 非 UTF-8 的 `CHARSET`（如 GB2312）不转码、跳过该卡（首批只保证 UTF-8，
//!   见 docs/通讯录设计.md §8 风险记录）。
//!
//! 解析结果只含姓名，`keys`（拼音键集合）由 contacts 索引构建（T-071-2）填充。

use std::fmt;

/// 单条联系人解析结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VCardContact {
    /// 联系人姓名（`FN` 或 `N` 拼接结果；空名卡不产出）。
    pub name: String,
    /// 检索键集合：解析阶段为空，由 `build_contact_index` 的注音回调填充。
    pub keys: Vec<String>,
}

/// vCard 解析错误（结构损坏才报错；语义空白如空名卡按跳过处理）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VCardError {
    /// `END:VCARD` 之前文件结束（卡未闭合）。
    UnclosedCard(usize),
    /// 出现 `END:VCARD` 但当前没有打开的卡。
    OrphanEnd(usize),
    /// 行缺少 `:` 分隔符（既不是属性也不是版本/边界行）。
    MalformedLine(usize),
}

impl fmt::Display for VCardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnclosedCard(line) => write!(f, "vCard 未闭合（第 {line} 行附近）"),
            Self::OrphanEnd(line) => write!(f, "多余 END:VCARD（第 {line} 行）"),
            Self::MalformedLine(line) => write!(f, "vCard 行格式错误（第 {line} 行）"),
        }
    }
}

impl std::error::Error for VCardError {}

type VCardResult<T> = std::result::Result<T, VCardError>;

/// 解析 vCard 文本，返回全部有效联系人（按出现顺序）。
///
/// 确定性契约：同一输入两次解析结果逐位一致（FR-036 可复现）。
/// 结构损坏（未闭合/孤立 END/无法解析的属性行）返回错误；语义空白
/// （无 `FN` 且 `N` 无内容、非 UTF-8 `CHARSET`）跳过该卡而不报错。
pub fn parse_vcard(input: &str) -> VCardResult<Vec<VCardContact>> {
    let input = input.strip_prefix('\u{feff}').unwrap_or(input);
    let mut contacts = Vec::new();

    // 段级状态：当前打开的卡（内容行累加，已折叠 + 已转义）。
    let mut in_card = false;
    let mut card_lines: Vec<(usize, String)> = Vec::new();
    let mut logical_line: Option<(usize, String)> = None;

    // 统一按 \n 分行后修掉行尾 \r（覆盖 \r\n 与裸 \n 两种形态）。
    let physical = input
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .collect::<Vec<&str>>();

    for (index, raw) in physical.iter().enumerate() {
        let line_no = index + 1;
        // 行折叠：后行以空格/制表符开头 → 拼到上一逻辑行（RFC 2426 §2.4.1）。
        if let Some(ch) = raw.chars().next() {
            if ch == ' ' || ch == '\t' {
                if let Some((_, acc)) = logical_line.as_mut() {
                    acc.push_str(raw.trim_start_matches([' ', '\t']));
                    continue;
                }
                // 无上一逻辑行的续行：按畸形行处理（防御，不崩溃）。
                return Err(VCardError::MalformedLine(line_no));
            }
        }
        // 收尾上一逻辑行。
        if let Some((first, acc)) = logical_line.take() {
            dispatch_line(acc, first, &mut in_card, &mut card_lines, &mut contacts)?;
        }
        logical_line = Some((line_no, raw.to_string()));
    }
    if let Some((first, acc)) = logical_line.take() {
        dispatch_line(acc, first, &mut in_card, &mut card_lines, &mut contacts)?;
    }

    if in_card {
        return Err(VCardError::UnclosedCard(input.lines().count()));
    }
    Ok(contacts)
}

/// 处理一条（已折叠的）逻辑行：边界、属性收集与卡收尾。
#[allow(clippy::too_many_arguments)]
fn dispatch_line(
    line: String,
    line_no: usize,
    in_card: &mut bool,
    card_lines: &mut Vec<(usize, String)>,
    contacts: &mut Vec<VCardContact>,
) -> VCardResult<()> {
    let upper = line.to_ascii_uppercase();
    if upper.starts_with("BEGIN:VCARD") {
        if *in_card {
            // 卡中再开卡：按畸形处理（防御）。
            return Err(VCardError::MalformedLine(line_no));
        }
        *in_card = true;
        card_lines.clear();
        return Ok(());
    }
    if upper.starts_with("END:VCARD") {
        if !*in_card {
            return Err(VCardError::OrphanEnd(line_no));
        }
        *in_card = false;
        let name = extract_name(card_lines);
        if let Some(name) = name {
            contacts.push(VCardContact {
                name,
                keys: Vec::new(),
            });
        }
        card_lines.clear();
        return Ok(());
    }
    if !*in_card {
        // 卡外内容（VERSION 头、空白、注释）：忽略。
        return Ok(());
    }
    // 卡内空行：忽略（一些导出工具会插入空行）。
    if line.trim().is_empty() {
        return Ok(());
    }
    // 卡内属性行（属性名可能带参数与组前缀）。
    if let Some((attr, param_text, value)) = split_property(&line) {
        // 非 UTF-8 CHARSET：跳过该卡（首批不转码，保持数据完整性）。
        if charset_is_non_utf8(param_text) {
            // 标记卡内容作废：清空后仅继续收集直到 END。
            card_lines.clear();
            card_lines.push((line_no, String::new())); // 哨兵行，extract_name 视为空卡
        } else {
            card_lines.push((line_no, format!("{attr}:{value}")));
        }
        return Ok(());
    }
    Err(VCardError::MalformedLine(line_no))
}

/// 按 `属性[;参数]*:值` 拆行；返回（属性名小写, 参数文本, 值）。组前缀剥除。
fn split_property(line: &str) -> Option<(String, &str, &str)> {
    let colon = line.find(':')?;
    let head = &line[..colon];
    let value = &line[colon + 1..];
    let (ident, params) = match head.find(';') {
        Some(semi) => (&head[..semi], &head[semi + 1..]),
        None => (head, ""),
    };
    // 组前缀：`item1.FN` → 取最后一个 '.' 之后（RFC 2426 §3.2.3）。
    let name = ident.rsplit('.').next().unwrap_or(ident);
    if name.is_empty() {
        return None;
    }
    Some((name.to_ascii_lowercase(), params, value))
}

/// 参数文本里是否出现显式非 UTF-8 字符集声明（`CHARSET=GB*` 等）。
fn charset_is_non_utf8(params: &str) -> bool {
    params.split(';').any(|part| {
        let part = part.trim();
        let lower = part.to_ascii_lowercase();
        lower.starts_with("charset=") && !lower.ends_with("utf-8")
    })
}

/// 从卡内属性行提取姓名：`FN` 优先；无 `FN` 用 `N` 组件拼接（中文顺序：姓+名）。
fn extract_name(card_lines: &[(usize, String)]) -> Option<String> {
    let mut family = String::new();
    let mut given = String::new();
    let mut has_n = false;
    for (_, line) in card_lines {
        let (attr, _, value) = split_property(line)?;
        match attr.as_str() {
            "fn" => {
                let name = unescape(value);
                if !name.trim().is_empty() {
                    return Some(unescape(value.trim()));
                }
            }
            "n" => {
                // N:姓;名;中间名;前缀;后缀 （RFC 2426）。
                has_n = true;
                let mut parts = value.split(';');
                family = unescape(parts.next().unwrap_or("")).trim().to_owned();
                given = unescape(parts.next().unwrap_or("")).trim().to_owned();
            }
            _ => {}
        }
    }
    if !has_n {
        return None;
    }
    // 中文字符直接拼接（姓+名）；否则以空格连接 Given Family（西式阅读序）。
    let result = match (family.is_empty(), given.is_empty()) {
        (true, true) => return None,
        (false, true) => family,
        (true, false) => given,
        (false, false) => {
            let chinese = family
                .chars()
                .all(|c| ('\u{4e00}'..='\u{9fff}').contains(&c))
                && given
                    .chars()
                    .all(|c| ('\u{4e00}'..='\u{9fff}').contains(&c));
            if chinese {
                format!("{family}{given}")
            } else {
                format!("{given} {family}")
            }
        }
    };
    let result = result.trim();
    if result.is_empty() {
        None
    } else {
        Some(result.to_owned())
    }
}

/// 反转义值：`\\` `\,` `\;` `\n`（RFC 2426 §2.1.1）。
fn unescape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('\\') => out.push('\\'),
                Some(',') => out.push(','),
                Some(';') => out.push(';'),
                Some('n') | Some('N') => out.push('\n'),
                Some(other) => {
                    // 未知转义按字面保留反斜杠与字符（不丢内容）。
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(ch);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{parse_vcard, VCardContact, VCardError};

    fn names(input: &str) -> Vec<String> {
        parse_vcard(input)
            .expect("解析应成功")
            .into_iter()
            .map(|c| c.name)
            .collect()
    }

    fn contact(input: &str) -> VCardContact {
        let mut list = parse_vcard(input).expect("解析应成功");
        assert_eq!(list.len(), 1, "应恰好产出一条联系人");
        list.remove(0)
    }

    #[test]
    fn 标准_fn_单卡() {
        let card = "BEGIN:VCARD\nVERSION:3.0\nFN:张三\nN:张;三;;;\nEND:VCARD\n";
        let c = contact(card);
        assert_eq!(c.name, "张三");
        assert!(c.keys.is_empty(), "keys 由索引构建阶段填充");
    }

    #[test]
    fn 多卡按出现顺序() {
        let input = "BEGIN:VCARD\nVERSION:3.0\nFN:张三\nEND:VCARD\n\
                     BEGIN:VCARD\nVERSION:3.0\nFN:李四\nEND:VCARD\n";
        assert_eq!(names(input), vec!["张三", "李四"]);
    }

    #[test]
    fn 行折叠_crlf_空格与制表符() {
        // 属性值被折叠成两行（RFC 2426 §2.4.1），折叠后应还原为一个 FN。
        let input = "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:张\r\n 三\r\nEND:VCARD\r\n";
        assert_eq!(names(input), vec!["张三"]);
        let tab = "BEGIN:VCARD\nVERSION:3.0\nFN:张\n\t三\nEND:VCARD\n";
        assert_eq!(names(tab), vec!["张三"]);
    }

    #[test]
    fn 转义序列() {
        let input = "BEGIN:VCARD\nVERSION:3.0\nFN:王\\,小明\\;博士\\\\x\nEND:VCARD\n";
        assert_eq!(names(input), vec!["王,小明;博士\\x"]);
    }

    #[test]
    fn 未知字段与空行跳过() {
        let input = "BEGIN:VCARD\nVERSION:3.0\nFN:赵六\nORG:测试公司\n\
                     TEL:+86-10-12345678\nEMAIL:a@b.c\nADR:;;北京;;;\n\
                     X-CUSTOM:任意内容\n\nEND:VCARD\n";
        assert_eq!(names(input), vec!["赵六"]);
    }

    #[test]
    fn 无_fn_用_n_拼接_中文姓加名() {
        let input = "BEGIN:VCARD\nVERSION:3.0\nN:钱;七;;;\nEND:VCARD\n";
        assert_eq!(names(input), vec!["钱七"]);
    }

    #[test]
    fn 无_fn_用_n_拼接_西式给定名在前() {
        let input = "BEGIN:VCARD\nVERSION:3.0\nN:Smith;John;;;\nEND:VCARD\n";
        assert_eq!(names(input), vec!["John Smith"]);
    }

    #[test]
    fn 空名卡跳过() {
        // 无 FN 且 N 为空/缺失：语义空白，跳过不报错。
        let input = "BEGIN:VCARD\nVERSION:3.0\nTEL:+86\nEND:VCARD\n";
        assert!(parse_vcard(input).expect("解析应成功").is_empty());
    }

    #[test]
    fn 损坏输入_未闭合卡() {
        let input = "BEGIN:VCARD\nVERSION:3.0\nFN:张三\n";
        assert_eq!(parse_vcard(input), Err(VCardError::UnclosedCard(3)));
    }

    #[test]
    fn 损坏输入_孤立_end() {
        let input = "END:VCARD\n";
        assert_eq!(parse_vcard(input), Err(VCardError::OrphanEnd(1)));
    }

    #[test]
    fn 卡外内容忽略() {
        let input = "some trailing text\nBEGIN:VCARD\nVERSION:3.0\nFN:孙八\nEND:VCARD\n";
        assert_eq!(names(input), vec!["孙八"]);
    }

    #[test]
    fn 显式_utf8_charset_正常解析() {
        let input = "BEGIN:VCARD\nVERSION:3.0\nFN;CHARSET=UTF-8:周五\nEND:VCARD\n";
        assert_eq!(names(input), vec!["周五"]);
    }

    #[test]
    fn 非_utf8_charset_跳过该卡() {
        let input = "BEGIN:VCARD\nVERSION:3.0\nFN;CHARSET=GB2312:吴十\nEND:VCARD\n\
                     BEGIN:VCARD\nVERSION:3.0\nFN:郑十一\nEND:VCARD\n";
        assert_eq!(names(input), vec!["郑十一"]);
    }

    #[test]
    fn 组前缀_item_fn() {
        let input = "BEGIN:VCARD\nVERSION:3.0\nitem1.FN:冯十二\nEND:VCARD\n";
        assert_eq!(names(input), vec!["冯十二"]);
    }

    #[test]
    fn 裸_lf_与_bom() {
        let input = "\u{feff}BEGIN:VCARD\nVERSION:3.0\nFN:陈十三\nEND:VCARD\n";
        assert_eq!(names(input), vec!["陈十三"]);
    }

    #[test]
    fn 确定性两次解析逐位一致() {
        let input = "BEGIN:VCARD\nVERSION:3.0\nFN:张三\nN:张;三;;;\nEND:VCARD\n";
        assert_eq!(parse_vcard(input), parse_vcard(input));
    }
}
