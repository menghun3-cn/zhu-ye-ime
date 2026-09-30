# 英文词表构建脚本（T-064）：生成 crates/zhu-ye-core/src/en_words.rs
# 输入：
#   data/cache/frequencywords-en.txt   FrequencyWords 英文词频（D-018，CC BY-SA 4.0）
#   data/patches/en-capitals.tsv       大小写补丁（人工维护，随 git 版本管理）
#   data/raw/cedict_ts.u8              CC-CEDICT 原文（D-001，CC BY-SA 4.0，英文侧提取）
# 输出：
#   crates/zhu-ye-core/src/en_words.rs（表 + 查询函数 + 测试；由本脚本整体生成，变更应改脚本后重跑）
# 规则：小写 ASCII 查键唯一、按字节序升序（二分依赖）；top10000 由频次截断；补丁只改原形/排序，
#       可强推低频词进入；CEDICT 英文侧仅补充纯单词词目（过滤括号注释/空格短语）。
# 兼容：Windows PowerShell 5.1+；脚本文件使用 UTF-8 BOM
$ErrorActionPreference = 'Stop'

$root = (Get-Location).Path
$freqPath = Join-Path $root 'data/cache/frequencywords-en.txt'
$patchPath = Join-Path $root 'data/patches/en-capitals.tsv'
$cedictPath = Join-Path $root 'data/raw/cedict_ts.u8'
$outPath = Join-Path $root 'crates/zhu-ye-core/src/en_words.rs'

if (-not (Test-Path $freqPath)) { throw "缺少词频文件：$freqPath（先跑 scripts/fetch-sources.ps1 -WritePins）" }
if (-not (Test-Path $cedictPath)) { throw "缺少 CEDICT 原文：$cedictPath" }

# ---------- 1. FrequencyWords 清洗 + top10000 ----------
$words = [System.Collections.Generic.List[object]]::new()
$reader = [System.IO.StreamReader]::new($freqPath, [System.Text.UTF8Encoding]::new($false))
try {
    while ($null -ne ($line = $reader.ReadLine())) {
        if ($line.Length -eq 0) { continue }
        # FrequencyWords 行格式：word<空格>frequency（词不含空格，按最后一个空格分割）
        $sp = $line.LastIndexOf(' ')
        if ($sp -lt 1) { continue }
        $w = $line.Substring(0, $sp)
        $freq = 0L
        if (-not [long]::TryParse($line.Substring($sp + 1), [ref]$freq)) { continue }
        # 清洗：纯 ASCII a-z 与撇号、长度 2-32
        $ok = $true
        foreach ($ch in $w.ToCharArray()) {
            if (-not (($ch -ge 'a' -and $ch -le 'z') -or $ch -eq "'")) { $ok = $false; break }
        }
        if (-not $ok) { continue }
        if ($w.Length -lt 2 -or $w.Length -gt 32) { continue }
        $words.Add([pscustomobject]@{ norm = $w; word = $w; freq = $freq })
    }
}
finally { $reader.Dispose() }

Write-Host ("FrequencyWords 清洗后：{0} 词" -f $words.Count)

# 去重（同 norm 取最高 freq）并按 freq 降序截断 top10000
# 注意：键比较一律用 Ordinal（PS hashtable 默认大小写不敏感，会造成查键误判）
$byNorm = [System.Collections.Generic.Dictionary[string, object]]::new([System.StringComparer]::Ordinal)
foreach ($w in $words) {
    if (-not $byNorm.ContainsKey($w.norm) -or $w.freq -gt $byNorm[$w.norm].freq) { $byNorm[$w.norm] = $w }
}
$ranked = [System.Collections.Generic.List[object]]::new()
foreach ($kv in $byNorm.GetEnumerator()) { $ranked.Add($kv.Value) }
$ranked.Sort({ param($a, $b) if ($a.freq -ne $b.freq) { return [int]([Math]::Sign($b.freq - $a.freq)) }; return [string]::CompareOrdinal($a.norm, $b.norm) })
if ($ranked.Count -gt 10000) { $ranked.RemoveRange(10000, $ranked.Count - 10000) }
Write-Host ("top10000 截断后：{0} 词" -f $ranked.Count)

# 统一为 { norm, word, rank } 结构；排名 1..N（越小越靠前），norm -> 对象引用
$rankMap = [System.Collections.Generic.Dictionary[string, object]]::new([System.StringComparer]::Ordinal)
$normalized = [System.Collections.Generic.List[object]]::new()
for ($i = 0; $i -lt $ranked.Count; $i++) {
    $obj = [pscustomobject]@{ norm = $ranked[$i].norm; word = $ranked[$i].word; rank = $i + 1 }
    $normalized.Add($obj)
    $rankMap[$obj.norm] = $obj
}
$ranked = $normalized
$nextRank = $ranked.Count + 1

# ---------- 2. 大小写补丁 ----------
$patches = [System.Collections.Generic.List[object]]::new()
foreach ($line in [System.IO.File]::ReadAllLines($patchPath)) {
    $line = $line.Trim()
    if ($line.Length -eq 0 -or $line.StartsWith('#')) { continue }
    $cols = $line -split "`t"
    if ($cols.Count -lt 2) { throw "补丁行格式错误：$line" }
    $norm = $cols[0].ToLowerInvariant()
    if ($norm -ne $cols[0]) { throw "补丁小写键必须全小写：$line" }
    $rank = $null
    if ($cols.Count -ge 3 -and $cols[2].Length -gt 0) {
        $r = 0
        if (-not [int]::TryParse($cols[2], [ref]$r) -or $r -lt 1) { throw "补丁排名必须为正整数：$line" }
        $rank = $r
    }
    $patches.Add([pscustomobject]@{ norm = $norm; word = $cols[1]; rank = $rank })
}
Write-Host ("大小写补丁：{0} 条" -f $patches.Count)

$nextRank = 10000 + 1
foreach ($p in $patches) {
    if ($rankMap.ContainsKey($p.norm)) {
        $obj = $rankMap[$p.norm]
        $obj.word = $p.word
        if ($p.rank) {
            $obj.rank = $p.rank
            $nextRank = [Math]::Max($nextRank, $p.rank + 1)
        }
    } else {
        $r = $p.rank ? $p.rank : $nextRank
        if ($p.rank) { $nextRank = [Math]::Max($nextRank, $p.rank + 1) } else { $nextRank++ }
        $obj = [pscustomobject]@{ norm = $p.norm; word = $p.word; rank = $r }
        $rankMap[$p.norm] = $obj
        $ranked.Add($obj)
    }
}
# CEDICT 补充的排名从 nextRank 起
$cedictExtra = [System.Collections.Generic.List[object]]::new()

# ---------- 3. CEDICT 英文侧提取 ----------
$saw = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::Ordinal)
foreach ($k in $rankMap.Keys) { [void]$saw.Add($k) }
$cedictExtra = [System.Collections.Generic.List[object]]::new()
$sr = [System.IO.StreamReader]::new($cedictPath, [System.Text.UTF8Encoding]::new($false))
try {
    while ($null -ne ($line = $sr.ReadLine())) {
        if ($line.Length -eq 0 -or $line.StartsWith('#')) { continue }
        $open = $line.IndexOf('/')
        if ($open -lt 0) { continue }
        $gloss = $line.Substring($open + 1).TrimEnd('/')
        # 只取第一条译文；过滤注释括号
        $semi = $gloss.IndexOf(';')
        if ($semi -ge 0) { $gloss = $gloss.Substring(0, $semi) }
        if ($gloss.IndexOf('(') -ge 0 -or $gloss.IndexOf(' ') -ge 0) { continue }
        $ok = $true
        foreach ($ch in $gloss.ToCharArray()) {
            if (-not (($ch -ge 'a' -and $ch -le 'z') -or ($ch -ge 'A' -and $ch -le 'Z'))) { $ok = $false; break }
        }
        if (-not $ok) { continue }
        if ($gloss.Length -lt 2 -or $gloss.Length -gt 32) { continue }
        $norm = $gloss.ToLowerInvariant()
        if ($saw.Contains($norm)) { continue }
        [void]$saw.Add($norm)
        $cedictExtra.Add([pscustomobject]@{ norm = $norm; word = $gloss; rank = $nextRank })
        $nextRank++
    }
}
finally { $sr.Dispose() }
Write-Host ("CEDICT 英文侧补充：{0} 词" -f $cedictExtra.Count)

foreach ($e in $cedictExtra) { $ranked.Add($e) }

# ---------- 4. 按 norm 字节序排序并生成 en_words.rs ----------
$ranked.Sort({ param($a, $b) [string]::CompareOrdinal($a.norm, $b.norm) })

# 重复 norm 防御（排序后相邻检查）
$lines = [System.Collections.Generic.List[string]]::new()
$lines.Add('//! 英文词候选表（FR-030，场景 6）。')
$lines.Add('//!')
$lines.Add('//! 由 scripts/build-en-words.ps1 生成，请勿手改；改数据源/清洗/补丁后重跑脚本。')
$lines.Add('//! 数据来源：FrequencyWords 英文词频（D-018，CC BY-SA 4.0）+ 人工大小写补丁')
$lines.Add('//! （data/patches/en-capitals.tsv）+ CC-CEDICT 英文侧（D-001，CC BY-SA 4.0）。')
$lines.Add('//! 查键 norm 为小写 ASCII，按字节序升序（二分依赖）；word 保留原形大小写（D-09）；')
$lines.Add('//! freq_rank 仅组内排序用（越小越常用，不参与中文静态排序，D-10）。')
$lines.Add('')
$lines.Add('/// 英文词候选条目。')
$lines.Add('#[derive(Debug, Clone, Copy)]')
$lines.Add('pub struct EnWordEntry {')
$lines.Add('    /// 小写归一化查键（ASCII），表按此字节序升序。')
$lines.Add("    pub norm: &'static str,")
$lines.Add('    /// 上屏原形（保留大小写）。')
$lines.Add("    pub word: &'static str,")
$lines.Add('    /// 组内排序号（越小越常用）。')
$lines.Add("    pub freq_rank: u16,")
$lines.Add('}')
$lines.Add('')
$lines.Add('pub static EN_WORDS: &[EnWordEntry] = &[')

$prevNorm = $null
$prevWord = $null
$count = 0
foreach ($e in $ranked) {
    if ($null -ne $prevNorm -and [string]::CompareOrdinal($prevNorm, $e.norm) -eq 0) {
        throw ("重复查键（generate 防御）：{0}（word1={1} rank1={2} / word2={3} rank2={4}）" -f $e.norm, $prevWord, $count, $e.word, $e.rank)
    }
    $prevNorm = $e.norm
    $lines.Add(('    EnWordEntry {{ norm: "{0}", word: "{1}", freq_rank: {2} }},' -f $e.norm, $e.word, [Math]::Min($e.rank, 65535)))
    $count++
}
$lines.Add('];')
$lines.Add('')
$lines.Add('/// 按输入前缀查询英文候选（命中 ≤limit 条，按 freq_rank 升序；未命中返回空）。')
$lines.Add('/// 输入任意大小写，内部小写化后匹配查键。')
$lines.Add('#[must_use]')
$lines.Add("pub fn en_words_with_prefix(prefix: &str, limit: usize) -> Vec<(&'static str, u16)> {")
$lines.Add('    if prefix.is_empty() || limit == 0 { return Vec::new(); }')
$lines.Add('    let p = prefix.to_ascii_lowercase();')
$lines.Add('    let start = EN_WORDS.partition_point(|e| prefix_lt(e.norm, &p));')
$lines.Add("    let mut hits: Vec<(&'static str, u16)> = Vec::new();")
$lines.Add('    for e in &EN_WORDS[start..] {')
$lines.Add('        if !e.norm.starts_with(&p) { break; }')
$lines.Add('        hits.push((e.word, e.freq_rank));')
$lines.Add('    }')
$lines.Add('    hits.sort_by_key(|(_, rank)| *rank);')
$lines.Add('    hits.truncate(limit);')
$lines.Add('    hits')
$lines.Add('}')
$lines.Add('')
$lines.Add('/// 字节序前缀严格小于比较（`a < b` 且 `a` 不是 `b` 的前缀）。')
$lines.Add('fn prefix_lt(a: &str, b: &str) -> bool {')
$lines.Add('    match a.as_bytes().cmp(b.as_bytes()) {')
$lines.Add('        std::cmp::Ordering::Less => true,')
$lines.Add('        std::cmp::Ordering::Greater => false,')
$lines.Add('        std::cmp::Ordering::Equal => false,')
$lines.Add('    }')
$lines.Add('}')
$lines.Add('')
$lines.Add('#[cfg(test)]')
$lines.Add('mod tests {')
$lines.Add('    use super::{en_words_with_prefix, EN_WORDS};')
$lines.Add('')
$lines.Add('    #[test]')
$lines.Add('    fn 表有序且无重复查键() {')
$lines.Add('        for pair in EN_WORDS.windows(2) {')
$lines.Add('            assert!(pair[0].norm < pair[1].norm, "表未按 norm 升序: {} vs {}", pair[0].norm, pair[1].norm);')
$lines.Add('        }')
$lines.Add('        let mut seen = std::collections::HashSet::new();')
$lines.Add('        for e in EN_WORDS {')
$lines.Add('            assert!(seen.insert(e.norm), "重复查键: {}", e.norm);')
$lines.Add('            assert!(!e.norm.is_empty() && !e.word.is_empty());')
$lines.Add('        }')
$lines.Add('    }')
$lines.Add('')
$lines.Add('    #[test]')
$lines.Add('    fn 词表规模达标() {')
$lines.Add('        assert!(EN_WORDS.len() >= 10000, "D-12 应 ≥10000 词，实际 {}", EN_WORDS.len());')
$lines.Add('    }')
$lines.Add('')
$lines.Add('    #[test]')
$lines.Add('    fn 验收命中() {')
$lines.Add('        assert_eq!(en_words_with_prefix("python", 8).first().map(|(w, _)| *w), Some("python"));')
$lines.Add('        assert_eq!(en_words_with_prefix("pytho", 8).first().map(|(w, _)| *w), Some("python"));')
$lines.Add('        assert_eq!(en_words_with_prefix("iphone", 8).first().map(|(w, _)| *w), Some("iPhone"));')
$lines.Add('        assert_eq!(en_words_with_prefix("iphon", 8).first().map(|(w, _)| *w), Some("iPhone"));')
$lines.Add('        assert_eq!(en_words_with_prefix("api", 8).first().map(|(w, _)| *w), Some("API"));')
$lines.Add('        // 大小写输入同命中（D-09）')
$lines.Add('        assert_eq!(en_words_with_prefix("PYTHO", 8).first().map(|(w, _)| *w), Some("python"));')
$lines.Add('        assert_eq!(en_words_with_prefix("PYTHON", 8).first().map(|(w, _)| *w), Some("python"));')
$lines.Add('        // 前缀补充')
$lines.Add('        assert!(!en_words_with_prefix("py", 8).is_empty());')
$lines.Add('    }')
$lines.Add('')
$lines.Add('    #[test]')
$lines.Add('    fn 不命中返回空() {')
$lines.Add('        assert!(en_words_with_prefix("", 8).is_empty());')
$lines.Add('        assert!(en_words_with_prefix("xyzabc123", 8).is_empty());')
$lines.Add('        assert!(en_words_with_prefix("zzzz9", 8).is_empty());')
$lines.Add('    }')
$lines.Add('')
$lines.Add('    #[test]')
$lines.Add('    fn 候选数量受限于上限() {')
$lines.Add('        let hits = en_words_with_prefix("a", 3);')
$lines.Add('        assert!(hits.len() <= 3);')
$lines.Add('    }')
$lines.Add('}')
$lines.Add('')

[System.IO.File]::WriteAllLines($outPath, $lines, [System.Text.UTF8Encoding]::new($false))
# 入库文件必须 rustfmt 干净（门禁）；用工具链自带的 rustfmt 格式化生成的代码
& rustfmt $outPath 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) { throw "rustfmt 失败（$LASTEXITCODE）：检查 rustfmt 是否在 PATH（cargo 工具链）" }
Write-Host ("已生成 {0}：{1} 条（含测试与查询函数，已 rustfmt）" -f $outPath.Replace($root + '\', ''), $count)
