# 单字拼音注音表构建脚本（T-071-2）：生成 crates/zhu-ye-core/src/char_pinyin.rs
# 输入：
#   data/cache/kTGHZ2013.txt   Unihan kTGHZ2013 单字拼音（通用规范汉字表读音，MIT）
#     参考 data/pins/pinyin-data-kTGHZ2013.json（pin 哈希锁定）
# 输出：
#   crates/zhu-ye-core/src/char_pinyin.rs（表 + 查询函数 + 抽查测试；由本脚本整体生成，
#     变更应改脚本后重跑，生成文件不得手改）
# 规则：
#   - 只收"通用规范汉字"区（U+3442..U+9FA5 等）中解析出读音的行；行内 `#` 后为注释
#   - 读音剥离预组声调字母与组合调符（zhòu -> zhou）；ü 系归一为输入法惯例 v（lǜ -> lv）
#   - 读音去重保序；过滤空串/含非 ASCII 的脏读音
#   - 条目按 char 标量序升序（binary_search_by_key 依赖）；查询返回全读音（多音字全形态，
#     D-22）
# 兼容：Windows PowerShell 5.1+；脚本文件使用 UTF-8 BOM
$ErrorActionPreference = 'Stop'

$root = (Get-Location).Path
$srcPath = Join-Path $root 'data/cache/kTGHZ2013.txt'
$pinPath = Join-Path $root 'data/pins/pinyin-data-kTGHZ2013.json'
$outPath = Join-Path $root 'crates/zhu-ye-core/src/char_pinyin.rs'

if (-not (Test-Path $srcPath)) { throw "缺少 kTGHZ2013：$srcPath（先跑 scripts/fetch-sources.ps1 -WritePins）" }

# ---------- 0. pin 哈希校验（数据不变则生成表不变，可复现） ----------
$pin = Get-Content $pinPath -Raw -Encoding UTF8 | ConvertFrom-Json
$expected = $pin.sha256
$actual = (Get-FileHash $srcPath -Algorithm SHA256).Hash
if ($expected -and ($actual -ne $expected)) {
    throw "kTGHZ2013.txt 哈希不匹配（期望 $expected，实际 $actual）：数据漂移，禁止生成"
}

# ---------- 1. 预组调符字符映射（字母调符 -> 无调字母；ü -> v） ----------
$toneMap = @{
    [char]0x0101 = 'a'; [char]0x00E1 = 'a'; [char]0x01CE = 'a'; [char]0x00E0 = 'a'  # ā á ǎ à
    [char]0x0113 = 'e'; [char]0x00E9 = 'e'; [char]0x011B = 'e'; [char]0x00E8 = 'e'  # ē é ě è
    [char]0x012B = 'i'; [char]0x00ED = 'i'; [char]0x01D0 = 'i'; [char]0x00EC = 'i'  # ī í ǐ ì
    [char]0x014D = 'o'; [char]0x00F3 = 'o'; [char]0x01D2 = 'o'; [char]0x00F2 = 'o'  # ō ó ǒ ò
    [char]0x016B = 'u'; [char]0x00FA = 'u'; [char]0x01D4 = 'u'; [char]0x00F9 = 'u'  # ū ú ǔ ù
    [char]0x01D6 = 'v'; [char]0x01D8 = 'v'; [char]0x01DA = 'v'; [char]0x01DC = 'v'  # ǖ ǘ ǚ ǜ
    [char]0x00FC = 'v';                                                             # ü
    [char]0x1E25 = 'm';                                                             # ḿ
    [char]0x0144 = 'n'; [char]0x0148 = 'n'; [char]0x01F9 = 'n'                       # ń ň ǹ
}
$combiningTones = @{
    [char]0x0300 = $true; [char]0x0301 = $true; [char]0x0302 = $true
    [char]0x0303 = $true; [char]0x0304 = $true; [char]0x0306 = $true
    [char]0x030C = $true
}

function Strip-Tone([string]$syllable) {
    $sb = [System.Text.StringBuilder]::new()
    foreach ($ch in $syllable.ToCharArray()) {
        if ($toneMap.ContainsKey($ch)) { [void]$sb.Append($toneMap[$ch]) }
        elseif ($combiningTones.ContainsKey($ch)) { }  # 组合调符删除
        else { [void]$sb.Append($ch) }
    }
    return $sb.ToString()
}

# ---------- 2. 解析行并剥离调符 ----------
$entries = [System.Collections.Generic.List[object]]::new()
$reader = [System.IO.StreamReader]::new($srcPath, [System.Text.UTF8Encoding]::new($false))
try {
    while ($null -ne ($line = $reader.ReadLine())) {
        $line = $line.Trim()
        if ($line.Length -eq 0 -or $line.StartsWith('#')) { continue }
        # U+XXXX: 拼音1,拼音2  # 注释
        $m = [regex]::Match($line, '^U\+([0-9A-Fa-f]{4,5})\s*:\s*([^\#]+?)\s*(\#.*)?$')
        if (-not $m.Success) { continue }
        $code = [Convert]::ToInt32($m.Groups[1].Value, 16)
        # 非 BMP 码位（扩展区字）在 .NET 里是 UTF-16 代理对（string），Rust 侧以 char 标量存放。
        $ch = [System.Char]::ConvertFromUtf32($code)
        $readings = [System.Collections.Generic.List[string]]::new()
        $seen = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::Ordinal)
        foreach ($part in $m.Groups[2].Value.Split(',')) {
            $clean = (Strip-Tone $part).Trim()
            if ($clean.Length -eq 0) { continue }
            $isAscii = $true
            foreach ($c in $clean.ToCharArray()) { if ([int]$c -gt 127) { $isAscii = $false; break } }
            if (-not $isAscii) { continue }
            if ($seen.Add($clean)) { [void]$readings.Add($clean) }
        }
        if ($readings.Count -eq 0) { continue }
        $entries.Add([pscustomobject]@{ Ch = $ch; Readings = $readings })
    }
}
finally { $reader.Dispose() }

# 按 char 标量序升序（binary_search 依赖）。UTF-16 码元序与 Unicode 标量序同序，
# 用 Ordinal 比较保证排序确定性（不随区域设置变化）。
$sorted = [System.Linq.Enumerable]::OrderBy(
    $entries,
    [System.Func[object, System.String]] { param($e) $e.Ch },
    [System.StringComparer]::Ordinal
)
$count = $sorted.Count

# ---------- 3. 生成 Rust 源 ----------
$lines = [System.Collections.Generic.List[string]]::new()
$lines.Add('//! 单字拼音注音表（通用规范汉字表读音，kTGHZ2013）。')
$lines.Add('//!')
$lines.Add('//! 本文件由 scripts/build-char-pinyin.ps1 从 data/cache/kTGHZ2013.txt 生成，')
$lines.Add('//! 不得手改；变更需改脚本后重跑（输入哈希由 data/pins/pinyin-data-kTGHZ2013.json 锁定）。')
$lines.Add('//!')
$lines.Add('//! 供场景 9 通讯录（FR-036/FR-037）联系人姓名注音建键：返回某字**全部**读音形态')
$lines.Add('//! （无调 ASCII，ü 按输入法惯例写作 v），多音字任一读音均可命中（D-22）。')
$lines.Add('')
$lines.Add('/// 单条字-读音表项（按字标量序升序，二分查询）。')
$lines.Add('#[derive(Debug, Clone, Copy, PartialEq, Eq)]')
$lines.Add('pub struct CharPinyinEntry {')
$lines.Add('    /// 汉字。')
$lines.Add('    pub ch: char,')
$lines.Add('    /// 全部读音形态（无调，ü->v）。
    pub readings: &''static [&''static str],')
$lines.Add('}')
$lines.Add('')
$lines.Add('/// 单字注音表（kTGHZ2013，经脚本清洗）。')
$lines.Add('pub static CHAR_PINYIN: &[CharPinyinEntry] = &[')
foreach ($e in $sorted) {
    $chLit = "'\u{$('{0:X4}' -f [System.Char]::ConvertToUtf32($e.Ch, 0))}'"
    $readings = ($e.Readings | ForEach-Object { '"' + $_ + '"' }) -join ', '
    $lines.Add("    CharPinyinEntry { ch: $chLit, readings: &[$readings] },")
}
$lines.Add('];')
$lines.Add('')
$lines.Add('/// 查询单字全部读音；未收录返回 `None`（kTGHZ 外生僻字）。')
$lines.Add('#[must_use]')
$lines.Add('pub fn char_pinyin(ch: char) -> Option<&''static [&''static str]> {')
$lines.Add('    CHAR_PINYIN')
$lines.Add('        .binary_search_by_key(&ch, |entry| entry.ch)')
$lines.Add('        .ok()')
$lines.Add('        .map(|index| CHAR_PINYIN[index].readings)')
$lines.Add('}')
$lines.Add('')
$lines.Add('#[cfg(test)]')
$lines.Add('mod tests {')
$lines.Add('    use super::{char_pinyin, CHAR_PINYIN};')
$lines.Add('')
$lines.Add('    #[test]')
$lines.Add('    fn 表按字升序() {')
$lines.Add('        for pair in CHAR_PINYIN.windows(2) {')
$lines.Add('            assert!(pair[0].ch < pair[1].ch, "表必须按 char 升序");')
$lines.Add('        }')
$lines.Add('    }')
$lines.Add('')
$lines.Add('    #[test]')
$lines.Add('    fn 多音字全形态() {')
$lines.Add('        // D-22：全部读音形态建键（集合语义断言，不依赖 kTGHZ 行的读音顺序）。')
$lines.Add('        for (ch, readings) in [')
$lines.Add('            (''谁'', &["shei", "shui"]),')
$lines.Add('            (''曾'', &["ceng", "zeng"]),')
$lines.Add('            (''了'', &["le", "liao"]),')
$lines.Add('            (''乐'', &["le", "yue"]),')
$lines.Add('        ] {')
$lines.Add('            let got = char_pinyin(ch).expect("常见多音字应被收录");')
$lines.Add('            for reading in readings {')
$lines.Add('                assert!(got.contains(reading), "字 {ch} 应含读音 {reading}，实际 {got:?}");')
$lines.Add('            }')
$lines.Add('        }')
$lines.Add('    }')
$lines.Add('')
$lines.Add('    #[test]')
$lines.Add('    fn ü归一为v() {')
$lines.Add('        assert!(char_pinyin(''绿'').unwrap().contains(&"lv"), "绿 应含 lv");')
$lines.Add('        assert_eq!(char_pinyin(''女''), Some(&["nv"][..]));')
$lines.Add('    }')
$lines.Add('')
$lines.Add('    #[test]')
$lines.Add('    fn 调符已剥离() {')
$lines.Add('        assert_eq!(char_pinyin(''张''), Some(&["zhang"][..]));')
$lines.Add('        assert_eq!(char_pinyin(''三''), Some(&["san"][..]));')
$lines.Add('    }')
$lines.Add('')
$lines.Add('    #[test]')
$lines.Add('    fn 未收录字返回none() {')
$lines.Add('        assert_eq!(char_pinyin(''𠀀''), None);')
$lines.Add('    }')
$lines.Add('')
$lines.Add('    #[test]')
$lines.Add('    fn 常见字抽查() {')
$lines.Add('        for (ch, expect) in [')
$lines.Add('            (''你'', &["ni"][..]),')
$lines.Add('            (''好'', &["hao"][..]),')
$lines.Add('            (''李'', &["li"][..]),')
$lines.Add('            (''王'', &["wang"][..]),')
$lines.Add('            (''国'', &["guo"][..]),')
$lines.Add('        ] {')
$lines.Add('            assert_eq!(char_pinyin(ch), Some(expect), "字 {ch} 读音不符");')
$lines.Add('        }')
$lines.Add('    }')
$lines.Add('}')
$lines.Add('')

[System.IO.File]::WriteAllLines($outPath, $lines, [System.Text.UTF8Encoding]::new($false))
# 入库文件必须 rustfmt 干净（门禁）；用工具链自带的 rustfmt 格式化生成的代码
& rustfmt $outPath 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) { throw "rustfmt 失败（$LASTEXITCODE）：检查 rustfmt 是否在 PATH（cargo 工具链）" }
Write-Host ("已生成 {0}：{1} 条（含测试与查询函数，已 rustfmt）" -f $outPath.Replace($root + '\', ''), $count)
