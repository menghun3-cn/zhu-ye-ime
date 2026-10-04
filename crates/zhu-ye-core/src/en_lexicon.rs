//! 英文词表文件（`en.zyen`）格式定义与 mmap 只读加载器（T-085，方案设计 14.2）。
//!
//! 背景：第五期（T-064）英文词表以静态表内嵌于 `en_words.rs`（10k 规模）。
//! 第九期 ECDICT 全量约 77 万词条，静态内嵌不再成立（源码体积/编译时间），
//! 改为发行包内独立只读文件 `en.zyen`，由 `zhu-ye-dict en-build` 编译、
//! core 侧 mmap 加载；`en_words.rs` 静态表保留为**无文件回退**（启动不依赖）。
//!
//! 格式 v1（魔数 `ZYEN`，全部多字节字段小端）：
//!
//! ```text
//! 头部 96 字节：
//!   [0..4)    magic         "ZYEN"
//!   [4..8)    version       u32 = 1
//!   [8..16)   count         u64 记录数
//!   [16..24)  reserved      u64 = 0
//!   [24..32)  records_off   u64 记录区起始（= 96）
//!   [32..40)  pool_off      u64 文本池起始
//!   [40..48)  anchors_off   u64 锚区起始（锚在池之后）
//!   [48..56)  pool_len      u64 文本池字节数
//!   [56..60)  anchor_stride u32 锚步长（每条记录一个锚；0 = 无锚）
//!   [60..64)  anchor_count  u32 锚条数
//!   [64..96)  content_hash  [u8;32] 覆盖 [96..文件尾)
//! 记录区：count 条 × 11 字节，按 norm 字节序升序：
//!   [0..3) norm_off u24（相对池首）
//!   [3..6) word_off u24（相对池首；word_len == 0 时忽略）
//!   [6)    norm_len u8
//!   [7)    word_len u8（0 = word 与 norm 共享同一段，如全小写原形）
//!   [8..11) rank u24（常用度排序号，越小越常用；norm 升序与 rank 序正交，
//!          查询命中区间后按 rank 排序截断——与第五期静态表口径一致）
//! 锚区：anchor_count 条 × 12 字节，桶首记录的下标与 norm 前 8 字节：
//!   [0..8)  key_pfx [u8;8]（norm 前 8 字节，不足零填充）
//!   [8..12) idx u32
//! ```
//!
//! 查询策略：先按锚定位桶首（约少 10 次比较），再做 `partition_point` 二分
//! 找到第一个 `norm >= 前缀` 的记录，随后线性扫描前缀区间并提前截断。
//! 加载时全量校验：魔数/版本/内容 SHA-256/分区边界/池 UTF-8/记录升序/锚单调。

use std::fmt;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;

use memmap2::{Mmap, MmapOptions};
use sha2::{Digest, Sha256};

use crate::{Error, Result};

/// 文件魔数 `ZYEN`。
pub const EN_MAGIC: [u8; 4] = *b"ZYEN";

/// 当前英文词表文件格式版本。
pub const EN_VERSION: u32 = 1;

/// 头部固定大小。
pub const EN_HEADER_SIZE: usize = 96;

/// 记录固定大小（11 字节：2×u24 偏移 + 2×u8 长度 + u24 rank）。
pub const EN_RECORD_SIZE: usize = 11;

/// 锚记录固定大小。
pub const EN_ANCHOR_SIZE: usize = 12;

/// 默认锚步长：每 1024 条记录一个桶。
pub const EN_ANCHOR_STRIDE: u32 = 1024;

/// 词条长度上限（清洗期保证；u8 长度字段可容纳）。
const MAX_WORD_BYTES: usize = 255;

/// 编译输入：一条英文词条，`records` 必须按 **norm 字节序升序**传入
/// （二分依赖）；`rank` 为常用度排序号（越小越常用，构建期按词频赋值，
/// 决定候选展示顺序）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnRecord<'a> {
    /// 小写归一化查键（ASCII，唯一）。
    pub norm: &'a str,
    /// 上屏原形（保留大小写；与 norm 相同时 word_len 编译为 0 共享池段）。
    pub word: &'a str,
    /// 常用度排序号（越小越常用）。
    pub rank: u32,
}

/// 词条长度上限常量暴露给构建侧校验。
#[must_use]
pub const fn en_word_length_limit() -> usize {
    MAX_WORD_BYTES
}

/// u24 小端读取（3 字节，0..2^24）。
fn u24_at(bytes: &[u8], at: usize) -> u32 {
    u32::from(bytes[at]) | (u32::from(bytes[at + 1]) << 8) | (u32::from(bytes[at + 2]) << 16)
}

/// u24 小端写入（值必须 < 2^24）。
fn u24_put(out: &mut Vec<u8>, value: u32) {
    debug_assert!(value < (1 << 24), "u24 溢出: {value}");
    out.push((value & 0xFF) as u8);
    out.push(((value >> 8) & 0xFF) as u8);
    out.push(((value >> 16) & 0xFF) as u8);
}

/// 解析后的文件头。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnHeader {
    pub count: u64,
    pub records_off: u64,
    pub pool_off: u64,
    pub anchors_off: u64,
    pub pool_len: u64,
    pub anchor_stride: u32,
    pub anchor_count: u32,
    pub content_hash: [u8; 32],
}

impl EnHeader {
    /// 从文件头 96 字节解析；校验魔数/版本/边界。
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < EN_HEADER_SIZE {
            return Err(Error::Dictionary("en 词表文件过短".to_owned()));
        }
        if bytes[0..4] != EN_MAGIC {
            return Err(Error::Dictionary(
                "en 词表魔数不符（非 ZYEN 文件）".to_owned(),
            ));
        }
        let version = u32::from_le_bytes(bytes[4..8].try_into().expect("4 字节"));
        if version != EN_VERSION {
            return Err(Error::Dictionary(format!(
                "en 词表版本不符：文件 {version}，支持 {EN_VERSION}"
            )));
        }
        let read_u64 =
            |at: usize| u64::from_le_bytes(bytes[at..at + 8].try_into().expect("8 字节"));
        let read_u32 =
            |at: usize| u32::from_le_bytes(bytes[at..at + 4].try_into().expect("4 字节"));
        let header = Self {
            count: read_u64(8),
            records_off: read_u64(24),
            pool_off: read_u64(32),
            anchors_off: read_u64(40),
            pool_len: read_u64(48),
            anchor_stride: read_u32(56),
            anchor_count: read_u32(60),
            content_hash: bytes[64..96].try_into().expect("32 字节"),
        };
        header.validate_boundaries()?;
        Ok(header)
    }

    fn validate_boundaries(&self) -> Result<()> {
        if self.count == 0 {
            return Err(Error::Dictionary("en 词表为空（count=0）".to_owned()));
        }
        let records_bytes = self
            .count
            .checked_mul(EN_RECORD_SIZE as u64)
            .ok_or_else(|| Error::Dictionary("en 词表记录区溢出".to_owned()))?;
        let records_end = self
            .records_off
            .checked_add(records_bytes)
            .ok_or_else(|| Error::Dictionary("en 词表记录区偏移溢出".to_owned()))?;
        if records_end != self.pool_off {
            return Err(Error::Dictionary(format!(
                "en 词表格局异常：记录区结束 {} 与池起始 {} 不衔接",
                records_end, self.pool_off
            )));
        }
        let pool_end = self
            .pool_off
            .checked_add(self.pool_len)
            .ok_or_else(|| Error::Dictionary("en 词表池区溢出".to_owned()))?;
        let anchor_bytes = (self.anchor_count as u64)
            .checked_mul(EN_ANCHOR_SIZE as u64)
            .ok_or_else(|| Error::Dictionary("en 词表锚区溢出".to_owned()))?;
        let expected_anchors = if self.anchor_stride > 0 {
            self.count.div_ceil(u64::from(self.anchor_stride))
        } else {
            0
        };
        if u64::from(self.anchor_count) != expected_anchors {
            return Err(Error::Dictionary(format!(
                "en 词表锚数不符：文件 {}，期望 {}（stride={}）",
                self.anchor_count, expected_anchors, self.anchor_stride
            )));
        }
        let _ = self
            .anchors_off
            .checked_add(anchor_bytes)
            .ok_or_else(|| Error::Dictionary("en 词表锚区偏移溢出".to_owned()))?;
        if pool_end != self.anchors_off {
            return Err(Error::Dictionary(format!(
                "en 词表格局异常：池结束 {} 与锚起始 {} 不衔接",
                pool_end, self.anchors_off
            )));
        }
        if self.pool_len == 0 {
            return Err(Error::Dictionary("en 词表文本池为空".to_owned()));
        }
        Ok(())
    }
}

/// 把记录编译为内存中的完整 `en.zyen` 字节流（构建侧与测试共用）。
///
/// `records` 必须：非空、按 norm 字节序**升序**排列（二分依赖）、norm 小写
/// ASCII 唯一、每条 norm 非空且 ≤ `MAX_WORD_BYTES` 字节。rank 由调用方按
/// 常用度赋值（不影响存储顺序）。
#[must_use]
pub fn compile_en_records(records: &[EnRecord<'_>]) -> Vec<u8> {
    assert!(!records.is_empty(), "en 词表编译输入为空");
    for (i, r) in records.iter().enumerate() {
        assert!(r.norm.len() <= MAX_WORD_BYTES, "norm 超长: {}", r.norm);
        assert!(r.word.len() <= MAX_WORD_BYTES, "word 超长: {}", r.word);
        assert!(!r.norm.is_empty(), "norm 为空（第 {i} 条）");
        assert!(
            r.norm
                .bytes()
                .all(|b| b.is_ascii() && !b.is_ascii_uppercase()),
            "norm 必须 ASCII 无大写（允许 '、-、空格等）: {}",
            r.norm
        );
        if i > 0 {
            assert!(
                r.norm.as_bytes() > records[i - 1].norm.as_bytes(),
                "norm 必须严格升序（重复或乱序）: {}",
                r.norm
            );
        }
    }

    // 单文本池：norm 全量入池；word 与 norm 相同时共享池段（word_len = 0）。
    let mut records_bytes: Vec<u8> = Vec::with_capacity(records.len() * EN_RECORD_SIZE);
    let mut pool: Vec<u8> = Vec::new();
    let mut raw: Vec<([u8; 8], u32)> = Vec::new(); // (桶首 key_pfx, idx)

    for (idx, r) in records.iter().enumerate() {
        let norm_off = pool.len() as u32;
        pool.extend_from_slice(r.norm.as_bytes());
        let (word_off, word_len) = if r.word == r.norm {
            (0u32, 0u8) // 共享
        } else {
            let wo = pool.len() as u32;
            pool.extend_from_slice(r.word.as_bytes());
            (wo, r.word.len() as u8)
        };
        u24_put(&mut records_bytes, norm_off);
        u24_put(&mut records_bytes, word_off);
        records_bytes.push(r.norm.len() as u8);
        records_bytes.push(word_len);
        u24_put(&mut records_bytes, r.rank);
        if (idx % EN_ANCHOR_STRIDE as usize) == 0 {
            let mut pfx = [0u8; 8];
            let n = r.norm.as_bytes();
            let take = n.len().min(8);
            pfx[..take].copy_from_slice(&n[..take]);
            raw.push((pfx, idx as u32));
        }
    }
    debug_assert_eq!(
        raw.len() as u32,
        (records.len() as u32).div_ceil(EN_ANCHOR_STRIDE)
    );

    let mut anchors: Vec<u8> = Vec::with_capacity(raw.len() * EN_ANCHOR_SIZE);
    for (pfx, idx) in &raw {
        anchors.extend_from_slice(pfx);
        anchors.extend_from_slice(&idx.to_le_bytes());
    }

    let count = records.len() as u64;
    let records_off = EN_HEADER_SIZE as u64;
    let pool_off = records_off + count * EN_RECORD_SIZE as u64;
    let anchors_off = pool_off + pool.len() as u64;
    let mut out =
        Vec::with_capacity(EN_HEADER_SIZE + records_bytes.len() + pool.len() + anchors.len());
    out.extend_from_slice(&EN_MAGIC);
    out.extend_from_slice(&EN_VERSION.to_le_bytes());
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&0u64.to_le_bytes()); // reserved
    out.extend_from_slice(&records_off.to_le_bytes());
    out.extend_from_slice(&pool_off.to_le_bytes());
    out.extend_from_slice(&anchors_off.to_le_bytes());
    out.extend_from_slice(&(pool.len() as u64).to_le_bytes());
    out.extend_from_slice(&EN_ANCHOR_STRIDE.to_le_bytes());
    out.extend_from_slice(&(raw.len() as u32).to_le_bytes());
    out.resize(EN_HEADER_SIZE, 0u8); // content_hash 占位
    out.extend_from_slice(&records_bytes);
    out.extend_from_slice(&pool);
    out.extend_from_slice(&anchors);
    let content_hash: [u8; 32] = Sha256::digest(&out[EN_HEADER_SIZE..]).into();
    out[64..96].copy_from_slice(&content_hash);
    out
}

/// 数据后端：发行运行时文件 mmap；测试/构建自检用堆内存。
#[derive(Clone)]
enum MapData {
    File(Arc<Mmap>),
    Heap(Arc<Vec<u8>>),
}

impl MapData {
    #[allow(clippy::needless_borrows_for_generic_args)]
    fn as_slice(&self) -> &[u8] {
        match self {
            Self::File(m) => m.as_ref(),
            Self::Heap(v) => v.as_slice(),
        }
    }

    fn len(&self) -> usize {
        self.as_slice().len()
    }
}

/// mmap 英文词表；加载方持有数据与头部，字符串按偏移即时读取。
#[derive(Clone)]
pub struct EnLexicon {
    data: MapData,
    header: EnHeader,
    /// 来源路径；内存构造（测试）时为 `None`。
    path: Option<Arc<std::path::PathBuf>>,
}

impl fmt::Debug for EnLexicon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EnLexicon")
            .field("file_size", &self.data.len())
            .field("count", &self.header.count)
            .field("anchor_count", &self.header.anchor_count)
            .finish()
    }
}

impl EnLexicon {
    /// 打开并校验英文词表；失败返回可诊断错误，不产生部分可用状态。
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path)
            .map_err(|error| Error::Dictionary(format!("打开英文词表失败: {error}")))?;
        let map = unsafe { MmapOptions::new().map(&file) }
            .map_err(|error| Error::Dictionary(format!("英文词表内存映射失败: {error}")))?;
        let mut lexicon = Self::from_map(map)?;
        lexicon.path = Some(Arc::new(path.to_path_buf()));
        Ok(lexicon)
    }

    /// 从内存字节构造（构建侧自检与测试用）。
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let backed = MapData::Heap(Arc::new(bytes.to_vec()));
        let header = EnHeader::from_bytes(backed.as_slice())?;
        let actual_hash: [u8; 32] = Sha256::digest(&backed.as_slice()[EN_HEADER_SIZE..]).into();
        if actual_hash != header.content_hash {
            return Err(Error::Dictionary(
                "en 词表内容 SHA-256 不匹配，文件可能已损坏".to_owned(),
            ));
        }
        let lexicon = Self {
            data: backed,
            header,
            path: None,
        };
        lexicon.validate_layout()?;
        Ok(lexicon)
    }

    fn from_map(map: Mmap) -> Result<Self> {
        let header = EnHeader::from_bytes(&map)?;
        let actual_hash: [u8; 32] = Sha256::digest(&map[EN_HEADER_SIZE..]).into();
        if actual_hash != header.content_hash {
            return Err(Error::Dictionary(
                "en 词表内容 SHA-256 不匹配，文件可能已损坏".to_owned(),
            ));
        }
        let lexicon = Self {
            data: MapData::File(Arc::new(map)),
            header,
            path: None,
        };
        lexicon.validate_layout()?;
        Ok(lexicon)
    }

    /// 返回来源路径；内存构造的实例返回 `None`。
    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref().map(std::path::PathBuf::as_path)
    }

    /// 返回词条总数。
    #[must_use]
    pub fn count(&self) -> u64 {
        self.header.count
    }

    /// 返回数据总字节数。
    #[must_use]
    pub fn file_size(&self) -> usize {
        self.data.len()
    }

    /// 返回解析后的头部元数据。
    #[must_use]
    pub fn header(&self) -> &EnHeader {
        &self.header
    }

    fn records(&self) -> &[u8] {
        let start = self.header.records_off as usize;
        let end = start + self.header.count as usize * EN_RECORD_SIZE;
        &self.data.as_slice()[start..end]
    }

    fn pool(&self) -> &[u8] {
        let start = self.header.pool_off as usize;
        let end = start + self.header.pool_len as usize;
        &self.data.as_slice()[start..end]
    }

    fn anchors(&self) -> &[u8] {
        let start = self.header.anchors_off as usize;
        let end = start + self.header.anchor_count as usize * EN_ANCHOR_SIZE;
        &self.data.as_slice()[start..end]
    }

    /// 读取第 `idx` 条记录的三要素；校验期已保证边界合法（panic-safe by layout 校验）。
    fn record_texts(&self, idx: usize) -> (&str, &str, u32) {
        let rec = &self.records()[idx * EN_RECORD_SIZE..(idx + 1) * EN_RECORD_SIZE];
        let norm_off = u24_at(rec, 0) as usize;
        let word_off = u24_at(rec, 3) as usize;
        let norm_len = rec[6] as usize;
        let word_len = rec[7] as usize;
        let rank = u24_at(rec, 8);
        let pool = self.pool();
        let norm = std::str::from_utf8(&pool[norm_off..norm_off + norm_len])
            .expect("池 UTF-8 已在校验期确认");
        let word = if word_len == 0 {
            norm
        } else {
            std::str::from_utf8(&pool[word_off..word_off + word_len])
                .expect("池 UTF-8 已在校验期确认")
        };
        (norm, word, rank)
    }

    fn norm_at(&self, idx: usize) -> &str {
        self.record_texts(idx).0
    }

    fn validate_layout(&self) -> Result<()> {
        // 文件总长匹配（头部 + 记录 + 池 + 锚）。
        let expected_len =
            (self.header.anchors_off as usize) + self.header.anchor_count as usize * EN_ANCHOR_SIZE;
        if self.data.len() != expected_len {
            return Err(Error::Dictionary(format!(
                "en 词表总长不符：文件 {}，布局期望 {}",
                self.data.len(),
                expected_len
            )));
        }
        // 池必须是合法 UTF-8（所有文本即时读取依赖此前提）。
        let pool = self.pool();
        std::str::from_utf8(pool)
            .map_err(|_| Error::Dictionary("en 词表文本池非 UTF-8".to_owned()))?;
        // 逐条记录校验（并发分块）：偏移/长度边界 + ASCII norm + rank 上限；
        // 跨块边界只校验各块首尾 norm 的升序衔接。
        let count = self.header.count as usize;
        let records = self.records();
        let chunks = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
            .min(8);
        let per_chunk = count.div_ceil(chunks.max(1));
        let mut first_errors: Vec<(usize, String)> = Vec::new();
        std::thread::scope(|scope| {
            let mut handles = Vec::with_capacity(chunks);
            for c in 0..chunks {
                let start = c * per_chunk;
                let end = (start + per_chunk).min(count);
                if start >= end {
                    continue;
                }
                handles.push(scope.spawn(move || {
                    let mut prev: Option<&[u8]> = None;
                    let mut error: Option<(usize, String)> = None;
                    for idx in start..end {
                        let rec = &records[idx * EN_RECORD_SIZE..(idx + 1) * EN_RECORD_SIZE];
                        let norm_off = u24_at(rec, 0) as usize;
                        let word_off = u24_at(rec, 3) as usize;
                        let norm_len = rec[6] as usize;
                        let word_len = rec[7] as usize;
                        let rank = u24_at(rec, 8);
                        if norm_len == 0
                            || norm_off + norm_len > pool.len()
                            || (word_len > 0 && word_off + word_len > pool.len())
                        {
                            error = Some((
                                idx,
                                format!(
                                    "en 词表记录 {idx} 越界（norm@{norm_off}+{norm_len} word@{word_off}+{word_len}，池 {}）",
                                    pool.len()
                                ),
                            ));
                            break;
                        }
                        if rank >= count as u32 {
                            error = Some((
                                idx,
                                format!("en 词表记录 {idx} rank 越界（{rank} ≥ count {count}）"),
                            ));
                            break;
                        }
                        let norm = &pool[norm_off..norm_off + norm_len];
                        if !norm.iter().all(|b| b.is_ascii() && !b.is_ascii_uppercase()) {
                            error = Some((
                                idx,
                                format!(
                                    "en 词表记录 {idx} norm 含大写或非 ASCII: {}",
                                    String::from_utf8_lossy(norm)
                                ),
                            ));
                            break;
                        }
                        if let Some(prev) = prev {
                            if prev >= norm {
                                error = Some((
                                    idx,
                                    format!(
                                        "en 词表记录 {idx} 未按 norm 严格升序（{} vs {}）",
                                        String::from_utf8_lossy(prev),
                                        String::from_utf8_lossy(norm)
                                    ),
                                ));
                                break;
                            }
                        }
                        prev = Some(norm);
                    }
                    (start, error)
                }));
            }
            for handle in handles {
                let (start, error) = handle.join().expect("校验线程运行");
                if let Some((_, message)) = error {
                    first_errors.push((start, message));
                }
            }
        });
        if let Some((_, message)) = first_errors.into_iter().min() {
            return Err(Error::Dictionary(message));
        }
        // 升序链跨块衔接：逐块首 norm 必须 > 前块尾 norm（块内已自检升序）。
        let mut prev_tail: Option<&[u8]> = None;
        for chunk_start in (0..count).step_by(per_chunk) {
            let end = (chunk_start + per_chunk).min(count);
            if chunk_start >= end {
                break;
            }
            let first_rec =
                &records[chunk_start * EN_RECORD_SIZE..(chunk_start + 1) * EN_RECORD_SIZE];
            let first_norm = &pool[u24_at(first_rec, 0) as usize
                ..u24_at(first_rec, 0) as usize + first_rec[6] as usize];
            if let Some(prev_tail) = prev_tail {
                if prev_tail >= first_norm {
                    return Err(Error::Dictionary(format!(
                        "en 词表记录 {chunk_start} 跨块升序破坏（{prev_tail:?} vs {first_norm:?}）"
                    )));
                }
            }
            let last_rec = &records[(end - 1) * EN_RECORD_SIZE..end * EN_RECORD_SIZE];
            prev_tail = Some(
                &pool[u24_at(last_rec, 0) as usize
                    ..u24_at(last_rec, 0) as usize + last_rec[6] as usize],
            );
        }
        // 锚桶：idx 单调且复位到桶首记录；key_pfx 单调。
        if self.header.anchor_count > 0 {
            let stride = self.header.anchor_stride as u64;
            let mut prev_pfx: Option<[u8; 8]> = None;
            let mut prev_idx: Option<u32> = None;
            for k in 0..self.header.anchor_count as usize {
                let anchor = &self.anchors()[k * EN_ANCHOR_SIZE..(k + 1) * EN_ANCHOR_SIZE];
                let pfx: [u8; 8] = anchor[0..8].try_into().expect("8 字节");
                let idx = u32::from_le_bytes(anchor[8..12].try_into().expect("4 字节")) as u64;
                if idx != k as u64 * stride {
                    return Err(Error::Dictionary(format!(
                        "en 词表锚 {k} 下标不符（{idx}，期望 {}）",
                        k as u64 * stride
                    )));
                }
                if let Some(prev_pfx) = prev_pfx {
                    if prev_pfx.as_slice() >= pfx.as_slice() {
                        return Err(Error::Dictionary("en 词表锚 key_pfx 未单调递增".to_owned()));
                    }
                }
                if let Some(prev_idx) = prev_idx {
                    if idx <= u64::from(prev_idx) {
                        return Err(Error::Dictionary("en 词表锚 idx 未递增".to_owned()));
                    }
                }
                // 桶首记录的 norm 前 8 字节必须与 key_pfx 一致。
                let norm = self.norm_at(idx as usize);
                let mut expect = [0u8; 8];
                let n = norm.as_bytes();
                let take = n.len().min(8);
                expect[..take].copy_from_slice(&n[..take]);
                if expect != pfx {
                    return Err(Error::Dictionary(format!(
                        "en 词表锚 {k} key_pfx 与桶首 norm 不符"
                    )));
                }
                prev_pfx = Some(pfx);
                prev_idx = Some(idx as u32);
            }
        }
        Ok(())
    }

    /// 第一个 `norm >= 前缀` 的记录下标（利用锚桶定位缩小二分范围）。
    fn lower_bound(&self, prefix: &str) -> usize {
        let count = self.header.count as usize;
        let from = if self.header.anchor_count > 0 {
            let p8 = pfx8(prefix);
            let anchors = self.anchors();
            // 手写二分：最后一个 key_pfx <= p8 的锚（桶首起点）。
            let mut lo = 0usize;
            let mut hi = self.header.anchor_count as usize;
            while lo < hi {
                let mid = (lo + hi) / 2;
                let anchor = &anchors[mid * EN_ANCHOR_SIZE..(mid + 1) * EN_ANCHOR_SIZE];
                let pfx: [u8; 8] = anchor[0..8].try_into().expect("8 字节");
                if pfx <= p8 {
                    lo = mid + 1;
                } else {
                    hi = mid;
                }
            }
            if lo == 0 {
                0
            } else {
                let anchor = &anchors[(lo - 1) * EN_ANCHOR_SIZE..lo * EN_ANCHOR_SIZE];
                u32::from_le_bytes(anchor[8..12].try_into().expect("4 字节")) as usize
            }
        } else {
            0
        };
        // 从桶首起二分：第一个 norm >= prefix 的记录。
        let mut lo = from;
        let mut hi = count;
        while lo < hi {
            let mid = (lo + hi) / 2;
            if is_norm_less_than(self.norm_at(mid), prefix) {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        lo
    }

    /// 按输入前缀查询英文候选（命中 ≤limit 条，按 rank 升序即常用度序；未命中返回空）。
    ///
    /// 输入任意大小写，内部小写化后匹配查键；前缀区间内按 rank 排序后截取
    /// 前 `limit` 条（与第五期静态表口径一致，不依赖区间顺序）。区间命中
    /// 超出 `k` 时用最大堆保顶 `limit` 个最小 rank，避免整区间排序的长尾
    /// （短前缀如 "a" 命中逾十万条）。
    #[must_use]
    pub fn words_with_prefix(&self, prefix: &str, limit: usize) -> Vec<(&str, u32)> {
        if prefix.is_empty() || limit == 0 {
            return Vec::new();
        }
        let p = prefix.to_ascii_lowercase();
        let start = self.lower_bound(&p);
        let count = self.header.count as usize;
        // 小堆（最大堆）：堆顶是当前最差（rank 最大）的候选；超出 limit 时挤出。
        let mut top: std::collections::BinaryHeap<(u32, usize)> =
            std::collections::BinaryHeap::with_capacity(limit + 1);
        for idx in start..count {
            let (norm, word, rank) = self.record_texts(idx);
            if !norm.starts_with(p.as_str()) {
                break;
            }
            let _ = word;
            if top.len() < limit {
                top.push((rank, idx));
            } else if let Some(&(worst, _)) = top.peek() {
                if rank < worst {
                    top.pop();
                    top.push((rank, idx));
                }
            }
        }
        // 收集堆中最小 rank 的 limit 个，按 rank 升序输出。
        let mut chosen: Vec<(u32, usize)> = top.into_iter().collect();
        chosen.sort_unstable_by_key(|(rank, _)| *rank);
        chosen
            .into_iter()
            .map(|(rank, idx)| (self.record_texts(idx).1, rank))
            .collect()
    }
}

/// 取字符串前 8 字节，不足零填充（锚比较键）。
fn pfx8(s: &str) -> [u8; 8] {
    let mut out = [0u8; 8];
    let bytes = s.as_bytes();
    let take = bytes.len().min(8);
    out[..take].copy_from_slice(&bytes[..take]);
    out
}

/// 字节序前缀严格小于：`a < b` 且 `a` 不是 `b` 的前缀。
/// `partition_point` 单调性依赖：前缀匹配区（a 以 b 开头）整体返回 false。
fn is_norm_less_than(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    match a.cmp(b) {
        std::cmp::Ordering::Less => true,
        std::cmp::Ordering::Greater => false,
        std::cmp::Ordering::Equal => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_records() -> Vec<EnRecord<'static>> {
        // 按 norm 字节序升序传入；rank 为常用度排序号（越小越常用）。
        vec![
            EnRecord {
                norm: "'a",
                word: "'a",
                rank: 8,
            },
            EnRecord {
                norm: "api",
                word: "API",
                rank: 3,
            },
            EnRecord {
                norm: "apiary",
                word: "apiary",
                rank: 5,
            },
            EnRecord {
                norm: "apple",
                word: "Apple",
                rank: 4,
            },
            EnRecord {
                norm: "good morning",
                word: "good morning",
                rank: 7,
            },
            EnRecord {
                norm: "iphone",
                word: "iPhone",
                rank: 1,
            },
            EnRecord {
                norm: "merry-go-round",
                word: "merry-go-round",
                rank: 6,
            },
            EnRecord {
                norm: "python",
                word: "python",
                rank: 0,
            },
            EnRecord {
                norm: "the",
                word: "the",
                rank: 2,
            },
        ]
    }

    fn lex() -> EnLexicon {
        EnLexicon::from_bytes(&compile_en_records(&sample_records())).expect("编译加载")
    }

    #[test]
    fn 加载校验通过() {
        let l = lex();
        assert_eq!(l.count(), 9);
        assert!(l.path().is_none());
    }

    #[test]
    fn 验收命中与大小写无关() {
        let l = lex();
        let hit = |p: &str| {
            l.words_with_prefix(p, 8)
                .into_iter()
                .map(|(w, _)| w.to_owned())
                .collect::<Vec<_>>()
        };
        assert_eq!(hit("python"), vec!["python"]);
        assert_eq!(hit("pytho"), vec!["python"]);
        assert_eq!(hit("iphone"), vec!["iPhone"]);
        assert_eq!(hit("iphon"), vec!["iPhone"]);
        assert_eq!(hit("api"), vec!["API", "apiary"]); // 原形大小写保留（D-09）
        assert_eq!(hit("PYTHON"), vec!["python"]); // 输入大小写不敏感
        assert_eq!(hit("good"), vec!["good morning"]); // 带空格词
        assert_eq!(hit("merry"), vec!["merry-go-round"]); // 连字符词
        assert_eq!(hit("'a"), vec!["'a"]); // 撇号词
        assert_eq!(
            hit("ap"),
            vec!["API", "Apple", "apiary"] // 按 rank 排序：3, 4, 5
        );
    }

    #[test]
    fn 排序按常用度截断() {
        // "a" 前缀命中 API(rank3)/Apple(rank4)/apiary(rank5)，取前 2 = API、Apple。
        let l = lex();
        let hits = l.words_with_prefix("a", 2);
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].0, "API");
        assert_eq!(hits[0].1, 3);
        assert_eq!(hits[1].0, "Apple");
        assert_eq!(hits[1].1, 4);
        // limit=0 返回空。
        assert!(l.words_with_prefix("a", 0).is_empty());
    }

    #[test]
    fn 不命中返回空() {
        let l = lex();
        assert!(l.words_with_prefix("", 8).is_empty());
        assert!(l.words_with_prefix("xyzabc", 8).is_empty());
        assert!(l.words_with_prefix("zzzz", 8).is_empty());
        assert!(l.words_with_prefix("thex", 8).is_empty());
    }

    #[test]
    fn 精确与前缀边界() {
        let l = lex();
        // "the" 精确命中；"the " 无命中（无词以 "the " 开头）。
        assert_eq!(l.words_with_prefix("the", 8).len(), 1);
        assert!(l.words_with_prefix("the ", 8).is_empty());
        // 前缀区慢于单词：go→"good morning"，goo 同样。
        assert_eq!(l.words_with_prefix("goo", 8).len(), 1);
    }

    #[test]
    fn 首尾锚桶边界() {
        // 锚桶仅 1 桶（9 条 < 1024）；验证从桶首定位与尾桶行为。
        let l = lex();
        assert_eq!(l.header.anchor_count, 1);
        assert_eq!(l.words_with_prefix("'", 8).len(), 1); // 撇号桶首
                                                          // 前缀大于全部键 → 空。
        assert!(l.words_with_prefix("zzzzzz", 8).is_empty());
    }

    #[test]
    fn 损坏文件被拒() {
        let bytes = compile_en_records(&sample_records());
        // 篡改一条 norm（改池字节后哈希失配）。
        let mut bad = bytes.clone();
        let pool_off = EN_HEADER_SIZE + 9 * EN_RECORD_SIZE;
        bad[pool_off] = b'Z'; // "the" → "Zhe"，哈希必失配
        assert!(EnLexicon::from_bytes(&bad).is_err());

        // 截断。
        let truncated = &bytes[..bytes.len() - 3];
        assert!(EnLexicon::from_bytes(truncated).is_err());

        // 魔数错误。
        let mut bad_magic = bytes.clone();
        bad_magic[0] = b'X';
        assert!(EnLexicon::from_bytes(&bad_magic).is_err());
    }

    #[test]
    fn 超大锚桶跨前缀仍正确() {
        // 构造 3000 条记录（3 个锚桶），验证跨桶前缀查询。
        let records: Vec<EnRecord<'static>> = (0..3000)
            .map(|i| {
                let norm = format!("word{i:04}");
                let word = format!("Word{i:04}");
                EnRecord {
                    norm: Box::leak(norm.into_boxed_str()),
                    word: Box::leak(word.into_boxed_str()),
                    rank: i,
                }
            })
            .collect();
        let bytes = compile_en_records(&records);
        let l = EnLexicon::from_bytes(&bytes).expect("编译加载");
        assert_eq!(l.header.anchor_count, 3);
        // 前缀 "word0" 命中 word0000..word0999。
        let hits = l.words_with_prefix("word0", 1200);
        assert_eq!(hits.len(), 1000);
        assert_eq!(hits[0].0, "Word0000");
        assert_eq!(hits[999].0, "Word0999");
        assert_eq!(hits[0].1, 0);
        // 截断生效。
        assert_eq!(l.words_with_prefix("word", 5).len(), 5);
    }

    #[test]
    fn 非utf8池被拒() {
        // 手动构造：把池首字节改成 0xFF 再重算哈希 → 加载应报“非 UTF-8”。
        let records = sample_records();
        let mut bytes = compile_en_records(&records);
        let pool_off = EN_HEADER_SIZE + records.len() * EN_RECORD_SIZE;
        bytes[pool_off] = 0xFF;
        let hash: [u8; 32] = Sha256::digest(&bytes[EN_HEADER_SIZE..]).into();
        bytes[64..96].copy_from_slice(&hash);
        assert!(EnLexicon::from_bytes(&bytes).is_err());
    }
}
