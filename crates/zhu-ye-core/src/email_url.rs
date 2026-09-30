//! 邮箱/网址格式候选（FR-031，场景 6）。
//!
//! 纯字符串规则、确定性、零 IO。判定与补全规则：
//! - `@` 出现且前面有至少 1 个字符 → 邮箱态；`@` 后无 `.` 时补全 `.com/.cn/.net`（至多
//!   3 条），已含 `.` 视为完整串直接上屏；
//! - `www.` 或 `http://` / `https://` 前缀（大小写不敏感）→ 网址态；主机段无 `.` 时补全
//!   `.com/.cn/.org`（至多 3 条），已含 `.` 视为完整串直接上屏；
//! - 其余输入返回 `None`，由上层走既有拼音/缩写路径（D-10：普通中文输入不介入）。
//!
//! 邮箱/网址候选 `pinyin = None`，不进入用户词学习（commit 路径防御）。上屏串保留用户
//! 已输入部分的大小写，仅追加小写后缀；候选由上层以 `CandidateSource::EmailUrl` 呈现。

/// 组合串的格式类别（FR-031）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatKind {
    /// 非邮箱/网址，走既有路径。
    None,
    /// 邮箱态（组合串含 `@`）。
    Email,
    /// 网址态（`www.` / `http://` / `https://` 前缀）。
    Url,
}

/// 邮箱补全候选后缀（至多 3 条，顺序即展示顺序）。
pub const EMAIL_TLDS: [&str; 3] = [".com", ".cn", ".net"];
/// 网址补全候选后缀（至多 3 条，顺序即展示顺序）。
pub const URL_TLDS: [&str; 3] = [".com", ".cn", ".org"];

/// 判定组合串是否进入邮箱/网址格式路径。
#[must_use]
pub fn detect_format(input: &str) -> FormatKind {
    if is_url_like(input) {
        return FormatKind::Url;
    }
    if let Some(at) = input.find('@') {
        // `@` 前必须有至少 1 个字符（纯 `@`/`@xx` 不成邮箱，不介入）。
        if at > 0 {
            return FormatKind::Email;
        }
    }
    FormatKind::None
}

/// 大小写不敏感前缀判定（仅 ASCII）。
fn has_ci_prefix(input: &str, prefix: &str) -> bool {
    input.len() >= prefix.len()
        && input
            .as_bytes()
            .iter()
            .zip(prefix.as_bytes())
            .all(|(a, b)| a.eq_ignore_ascii_case(b))
}

/// 网址前缀判定：`www.` / `http://` / `https://`（大小写不敏感）。
fn is_url_like(input: &str) -> bool {
    has_ci_prefix(input, "www.")
        || has_ci_prefix(input, "http://")
        || has_ci_prefix(input, "https://")
}

/// 组合串是否已含 `.`（视为格式完整，直接上屏不补全）。
fn contains_dot(input: &str) -> bool {
    input.contains('.')
}

/// 邮箱补全候选：`me@163` → `me@163.com/.cn/.net`（至多 3 条，顺序即展示顺序）；
/// `@` 后已含 `.`（如 `me@163.com`）→ 返回完整串本身（直通，不重复补全）；
/// 畸形（多余 `@` / `@` 前空）→ 返回完整串本身交由上层直通。
#[must_use]
pub fn email_candidates(input: &str) -> Vec<String> {
    let at_count = input.matches('@').count();
    let valid = at_count == 1
        && input
            .find('@')
            .is_some_and(|at| at > 0 && at < input.len() - 1);
    if !valid {
        // 防御返回原文直通，避免把畸形串当后缀拼接。
        return vec![input.to_owned()];
    }
    if contains_dot(input) {
        return vec![input.to_owned()];
    }
    EMAIL_TLDS
        .iter()
        .map(|tld| format!("{input}{tld}"))
        .collect()
}

/// 网址补全候选：`www.exa` → `www.exa.com/.cn/.org`（至多 3 条）；
/// `www.exa.com`（主机段已含 `.`）→ 返回完整串本身（直通，不重复补全）。
/// `http://` / `https://` 前缀同样处理；输出保留用户输入大小写，仅追加小写后缀。
#[must_use]
pub fn url_candidates(input: &str) -> Vec<String> {
    // 主机段：去掉 scheme 前缀后的部分（到第一个 `/` 为止，组合态一般无路径）。
    let rest = if has_ci_prefix(input, "https://") {
        &input["https://".len()..]
    } else if has_ci_prefix(input, "http://") {
        &input["http://".len()..]
    } else if has_ci_prefix(input, "www.") {
        &input["www.".len()..]
    } else {
        // 防御：非网址前缀时原样直通（上层应先经 detect_format）。
        return vec![input.to_owned()];
    };
    if rest.contains('.') {
        return vec![input.to_owned()];
    }
    URL_TLDS.iter().map(|tld| format!("{input}{tld}")).collect()
}

#[cfg(test)]
mod tests {
    use super::{detect_format, email_candidates, url_candidates, FormatKind};

    #[test]
    fn 格式识别() {
        assert_eq!(detect_format("me@163"), FormatKind::Email);
        assert_eq!(detect_format("ab@cd"), FormatKind::Email);
        assert_eq!(detect_format("www.exa"), FormatKind::Url);
        assert_eq!(detect_format("http://exa"), FormatKind::Url);
        assert_eq!(detect_format("https://exa"), FormatKind::Url);
        assert_eq!(detect_format("HTTP://EXA"), FormatKind::Url);
        assert_eq!(detect_format("nihao"), FormatKind::None);
        assert_eq!(detect_format(""), FormatKind::None);
        assert_eq!(detect_format("@163"), FormatKind::None);
        assert_eq!(detect_format("@"), FormatKind::None);
        // 网址前缀优先（www. 开头即使含 @ 也判网址）
        assert_eq!(detect_format("www.a@b"), FormatKind::Url);
    }

    #[test]
    fn 邮箱补全() {
        assert_eq!(
            email_candidates("me@163"),
            vec!["me@163.com", "me@163.cn", "me@163.net"]
        );
        // 已含点：完整直通，不重复补全
        assert_eq!(email_candidates("me@163.com"), vec!["me@163.com"]);
        assert_eq!(email_candidates("ab@c.d"), vec!["ab@c.d"]);
        // 畸形防御：多余 @ / @ 前空 → 原文直通
        assert_eq!(email_candidates("me@163@x"), vec!["me@163@x"]);
        assert_eq!(email_candidates("@163"), vec!["@163"]);
        assert_eq!(email_candidates(""), vec![""]);
    }

    #[test]
    fn 网址补全() {
        assert_eq!(
            url_candidates("www.exa"),
            vec!["www.exa.com", "www.exa.cn", "www.exa.org"]
        );
        assert_eq!(
            url_candidates("http://exa"),
            vec!["http://exa.com", "http://exa.cn", "http://exa.org"]
        );
        assert_eq!(
            url_candidates("https://exa"),
            vec!["https://exa.com", "https://exa.cn", "https://exa.org"]
        );
        // 已含点：完整直通
        assert_eq!(url_candidates("www.exa.com"), vec!["www.exa.com"]);
        assert_eq!(
            url_candidates("http://example.com"),
            vec!["http://example.com"]
        );
        // 大小写不敏感判定，输出保留输入大小写
        assert_eq!(
            url_candidates("HTTP://EXA"),
            vec!["HTTP://EXA.com", "HTTP://EXA.cn", "HTTP://EXA.org"]
        );
        // 防御：非网址前缀原样
        assert_eq!(url_candidates("exa"), vec!["exa"]);
    }
}
