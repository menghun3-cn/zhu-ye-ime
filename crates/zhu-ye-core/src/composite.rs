//! 复合词典：把多个 v2 词典包合并为一个 `Dictionary + BigramModel + Translator`。
//!
//! M6-R（FR-015、方案设计 11.3）：运行时按配置启用的包列表依次 mmap 打开，
//! 查询时跨文件合并，候选生成/排序/分页/译文层完全不感知多文件存在。
//!
//! 合并规则：
//! - 同「拼音 + 词」去重，词频取 **max**（S-6 定稿）；
//! - bigram 同词对取 **max**；
//! - 译文取首个非空。
//!
//! **等价性门禁**：仅持有单个包时，`lookup`/`lookup_prefix` 的输出与直接使用
//! 该 `DictionaryFile` 逐项一致——合并保持文件内原顺序、再按词频降序稳定排序，
//! 与该实现自身的排序方式相同。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::bigram::BigramModel;
use crate::dict::{Dictionary, DictionaryEntry};
use crate::dict_loader::DictionaryFile;
use crate::translate::{TranslationDirection, Translator};

/// 跨文件合并的词典集合；每个包各自 mmap 只读打开。
#[derive(Debug, Clone, Default)]
pub struct CompositeDictionary {
    files: Vec<Arc<DictionaryFile>>,
}

impl CompositeDictionary {
    /// 用已打开的包构造。
    #[must_use]
    pub fn new(files: Vec<Arc<DictionaryFile>>) -> Self {
        Self { files }
    }

    /// 依次打开包文件；**任一包打开失败则跳过该包并返回诊断信息**，不阻断输入
    /// （NFR-009：绝不半加载，也不因单个坏包让输入法不可用）。
    ///
    /// 返回的第二个元素是每个被跳过包的说明，调用方负责记日志。
    #[must_use]
    pub fn from_paths(paths: &[PathBuf]) -> (Self, Vec<String>) {
        let mut files = Vec::new();
        let mut skipped = Vec::new();
        for path in paths {
            match DictionaryFile::open(path) {
                Ok(file) => files.push(Arc::new(file)),
                Err(error) => skipped.push(format!("跳过词典包 {}: {error}", path.display())),
            }
        }
        (Self { files }, skipped)
    }

    /// 已加载的包数量。
    #[must_use]
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// 是否没有任何可用包。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// 已加载包的文件路径与词条数，供自检与日志输出。
    #[must_use]
    pub fn describe(&self) -> Vec<(String, u64)> {
        self.files
            .iter()
            .map(|file| {
                let name = file
                    .path()
                    .map_or_else(|| "(内存)".to_owned(), |path| path.display().to_string());
                (name, file.entry_count())
            })
            .collect()
    }

    /// 收集各包查询结果并在需要时合并。
    ///
    /// **热路径快速通道**：只有一个包返回非空时直接返回该结果——它已经是该实现
    /// 自身的顺序（按词频降序），与合并结果逐项一致，因此无需分配哈希表。
    /// 未启用任何领域包（只开基础包）的常见场景走的就是这条路径。
    fn collect_merging<F>(&self, mut query: F) -> Vec<DictionaryEntry>
    where
        F: FnMut(&DictionaryFile) -> Vec<DictionaryEntry>,
    {
        let mut first: Option<Vec<DictionaryEntry>> = None;
        let mut all: Vec<DictionaryEntry> = Vec::new();
        let mut sources = 0usize;
        for file in &self.files {
            let found = query(file);
            if found.is_empty() {
                continue;
            }
            sources += 1;
            if sources == 1 {
                first = Some(found);
            } else {
                if let Some(pending) = first.take() {
                    all.extend(pending);
                }
                all.extend(found);
            }
        }
        match sources {
            0 => Vec::new(),
            1 => first.unwrap_or_default(),
            _ => merge_entries(all),
        }
    }
}

/// 跨文件合并词条：按「拼音 + 词」去重，词频取 max，译文取首个非空。
///
/// 保持首次出现顺序后再按词频降序**稳定**排序，因此单个包时与
/// `DictionaryFile` 自身的输出顺序完全一致。
fn merge_entries(entries: Vec<DictionaryEntry>) -> Vec<DictionaryEntry> {
    let mut order: Vec<(String, String)> = Vec::new();
    let mut merged: HashMap<(String, String), DictionaryEntry> = HashMap::new();
    for entry in entries {
        let key = (entry.pinyin.clone(), entry.word.clone());
        match merged.get_mut(&key) {
            Some(existing) => {
                existing.frequency = existing.frequency.max(entry.frequency);
                if existing.translation.is_none() {
                    existing.translation = entry.translation;
                }
            }
            None => {
                order.push(key.clone());
                merged.insert(key, entry);
            }
        }
    }
    let mut out: Vec<DictionaryEntry> = order
        .into_iter()
        .filter_map(|key| merged.remove(&key))
        .collect();
    out.sort_by_key(|entry| std::cmp::Reverse(entry.frequency));
    out
}

impl Dictionary for CompositeDictionary {
    fn lookup(&self, pinyin: &str) -> Vec<DictionaryEntry> {
        self.collect_merging(|file| file.lookup(pinyin))
    }

    fn lookup_prefix(&self, pinyin_prefix: &str) -> Vec<DictionaryEntry> {
        self.collect_merging(|file| file.lookup_prefix(pinyin_prefix))
    }
}

impl BigramModel for CompositeDictionary {
    fn frequency(&self, previous: &str, word: &str) -> u64 {
        self.files
            .iter()
            .map(|file| file.frequency(previous, word))
            .max()
            .unwrap_or(0)
    }

    fn successors(&self, previous: &str, limit: usize) -> Vec<(String, u64)> {
        // 各源分别取后继后按词合并取 max（与 frequency 同口径），再按频率降序截取。
        let mut merged: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
        for file in &self.files {
            for (word, frequency) in file.successors(previous, limit) {
                let entry = merged.entry(word).or_insert(0);
                *entry = (*entry).max(frequency);
            }
        }
        let mut items: Vec<(String, u64)> = merged.into_iter().collect();
        items.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        items.truncate(limit);
        items
    }
}

impl Translator for CompositeDictionary {
    fn translate(&self, text: &str, direction: TranslationDirection) -> Option<String> {
        self.files
            .iter()
            .find_map(|file| file.translate(text, direction))
    }
}

/// 判断路径列表里是否存在可读的包文件；供装配前做快速决策。
#[must_use]
pub fn any_exists(paths: &[PathBuf]) -> bool {
    paths.iter().any(|path| path.is_file())
}

/// 把包 id 拼成 `<dir>/<id>.zyct`（方案设计 11.3 的包文件命名约定）。
#[must_use]
pub fn pack_path(dir: &Path, pack_id: &str) -> PathBuf {
    dir.join(format!("{pack_id}.zyct"))
}

#[cfg(test)]
mod tests {
    use super::{merge_entries, pack_path, CompositeDictionary};
    use crate::bigram::BigramModel;
    use crate::dict::{Dictionary, DictionaryEntry, InMemoryDictionary};
    use crate::translate::{TranslationDirection, Translator};
    use std::path::Path;
    use std::sync::Arc;

    fn entry(word: &str, pinyin: &str, frequency: u64) -> DictionaryEntry {
        DictionaryEntry::new(word, pinyin, frequency)
    }

    #[test]
    fn 合并去重且词频取max() {
        let merged = merge_entries(vec![
            entry("中国", "zhongguo", 10),
            entry("中国", "zhongguo", 30),
            entry("中", "zhong", 5),
        ]);
        assert_eq!(merged.len(), 2);
        let china = merged.iter().find(|e| e.word == "中国").unwrap();
        assert_eq!(china.frequency, 30);
    }

    #[test]
    fn 合并保留译文且取首个非空() {
        let merged = merge_entries(vec![
            entry("中国", "zhongguo", 10),
            entry("中国", "zhongguo", 30).with_translation("China"),
        ]);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].translation.as_deref(), Some("China"));
        assert_eq!(merged[0].frequency, 30);
    }

    #[test]
    fn 合并按词频降序且同频保持原顺序() {
        let merged = merge_entries(vec![
            entry("低", "di", 1),
            entry("高", "gao", 9),
            entry("同甲", "tong", 5),
            entry("同乙", "tong2", 5),
        ]);
        let words: Vec<&str> = merged.iter().map(|e| e.word.as_str()).collect();
        assert_eq!(words, vec!["高", "同甲", "同乙", "低"]);
    }

    #[test]
    fn 同拼音不同词不去重() {
        let merged = merge_entries(vec![entry("先", "xian", 70), entry("西安", "xian", 60)]);
        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn 空集合查询安全() {
        let composite = CompositeDictionary::default();
        assert!(composite.is_empty());
        assert_eq!(composite.file_count(), 0);
        assert!(composite.lookup("nihao").is_empty());
        assert!(composite.lookup_prefix("ni").is_empty());
        assert_eq!(composite.frequency("你好", "世界"), 0);
        assert_eq!(
            composite.translate("你好", TranslationDirection::ZhToEn),
            None
        );
    }

    /// 等价性门禁：单个包时输出与直接查询该包逐项一致。
    #[test]
    fn 单包时与直接查询逐项一致() {
        let dir = std::env::temp_dir().join(format!("zhu-ye-composite-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("equiv.zyct");
        let entries = vec![
            entry("你好", "nihao", 100).with_translation("hello"),
            entry("尼好", "nihao", 1),
            entry("你", "ni", 65),
            entry("泥", "ni", 20),
            entry("拟", "ni", 15),
        ];
        let bytes = crate::build_v2(&entries, &[]).unwrap();
        std::fs::write(&path, &bytes).unwrap();

        let (composite, skipped) = CompositeDictionary::from_paths(std::slice::from_ref(&path));
        assert!(skipped.is_empty(), "不应跳过任何包: {skipped:?}");
        assert_eq!(composite.file_count(), 1);

        let single = crate::DictionaryFile::open(&path).unwrap();
        for key in ["nihao", "ni", "zzz"] {
            assert_eq!(
                composite.lookup(key),
                single.lookup(key),
                "lookup({key}) 应与单词典逐项一致"
            );
        }
        for key in ["n", "ni", "nih", "zzz"] {
            assert_eq!(
                composite.lookup_prefix(key),
                single.lookup_prefix(key),
                "lookup_prefix({key}) 应与单词典逐项一致"
            );
        }

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 坏包被跳过且不阻断其余包() {
        let dir = std::env::temp_dir().join(format!("zhu-ye-composite-bad-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let good = dir.join("good.zyct");
        let bad = dir.join("bad.zyct");
        let bytes = crate::build_v2(&[entry("你好", "nihao", 100)], &[]).unwrap();
        std::fs::write(&good, &bytes).unwrap();
        std::fs::write(&bad, b"not a dictionary").unwrap();

        let (composite, skipped) =
            CompositeDictionary::from_paths(&[bad.clone(), good.clone(), dir.join("absent.zyct")]);
        assert_eq!(composite.file_count(), 1);
        assert_eq!(skipped.len(), 2, "坏包与缺失包都应记入诊断: {skipped:?}");
        assert!(composite.lookup("nihao").iter().any(|e| e.word == "你好"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 包路径按id拼接() {
        assert_eq!(
            pack_path(Path::new("C:\\packs"), "it"),
            Path::new("C:\\packs\\it.zyct")
        );
    }

    #[test]
    fn 跨包查询合并两包词条() {
        // 直接验证合并语义：两个内存词典的内容合并后，同词取 max、异词并存。
        let left = InMemoryDictionary::from_entries(vec![
            entry("编程", "biancheng", 100),
            entry("计算", "jisuan", 50),
        ]);
        let right = InMemoryDictionary::from_entries(vec![
            entry("编程", "biancheng", 900),
            entry("算法", "suanfa", 80),
        ]);
        let mut collected = left.lookup("biancheng");
        collected.extend(right.lookup("biancheng"));
        let merged = merge_entries(collected);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].frequency, 900);
    }

    #[test]
    fn 描述输出包名与词条数() {
        let dir =
            std::env::temp_dir().join(format!("zhu-ye-composite-desc-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("it.zyct");
        let bytes = crate::build_v2(&[entry("编程", "biancheng", 100)], &[]).unwrap();
        std::fs::write(&path, &bytes).unwrap();

        let (composite, _) = CompositeDictionary::from_paths(&[path]);
        let described = composite.describe();
        assert_eq!(described.len(), 1);
        assert_eq!(described[0].1, 1);
        assert!(described[0].0.ends_with("it.zyct"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 可用集合判断跳过缺失路径() {
        let dir = std::env::temp_dir().join(format!("zhu-ye-composite-any-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let existing = dir.join("a.zyct");
        std::fs::write(&existing, b"x").unwrap();
        assert!(super::any_exists(&[dir.join("absent.zyct"), existing]));
        assert!(!super::any_exists(&[dir.join("absent.zyct")]));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 复合词典实现三大接口() {
        let composite: Arc<dyn Dictionary> = Arc::new(CompositeDictionary::default());
        assert!(composite.lookup("x").is_empty());
        let bigram: Arc<dyn BigramModel> = Arc::new(CompositeDictionary::default());
        assert_eq!(bigram.frequency("a", "b"), 0);
        let translator: Arc<dyn Translator> = Arc::new(CompositeDictionary::default());
        assert_eq!(translator.zh_to_en("a"), None);
    }
}
