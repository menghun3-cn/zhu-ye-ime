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
//! **默认关闭**（P-03）：`config.json` 的 `online_update` 为 `false` 时，
//! `check`/`apply` 直接退出且不发起任何网络请求；`check-once` 同样直接退出
//! （TSF 侧也只在开关开启时才会 spawn 本程序，验收"默认关闭零 spawn/零网络"）。

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::ExitCode;

use zhu_ye_core::manifest::{
    parse_manifest, parse_public_key, verify_signature_with_key, version_at_least,
};
use zhu_ye_core::pack_config::{load_config, plan_packs, ConfigFile, PACKS_DIR_NAME};
use zhu_ye_core::update::{apply_release, find_outdated};
use zhu_ye_core::UpdateStatus;

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
        Some("check") => check_command(false),
        Some("apply") => check_command(true),
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
    println!("  zhu-ye-updater apply    下载并应用更新（需 online_update=true）");
    println!("  zhu-ye-updater check-once   启动时静默检查一次（结果写 update_status.json）");
}

/// `%APPDATA%\ai-zhu-ye-ime`。
fn appdata_root() -> Result<PathBuf, String> {
    std::env::var_os("APPDATA")
        .map(|root| PathBuf::from(root).join("ai-zhu-ye-ime"))
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

/// 检查或应用更新。`apply` 为 `false` 时只报告。
fn check_command(apply: bool) -> Result<(), String> {
    let config_path = config_path()?;
    let (config, diagnostic) = load_config(&config_path);
    if let Some(diagnostic) = diagnostic {
        eprintln!("配置警告: {diagnostic}");
    }

    // 默认关闭原则（P-03）：未显式开启时不发起任何网络请求。
    if !config.online_update {
        println!("在线更新已关闭（online_update=false），未发起任何网络请求。");
        println!("如需检查更新，请把 config.json 的 online_update 设为 true。");
        return Ok(());
    }

    let url =
        std::env::var("ZHU_YE_MANIFEST_URL").unwrap_or_else(|_| DEFAULT_MANIFEST_URL.to_owned());
    println!("清单地址: {url}");

    let manifest_text = http_get_text(&url)?;
    let manifest = parse_manifest(&manifest_text).map_err(|error| error.to_string())?;
    println!(
        "清单: schema={} 发布时间={} 包数={}",
        manifest.schema,
        manifest.published_at,
        manifest.packs.len()
    );

    let key = trusted_key()?;
    verify_signature_with_key(&manifest, &key).map_err(|error| error.to_string())?;
    println!("签名校验: 通过（内置公钥）");

    let engine_version = zhu_ye_core::core_version();
    let packs_dir = packs_dir()?;
    // 只考虑可分发包：基础包随安装只读交付，real/seed 是开发产物，都不在更新范围。
    let outdated: Vec<String> = find_outdated(&manifest, &packs_dir)
        .into_iter()
        .filter(|id| zhu_ye_core::is_distributable_pack(id))
        .collect();

    if outdated.is_empty() {
        println!("所有可更新包均为最新，无需更新。");
        return Ok(());
    }
    println!("可用更新: {outdated:?}");

    if !apply {
        println!("（未应用；执行 `zhu-ye-updater apply` 下载并应用）");
        return Ok(());
    }

    // 下载各包；跳过版本门槛不足的包。
    let base_url = url.rsplit_once('/').map(|(prefix, _)| prefix).unwrap_or("");
    let mut downloads: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for pack in &manifest.packs {
        if !outdated.contains(&pack.id) {
            continue;
        }
        if !version_at_least(engine_version, &pack.min_engine_version) {
            println!(
                "跳过 {}：需要引擎 >= {}，当前 {}",
                pack.id, pack.min_engine_version, engine_version
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
    let outcome = apply_release(&manifest, &key, &downloads, &packs_dir, engine_version)
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
    println!("更新完成。重启输入法后生效（P-12）。");
    Ok(())
}

/// 启动时静默检查一次（T-088 / FR-048，方案设计 §14.5.4）。
///
/// - `online_update=false` → 直接退出：不联网、不写状态文件（配合 TSF 侧开关判断，
///   达成验收"默认关闭零 spawn/零网络"）；
/// - 距上次检查不足 [`zhu_ye_core::CHECK_INTERVAL_DAYS`] 天 → 直接退出（保留旧状态）；
/// - 否则联网拉取并验签 manifest，把结果原子写入 `update_status.json`，并回写
///   `config.json` 的 `last_check`（S-8：以进入本命令时重读的配置为基线，保存前不再整体覆盖）；
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
        if outdated.is_empty() {
            return Ok(UpdateStatus::up_to_date(now));
        }
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

/// 供自检：确认更新器不会在关闭状态下联网（验收标准 FR-020「默认关闭」）。
#[must_use]
pub fn network_required(config: &ConfigFile) -> bool {
    config.online_update
}

#[cfg(test)]
mod tests {
    use super::network_required;
    use zhu_ye_core::pack_config::ConfigFile;

    #[test]
    fn 默认配置不要求网络() {
        assert!(!network_required(&ConfigFile::default()));
    }

    #[test]
    fn 显式开启后才要求网络() {
        let config = ConfigFile {
            online_update: true,
            ..ConfigFile::default()
        };
        assert!(network_required(&config));
    }
}
