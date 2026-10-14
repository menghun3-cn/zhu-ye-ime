//! 词典更新器（M6-U，FR-020、方案设计 11.7）。
//!
//! **唯一允许联网的组件**：TSF DLL 在宿主进程内绝不做网络/TLS 操作
//! （S-4：防挂起宿主、杀软拦截、影响输入）。引擎启动时可异步 spawn 本程序
//! 一次；本程序完成检查/下载/应用后立即退出，不驻留。
//!
//! 子命令：
//! - `status`：打印当前配置、已安装包与 manifest 差异（不联网）
//! - `check`：联网拉取 manifest，报告可用更新（不下载包）
//! - `apply`：联网拉取 manifest 与包，验签验哈希后原子应用
//! - `check-once`：启动异步检查一次（T-088 / FR-048）：无 UI、无 stdout，结果只写
//!   `update_status.json` 并记录 `config.json` 的 `last_check`
//!
//! **默认开启**（T-154 反转 P-03）：`config.json` 的 `online_update` 缺失或为
//! `true` 时检查/应用可用；显式置 `false` 时 `check`/`apply` 直接退出且不发起
//! 任何网络请求，`check-once` 同样直接退出（TSF 侧也只会在开关开启时 spawn）。
//!
//! 除词典包 manifest 外，检查还会拉取**独立签名**的 `app.json`（T-154）：
//! 镜像仓库随发布附带输入法应用版本与更新日志，验签后用于"发现新版本 +
//! 更新日志"展示；该文件缺失或损坏只降级应用信息，不阻断词典包检查/应用。

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::ExitCode;

use zhu_ye_core::manifest::{
    parse_manifest, parse_public_key, verify_signature_with_key, version_at_least,
};
use zhu_ye_core::pack_config::{load_config, plan_packs, ConfigFile, PACKS_DIR_NAME};
use zhu_ye_core::update::{apply_release, find_outdated};
use zhu_ye_core::{AppMeta, UpdateStatus};

/// 发布清单默认 URL（T-098：镜像分发仓库 menghun3-cn/zhu-ye-updates；
/// 该仓库 feed release 不带 prerelease 标记 → `releases/latest` 语义可用，
/// 绕开主仓库 latest 拒绝 prerelease 的缺陷（T-096 文档 §21.1）。
/// 可用 `ZHU_YE_MANIFEST_URL` 覆盖（测试与私有渠道）。
const DEFAULT_MANIFEST_URL: &str =
    "https://github.com/menghun3-cn/zhu-ye-updates/releases/latest/download/manifest.json";

/// 内置发布公钥（十六进制）。发布私钥只在发布环境，绝不入库。
///
/// 未配置时更新器拒绝应用任何 manifest——宁可不可用，也不接受未固定信任锚的包。
const BUILTIN_PUBLIC_KEY: &str = match option_env!("ZHU_YE_RELEASE_PUBLIC_KEY") {
    Some(key) => key,
    None => "",
};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("status") | None => status_command(),
        Some("check") | Some("apply") => {
            let apply = args.first().map(String::as_str) == Some("apply");
            // T-161：`check --json` 输出结构化 UpdateStatus，供设置窗口渲染用户摘要。
            let json = args.get(1).map(String::as_str) == Some("--json");
            check_command(apply, json)
        }
        Some("check-once") => check_once_command(),
        Some("--help") | Some("-h") => {
            print_usage();
            Ok(())
        }
        Some(other) => Err(format!("未知子命令: {other}")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("zhu-ye-updater: {message}");
            ExitCode::from(1)
        }
    }
}

fn print_usage() {
    println!("zhu-ye-updater — 竹叶输入法词典更新器");
    println!("用法:");
    println!("  zhu-ye-updater status   查看配置与已安装包状态（不联网）");
    println!("  zhu-ye-updater check    检查可用更新（需 online_update=true）");
    println!("  zhu-ye-updater check --json   检查并以 UpdateStatus JSON 输出（设置窗口契约）");
    println!("  zhu-ye-updater apply    下载并应用更新（需 online_update=true）");
    println!("  zhu-ye-updater check-once   启动时静默检查一次（结果写 update_status.json）");
}

/// `%APPDATA%\zhu-ye-ime`。
fn appdata_root() -> Result<PathBuf, String> {
    std::env::var_os("APPDATA")
        .map(|root| PathBuf::from(root).join("zhu-ye-ime"))
        .ok_or_else(|| "未设置 APPDATA，无法定位配置目录".to_owned())
}

fn config_path() -> Result<PathBuf, String> {
    Ok(appdata_root()?.join("config.json"))
}

/// 已安装包的目录（领域包可写目录）。
fn packs_dir() -> Result<PathBuf, String> {
    Ok(appdata_root()?.join(PACKS_DIR_NAME))
}

/// 内置公钥；未配置或格式非法时返回明确错误。
fn trusted_key() -> Result<ed25519_dalek::VerifyingKey, String> {
    if BUILTIN_PUBLIC_KEY.is_empty() {
        return Err(
            "未内置发布公钥（构建时未设置 ZHU_YE_RELEASE_PUBLIC_KEY），拒绝应用任何更新".to_owned(),
        );
    }
    parse_public_key(BUILTIN_PUBLIC_KEY).map_err(|error| format!("内置公钥无效: {error}"))
}

fn status_command() -> Result<(), String> {
    let config_path = config_path()?;
    let (config, diagnostic) = load_config(&config_path);
    println!("配置: {}", config_path.display());
    if let Some(diagnostic) = diagnostic {
        println!("  警告: {diagnostic}");
    }
    println!(
        "  在线更新: {}",
        if config.online_update {
            "已开启"
        } else {
            "已关闭（默认）"
        }
    );
    println!("  启用包: {:?}", config.enabled_packs);
    println!(
        "  上次检查: {}",
        config
            .last_check
            .map_or_else(|| "从未".to_owned(), |seconds| seconds.to_string())
    );

    let packs_dir = packs_dir()?;
    println!("包目录: {}", packs_dir.display());
    if packs_dir.is_dir() {
        let mut names: Vec<String> = std::fs::read_dir(&packs_dir)
            .map_err(|error| format!("读取包目录失败: {error}"))?
            .filter_map(Result::ok)
            .filter(|entry| entry.path().extension().and_then(|e| e.to_str()) == Some("zyct"))
            .filter_map(|entry| entry.file_name().into_string().ok())
            .collect();
        names.sort();
        if names.is_empty() {
            println!("  已安装包: （无）");
        } else {
            for name in names {
                let path = packs_dir.join(&name);
                let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                println!("  已安装: {name}（{size} 字节）");
            }
        }
    } else {
        println!("  已安装包: （目录不存在）");
    }

    // 报告装配计划中被忽略/缺失的包，便于排查配置问题。
    let (base_dir, packs) = (appdata_root()?, packs_dir.clone());
    let plan = plan_packs(&config, &base_dir, &packs);
    if !plan.unknown.is_empty() {
        println!("  未知包 id（已忽略）: {:?}", plan.unknown);
    }
    if !plan.missing.is_empty() {
        println!("  配置了但文件缺失: {:?}", plan.missing);
    }
    Ok(())
}

/// 检查收集阶段的载荷：网络/验签/版本比较的结果，人类检查行一并收集
/// （T-161：`--json` 时静默收集，末尾统一输出结构化状态）。
struct CheckInfo {
    manifest: zhu_ye_core::manifest::Manifest,
    key: ed25519_dalek::VerifyingKey,
    engine_version: String,
    /// 输入法应用元数据（T-154）：`Err` 为拉取/验签失败摘要（人类模式降级打印）。
    app_meta: Result<Option<AppMeta>, String>,
    /// 可更新的包 id 列表（基础包随安装只读交付，real/seed 是开发产物，不在更新范围）。
    outdated: Vec<String>,
    /// 可更新包中的最大版本号（比较口径同 `version_at_least`；无可更新为空串）。
    latest: String,
    /// 人类检查行（清单地址/清单/签名校验），由调用方统一输出。
    human_lines: Vec<String>,
}

impl CheckInfo {
    /// 检查结果 → 结构化更新状态（口径与 `check-once` 一致，T-154）。
    #[must_use]
    fn to_status(&self) -> UpdateStatus {
        let now = zhu_ye_core::unix_now();
        if let Ok(Some(meta)) = &self.app_meta {
            if zhu_ye_core::app_update_available(&meta.app_version, &self.engine_version) {
                return UpdateStatus::available_app(
                    now,
                    meta.app_version.clone(),
                    meta.release_notes.clone(),
                    self.outdated.clone(),
                    (!self.latest.is_empty()).then_some(self.latest.clone()),
                );
            }
        }
        if self.outdated.is_empty() {
            return UpdateStatus::up_to_date(now);
        }
        UpdateStatus::available(now, self.outdated.clone(), self.latest.clone())
    }
}

/// 拉取、验签并比较版本，收集检查结果；`silent` 时不收集人类行（T-161）。
fn check_collect(url: &str, silent: bool) -> Result<CheckInfo, String> {
    let mut human_lines = Vec::new();
    let mut ui = |line: String| {
        if !silent {
            human_lines.push(line);
        }
    };

    ui(format!("清单地址: {url}"));
    let manifest_text = http_get_text(url)?;
    let manifest = parse_manifest(&manifest_text).map_err(|error| error.to_string())?;
    ui(format!(
        "清单: schema={} 发布时间={} 包数={}",
        manifest.schema,
        manifest.published_at,
        manifest.packs.len()
    ));

    let key = trusted_key()?;
    verify_signature_with_key(&manifest, &key).map_err(|error| error.to_string())?;
    ui("签名校验: 通过（内置公钥）".to_owned());

    let engine_version = zhu_ye_core::core_version();
    // 输入法应用版本信息（T-154）：独立签名的 app.json，仅展示用途；缺失/损坏/
    // 验签失败只降级，不阻断词典包检查与应用。
    let app_base = url.rsplit_once('/').map(|(prefix, _)| prefix).unwrap_or("");
    let app_meta = fetch_app_meta(app_base, &key);

    let packs_dir = packs_dir()?;
    // 只考虑可分发包：基础包随安装只读交付，real/seed 是开发产物，都不在更新范围。
    let outdated: Vec<String> = find_outdated(&manifest, &packs_dir)
        .into_iter()
        .filter(|id| zhu_ye_core::is_distributable_pack(id))
        .collect();
    // 可更新包中的最大版本号（与 `check-once` 相同口径）。
    let latest = manifest
        .packs
        .iter()
        .filter(|pack| outdated.contains(&pack.id))
        .map(|pack| pack.version.as_str())
        .max_by(|a, b| {
            if version_at_least(a, b) {
                std::cmp::Ordering::Greater
            } else if version_at_least(b, a) {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            }
        })
        .unwrap_or_default()
        .to_owned();

    Ok(CheckInfo {
        manifest,
        key,
        engine_version: engine_version.to_owned(),
        app_meta,
        outdated,
        latest,
        human_lines,
    })
}

/// 输出结构化检查结果（T-161：设置窗口 `check --json` 契约）。
fn print_status_json(status: &UpdateStatus) -> Result<(), String> {
    let text = zhu_ye_core::update_status_to_json(status)?;
    println!("{text}");
    Ok(())
}

/// 检查或应用更新。`apply` 为 `false` 时只报告；`json` 为 `true` 时只输出
/// 机器可读 `UpdateStatus`（T-161：设置窗口按结构化结果渲染，不解析人类文本）。
fn check_command(apply: bool, json: bool) -> Result<(), String> {
    let config_path = config_path()?;
    let (config, diagnostic) = load_config(&config_path);
    if let Some(diagnostic) = diagnostic {
        eprintln!("配置警告: {diagnostic}");
    }

    // 关闭原则（T-154）：显式关闭时不发起任何网络请求。
    if !config.online_update {
        if json {
            print_status_json(&UpdateStatus::failed(
                zhu_ye_core::unix_now(),
                "在线更新已关闭（online_update=false），未发起任何网络请求",
            ))?;
        } else {
            println!("在线更新已关闭（online_update=false），未发起任何网络请求。");
            println!("如需检查更新，请在设置「关于与更新」中启用在线更新。");
        }
        return Ok(());
    }

    let url =
        std::env::var("ZHU_YE_MANIFEST_URL").unwrap_or_else(|_| DEFAULT_MANIFEST_URL.to_owned());
    let info = match check_collect(&url, json) {
        Ok(info) => info,
        Err(message) => {
            if json {
                // JSON 契约：失败以 `error` 字段表述，进程仍以 0 退出，
                // 设置窗口无需拼装退出码与输出文本。
                print_status_json(&UpdateStatus::failed(zhu_ye_core::unix_now(), message))?;
                return Ok(());
            }
            return Err(message);
        }
    };
    for line in &info.human_lines {
        println!("{line}");
    }

    // 输入法应用版本信息（人类检查行；json 模式载荷已含，不再输出）。
    if !json {
        match &info.app_meta {
            Ok(Some(meta))
                if zhu_ye_core::app_update_available(&meta.app_version, &info.engine_version) =>
            {
                println!(
                    "发现输入法应用新版本: {}（当前 {}）",
                    meta.app_version, info.engine_version
                );
                println!("更新日志:");
                for line in meta.release_notes.lines() {
                    println!("  {line}");
                }
                println!("（输入法主程序升级请到官网或更新页下载新版安装包）");
            }
            Ok(Some(_)) => println!("输入法应用已是最新版本（{}）。", info.engine_version),
            Ok(None) => println!("（发布源未提供应用版本信息）"),
            Err(error) => println!("应用版本信息检查失败: {error}（不影响词典包检查）"),
        }
    }

    if json {
        return print_status_json(&info.to_status());
    }
    if info.outdated.is_empty() {
        println!("所有可更新包均为最新，无需更新。");
        return Ok(());
    }
    println!("可用更新: {:?}", info.outdated);

    if !apply {
        println!("（未应用；执行 `zhu-ye-updater apply` 下载并应用）");
        return Ok(());
    }

    // 下载各包；跳过版本门槛不足的包。
    let base_url = url.rsplit_once('/').map(|(prefix, _)| prefix).unwrap_or("");
    let mut downloads: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for pack in &info.manifest.packs {
        if !info.outdated.contains(&pack.id) {
            continue;
        }
        if !version_at_least(&info.engine_version, &pack.min_engine_version) {
            println!(
                "跳过 {}：需要引擎 >= {}，当前 {}",
                pack.id, pack.min_engine_version, info.engine_version
            );
            continue;
        }
        let pack_url = format!("{base_url}/{}", pack.file);
        println!("下载 {} <- {}", pack.id, pack_url);
        let bytes = http_get_bytes(&pack_url)?;
        println!("  {} 字节", bytes.len());
        downloads.insert(pack.id.clone(), bytes);
    }

    if downloads.is_empty() {
        println!("没有可应用的包。");
        return Ok(());
    }

    // 传**原始** manifest：签名是对完整清单计算的，裁剪包列表会让验签失败。
    // `apply_release` 内部只校验本次下载的包。
    let packs_dir = packs_dir()?;
    let outcome = apply_release(
        &info.manifest,
        &info.key,
        &downloads,
        &packs_dir,
        &info.engine_version,
    )
    .map_err(|error| error.to_string())?;
    println!("已应用: {:?}", outcome.applied);
    if !outcome.skipped_by_version.is_empty() {
        println!("版本门槛跳过: {:?}", outcome.skipped_by_version);
    }
    if !outcome.unchanged.is_empty() {
        println!("无需变更: {:?}", outcome.unchanged);
    }

    // 记录检查时间；失败不影响已完成的更新。
    let updated = ConfigFile {
        last_check: Some(zhu_ye_core::unix_now()),
        ..config
    };
    if let Err(error) = zhu_ye_core::pack_config::save_config(&config_path, &updated) {
        eprintln!("警告: 写入配置失败: {error}");
    }
    println!("更新完成。重启输入法后生效。");
    Ok(())
}

/// 启动时静默检查一次（T-088 / FR-048，方案设计 §14.5.4）。
///
/// - `online_update=false` → 直接退出：不联网、不写状态文件（配合 TSF 侧开关判断，
///   达成验收"显式关闭时零 spawn/零网络"）；
/// - 距上次检查不足 [`zhu_ye_core::CHECK_INTERVAL_DAYS`] 天 → 直接退出（保留旧状态）；
/// - 否则联网拉取并验签 manifest 与应用版本元数据（T-154），把结果原子写入
///   `update_status.json`，并回写 `config.json` 的 `last_check`（S-8：以进入本命令时
///   重读的配置为基线，保存前不再整体覆盖）；
/// - 网络/验签失败也写状态文件（含错误摘要，关于页可见），`last_check` 照记，避免故障期
///   每次启动都重试；
/// - 全程无 stdout（无 UI、无打扰）。
fn check_once_command() -> Result<(), String> {
    let config_path = config_path()?;
    let (config, _diagnostic) = load_config(&config_path);
    if !config.online_update {
        return Ok(());
    }

    let now = zhu_ye_core::unix_now();
    if !zhu_ye_core::check_due(config.last_check, now) {
        return Ok(());
    }

    let url =
        std::env::var("ZHU_YE_MANIFEST_URL").unwrap_or_else(|_| DEFAULT_MANIFEST_URL.to_owned());
    let outcome = (|| -> Result<UpdateStatus, String> {
        let manifest_text = http_get_text(&url)?;
        let manifest = parse_manifest(&manifest_text).map_err(|error| error.to_string())?;
        let key = trusted_key()?;
        verify_signature_with_key(&manifest, &key).map_err(|error| error.to_string())?;
        let packs_dir = packs_dir()?;
        // 只考虑可分发包：基础包随安装只读交付，real/seed 是开发产物，都不在更新范围。
        let outdated: Vec<String> = find_outdated(&manifest, &packs_dir)
            .into_iter()
            .filter(|id| zhu_ye_core::is_distributable_pack(id))
            .collect();
        // 可更新包中的最大版本号（比较口径同 `version_at_least`）。
        let latest = manifest
            .packs
            .iter()
            .filter(|pack| outdated.contains(&pack.id))
            .map(|pack| pack.version.as_str())
            .max_by(|a, b| {
                if version_at_least(a, b) {
                    std::cmp::Ordering::Greater
                } else if version_at_least(b, a) {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Equal
                }
            })
            .unwrap_or_default()
            .to_owned();
        // 输入法应用版本信息（T-154）：缺失/失败降级为"无应用信息"，不影响包状态。
        let app_base = url.rsplit_once('/').map(|(prefix, _)| prefix).unwrap_or("");
        let app_meta = fetch_app_meta(app_base, &key).ok().flatten();
        if let Some(meta) = app_meta.as_ref().filter(|meta| {
            zhu_ye_core::app_update_available(&meta.app_version, zhu_ye_core::core_version())
        }) {
            // 应用新版本优先作为"有更新"状态展示（词典包信息一并携带）。
            return Ok(UpdateStatus::available_app(
                now,
                meta.app_version.clone(),
                meta.release_notes.clone(),
                if outdated.is_empty() {
                    Vec::new()
                } else {
                    outdated
                },
                (!latest.is_empty()).then_some(latest),
            ));
        }
        if outdated.is_empty() {
            return Ok(UpdateStatus::up_to_date(now));
        }
        Ok(UpdateStatus::available(now, outdated, latest))
    })();
    let status = outcome.unwrap_or_else(|error| UpdateStatus::failed(now, error));

    // 写状态文件失败不阻断：尽力而为，问题留给日志与关于页的旧状态。
    let status_path = appdata_root()?.join(zhu_ye_core::UPDATE_STATUS_FILE_NAME);
    if let Err(error) = zhu_ye_core::write_update_status(&status_path, &status) {
        eprintln!("zhu-ye-updater: 写入更新状态失败: {error}");
    }

    // 回写检查时间（间隔判断的数据源）。
    let updated = ConfigFile {
        last_check: Some(now),
        ..config
    };
    if let Err(error) = zhu_ye_core::pack_config::save_config(&config_path, &updated) {
        eprintln!("zhu-ye-updater: 写入配置失败: {error}");
    }
    Ok(())
}

/// 最小 HTTP GET：用系统自带 `curl.exe`（Windows 10+ 内置），
/// 避免为更新器引入 TLS 依赖栈。
///
/// 失败时返回 stderr 摘要，便于日志诊断。
fn http_get_bytes(url: &str) -> Result<Vec<u8>, String> {
    let output = std::process::Command::new("curl.exe")
        .args(["-fsSL", "--max-time", "60", url])
        .output()
        .map_err(|error| format!("调用 curl 失败: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("下载失败（{}）: {}", output.status, stderr.trim()));
    }
    Ok(output.stdout)
}

fn http_get_text(url: &str) -> Result<String, String> {
    let bytes = http_get_bytes(url)?;
    String::from_utf8(bytes).map_err(|error| format!("响应不是有效 UTF-8: {error}"))
}

/// 拉取并验签输入法应用版本元数据（`app.json`，T-154）。
///
/// `app_base` 是与 manifest 同目录的 URL 前缀（如 `…/releases/latest/download`）。
/// - 拉取失败（旧发布源没有该资产、网络故障）→ `Ok(None)`：降级为"无应用信息"，
///   不阻断词典包检查/应用（应用信息仅供展示）；
/// - 拉取成功但格式/验签失败 → `Err` 摘要（调用方决定如何提示）。
fn fetch_app_meta(
    app_base: &str,
    key: &ed25519_dalek::VerifyingKey,
) -> Result<Option<AppMeta>, String> {
    let url = format!("{app_base}/{}", zhu_ye_core::APP_META_FILE_NAME);
    let text = match http_get_text(&url) {
        Ok(text) => text,
        Err(_) => return Ok(None),
    };
    let meta = zhu_ye_core::parse_app_meta(&text)?;
    meta.verify(key).map_err(|error| error.to_string())?;
    Ok(Some(meta))
}

/// 供自检：确认更新器在显式关闭时不会联网（验收标准 FR-020「关闭零网络」）。
#[must_use]
pub fn network_required(config: &ConfigFile) -> bool {
    config.online_update
}

#[cfg(test)]
mod tests {
    use super::network_required;
    use zhu_ye_core::pack_config::ConfigFile;

    #[test]
    fn 默认配置要求网络() {
        // T-154 反转 P-03：在线更新默认开启。
        assert!(network_required(&ConfigFile::default()));
    }

    #[test]
    fn 显式关闭后不要求网络() {
        let config = ConfigFile {
            online_update: false,
            ..ConfigFile::default()
        };
        assert!(!network_required(&config));
    }

    #[test]
    fn update_status_json可被同源解析回来() {
        // T-161：`check --json` 输出契约 = UpdateStatus（同 check-once 写文件的口径）。
        let status = zhu_ye_core::UpdateStatus::available_app(
            1,
            "0.2.0".to_owned(),
            "第一行\n第二行".to_owned(),
            vec!["it".to_owned(), "med".to_owned()],
            Some("2026-10-11".to_owned()),
        );
        let text = zhu_ye_core::update_status_to_json(&status).unwrap();
        let back = zhu_ye_core::parse_update_status(&text).unwrap();
        assert_eq!(back, status, "序列化后应能原样解析回同一状态");
    }
}
