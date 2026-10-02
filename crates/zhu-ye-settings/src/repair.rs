//! 输入法注册状态判据与两级修复（T-076 / FR-043）。
//!
//! 分层：`super::registry` 负责 Win32 注册表的读取与重注册（薄层），本模块负责判据、
//! 一级检测与修复决策（纯逻辑 + 文件 IO，可无 GUI、无注册表单测），窗口层负责
//! "先报告后动手"（D-41）的交互编排。
//!
//! 判据与 `scripts/ime-identity.ps1` 的 `Test-TsfRegistration` 逐条一致（设计 §6.1）：
//! `LanguageProfile\{语言}\{ProfileGuid}` 存在且 `Enable == 1`；`InProcServer32` 默认值
//! 指向**实际存在**的文件（S-9 强化）且 `ThreadingModel == 'Apartment'`。
//!
//! 一级修复只"改名、绝不删除"（D-41）：损坏的配置文件与用户词库改名为 `.bak` 后重建，
//! 原内容始终保留；二级修复（提权重注册）见 `registry::re_register`。

use std::path::{Path, PathBuf};

use zhu_ye_core::{
    load_config, probe_user_words_file, DictionaryFile, UserDictStore, UserWordsProbe,
};

/// 一次注册表探测的原始值（由 `registry::probe_registration` 读取；测试可构造）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryProbe {
    /// `LanguageProfile\{语言}\{ProfileGuid}` 键是否存在。
    pub language_profile_exists: bool,
    /// 该键的 `Enable` 值；键存在但读不到时视为 `None`。
    pub enable: Option<u32>,
    /// `InProcServer32` 的默认值（DLL 路径）；键存在但默认值为空时视为 `None`。
    pub inproc_server: Option<String>,
    /// `InProcServer32` 的 `ThreadingModel` 值。
    pub threading_model: Option<String>,
}

/// 注册状态判据的单项问题。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegisterIssue {
    /// 语言配置文件键缺失。
    ProfileMissing,
    /// `Enable` 不存在或不是 `1`。
    ProfileDisabled,
    /// `InProcServer32` 缺失或默认值为空。
    InprocMissing,
    /// `InProcServer32` 指向的文件不存在。
    DllMissing(String),
    /// `ThreadingModel` 不是 `Apartment`（附实际值）。
    ThreadingModel(String),
}

/// 注册状态判定结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationStatus {
    /// 全部问题；为空即已注册。
    pub issues: Vec<RegisterIssue>,
    /// 判据指向的 DLL 路径（仅当 `InProcServer32` 存在且有值）。
    pub dll_path: Option<PathBuf>,
}

impl RegistrationStatus {
    /// 是否已注册（判据全部通过）。
    #[must_use]
    pub fn registered(&self) -> bool {
        self.issues.is_empty()
    }

    /// 一行状态文案（供条目行与子视图标题）。
    #[must_use]
    pub fn headline(&self) -> String {
        if self.registered() {
            "已注册".to_owned()
        } else if self
            .issues
            .iter()
            .any(|issue| matches!(issue, RegisterIssue::DllMissing(_)))
        {
            "未注册（输入法文件缺失）".to_owned()
        } else {
            "未注册（注册表异常）".to_owned()
        }
    }

    /// 多行详情（注册状态行：第一行状态，其后逐行列出问题）。
    #[must_use]
    pub fn detail_lines(&self) -> Vec<String> {
        let mut lines = vec![self.headline()];
        for issue in &self.issues {
            lines.push(match issue {
                RegisterIssue::ProfileMissing => "语言配置文件键缺失".to_owned(),
                RegisterIssue::ProfileDisabled => "语言配置文件 Enable 不是 1".to_owned(),
                RegisterIssue::InprocMissing => "InProcServer32 缺失或为空".to_owned(),
                RegisterIssue::DllMissing(path) => {
                    format!("DLL 文件不存在：{path}")
                }
                RegisterIssue::ThreadingModel(actual) => {
                    format!("ThreadingModel 应为 Apartment，实际为：{actual}")
                }
            });
        }
        lines
    }
}

/// 按判据判定注册状态（纯逻辑）。
///
/// DLL 存在性检查读取文件系统；测试用临时文件覆盖该分支。
#[must_use]
pub fn evaluate_registration(probe: &RegistryProbe) -> RegistrationStatus {
    let mut issues = Vec::new();
    if !probe.language_profile_exists {
        issues.push(RegisterIssue::ProfileMissing);
    } else if probe.enable != Some(1) {
        issues.push(RegisterIssue::ProfileDisabled);
    }

    let mut dll_path = None;
    let inproc = probe
        .inproc_server
        .as_deref()
        .filter(|value| !value.is_empty());
    match inproc {
        None => issues.push(RegisterIssue::InprocMissing),
        Some(path) => {
            if probe.threading_model.as_deref() != Some("Apartment") {
                issues.push(RegisterIssue::ThreadingModel(
                    probe.threading_model.clone().unwrap_or_default(),
                ));
            }
            if Path::new(path).is_file() {
                dll_path = Some(PathBuf::from(path));
            } else {
                issues.push(RegisterIssue::DllMissing(path.to_owned()));
            }
        }
    }

    RegistrationStatus { issues, dll_path }
}

/// 一个校验失败的文件及其原因。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrokenFile {
    /// 文件路径。
    pub path: PathBuf,
    /// 失败原因（可读文案）。
    pub reason: String,
}

/// 一级检测结果（先报告后动手：检测不产生任何写盘/改名动作）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RepairScan {
    /// `packs` 目录是否存在（不存在 = 需重建）。
    pub packs_dir_missing: bool,
    /// `packs` 目录下校验失败的领域包（`DictionaryFile::open` 全量校验：魔数/格式版本/
    /// 布局/内容 SHA-256）。
    pub broken_packs: Vec<BrokenFile>,
    /// `config.json` 是否损坏（不存在不算损坏）。
    pub config_broken: bool,
    /// `user_words.json` 状态。
    pub user_words_probe: UserWordsProbe,
    /// 基础包存在但校验失败（不存在交给安装器，不在本入口修复）。
    pub base_dict_broken: Option<BrokenFile>,
}

impl RepairScan {
    /// 是否没有任何可修复项（注册状态不在此判定）。
    #[must_use]
    pub fn has_issues(&self) -> bool {
        self.packs_dir_missing
            || !self.broken_packs.is_empty()
            || self.config_broken
            || self.user_words_probe != UserWordsProbe::Ok
            || self.base_dict_broken.is_some()
    }

    /// 逐行检测结果（供「修复输入法」子视图展示；行数与 `repair_layout` 的行列表对应）。
    #[must_use]
    pub fn summary_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        if self.packs_dir_missing {
            lines.push("packs 目录缺失：将被重建（不删除任何文件）".to_owned());
        } else {
            lines.push("packs 目录存在".to_owned());
        }
        if self.broken_packs.is_empty() {
            lines.push("领域包校验通过".to_owned());
        } else {
            for broken in &self.broken_packs {
                let name = broken.path.file_stem().map_or_else(
                    || broken.path.display().to_string(),
                    |stem| stem.to_string_lossy().into_owned(),
                );
                lines.push(format!(
                    "领域包 {name} 损坏（{reason}）：将被隔离为 .bak，请重新导入或更新恢复",
                    reason = broken.reason
                ));
            }
        }
        if self.config_broken {
            lines.push("config.json 损坏：将被改名为 .bak 后重建默认配置".to_owned());
        } else {
            lines.push("config.json 可解析".to_owned());
        }
        match self.user_words_probe {
            UserWordsProbe::Ok => lines.push("user_words.json 可解析".to_owned()),
            UserWordsProbe::Corrupt => {
                lines.push("user_words.json 损坏：将被改名为 .bak 后重建空库".to_owned())
            }
            UserWordsProbe::NewerVersion { found } => lines.push(format!(
                "user_words.json 版本 {found} 高于当前支持：请升级输入法，本入口不改动它"
            )),
        }
        if let Some(broken) = &self.base_dict_broken {
            lines.push(format!(
                "基础包损坏（{}）：无法在本窗口修复，请重新安装",
                broken.reason
            ));
        }
        lines
    }
}

/// 执行一级检测（纯 IO，无副作用）。
#[must_use]
pub fn scan_l1(
    packs_dir: &Path,
    config_path: Option<&Path>,
    user_words_path: &Path,
    base_dict_path: &Path,
) -> RepairScan {
    let packs_dir_missing = !packs_dir.is_dir();
    let mut broken_packs = Vec::new();
    if !packs_dir_missing {
        if let Ok(entries) = std::fs::read_dir(packs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) != Some("zyct") {
                    continue;
                }
                if let Err(reason) = DictionaryFile::open(&path) {
                    broken_packs.push(BrokenFile {
                        path,
                        reason: reason.to_string(),
                    });
                }
            }
        }
    }
    let config_broken = if let Some(config_path) = config_path {
        config_path.is_file() && load_config(config_path).1.is_some()
    } else {
        false
    };
    let user_words_probe = probe_user_words_file(user_words_path);
    let base_dict_broken = if base_dict_path.is_file() {
        DictionaryFile::open(base_dict_path)
            .err()
            .map(|reason| BrokenFile {
                path: base_dict_path.to_path_buf(),
                reason: reason.to_string(),
            })
    } else {
        None
    };
    RepairScan {
        packs_dir_missing,
        broken_packs,
        config_broken,
        user_words_probe,
        base_dict_broken,
    }
}

/// 把损坏文件改名为 `.bak` 序列之一（目标已被占用时追加 `.1`/`.2`，绝不覆盖删除）。
///
/// 命名沿用 `UserDictStore::load` 的恢复备份先例（`user_words.json` → `user_words.bak`，
/// 扩展名被替换），保证运行时自动恢复与修复入口产生同一种备份名，不并存两套命名。
/// 返回改名后的路径。
fn quarantine(path: &Path) -> Result<PathBuf, String> {
    let mut candidate = path.with_extension("bak");
    let mut counter = 1u32;
    while candidate.exists() {
        candidate = path.with_extension(format!("bak.{counter}"));
        counter += 1;
    }
    std::fs::rename(path, &candidate).map_err(|error| {
        format!(
            "把 {} 改名为 {} 失败：{error}",
            path.display(),
            candidate.display()
        )
    })?;
    Ok(candidate)
}

/// 一项一级修复动作的结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum L1Outcome {
    /// 成功；附用户可见的说明。
    Done(String),
    /// 失败；附原因。
    Failed(String),
}

/// 执行一级修复（由用户显式触发；D-41"先报告后动手"）。
///
/// 每个动作返回独立结果，互不拖累：某一步失败不会阻止其余动作执行。
pub fn apply_l1(
    scan: &RepairScan,
    packs_dir: &Path,
    config_path: Option<&Path>,
    user_words_path: &Path,
) -> Vec<L1Outcome> {
    let mut outcomes = Vec::new();
    if scan.packs_dir_missing {
        match std::fs::create_dir_all(packs_dir) {
            Ok(()) => outcomes.push(L1Outcome::Done(format!(
                "已重建 packs 目录：{}",
                packs_dir.display()
            ))),
            Err(error) => outcomes.push(L1Outcome::Failed(format!("重建 packs 目录失败：{error}"))),
        }
    }
    for broken in &scan.broken_packs {
        match quarantine(&broken.path) {
            Ok(backup) => {
                let name = broken.path.file_stem().map_or_else(
                    || broken.path.display().to_string(),
                    |stem| stem.to_string_lossy().into_owned(),
                );
                outcomes.push(L1Outcome::Done(format!(
                    "已隔离损坏的领域包 {name}（原文件保留为 {}）",
                    backup.display()
                )));
            }
            Err(error) => outcomes.push(L1Outcome::Failed(error)),
        }
    }
    if scan.config_broken {
        if let Some(config_path) = config_path {
            match quarantine(config_path) {
                Ok(backup) => {
                    match zhu_ye_core::save_config(config_path, &zhu_ye_core::ConfigFile::default())
                    {
                        Ok(()) => outcomes.push(L1Outcome::Done(format!(
                            "已重建 config.json（原文件保留为 {}）",
                            backup.display()
                        ))),
                        Err(error) => outcomes
                            .push(L1Outcome::Failed(format!("重建 config.json 失败：{error}"))),
                    }
                }
                Err(error) => outcomes.push(L1Outcome::Failed(error)),
            }
        }
    }
    if scan.user_words_probe == UserWordsProbe::Corrupt {
        match quarantine(user_words_path) {
            Ok(backup) => {
                let store = UserDictStore::new(user_words_path);
                match store.save(&zhu_ye_core::UserDictionary::new()) {
                    Ok(()) => outcomes.push(L1Outcome::Done(format!(
                        "已重建 user_words.json（原文件保留为 {}）",
                        backup.display()
                    ))),
                    Err(error) => outcomes.push(L1Outcome::Failed(format!(
                        "重建 user_words.json 失败：{error}"
                    ))),
                }
            }
            Err(error) => outcomes.push(L1Outcome::Failed(error)),
        }
    }
    outcomes
}

#[cfg(test)]
mod tests {
    use super::{
        apply_l1, evaluate_registration, quarantine, scan_l1, RegisterIssue, RegistryProbe,
    };
    use zhu_ye_core::UserWordsProbe;

    fn probe() -> RegistryProbe {
        RegistryProbe {
            language_profile_exists: true,
            enable: Some(1),
            inproc_server: Some("C:\\nonexistent\\zhu-ye-ime.dll".to_owned()),
            threading_model: Some("Apartment".to_owned()),
        }
    }

    #[test]
    fn 判据全部通过为已注册() {
        let dir = std::env::temp_dir().join(format!(
            "settings-reg-ok-{}-{}",
            std::process::id(),
            zhu_ye_core::unix_now()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let dll = dir.join("zhu-ye-ime.dll");
        std::fs::write(&dll, b"dummy").unwrap();
        let mut p = probe();
        p.inproc_server = Some(dll.to_string_lossy().into_owned());

        let status = evaluate_registration(&p);
        assert!(status.registered());
        assert_eq!(status.dll_path.as_deref(), Some(dll.as_path()));
        assert_eq!(status.headline(), "已注册");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 判据逐条报出问题() {
        // Profile 缺失 + InProcServer32 缺失。
        let mut p = probe();
        p.language_profile_exists = false;
        p.inproc_server = None;
        let status = evaluate_registration(&p);
        assert!(!status.registered());
        assert!(status.issues.contains(&RegisterIssue::ProfileMissing));
        assert!(status.issues.contains(&RegisterIssue::InprocMissing));
        assert!(status.headline().contains("未注册"));

        // Enable 不是 1。
        let mut p = probe();
        p.enable = Some(0);
        let status = evaluate_registration(&p);
        assert!(status.issues.contains(&RegisterIssue::ProfileDisabled));

        // ThreadingModel 不符。
        let mut p = probe();
        p.threading_model = Some("Both".to_owned());
        assert!(evaluate_registration(&p)
            .issues
            .contains(&RegisterIssue::ThreadingModel("Both".to_owned())));

        // 默认值为空视为缺失。
        let mut p = probe();
        p.inproc_server = Some(String::new());
        assert!(evaluate_registration(&p)
            .issues
            .contains(&RegisterIssue::InprocMissing));

        // DLL 不存在（文件系统分支）。
        let mut p = probe();
        p.inproc_server = Some("C:\\definitely\\missing.dll".to_owned());
        let status = evaluate_registration(&p);
        assert!(status.issues.contains(&RegisterIssue::DllMissing(
            "C:\\definitely\\missing.dll".to_owned()
        )));
        assert!(status.headline().contains("文件缺失"));
    }

    #[test]
    fn 一级检测逐项报告且不写盘() {
        let dir = std::env::temp_dir().join(format!(
            "settings-scan-{}-{}",
            std::process::id(),
            zhu_ye_core::unix_now()
        ));
        let packs = dir.join("packs");
        std::fs::create_dir_all(&packs).unwrap();
        let config = dir.join("config.json");
        let words = dir.join("user_words.json");

        // 全部健康：packs 存在、无可修项。
        std::fs::write(&config, r#"{"version":1}"#).unwrap();
        std::fs::write(&words, r#"{"version":1,"entries":[]}"#).unwrap();
        let scan = scan_l1(&packs, Some(&config), &words, &packs.join("base.zyct"));
        assert!(!scan.packs_dir_missing);
        assert!(scan.broken_packs.is_empty());
        assert!(!scan.config_broken);
        assert_eq!(scan.user_words_probe, UserWordsProbe::Ok);
        assert_eq!(scan.base_dict_broken, None);
        assert!(!scan.has_issues());

        // 坏 config 与坏 user_words、虚假领域包（非 ZYDT 魔数）、缺失 packs。
        std::fs::remove_dir_all(&packs).unwrap();
        std::fs::write(dir.join("packs_broken.zyct"), b"not a dictionary").unwrap();
        std::fs::write(&config, "{ not json").unwrap();
        std::fs::write(&words, "also not json").unwrap();
        let scan = scan_l1(
            &packs,
            Some(&config),
            &words,
            &dir.join("packs_broken.zyct"),
        );
        assert!(scan.packs_dir_missing);
        // packs 目录缺失时不扫包（避免误报）。
        assert!(scan.broken_packs.is_empty());
        assert!(scan.config_broken);
        assert_eq!(scan.user_words_probe, UserWordsProbe::Corrupt);
        assert!(scan.base_dict_broken.is_some());
        assert!(scan.has_issues());
        assert!(!scan.summary_lines().is_empty());

        // 探针阶段必须零副作用：坏文件原样保留，无任何 .bak。
        assert!(config.exists());
        assert!(std::fs::read_to_string(&config)
            .unwrap()
            .contains("not json"));
        assert!(!dir.join("config.bak").exists());
        assert!(!dir.join("user_words.bak").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 一级修复合规_改名重建而不删除() {
        let dir = std::env::temp_dir().join(format!(
            "settings-fix-{}-{}",
            std::process::id(),
            zhu_ye_core::unix_now()
        ));
        let packs = dir.join("packs");
        let config = dir.join("config.json");
        let words = dir.join("user_words.json");

        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(&config, "{ not json").unwrap();
        std::fs::write(&words, "not json").unwrap();
        let broken_pack = packs.join("it.zyct");
        std::fs::create_dir_all(&packs).unwrap();
        std::fs::write(&broken_pack, b"not a dictionary").unwrap();

        let scan = scan_l1(&packs, Some(&config), &words, &dir.join("none.zyct"));
        let outcomes = apply_l1(&scan, &packs, Some(&config), &words);
        assert_eq!(
            outcomes.len(),
            3,
            "packs 缺失修复 + 坏包隔离 + config + user_words"
        );

        // 原文件从未被删除：全部以 .bak 保留（命名沿用 UserDictStore 的恢复备份先例）。
        assert!(dir.join("config.bak").exists());
        assert!(dir.join("user_words.bak").exists());
        assert!(packs.join("it.bak").exists());
        // 重建后：
        assert!(config.exists());
        assert!(!std::fs::read_to_string(&config)
            .unwrap()
            .contains("not json"));
        assert!(words.exists());
        assert!(packs.is_dir());
        // 坏包被隔离后不再留在原位置。
        assert!(!packs.join("it.zyct").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 隔离改名防覆盖递增序号() {
        let dir = std::env::temp_dir().join(format!(
            "settings-quar-{}-{}",
            std::process::id(),
            zhu_ye_core::unix_now()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("config.json");
        std::fs::write(&file, "one").unwrap();

        let first = quarantine(&file).unwrap();
        assert_eq!(first, dir.join("config.bak"));
        std::fs::write(&file, "two").unwrap();
        let second = quarantine(&file).unwrap();
        assert_eq!(second, dir.join("config.bak.1"));
        // 两次内容都保留。
        assert_eq!(std::fs::read_to_string(&first).unwrap(), "one");
        assert_eq!(std::fs::read_to_string(&second).unwrap(), "two");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 版本过高只报告不改名() {
        let dir = std::env::temp_dir().join(format!(
            "settings-newer-{}-{}",
            std::process::id(),
            zhu_ye_core::unix_now()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let words = dir.join("user_words.json");
        std::fs::write(&words, r#"{"version":99,"entries":[]}"#).unwrap();

        let scan = scan_l1(&dir, None, &words, &dir.join("none.zyct"));
        assert_eq!(
            scan.user_words_probe,
            UserWordsProbe::NewerVersion { found: 99 }
        );
        // apply_l1 对 NewerVersion 无动作。
        assert!(apply_l1(&scan, &dir, None, &words).is_empty());
        assert!(!dir.join("user_words.json.bak").exists());
        assert!(std::fs::read_to_string(&words).unwrap().contains("99"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
