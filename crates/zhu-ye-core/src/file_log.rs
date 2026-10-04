//! 单文件文件日志（第十一期 FR-060，T-091）：目录准备 / 级别门 / 追加写 /
//! 1 MiB 轮转。
//!
//! 「尽力而为、零失败影响」（诊断产品化设计 §2）：任何 IO 失败静默返回，
//! 不向调用方传播、不影响输入功能。无内部锁——调用方（TSF 文本服务）在
//! 事件线程内串行调用；`%LOCALAPPDATA%` 的解析在消费方完成，本模块只拿
//! `PathBuf`，保持 core 零 Windows 专有 API（AGENTS §4）。

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use crate::log_level::LogLevel;
use crate::user_store::unix_now;

/// 默认单文件大小上限（诊断产品化设计 §3）：1 MiB。
pub const DEFAULT_LOG_SIZE_LIMIT: u64 = 1024 * 1024;

/// 轮转文件名（与主文件同目录）：`ime.1.log`。
const ROTATED_FILE_NAME: &str = "ime.1.log";

/// 单文件日志写入器。
///
/// 轮转策略：写前检查当前文件大小，超过上限时把旧文件覆盖为
/// `ime.1.log`（先删再改名，Windows 不支持 rename 覆盖），再开新
/// `ime.log` 继续写；允许丢最后一条（尽力而为，不引入临时文件复杂度）。
pub struct FileLogger {
    /// 当前级别（config 解析结果）。
    level: LogLevel,
    /// 产品日志主文件路径（`%LOCALAPPDATA%\ai-zhu-ye-ime\logs\ime.log`）。
    path: PathBuf,
    /// 大小上限；0 = 不限（测试注入）。
    size_limit: u64,
}

impl FileLogger {
    /// 以默认 1 MiB 上限创建。
    #[must_use]
    pub fn new(path: PathBuf, level: LogLevel) -> Self {
        Self {
            level,
            path,
            size_limit: DEFAULT_LOG_SIZE_LIMIT,
        }
    }

    /// 测试注入：覆盖大小上限（0 = 不轮转）。
    #[must_use]
    pub fn with_size_limit(mut self, bytes: u64) -> Self {
        self.size_limit = bytes;
        self
    }

    /// 当前级别。
    #[must_use]
    pub fn level(&self) -> LogLevel {
        self.level
    }

    /// 确保父目录存在（`create_dir_all` 对已存在目录无操作）；失败返回
    /// `Err` 由调用方忽略（尽力而为）。
    pub fn ensure_dir(&self) -> std::io::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(())
    }

    /// 级别门 + 追加写。消息级别不高于配置级别才落盘；写前大小超限先轮转。
    /// 全部尽力而为：任一失败静默跳过。`tid` 由调用方传给行格式（core 不依赖
    /// Windows 专有 API，TSF 侧传 `GetCurrentThreadId()`）。
    pub fn write(&mut self, level: LogLevel, tid: u32, line: &str) {
        if level > self.level {
            return;
        }
        if self.size_limit > 0 {
            match fs::metadata(&self.path) {
                Ok(meta) if meta.len() > self.size_limit => self.rotate(),
                Ok(_) | Err(_) => {}
            }
        }
        // `create_dir_all` 幂等：目录存在时 µs 级无操作，仅发生在达到级别的消息上。
        let _ = self.ensure_dir();
        let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        else {
            return;
        };
        let _ = writeln!(file, "{}", Self::format_line(level, tid, line));
    }

    /// 轮转：旧 `ime.log` 覆盖为 `ime.1.log`。
    fn rotate(&mut self) {
        let rotated = self.path.with_file_name(ROTATED_FILE_NAME);
        let _ = fs::remove_file(&rotated);
        let _ = fs::rename(&self.path, &rotated);
    }

    /// 行格式：`[unix秒] pid=N tid=N <LEVEL> message`。
    ///
    /// `tid` 由调用方提供——core 不依赖 Windows 专有 API（AGENTS §4），
    /// TSF 侧传 `GetCurrentThreadId()`；测试传任意值。
    #[must_use]
    pub fn format_line(level: LogLevel, tid: u32, message: &str) -> String {
        format!(
            "[{}] pid={} tid={} <{}> {}",
            unix_now(),
            std::process::id(),
            tid,
            level.as_str(),
            message
        )
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use crate::log_level::LogLevel;

    use super::{FileLogger, ROTATED_FILE_NAME};

    /// 独立临时子目录（进程内计数保证唯一），测试结束不清理（Windows 文件句柄
    /// 释放后可删，但保留更利于失败排查；产物不入库）。
    fn temp_dir() -> PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!(
            "zy-log-test-{}-{}-{}",
            std::process::id(),
            stamp,
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ))
    }

    #[test]
    fn 追加写并带时间戳前缀() {
        let dir = temp_dir();
        let logger = FileLogger::new(dir.join("ime.log"), LogLevel::Warn);
        let mut logger = logger.with_size_limit(0);

        logger.write(LogLevel::Error, 42, "第一条错误");
        logger.write(LogLevel::Warn, 42, "第二条警告");

        let content = fs::read_to_string(dir.join("ime.log")).expect("日志文件可读");
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);
        for (index, line) in lines.iter().enumerate() {
            assert!(
                line.starts_with('[') && line.contains("pid=") && line.contains("tid="),
                "第 {index} 行缺少时间戳/pid/tid 前缀: {line}"
            );
        }
        assert!(lines[0].ends_with("<error> 第一条错误"));
        assert!(lines[1].ends_with("<warn> 第二条警告"));
    }

    #[test]
    fn 级别门拦截低于配置级别的消息() {
        let dir = temp_dir();
        let mut logger = FileLogger::new(dir.join("ime.log"), LogLevel::Warn).with_size_limit(0);

        logger.write(LogLevel::Info, 42, "信息级");
        logger.write(LogLevel::Debug, 42, "调试级");
        assert!(
            !dir.join("ime.log").exists(),
            "低于默认级别的消息不得创建文件"
        );

        // 调低级别后同一条消息落盘。
        logger.write(LogLevel::Warn, 42, "警告级");
        let content = fs::read_to_string(dir.join("ime.log")).expect("日志文件可读");
        assert_eq!(content.lines().count(), 1);
        assert!(content.contains("警告级"));
    }

    #[test]
    fn 目录不存在时自动创建() {
        let dir = temp_dir().join("nested").join("logs");
        let mut logger = FileLogger::new(dir.join("ime.log"), LogLevel::Warn).with_size_limit(0);

        logger.write(LogLevel::Error, 42, "创建目录后写入");
        assert!(fs::read_to_string(dir.join("ime.log"))
            .expect("日志文件可读")
            .contains("创建目录后写入"));
    }

    #[test]
    fn 大小超限后轮转到一号档案() {
        let dir = temp_dir();
        let mut logger = FileLogger::new(dir.join("ime.log"), LogLevel::Error).with_size_limit(32); // 注：上限极小而每条消息约 60 字节
        logger.write(LogLevel::Error, 42, "第一代日志内容填充足够长度以触发轮转");
        logger.write(LogLevel::Error, 42, "第二代新日志内容");

        let rotated = fs::read_to_string(dir.join(ROTATED_FILE_NAME)).expect("轮转档案存在");
        assert!(
            rotated.contains("第一代"),
            "旧内容进入 ime.1.log: {rotated}"
        );
        let current = fs::read_to_string(dir.join("ime.log")).expect("新文件继续写");
        assert!(
            current.contains("第二代"),
            "新内容继续写 ime.log: {current}"
        );
    }

    #[test]
    fn 零上限不轮转() {
        let dir = temp_dir();
        let mut logger = FileLogger::new(dir.join("ime.log"), LogLevel::Error).with_size_limit(0);
        for index in 0..5 {
            logger.write(LogLevel::Error, 42, &format!("第 {index} 条"));
        }
        assert!(!dir.join(ROTATED_FILE_NAME).exists());
        let content = fs::read_to_string(dir.join("ime.log")).expect("日志文件可读");
        assert_eq!(content.lines().count(), 5);
    }

    #[test]
    fn 行格式携带级别与线程编号() {
        let line = FileLogger::format_line(LogLevel::Debug, 4242, "zhu-ye: key 0x20");
        assert_eq!(
            line,
            format!(
                "[{}] pid={} tid=4242 <debug> zhu-ye: key 0x20",
                crate::user_store::unix_now(),
                std::process::id()
            )
        );
    }
}
