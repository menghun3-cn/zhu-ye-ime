//! 主机侧端到端回归检查器。
//!
//! 使用 v2 词典文件驱动 `InputEngine`，覆盖核心输入闭环：
//! 拼音切分、候选排序、用户词学习、译文层与本地翻译反查。
//! 由 `scripts/e2e.ps1` 调用；本机运行不连接网络、不依赖 TSF 或虚拟机。

// 与 candidate_demo 相同，`#[path]` 独立编译库模块，允许库侧未使用项。
#![allow(dead_code)]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;

use zhu_ye_core::bigram::{BigramModel, InMemoryBigramModel};
use zhu_ye_core::candidate::CandidateSource;
use zhu_ye_core::dict::Dictionary;
use zhu_ye_core::translate::{TranslationDirection, Translator};
use zhu_ye_core::{
    build_contact_index, parse_vcard, DictionaryEntry, DictionaryFile, InMemoryDictionary,
    UserDictStore, VCardContact,
};

#[path = "../candidate_ui.rs"]
mod candidate_ui;

#[path = "../input.rs"]
mod input;

use input::{CandidateLayer, InputEngine, InputMode};

struct Runner {
    passed: usize,
    failed: usize,
}

impl Runner {
    fn pass(&mut self, name: &str) {
        println!("[PASS] {name}");
        self.passed += 1;
    }

    fn fail(&mut self, name: &str, detail: &str) {
        eprintln!("[FAIL] {name}: {detail}");
        self.failed += 1;
    }

    fn finish(self) -> ExitCode {
        let total = self.passed + self.failed;
        if self.failed == 0 {
            println!("E2E 结果: PASS {total}/{total}");
            ExitCode::SUCCESS
        } else {
            eprintln!("E2E 结果: FAIL {}/{}", self.passed, total);
            ExitCode::from(1)
        }
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--real-smoke") => {
            let Some(path) = args.get(1).map(String::as_str) else {
                eprintln!("用法: host-e2e --real-smoke <词典文件>");
                return ExitCode::from(2);
            };
            run_real_smoke(Path::new(path))
        }
        // M7 输入体验优化验收（FR-023 至 FR-025，验收标准 8.1）：简拼/模糊音纠错/整句。
        Some("--m7") => {
            let Some(path) = args.get(1).map(String::as_str) else {
                eprintln!("用法: host-e2e --m7 <词典文件>");
                return ExitCode::from(2);
            };
            run_m7_checks(Path::new(path))
        }
        // T-059 上屏联想验收（场景5）：上屏后空闲候选窗展示 bigram 后继联想。
        Some("--m8") => {
            let Some(path) = args.get(1).map(String::as_str) else {
                eprintln!("用法: host-e2e --m8 <词典文件>");
                return ExitCode::from(2);
            };
            run_m8_checks(Path::new(path))
        }
        // 场景7 格式候选验收（FR-027/028/029）：数字格式候选、v 模式符号、emoji 追尾。
        Some("--m9") => {
            let Some(path) = args.get(1).map(String::as_str) else {
                eprintln!("用法: host-e2e --m9 <词典文件>");
                return ExitCode::from(2);
            };
            run_m9_checks(Path::new(path))
        }
        // 场景6 中英混输验收（FR-030/031/032，验收标准 10.1）：英文拼写补全、
        // 大小写原形、拼音不介入、缩写不回退、邮箱/网址补全与直通、退出交互。
        Some("--m10") => {
            let Some(path) = args.get(1).map(String::as_str) else {
                eprintln!("用法: host-e2e --m10 <词典文件>");
                return ExitCode::from(2);
            };
            run_m10_checks(Path::new(path))
        }
        // 场景8 领域自动验收（FR-033/034/035，验收标准 11.1）：
        // --m11 [<词典文件>]，全内存可控领域包装配断言；提供真实词典时
        // 追加「无命中不漂移」复核（T-050）。
        Some("--m11") => {
            let optional = args.get(1).map(String::as_str);
            run_m11_checks(optional.map(Path::new))
        }
        // 场景9 通讯录提权验收（FR-036/037/038，验收标准 12.1）：
        // --m12 [<vcf 文件>]，内存构造联系人断言；提供 vcf 时追加真实导入链路复核。
        Some("--m12") => {
            let optional = args.get(1).map(String::as_str);
            run_m12_checks(optional.map(Path::new))
        }
        // 第十一期候选覆盖验收（FR-059，验收标准 16.1）：
        // --m14 [<词典文件>]，内存词表断言具体展开行为；提供真实词典时追加
        // 机制一致性复核（主组条数 <9 → 展开补足，否则零展开）。
        Some("--m14") => {
            let optional = args.get(1).map(String::as_str);
            run_m14_checks(optional.map(Path::new))
        }
        // M6-R 多包回归：--multi-pack <base.zyct> <pack1.zyct> [pack2.zyct ...]
        Some("--multi-pack") => {
            let paths: Vec<PathBuf> = args.iter().skip(1).map(PathBuf::from).collect();
            if paths.is_empty() {
                eprintln!("用法: host-e2e --multi-pack <base.zyct> [pack.zyct ...]");
                return ExitCode::from(2);
            }
            run_multi_pack(&paths)
        }
        Some(path) => run_seed_checks(Path::new(path)),
        None => run_seed_checks(Path::new("data/artifacts/seed.zyct")),
    }
}

/// M6-R 多包回归入口：基础包 + 领域包/网络语包组合。
fn run_multi_pack(paths: &[PathBuf]) -> ExitCode {
    let mut runner = Runner {
        passed: 0,
        failed: 0,
    };
    if let Err(error) = multi_pack_checks(paths, &mut runner) {
        runner.fail("多包回归执行", &error);
    }
    runner.finish()
}

/// 多包回归（FR-015/FR-016/FR-017、验收标准 7.3「多包回归」）：
///
/// 1. **等价性底线**：仅 base 时，复合词典候选与直接使用该包的单词典逐项一致；
/// 2. 启用领域包后领域词可达，基础排序不漂移；
/// 3. 同词跨包去重且词频取 max；
/// 4. 网络语包启用后缩写路径可达并标注 `Slang` 来源；
/// 5. 可切分串不触发缩写路径（防污染）。
fn multi_pack_checks(paths: &[PathBuf], runner: &mut Runner) -> Result<(), String> {
    let base_path = paths
        .first()
        .ok_or_else(|| "至少需要一个基础包路径".to_owned())?;
    let base_file = DictionaryFile::open(base_path)
        .map_err(|error| format!("打开基础包 {base_path:?} 失败: {error}"))?;

    // ---- 1. 等价性底线：仅 base 时复合与单词典逐项一致 ----
    let (composite, skipped) = zhu_ye_core::CompositeDictionary::from_paths(&paths[..1]);
    if !skipped.is_empty() {
        return Err(format!("基础包加载被跳过: {skipped:?}"));
    }
    let base_only: Arc<dyn Dictionary> = Arc::new(base_file.clone());
    let composite_only: Arc<dyn Dictionary> = Arc::new(composite);
    let mut identical = true;
    let mut mismatch = String::new();
    for key in ["nihao", "ni", "zhongguo", "xian", "de", "shijie"] {
        let direct = base_only.lookup(key);
        let merged = composite_only.lookup(key);
        if direct != merged {
            identical = false;
            mismatch = format!(
                "lookup({key}) 不一致: 单词典 {} 条 / 复合 {} 条",
                direct.len(),
                merged.len()
            );
            break;
        }
        let direct_prefix = base_only.lookup_prefix(key);
        let merged_prefix = composite_only.lookup_prefix(key);
        if direct_prefix != merged_prefix {
            identical = false;
            mismatch = format!("lookup_prefix({key}) 不一致");
            break;
        }
    }
    if identical {
        runner.pass("仅基础包时复合词典与单词典逐项一致");
    } else {
        runner.fail("仅基础包时复合词典与单词典逐项一致", &mismatch);
    }

    // ---- 2. 全包装配：基础包 + 其余包 ----
    let (full, full_skipped) = zhu_ye_core::CompositeDictionary::from_paths(paths);
    if !full_skipped.is_empty() {
        runner.fail("全部包成功加载", &format!("{full_skipped:?}"));
    } else {
        runner.pass("全部包成功加载");
    }
    let full_dictionary: Arc<dyn Dictionary> = Arc::new(full.clone());

    // ---- 3. 基础候选在全包模式下不漂移 ----
    let mut base_engine = InputEngine::with_bigram(base_only.clone(), Arc::new(base_file.clone()));
    let mut full_engine = InputEngine::with_bigram(full_dictionary.clone(), Arc::new(full.clone()));
    type_text(&mut base_engine, "nihao");
    type_text(&mut full_engine, "nihao");
    let base_texts: Vec<&str> = base_engine
        .candidates()
        .iter()
        .map(|c| c.text.as_str())
        .collect();
    let full_texts: Vec<&str> = full_engine
        .candidates()
        .iter()
        .map(|c| c.text.as_str())
        .collect();
    // 全包模式的前 N 项应与仅 base 一致（领域包只追加、不改动基础排序）。
    if !base_texts.is_empty() && full_texts.starts_with(&base_texts) {
        runner.pass("全包模式基础候选顺序不漂移");
    } else {
        runner.fail(
            "全包模式基础候选顺序不漂移",
            &format!("base={base_texts:?} full={full_texts:?}"),
        );
    }

    // ---- 4. 同词跨包去重且词频取 max ----
    let merged_nihao = full.lookup("nihao");
    let mut seen = std::collections::HashSet::new();
    let mut duplicated = false;
    for entry in &merged_nihao {
        if !seen.insert(entry.word.clone()) {
            duplicated = true;
        }
    }
    if duplicated {
        runner.fail("同词跨包去重", "合并结果出现重复词");
    } else {
        runner.pass("同词跨包去重");
    }

    // ---- 5. 网络语包缩写路径 ----
    if let Some(slang_path) = paths
        .iter()
        .find(|path| path.file_stem().and_then(|s| s.to_str()) == Some("slang"))
    {
        let slang_file = DictionaryFile::open(slang_path)
            .map_err(|error| format!("打开网络语包 {slang_path:?} 失败: {error}"))?;
        let slang: Arc<dyn Dictionary> = Arc::new(slang_file);
        let mut engine = InputEngine::with_bigram(full_dictionary.clone(), Arc::new(full.clone()))
            .with_slang(slang);
        type_text(&mut engine, "yyds");
        let slang_hit = engine.candidates().iter().find(|c| c.text == "永远的神");
        match slang_hit {
            Some(candidate) if candidate.source == CandidateSource::Slang => {
                runner.pass("网络语缩写精确命中并标注 Slang 来源");
            }
            other => runner.fail(
                "网络语缩写精确命中并标注 Slang 来源",
                &format!("实际: {other:?}"),
            ),
        }
        // 缩写候选应在尾部（独立组）。
        if engine.candidates().last().map(|c| c.text.as_str()) == Some("永远的神") {
            runner.pass("缩写候选位于候选尾部");
        } else {
            runner.fail("缩写候选位于候选尾部", "缩写候选不在尾部");
        }

        engine.handle_escape();
        type_text(&mut engine, "wo");
        let polluted = engine
            .candidates()
            .iter()
            .any(|c| c.source == CandidateSource::Slang);
        if polluted {
            runner.fail("可切分串不触发缩写路径", "wo 触发了缩写候选");
        } else {
            runner.pass("可切分串不触发缩写路径");
        }
    } else {
        println!("[SKIP] 未提供 slang 包，跳过缩写路径断言");
    }

    Ok(())
}

fn run_seed_checks(path: &Path) -> ExitCode {
    let mut runner = Runner {
        passed: 0,
        failed: 0,
    };
    if let Err(error) = seed_checks(path, &mut runner) {
        runner.fail("seed 检查执行", &error);
    }
    runner.finish()
}

fn run_real_smoke(path: &Path) -> ExitCode {
    let mut runner = Runner {
        passed: 0,
        failed: 0,
    };
    if let Err(error) = real_smoke(path, &mut runner) {
        runner.fail("真实词典 smoke 执行", &error);
    }
    runner.finish()
}

/// M7 体验优化验收入口（命令 `--m7 <词典文件>`）。
fn run_m7_checks(path: &Path) -> ExitCode {
    let mut runner = Runner {
        passed: 0,
        failed: 0,
    };
    if let Err(error) = m7_checks(path, &mut runner) {
        runner.fail("M7 体验优化检查执行", &error);
    }
    runner.finish()
}

/// M7 体验优化断言组（验收标准 8.1，FR-023 至 FR-025）。
///
/// 独立词典加载（简拼/纠错需要真实词库）；全组行为与 TSF 无关，主机侧即可断言。
fn m7_checks(path: &Path, runner: &mut Runner) -> Result<(), String> {
    let file = DictionaryFile::open(path)
        .map_err(|error| format!("打开 M7 验收词典 {path:?} 失败: {error}"))?;
    let dictionary: Arc<dyn Dictionary> = Arc::new(file.clone());
    let mut engine = InputEngine::with_bigram(dictionary.clone(), Arc::new(file.clone()));
    engine.handle_escape();

    let type_and = |engine: &mut InputEngine, text: &str| {
        engine.handle_escape();
        type_text(engine, text);
        engine
            .candidates()
            .iter()
            .map(|c| (c.text.clone(), c.source.clone()))
            .collect::<Vec<_>>()
    };

    // ---- FR-023 简拼/首字母输入 ----
    let nh = type_and(&mut engine, "nh");
    if nh.iter().any(|(text, _)| text == "你好") {
        runner.pass("简拼 nh 展开 ni+hao 命中你好");
    } else {
        runner.fail("简拼 nh 展开 ni+hao 命中你好", &format!("实际: {nh:?}"));
    }
    // 排序竞争：简拼候选参与正常排序（高频你好在前，不强制排序仅包含）。
    if nh
        .iter()
        .position(|(text, _)| text == "你好")
        .is_some_and(|index| index < 3)
    {
        runner.pass("简拼候选参与排序且高频词靠前");
    } else {
        runner.fail("简拼候选参与排序且高频词靠前", &format!("实际: {nh:?}"));
    }

    // 三字简拼：词典含该词时才断言（验收注「词典含该词时」）。
    if !dictionary.lookup("weishenme").is_empty() {
        let wsm = type_and(&mut engine, "wsm");
        if wsm.iter().any(|(text, _)| text == "为什么") {
            runner.pass("三字简拼 wsm 命中为什么");
        } else {
            runner.fail("三字简拼 wsm 命中为什么", &format!("实际: {wsm:?}"));
        }
    } else {
        println!("[SKIP] 三字简拼 wsm：词典无「为什么」词条");
    }

    // 单字符防泛滥：n、w 均不触发（长度 <2）。
    if type_and(&mut engine, "n").is_empty() && type_and(&mut engine, "w").is_empty() {
        runner.pass("单字符不触发简拼");
    } else {
        runner.fail("单字符不触发简拼", "n/w 不应产出简拼候选");
    }
    // 超长不触发（长度 >4）：简拼函数级直接拒绝（引擎对超长串走既有前缀/全拼路径）。
    if zhu_ye_core::initial_candidates(dictionary.as_ref(), "abcdefg").is_empty() {
        runner.pass("超长纯字母不触发简拼");
    } else {
        runner.fail(
            "超长纯字母不触发简拼",
            "initial_candidates 对 5 位以上应返回空",
        );
    }
    // 可切分防污染：wo/nihao 保持正常全拼路径，且无 Corrected 噪声。
    let wo = type_and(&mut engine, "wo");
    let nihao = type_and(&mut engine, "nihao");
    let wo_ok = wo.iter().any(|(text, _)| text == "我");
    let wo_clean = wo
        .iter()
        .all(|(_, source)| *source != CandidateSource::Corrected);
    let nihao_ok = nihao.iter().any(|(text, _)| text == "你好");
    if wo_ok && wo_clean && nihao_ok {
        runner.pass("可切分串走正常全拼且无简拼/纠错噪声");
    } else {
        runner.fail(
            "可切分串走正常全拼且无简拼/纠错噪声",
            &format!("wo={wo:?} nihao={nihao:?}"),
        );
    }
    // 含数字不触发简拼：引擎层数字键不进组合（T-049 由 TSF 层接管数字缩写语义，
    // M6-R 多包断言已覆盖 `u1s1` 缩写出「永远的神」），简拼函数自身也拒绝数字。
    engine.handle_escape();
    let digit_rejected = !engine.handle_letter('1');
    let initials_reject_digit =
        zhu_ye_core::initial_candidates(dictionary.as_ref(), "u1s1").is_empty();
    if digit_rejected && initials_reject_digit {
        runner.pass("含数字不触发简拼（数字缩写键语义保留）");
    } else {
        runner.fail(
            "含数字不触发简拼（数字缩写键语义保留）",
            &format!(
                "digit_rejected={digit_rejected} initials_reject_digit={initials_reject_digit}"
            ),
        );
    }
    // 确定性：同一串连续两次候选完全一致。
    type_and(&mut engine, "nh");
    let first = type_and(&mut engine, "nh");
    let second = type_and(&mut engine, "nh");
    if first == second {
        runner.pass("简拼候选确定性");
    } else {
        runner.fail("简拼候选确定性", "两次结果不一致");
    }

    // ---- FR-024 模糊音与纠错 ----
    let zongguo = type_and(&mut engine, "zongguo");
    match zongguo
        .iter()
        .find(|(text, _)| text == "中国")
        .map(|(_, source)| source)
    {
        Some(CandidateSource::Corrected) => {
            runner.pass("模糊替换 zong→zhong 出中国且标注 Corrected");
        }
        other => runner.fail(
            "模糊替换 zong→zhong 出中国且标注 Corrected",
            &format!("实际: {other:?} 全量: {zongguo:?}"),
        ),
    }
    // 模糊音映射 n↔l：`lan` 在真实词库整词命中（烂/蓝/兰…），按设计不触发纠错
    // （O-03 无整词命中才纠错），此处验证映射与无词场景的函数级行为。
    let lan_dictionary =
        InMemoryDictionary::from_entries(vec![DictionaryEntry::new("男", "nan", 600)]);
    let corrected_lan = zhu_ye_core::corrected_candidates(
        &zhu_ye_core::SyllableTable::standard(),
        &lan_dictionary,
        "lan",
    );
    let has_lan_mapping = corrected_lan
        .iter()
        .any(|c| c.text == "男" && c.source == CandidateSource::Corrected);
    if has_lan_mapping {
        runner.pass("模糊替换 n↔l：lan→nan 映射生效");
    } else {
        runner.fail(
            "模糊替换 n↔l：lan→nan 映射生效",
            &format!("实际: {corrected_lan:?}"),
        );
    }
    let lan = type_and(&mut engine, "lan");
    if lan
        .iter()
        .all(|(_, source)| *source != CandidateSource::Corrected)
    {
        runner.pass("lan 整词命中时纠错不介入（防漂移）");
    } else {
        runner.fail(
            "lan 整词命中时纠错不介入（防漂移）",
            &format!("实际: {lan:?}"),
        );
    }
    let niha = type_and(&mut engine, "niha");
    if niha
        .iter()
        .any(|(text, source)| text == "你好" && *source == CandidateSource::Corrected)
    {
        runner.pass("少字母补全 ha→hao：niha 出你好");
    } else {
        runner.fail("少字母补全 ha→hao：niha 出你好", &format!("实际: {niha:?}"));
    }
    // 少字母补全二：`zhonggu` 在真实词库因「中古」整词命中不触发纠错（设计内），
    // 函数级构造无「中古」词的词典验证 gu→guo 补全逻辑本身。
    let zhonggu_dictionary =
        InMemoryDictionary::from_entries(vec![DictionaryEntry::new("中国", "zhongguo", 5000)]);
    let corrected_zhonggu = zhu_ye_core::corrected_candidates(
        &zhu_ye_core::SyllableTable::standard(),
        &zhonggu_dictionary,
        "zhonggu",
    );
    let has_zhonggu = corrected_zhonggu
        .iter()
        .any(|c| c.text == "中国" && c.source == CandidateSource::Corrected);
    if has_zhonggu {
        runner.pass("少字母补全 gu→guo：zhonggu 出中国");
    } else {
        runner.fail(
            "少字母补全 gu→guo：zhonggu 出中国",
            &format!("实际: {corrected_zhonggu:?}"),
        );
    }
    let zhonggu = type_and(&mut engine, "zhonggu");
    if zhonggu
        .iter()
        .all(|(_, source)| *source != CandidateSource::Corrected)
    {
        runner.pass("zhonggu 整词命中（中古）时不叠加纠错");
    } else {
        runner.fail(
            "zhonggu 整词命中（中古）时不叠加纠错",
            &format!("实际: {zhonggu:?}"),
        );
    }
    if nihao
        .iter()
        .all(|(_, source)| *source != CandidateSource::Corrected)
    {
        runner.pass("整词命中不触发纠错（nihao）");
    } else {
        runner.fail("整词命中不触发纠错（nihao）", &format!("实际: {nihao:?}"));
    }
    // 不可切分串不触发纠错：`ww` 会走简拼（wowo 窝窝）但不得出现 Corrected 来源。
    let ww = type_and(&mut engine, "ww");
    if ww
        .iter()
        .all(|(_, source)| *source != CandidateSource::Corrected)
    {
        runner.pass("不可切分串不触发纠错");
    } else {
        runner.fail("不可切分串不触发纠错", &format!("实际: {ww:?}"));
    }
    // 排位：zongguo 的中国候选位于主候选（非 Corrected）之后。
    let zongguo_order = type_and(&mut engine, "zongguo");
    let main_last = zongguo_order
        .iter()
        .rposition(|(_, source)| *source != CandidateSource::Corrected);
    let corrected_first = zongguo_order
        .iter()
        .position(|(_, source)| *source == CandidateSource::Corrected);
    match (main_last, corrected_first) {
        (Some(main), Some(corrected)) if corrected >= main => {
            runner.pass("纠错组追加于主候选之后");
        }
        _ => runner.fail(
            "纠错组追加于主候选之后",
            &format!("实际顺序: {zongguo_order:?}"),
        ),
    }
    type_and(&mut engine, "zongguo");
    let first = type_and(&mut engine, "zongguo");
    let second = type_and(&mut engine, "zongguo");
    if first == second {
        runner.pass("纠错候选确定性");
    } else {
        runner.fail("纠错候选确定性", "两次结果不一致");
    }

    // ---- FR-025 整句/长句输入 ----
    let sentence: Vec<String> = type_and(&mut engine, "woxiangmingtianqubeijing")
        .into_iter()
        .map(|(text, _)| text)
        .collect();
    if sentence
        .first()
        .is_some_and(|text| text == "我想明天去北京")
    {
        runner.pass("整句 beam 出我想明天去北京且居首");
    } else {
        // 失败时输出 beam 顶层句子的分数，便于定位真实词库评分行为。
        let scored = zhu_ye_core::sentence_candidates(
            &zhu_ye_core::SyllableTable::standard(),
            dictionary.as_ref(),
            &file,
            "woxiangmingtianqubeijing",
        )
        .into_iter()
        .map(|c| format!("{}@{}", c.text, c.score))
        .collect::<Vec<_>>();
        // 诊断：尾部选择相关的 bigram 证据与整词词频。
        let diag = [
            ("我→想", file.frequency("我", "想")),
            ("想→明天", file.frequency("想", "明天")),
            ("明天→去", file.frequency("明天", "去")),
            ("去→北京", file.frequency("去", "北京")),
            ("去→被", file.frequency("去", "被")),
            ("被→敬", file.frequency("被", "敬")),
            ("想→名", file.frequency("想", "名")),
            ("名→天", file.frequency("名", "天")),
        ]
        .into_iter()
        .map(|(pair, freq)| format!("{pair}={freq}"))
        .collect::<Vec<_>>();
        let beijing = dictionary
            .lookup("beijing")
            .iter()
            .take(3)
            .map(|e| format!("beijing={}@{}", e.word, e.frequency))
            .collect::<Vec<_>>();
        let jing = dictionary
            .lookup("jing")
            .iter()
            .take(3)
            .map(|e| format!("jing={}@{}", e.word, e.frequency))
            .collect::<Vec<_>>();
        runner.fail(
            "整句 beam 出我想明天去北京且居首",
            &format!(
                "实际前 5: {:?}；beam 分数: {:?}；bigram: {:?}；lookup: {:?}",
                sentence.iter().take(5).collect::<Vec<_>>(),
                scored,
                diag,
                [beijing, jing].concat()
            ),
        );
    }
    // 短串不启用整句：nihao 走现状（第一个候选仍是整词你好）。
    let nihao_short: Vec<String> = type_and(&mut engine, "nihao")
        .into_iter()
        .map(|(text, _)| text)
        .collect();
    if nihao_short.first().is_some_and(|text| text == "你好") {
        runner.pass("短串不启用整句路径（行为不漂移）");
    } else {
        runner.fail(
            "短串不启用整句路径（行为不漂移）",
            &format!("实际: {nihao_short:?}"),
        );
    }
    // 回退安全：长串总能产出 ≥1 候选且不崩溃。
    let fallback = type_and(&mut engine, "woshiyigexuesheng");
    if !fallback.is_empty() {
        runner.pass("长串回退保底非空候选");
    } else {
        runner.fail("长串回退保底非空候选", "woshiyigexuesheng 无候选");
    }
    let sentence_first: Vec<String> = type_and(&mut engine, "woxiangmingtianqubeijing")
        .into_iter()
        .map(|(text, _)| text)
        .collect();
    let sentence_second: Vec<String> = type_and(&mut engine, "woxiangmingtianqubeijing")
        .into_iter()
        .map(|(text, _)| text)
        .collect();
    if sentence_first == sentence_second {
        runner.pass("整句候选确定性");
    } else {
        runner.fail("整句候选确定性", "两次结果不一致");
    }
    // ---- T-056 多音缺读补丁（谁 shui / 熟 shou）与反查越界回归 ----
    let shui: Vec<String> = type_and(&mut engine, "shui")
        .into_iter()
        .map(|(text, _)| text)
        .collect();
    if shui.iter().any(|text| text == "谁") {
        runner.pass("多音补丁 shui 出谁（主诉修复）");
    } else {
        runner.fail("多音补丁 shui 出谁（主诉修复）", &format!("实际: {shui:?}"));
    }
    let shei: Vec<String> = type_and(&mut engine, "shei")
        .into_iter()
        .map(|(text, _)| text)
        .collect();
    if shei.iter().any(|text| text == "谁") {
        runner.pass("多音补丁不覆盖原读音 shei 仍出谁");
    } else {
        runner.fail(
            "多音补丁不覆盖原读音 shei 仍出谁",
            &format!("实际: {shei:?}"),
        );
    }
    let shou: Vec<String> = type_and(&mut engine, "shou")
        .into_iter()
        .map(|(text, _)| text)
        .collect();
    if shou.iter().any(|text| text == "熟") {
        runner.pass("多音补丁 shou 出熟（口语音）");
    } else {
        runner.fail("多音补丁 shou 出熟（口语音）", &format!("实际: {shou:?}"));
    }
    // 反查越界回归：中文键（UTF-8 字节大于全部英文反查键）不得 panic。
    if file.en_to_zh("谁").is_none() {
        runner.pass("反查中文键不越界（T-056 回归）");
    } else {
        runner.fail("反查中文键不越界（T-056 回归）", "en_to_zh(谁) 预期 None");
    }
    Ok(())
}

/// T-059 上屏联想验收入口（命令 `--m8 <词典文件>`）。
fn run_m8_checks(path: &Path) -> ExitCode {
    let mut runner = Runner {
        passed: 0,
        failed: 0,
    };
    if let Err(error) = m8_checks(path, &mut runner) {
        runner.fail("上屏联想检查执行", &error);
    }
    runner.finish()
}

/// T-059 上屏联想断言组（场景5：上屏后空闲候选窗展示 bigram 后继联想）。
///
/// 独立词典加载，与 TSF 无关，主机侧即可断言：上屏进入联想态、
/// 整词在前短语置后、数字选择续联、输入字母退出、Esc 关闭。
fn m8_checks(path: &Path, runner: &mut Runner) -> Result<(), String> {
    let file = DictionaryFile::open(path)
        .map_err(|error| format!("打开联想验收词典 {path:?} 失败: {error}"))?;
    let dictionary: Arc<dyn Dictionary> = Arc::new(file.clone());
    let mut engine = InputEngine::with_bigram(dictionary.clone(), Arc::new(file.clone()));

    // 上屏「今天」，随后进入联想态。
    type_text(&mut engine, "jintian");
    let committed = engine.handle_space();
    if committed.as_deref() != Some("今天") {
        engine.handle_escape();
        return Err(format!("上屏 jintian 未得到「今天」: {committed:?}"));
    }
    if !engine.suggestion_active() {
        return Err("上屏后未进入联想态".to_owned());
    }
    runner.pass("上屏「今天」后进入联想态");
    let list: Vec<String> = engine.suggestion_list().to_vec();
    // T-058 实测契约：今天→的/是/早上/我/在（整词），今日短语置后。
    if list.first().map(String::as_str) == Some("的") {
        runner.pass("联想首条为高频后继「的」");
    } else {
        runner.fail("联想首条为高频后继「的」", &format!("实际: {list:?}"));
    }
    let first_five = list.iter().take(5).collect::<Vec<_>>();
    if first_five.iter().all(|w| !w.starts_with("今天"))
        && list.iter().any(|w| w.starts_with("今天"))
    {
        runner.pass("联想整词在前、短语（今天+后继）置后");
    } else {
        runner.fail(
            "联想整词在前、短语（今天+后继）置后",
            &format!("实际: {list:?}"),
        );
    }
    // 联想态候选窗快照：组合串为空但 items 携带联想列表（TSF 显示依据）。
    let view = engine.candidate_ui_view();
    if view.composition.is_empty() && !view.items.is_empty() {
        runner.pass("联想态候选窗快照组合串为空且 items 非空");
    } else {
        runner.fail(
            "联想态候选窗快照组合串为空且 items 非空",
            &format!("composition={:?} items={:?}", view.composition, view.items),
        );
    }

    // 数字选择第一条联想「的」上屏：作为新前词继续联想（连续联想）。
    let picked = engine.select_index(0);
    if picked.as_deref() != Some("的") {
        return Err(format!("选择联想首条未上屏「的」: {picked:?}"));
    }
    if engine.suggestion_active() && !engine.suggestion_list().is_empty() {
        runner.pass("选择联想词上屏并继续联想（前词更新为「的」）");
    } else {
        runner.fail(
            "选择联想词上屏并继续联想（前词更新为「的」）",
            "「的」的后继在真实语料中应非空",
        );
    }
    // Esc 关闭联想窗。
    if engine.handle_escape() && !engine.suggestion_active() {
        runner.pass("Esc 关闭联想窗");
    } else {
        runner.fail("Esc 关闭联想窗", "联想态 Esc 应清空联想列表");
    }

    // 再上屏一次，用输入字母验证退出联想（回到正常输入路径）。
    type_text(&mut engine, "jintian");
    engine.handle_space();
    if !engine.suggestion_active() {
        return Err("第二次上屏后未进入联想态".to_owned());
    }
    type_text(&mut engine, "n");
    if !engine.suggestion_active() && engine.composing() == "n" {
        runner.pass("输入字母退出联想态回到组合输入");
    } else {
        runner.fail(
            "输入字母退出联想态回到组合输入",
            "字母应清空联想并进入拼音组合",
        );
    }
    engine.handle_escape();
    Ok(())
}

fn run_m9_checks(path: &Path) -> ExitCode {
    let mut runner = Runner {
        passed: 0,
        failed: 0,
    };
    if let Err(error) = m9_checks(path, &mut runner) {
        runner.fail("格式候选检查执行", &error);
    }
    runner.finish()
}

/// 场景7 格式候选断言组（FR-027 数字格式 / FR-028 v 模式符号 / FR-029 emoji）。
///
/// 独立词典加载（验收标准 9.1 同口径），主机侧即可断言引擎完整闭环：
/// 数字边输边上屏、格式候选布局与替换长度、v 模式类型码与回退、emoji 队尾追加。
fn m9_checks(path: &Path, runner: &mut Runner) -> Result<(), String> {
    let mut engine = InputEngine::with_dictionary_file(path)
        .map_err(|error| format!("InputEngine 创建失败: {error}"))?;

    // ---- FR-027 数字格式候选：8 位日期 ----
    for digit in "20260930".chars() {
        assert!(
            engine.digit_append(digit),
            "数字 {digit} 未进入数字格式模式（20260930）"
        );
    }
    if engine.digit_active() && engine.composing().is_empty() {
        runner.pass("数字格式模式：8 位串累积且无拼音组合");
    } else {
        return Err("数字格式模式未激活或出现组合串".to_owned());
    }
    let date_candidates: Vec<String> = engine.candidates().iter().map(|c| c.text.clone()).collect();
    let expected = vec![
        "2026-09-30".to_owned(),
        "2026/09/30".to_owned(),
        "2026年9月30日".to_owned(),
        "2026.09.30".to_owned(),
    ];
    if date_candidates == expected {
        runner.pass("8 位日期 → 4 个格式候选（9.1-用例1）");
    } else {
        runner.fail(
            "8 位日期 → 4 个格式候选（9.1-用例1）",
            &format!("实际: {date_candidates:?}"),
        );
    }
    // 选择第 2 个日期：替换 8 位 buffer，格式文本作为新前词，无候选残留。
    let (committed, replace_len) = engine
        .commit_digit(1)
        .ok_or_else(|| "提交第 2 个日期候选失败".to_owned())?;
    if committed == "2026/09/30" && replace_len == 8 {
        runner.pass("选中第 2 式返回替换长度 8（9.1-用例2 替换链输入）");
    } else {
        runner.fail(
            "选中第 2 式返回替换长度 8（9.1-用例2 替换链输入）",
            &format!("text={committed:?} len={replace_len}"),
        );
    }
    if !engine.digit_active() && engine.candidates().is_empty() {
        runner.pass("提交后退出数字模式且无候选残留（9.1-用例2 无残留）");
    } else {
        runner.fail(
            "提交后退出数字模式且无候选残留（9.1-用例2 无残留）",
            "数字模式仍活跃或有候选",
        );
    }

    // ---- FR-027：数字模式后退格/不足不触发/越界 ----
    for digit in "20260930".chars() {
        engine.digit_append(digit);
    }
    if engine.digit_backspace() && engine.digit_text() == "2026093" {
        runner.pass("数字模式退格删除尾部位");
    } else {
        runner.fail("数字模式退格删除尾部位", "退格未生效");
    }
    engine.exit_digit();
    let mut short = InputEngine::with_dictionary_file(path)
        .map_err(|error| format!("InputEngine 创建失败: {error}"))?;
    for digit in "12".chars() {
        short.digit_append(digit);
    }
    if short.digit_active() && short.candidates().is_empty() {
        runner.pass("不足 5 位不触发格式候选（9.1-用例4）");
    } else {
        runner.fail("不足 5 位不触发格式候选（9.1-用例4）", "出现了格式候选");
    }
    short.exit_digit();

    // ---- FR-027：金额与电话 ----
    let mut amount = InputEngine::with_dictionary_file(path)
        .map_err(|error| format!("InputEngine 创建失败: {error}"))?;
    for digit in ['1', '2', '3', '4', '5', '.', '6'] {
        assert!(amount.digit_append(digit), "金额数字/小数点追加失败");
    }
    let amount_list: Vec<String> = amount.candidates().iter().map(|c| c.text.clone()).collect();
    if amount_list == vec!["12,345.6".to_owned(), "一万二千三百四十五点六".to_owned()] {
        runner.pass("金额 12345.6 → 千分位 + 中文读数（9.1-用例5）");
    } else {
        runner.fail(
            "金额 12345.6 → 千分位 + 中文读数（9.1-用例5）",
            &format!("实际: {amount_list:?}"),
        );
    }
    let mut phone = InputEngine::with_dictionary_file(path)
        .map_err(|error| format!("InputEngine 创建失败: {error}"))?;
    for digit in "13800138000".chars() {
        phone.digit_append(digit);
    }
    let phone_list: Vec<String> = phone.candidates().iter().map(|c| c.text.clone()).collect();
    if phone_list == vec!["138 0013 8000".to_owned(), "138-0013-8000".to_owned()] {
        runner.pass("11 位手机号 → 2 种分段（9.1-用例6）");
    } else {
        runner.fail(
            "11 位手机号 → 2 种分段（9.1-用例6）",
            &format!("实际: {phone_list:?}"),
        );
    }

    // ---- FR-028：v 模式符号 ----
    let mut v1 = InputEngine::with_dictionary_file(path)
        .map_err(|error| format!("InputEngine 创建失败: {error}"))?;
    if !v1.v_start() {
        return Err("空闲态 v_start 未启动 v 模式".to_owned());
    }
    if !v1.v_code('1') {
        return Err("v1 类型码未生效".to_owned());
    }
    if v1.v_symbol_count() == 9 && v1.candidates()[0].text == "①" {
        runner.pass("v1 → 序号符号组 9 项（9.1-用例7）");
    } else {
        runner.fail("v1 → 序号符号组 9 项（9.1-用例7）", "符号候选数或首项不符");
    }
    if v1.select_index(2).as_deref() == Some("③") && !v1.v_active() {
        runner.pass("选第 3 个符号上屏③并退出 v 模式（9.1-用例8）");
    } else {
        runner.fail("选第 3 个符号上屏③并退出 v 模式（9.1-用例8）", "选择异常");
    }
    // vx 数学组、vh 标点组。
    let mut vx = InputEngine::with_dictionary_file(path)
        .map_err(|error| format!("InputEngine 创建失败: {error}"))?;
    vx.v_start();
    vx.v_code('x');
    if vx.candidates()[0].text == "±" && vx.candidates().iter().any(|c| c.text == "∞") {
        runner.pass("vx → 数学符号组（±×÷≈≠≤≥∞％）");
    } else {
        runner.fail("vx → 数学符号组（±×÷≈≠≤≥∞％）", "数学组缺失");
    }
    let mut vh = InputEngine::with_dictionary_file(path)
        .map_err(|error| format!("InputEngine 创建失败: {error}"))?;
    vh.v_start();
    vh.v_code('h');
    if vh.candidates()[0].text == "，" {
        runner.pass("vh → 中文标点组（，。！？、；：\"\"）");
    } else {
        runner.fail("vh → 中文标点组（，。！？、；：\"\"）", "标点组缺失");
    }
    // vi 回退拼音。
    let mut vi = InputEngine::with_dictionary_file(path)
        .map_err(|error| format!("InputEngine 创建失败: {error}"))?;
    vi.v_start();
    if !vi.v_consume('i') || vi.composing() != "vi" || vi.v_active() {
        runner.fail("vi 回退拼音组合（9.1-用例10）", "vi 未进入拼音组合");
    } else {
        runner.pass("vi 回退拼音组合（9.1-用例10）");
    }

    // ---- FR-029：emoji 队尾追加 ----
    // 真实词典 xiao 有「小」；emoji 表别名 xiao → 😄 追候选尾部。
    let mut emoji_engine = InputEngine::with_dictionary_file(path)
        .map_err(|error| format!("InputEngine 创建失败: {error}"))?;
    type_text(&mut emoji_engine, "xiao");
    let tail = emoji_engine.candidates().last();
    if tail.map(|c| c.text.as_str()) == Some("😄")
        && tail.map(|c| c.source == zhu_ye_core::candidate::CandidateSource::Emoji) == Some(true)
        && emoji_engine.candidates().iter().any(|c| c.text == "小")
    {
        runner.pass("xiao → emoji😄追候选尾部且「小」保留在列（9.1-用例9）");
    } else {
        runner.fail(
            "xiao → emoji😄追候选尾部且「小」保留在列（9.1-用例9）",
            "emoji 未追加或拼音候选被挤掉",
        );
    }
    // 无别名命中不追加 emoji（来源过滤）。
    let mut miss = InputEngine::with_dictionary_file(path)
        .map_err(|error| format!("InputEngine 创建失败: {error}"))?;
    type_text(&mut miss, "nihao");
    if miss
        .candidates()
        .iter()
        .all(|c| c.source != zhu_ye_core::candidate::CandidateSource::Emoji)
    {
        runner.pass("无别名命中不追加 emoji（9.1-用例11）");
    } else {
        runner.fail("无别名命中不追加 emoji（9.1-用例11）", "出现了 emoji 候选");
    }
    engine.handle_escape();
    Ok(())
}

/// 场景6 中英混输验收入口（命令 `--m10 <词典文件>`）。
fn run_m10_checks(path: &Path) -> ExitCode {
    let mut runner = Runner {
        passed: 0,
        failed: 0,
    };
    if let Err(error) = m10_checks(path, &mut runner) {
        runner.fail("中英混输检查执行", &error);
    }
    runner.finish()
}

/// 场景6 中英混输断言组（FR-030/FR-031/FR-032，验收标准 10.1）。
///
/// 英文候选与邮箱/网址补全均不依赖词典（EN_WORDS 内嵌、格式规则纯字符串），
/// 因此任何词典文件（种子或真实）都能完整断言；`yyds` 缩写断言依赖词典含
/// 网络语词条，缺失时按 m7 惯例 SKIP。
fn m10_checks(path: &Path, runner: &mut Runner) -> Result<(), String> {
    let file = DictionaryFile::open(path)
        .map_err(|error| format!("打开 M10 验收词典 {path:?} 失败: {error}"))?;
    let dictionary: Arc<dyn Dictionary> = Arc::new(file.clone());

    let mut engine = InputEngine::with_dictionary_file(path)
        .map_err(|error| format!("InputEngine 创建失败: {error}"))?;
    engine.handle_escape();

    // ---- FR-030 英文拼写补全（不可切分整串才触发）----
    type_text(&mut engine, "pytho");
    let cands: Vec<(String, CandidateSource)> = engine
        .candidates()
        .iter()
        .map(|c| (c.text.clone(), c.source.clone()))
        .collect();
    if cands
        .first()
        .is_some_and(|(text, source)| text == "python" && *source == CandidateSource::EnWord)
    {
        runner.pass("拼写补全 pytho → python（10.1-用例1）");
    } else {
        runner.fail(
            "拼写补全 pytho → python（10.1-用例1）",
            &format!("实际: {cands:?}"),
        );
    }

    // 大小写原形：iphon → iPhone（D-09 不猜测大小写，表内原形）。
    engine.handle_escape();
    type_text(&mut engine, "iphon");
    let iphon = engine
        .candidates()
        .iter()
        .any(|c| c.text == "iPhone" && c.source == CandidateSource::EnWord);
    if iphon {
        runner.pass("大小写原形 iphon → iPhone（10.1-用例2）");
    } else {
        runner.fail("大小写原形 iphon → iPhone（10.1-用例2）", "未命中 iPhone");
    }

    // 拼音不介入：可切分串绝不进入英文路径（D-10）。
    engine.handle_escape();
    type_text(&mut engine, "nihao");
    let nihao_clean = engine
        .candidates()
        .iter()
        .all(|c| c.source != CandidateSource::EnWord && c.source != CandidateSource::EmailUrl);
    if nihao_clean && engine.candidates().iter().any(|c| c.text == "你好") {
        runner.pass("拼音串不介入英文/格式路径（10.1-用例3）");
    } else {
        runner.fail(
            "拼音串不介入英文/格式路径（10.1-用例3）",
            "出现英文/格式候选",
        );
    }

    // 缩写组不回退：yyds 无英文命中时原样走网络语缩写路径（保 Slang）。
    engine.handle_escape();
    type_text(&mut engine, "yyds");
    let yyds_hit = engine
        .candidates()
        .iter()
        .any(|c| c.text == "永远的神" && c.source == CandidateSource::Slang);
    let yyds_has_en = engine
        .candidates()
        .iter()
        .any(|c| c.source == CandidateSource::EnWord);
    if yyds_hit {
        runner.pass("缩写组不回退 yyds → 永远的神（10.1-用例4）");
    } else if !dictionary.lookup_prefix("yyds").is_empty() || yyds_has_en {
        runner.fail(
            "缩写组不回退 yyds → 永远的神（10.1-用例4）",
            &format!("实际: {:?}", engine.candidates()),
        );
    } else {
        println!("[SKIP] yyds 缩写断言：词典无网络语词条");
    }

    // ---- FR-031 邮箱补全与直通 ----
    engine.handle_escape();
    type_format(&mut engine, "me@163");
    let mail: Vec<String> = engine.candidates().iter().map(|c| c.text.clone()).collect();
    if mail == vec!["me@163.com", "me@163.cn", "me@163.net"]
        && engine
            .candidates()
            .iter()
            .all(|c| c.source == CandidateSource::EmailUrl)
    {
        runner.pass("邮箱补全 me@163 → .com/.cn/.net（10.1-用例5）");
    } else {
        runner.fail(
            "邮箱补全 me@163 → .com/.cn/.net（10.1-用例5）",
            &format!("实际: {mail:?}"),
        );
    }
    // 已含点完整串直通。
    engine.handle_escape();
    type_format(&mut engine, "a@b.c");
    if engine.candidates().len() == 1 && engine.candidates()[0].text == "a@b.c" {
        runner.pass("邮箱完整串直通（10.1-用例5 直通）");
    } else {
        runner.fail(
            "邮箱完整串直通（10.1-用例5 直通）",
            &format!("实际: {:?}", engine.candidates()),
        );
    }

    // ---- FR-031 网址补全与直通 ----
    engine.handle_escape();
    type_format(&mut engine, "www.exa");
    let www: Vec<String> = engine.candidates().iter().map(|c| c.text.clone()).collect();
    if www == vec!["www.exa.com", "www.exa.cn", "www.exa.org"] {
        runner.pass("网址补全 www.exa → .com/.cn/.org（10.1-用例6）");
    } else {
        runner.fail(
            "网址补全 www.exa → .com/.cn/.org（10.1-用例6）",
            &format!("实际: {www:?}"),
        );
    }
    engine.handle_escape();
    type_format(&mut engine, "http://exa");
    let http: Vec<String> = engine.candidates().iter().map(|c| c.text.clone()).collect();
    if http == vec!["http://exa.com", "http://exa.cn", "http://exa.org"] {
        runner.pass("网址补全 http://exa → 3 条（10.1-用例6 scheme）");
    } else {
        runner.fail(
            "网址补全 http://exa → 3 条（10.1-用例6 scheme）",
            &format!("实际: {http:?}"),
        );
    }
    engine.handle_escape();
    type_format(&mut engine, "www.exa.com");
    if engine.candidates().len() == 1 && engine.candidates()[0].text == "www.exa.com" {
        runner.pass("网址完整串直通（10.1-用例6 直通）");
    } else {
        runner.fail(
            "网址完整串直通（10.1-用例6 直通）",
            &format!("实际: {:?}", engine.candidates()),
        );
    }

    // ---- FR-032 提交/退出交互 ----
    engine.handle_escape();
    type_format(&mut engine, "me@163");
    let committed = engine
        .select_index(0)
        .ok_or_else(|| "选择邮箱补全首候选失败".to_owned())?;
    if committed == "me@163.com" && !engine.is_active() {
        runner.pass("选中补全尾候选上屏且组合清空（10.1-用例7）");
    } else {
        runner.fail(
            "选中补全尾候选上屏且组合清空（10.1-用例7）",
            &format!("text={committed:?}"),
        );
    }
    engine.handle_escape();
    type_format(&mut engine, "www.exa");
    assert!(engine.handle_escape());
    if !engine.is_active() && engine.candidates().is_empty() {
        runner.pass("Esc 清空邮箱/网址组合回空闲（10.1-用例7 退出）");
    } else {
        runner.fail(
            "Esc 清空邮箱/网址组合回空闲（10.1-用例7 退出）",
            "仍活跃或有候选",
        );
    }
    // 退格删掉 @ 退出邮箱态回到拼音（me 为可切分音节，按 D-10 走拼音路径）。
    engine.handle_escape();
    type_format(&mut engine, "me@163");
    for _ in 0..4 {
        assert!(engine.handle_backspace(), "邮箱组合退格未生效");
    }
    if engine.composing() == "me"
        && engine
            .candidates()
            .iter()
            .all(|c| c.source != CandidateSource::EmailUrl)
    {
        runner.pass("退格删除 @ 退出邮箱态（10.1-用例7 退格）");
    } else {
        runner.fail(
            "退格删除 @ 退出邮箱态（10.1-用例7 退格）",
            &format!("composing={}", engine.composing()),
        );
    }

    // 噪声：不可切分且无英文命中的串不出 EnWord/EmailUrl（空或既有拼音路径）。
    engine.handle_escape();
    type_text(&mut engine, "xjxq");
    let noise = engine
        .candidates()
        .iter()
        .all(|c| c.source != CandidateSource::EnWord && c.source != CandidateSource::EmailUrl);
    if noise {
        runner.pass("噪声串不产生英文/格式候选（10.1-用例8）");
    } else {
        runner.fail(
            "噪声串不产生英文/格式候选（10.1-用例8）",
            "出现英文/格式候选",
        );
    }

    Ok(())
}

/// 场景8 领域自动验收入口（命令 `--m11 [<词典文件>]`）。
fn run_m11_checks(path: Option<&Path>) -> ExitCode {
    let mut runner = Runner {
        passed: 0,
        failed: 0,
    };
    if let Err(error) = m11_checks(path, &mut runner) {
        runner.fail("领域自动检查执行", &error);
    }
    runner.finish()
}

/// 场景8 领域自动断言组（FR-033/FR-034/FR-035，验收标准 11.1）。
///
/// 基础包与领域包都用**内存构造词表**：真实 base 是 28 万词条大通用词典，
/// 对领域包拼音几乎总能整词/组合出同文本，提权样例会被「同文本去重」吸收
/// （这正是 T-050 基线的表现），无法稳定构造可观测的新文本提权；内存词表
/// 则全部可控、确定：
///
/// - `jubu → 局部`：基础无该拼音（基础候选仅由不可切分前缀路径回退），
///   it 域词「局部」整词命中 → 新文本提权（11.1-用例1）；
/// - `yuyan → 语言`：基楚有「语言」，it 域词「语研」不同文本 → D-15 前缀
///   验证用（yuyan 禁止提权）；完整词 `yuyanbiancheng` 不提权（med 无）；
/// - `ai → 欸`：基础有「爱」，it 域词「欸」不同文本；同时 `ai` 命中 emoji
///   别名 ❤️ → 验证 D-13 位次协调（基础 → 领域 → 追加组 emoji 队尾）；
/// - `nihao → 你好`：无领域命中 → T-050 逐位一致。
///
/// 提供真实词典文件时，额外用 `nihao` 复核真实上下文不漂移（见用例 3）。
fn m11_checks(path: Option<&Path>, runner: &mut Runner) -> Result<(), String> {
    let base_dict: Arc<dyn Dictionary> = Arc::new(InMemoryDictionary::from_entries(vec![
        DictionaryEntry::new("你好", "nihao", 100),
        DictionaryEntry::new("你", "ni", 90),
        DictionaryEntry::new("语言", "yuyan", 80),
        DictionaryEntry::new("爱", "ai", 85),
    ]));
    let it: Arc<dyn Dictionary> = Arc::new(InMemoryDictionary::from_entries(vec![
        DictionaryEntry::new("局部", "jubu", 800),
        DictionaryEntry::new("语研", "yuyan", 60),
        DictionaryEntry::new("欸", "ai", 30),
        DictionaryEntry::new("拟", "ni", 20),
    ]));
    let med: Arc<dyn Dictionary> = Arc::new(InMemoryDictionary::from_entries(vec![
        DictionaryEntry::new("局部麻醉", "jubumazui", 700),
        DictionaryEntry::new("医研", "yuyan", 50),
    ]));

    let mut engine =
        InputEngine::with_bigram(base_dict.clone(), Arc::new(InMemoryBigramModel::new()))
            .with_domain_packs(vec![("it".to_owned(), it.clone())]);
    engine.handle_escape();

    let snapshot = |engine: &mut InputEngine| {
        engine
            .candidates()
            .iter()
            .map(|c| (c.text.clone(), c.source.clone()))
            .collect::<Vec<_>>()
    };

    // ---- 11.1-用例1：完整词命中提权出新文本（D-15 完整词，D-13 位次）----
    type_text(&mut engine, "jubu");
    let cands = snapshot(&mut engine);
    let jb_idx = cands.iter().position(|(text, _)| text == "局部");
    match jb_idx {
        Some(index) if cands[index].1 == CandidateSource::Domain => {
            let before_clean = cands[..index]
                .iter()
                .all(|(_, source)| *source != CandidateSource::Domain);
            // 基础候选非空（jubumazui 的 jubu 前缀 → 基础可能为空或回退），
            // 只要领域候选之前无其他领域候选即满足组内前移语义。
            if before_clean {
                runner.pass("完整词命中提权出领域新候选（11.1-用例1）");
            } else {
                runner.fail(
                    "完整词命中提权出领域新候选（11.1-用例1）",
                    &format!("领域候选之前混入领域候选: {cands:?}"),
                );
            }
        }
        other => runner.fail(
            "完整词命中提权出领域新候选（11.1-用例1）",
            &format!("实际: {other:?} 候选: {cands:?}"),
        ),
    }

    // ---- D-13 位次协调：领域候选在基础之后、emoji 追加组之前 ----
    // ai：基础（爱）+ 领域（欸）+ emoji（❤️），顺序 = 爱 … 欸 … ❤️。
    engine.handle_escape();
    type_text(&mut engine, "ai");
    let ai_cands = snapshot(&mut engine);
    let ai_pos = ai_cands.iter().position(|(t, _)| t == "爱");
    let domain_pos = ai_cands.iter().position(|(t, _)| t == "欸");
    let emoji_pos = ai_cands
        .iter()
        .position(|(_, source)| *source == CandidateSource::Emoji);
    if ai_pos.is_some()
        && domain_pos.is_some_and(|index| ai_pos.unwrap() < index)
        && emoji_pos.is_some_and(|index| domain_pos.unwrap() < index)
    {
        runner.pass("位次协调：基础 < 领域 < emoji 追加组（11.1-用例1 位次）");
    } else {
        runner.fail(
            "位次协调：基础 < 领域 < emoji 追加组（11.1-用例1 位次）",
            &format!("ai={ai_pos:?} domain={domain_pos:?} emoji={emoji_pos:?} 候选: {ai_cands:?}"),
        );
    }

    // ---- 11.1-用例2：前缀命中不提权（D-15）----
    // yuyan 整词命中域包「语研」；yuyan 是完整拼音，lookup 精确命中 → 提权。
    // 前缀不说：yuy ？yuy 是 yuyan 前缀 → 域包 lookup("yuy") 空 → 不提权。
    engine.handle_escape();
    type_text(&mut engine, "yuy");
    let yuy_clean = engine
        .candidates()
        .iter()
        .all(|c| c.source != CandidateSource::Domain);
    if yuy_clean {
        runner.pass("前缀命中不提权（11.1-用例2）");
    } else {
        runner.fail("前缀命中不提权（11.1-用例2）", "yuy 出现领域候选");
    }

    // ---- 11.1-用例3：无命中不漂移（T-050 基线）----
    engine.handle_escape();
    type_text(&mut engine, "nihao");
    let boosted_texts: Vec<String> = engine.candidates().iter().map(|c| c.text.clone()).collect();
    let mut baseline =
        InputEngine::with_bigram(base_dict.clone(), Arc::new(InMemoryBigramModel::new()));
    baseline.handle_escape();
    type_text(&mut baseline, "nihao");
    let baseline_texts: Vec<String> = baseline
        .candidates()
        .iter()
        .map(|c| c.text.clone())
        .collect();
    let no_domain = engine
        .candidates()
        .iter()
        .all(|c| c.source != CandidateSource::Domain);
    if boosted_texts == baseline_texts && no_domain && boosted_texts.iter().any(|t| t == "你好") {
        runner.pass("无领域命中时与仅基础包逐位一致（11.1-用例3）");
    } else {
        runner.fail(
            "无领域命中时与仅基础包逐位一致（11.1-用例3）",
            &format!("boosted={boosted_texts:?} baseline={baseline_texts:?}"),
        );
    }

    // ---- 真实词典复核：无命中不漂移（提供文件时）----
    if let Some(real) = path {
        let file = DictionaryFile::open(real)
            .map_err(|error| format!("打开真实词典 {real:?} 失败: {error}"))?;
        let real_dict: Arc<dyn Dictionary> = Arc::new(file.clone());
        let mut real_boost = InputEngine::with_bigram(real_dict.clone(), Arc::new(file.clone()))
            .with_domain_packs(vec![("it".to_owned(), it.clone())]);
        real_boost.handle_escape();
        type_text(&mut real_boost, "nihao");
        let mut real_base = InputEngine::with_bigram(real_dict.clone(), Arc::new(file.clone()));
        real_base.handle_escape();
        type_text(&mut real_base, "nihao");
        let rb: Vec<String> = real_boost
            .candidates()
            .iter()
            .map(|c| c.text.clone())
            .collect();
        let rba: Vec<String> = real_base
            .candidates()
            .iter()
            .map(|c| c.text.clone())
            .collect();
        let real_no_domain = real_boost
            .candidates()
            .iter()
            .all(|c| c.source != CandidateSource::Domain);
        if rb == rba && real_no_domain {
            runner.pass("真实词典上下文无命中不漂移（11.1-用例3 真实复核）");
        } else {
            runner.fail(
                "真实词典上下文无命中不漂移（11.1-用例3 真实复核）",
                &format!("boosted={rb:?} baseline={rba:?}"),
            );
        }
    } else {
        println!("[SKIP] 未提供真实词典，跳过真实上下文复核");
    }

    // ---- 11.1-用例4：开关关闭恢复追加语义（D-14/D-17）----
    let mut off = InputEngine::with_bigram(base_dict.clone(), Arc::new(InMemoryBigramModel::new()))
        .with_domain_packs(vec![("it".to_owned(), it.clone())])
        .with_domain_boost(false);
    off.handle_escape();
    type_text(&mut off, "jubu");
    let off_clean = off
        .candidates()
        .iter()
        .all(|c| c.source != CandidateSource::Domain);
    if off_clean {
        runner.pass("提权开关关闭不产出领域候选（11.1-用例4）");
    } else {
        runner.fail(
            "提权开关关闭不产出领域候选（11.1-用例4）",
            "关闭后仍出现 Domain 候选",
        );
    }

    // ---- D-16：多包同整词命中取 id 字典序首个（it < med）----
    // yuyan：it「语研」与 med「医研」同时整词命中 → 取 it（字典序首个）。
    let mut multi =
        InputEngine::with_bigram(base_dict.clone(), Arc::new(InMemoryBigramModel::new()))
            .with_domain_packs(vec![
                ("it".to_owned(), it.clone()),
                ("med".to_owned(), med.clone()),
            ]);
    multi.handle_escape();
    type_text(&mut multi, "yuyan");
    let domain_texts: Vec<String> = multi
        .candidates()
        .iter()
        .filter(|c| c.source == CandidateSource::Domain)
        .map(|c| c.text.clone())
        .collect();
    if domain_texts.first().is_some_and(|t| t == "语研") {
        runner.pass("多包同命中取字典序首个包（11.1-用例5）");
    } else {
        runner.fail(
            "多包同命中取字典序首个包（11.1-用例5）",
            &format!("领域候选: {domain_texts:?}"),
        );
    }

    Ok(())
}

/// 场景9 通讯录提权验收入口（命令 `--m12 [<vcf 文件>]`）。
fn run_m12_checks(path: Option<&Path>) -> ExitCode {
    let mut runner = Runner {
        passed: 0,
        failed: 0,
    };
    if let Err(error) = m12_checks(path, &mut runner) {
        runner.fail("场景9 通讯录提权执行", &error);
    }
    runner.finish()
}

/// 场景9 通讯录提权断言组（FR-036/FR-037/FR-038，验收标准 12.1）。
///
/// 基础词表内存构造、联系人内存构造（`build_contact_index`），全部可控确定：
///
/// - `zhang → 张三`：全拼前缀命中，联系人候选以 `Contact` 来源插基础候选之后
///   （D-21 与 D-13 同层）；
/// - `zs → 张三`：简拼键命中（FR-037 简拼可达）；
/// - `zz/cz → 曾子`：多音字简拼多形态（D-22 全读音形态原则延伸）；
/// - `al → Alice`：英文名原文小写键（D-20 仅姓名）；
/// - `nihao → 你好`：无联系人命中 → 与无联系人引擎逐位一致（T-050 基线）；
/// - `clear_contacts` 后 `zhang` 不再出联系人（FR-038 清除能力）。
///
/// 提供真实 vcf 文件时，追加「真实导入 → 姓名可达」复核（走 `parse_vcard` 链路）。
fn m12_checks(path: Option<&Path>, runner: &mut Runner) -> Result<(), String> {
    let base_dict: Arc<dyn Dictionary> = Arc::new(InMemoryDictionary::from_entries(vec![
        DictionaryEntry::new("你好", "nihao", 100),
        DictionaryEntry::new("张", "zhang", 95),
        DictionaryEntry::new("章鱼", "zhangyu", 90),
        DictionaryEntry::new("阿里", "ali", 88),
        DictionaryEntry::new("爱", "ai", 85),
    ]));

    let contacts = vec![
        VCardContact {
            name: "张三".to_owned(),
            keys: Vec::new(),
        },
        VCardContact {
            name: "曾子".to_owned(),
            keys: Vec::new(),
        },
        VCardContact {
            name: "Alice".to_owned(),
            keys: Vec::new(),
        },
    ];
    let index = build_contact_index(&contacts);

    let mut engine =
        InputEngine::with_bigram(base_dict.clone(), Arc::new(InMemoryBigramModel::new()))
            .with_contacts(index);
    engine.handle_escape();

    let snapshot = |engine: &mut InputEngine| {
        engine
            .candidates()
            .iter()
            .map(|c| (c.text.clone(), c.source.clone()))
            .collect::<Vec<_>>()
    };

    // ---- 12.1-用例1：全拼前缀命中 + 位次（基础之后、Contact 来源）----
    type_text(&mut engine, "zhang");
    let cands = snapshot(&mut engine);
    let contact_pos = cands
        .iter()
        .position(|(t, s)| t == "张三" && *s == CandidateSource::Contact);
    let base_pos = cands.iter().position(|(t, _)| t == "张" || t == "章鱼");
    match (contact_pos, base_pos) {
        (Some(contact), Some(base)) if base < contact => {
            runner.pass("全拼前缀命中且排在基础候选之后（12.1-用例1）");
        }
        other => runner.fail(
            "全拼前缀命中且排在基础候选之后（12.1-用例1）",
            &format!("contact={:?} base={:?} 候选: {cands:?}", other.0, other.1),
        ),
    }

    // ---- 12.1-用例2：简拼键命中（FR-037）----
    engine.handle_escape();
    type_text(&mut engine, "zs");
    let zs_cands = snapshot(&mut engine);
    if zs_cands
        .iter()
        .any(|(t, s)| t == "张三" && *s == CandidateSource::Contact)
    {
        runner.pass("简拼 zs 命中张三（12.1-用例2）");
    } else {
        runner.fail(
            "简拼 zs 命中张三（12.1-用例2）",
            &format!("候选: {zs_cands:?}"),
        );
    }

    // ---- 12.1-用例3：多音字简拼多形态（D-22）----
    engine.handle_escape();
    type_text(&mut engine, "cz");
    let cz_cands = snapshot(&mut engine);
    if cz_cands
        .iter()
        .any(|(t, s)| t == "曾子" && *s == CandidateSource::Contact)
    {
        runner.pass("多音简拼 cz 命中曾子（12.1-用例3）");
    } else {
        runner.fail(
            "多音简拼 cz 命中曾子（12.1-用例3）",
            &format!("候选: {cz_cands:?}"),
        );
    }

    // ---- 12.1-用例4：英文名原文键（D-20）----
    engine.handle_escape();
    type_text(&mut engine, "al");
    let al_cands = snapshot(&mut engine);
    let alice_ok = al_cands
        .iter()
        .any(|(t, s)| t == "Alice" && *s == CandidateSource::Contact);
    // alice 与 base 阿里(ali) 同前缀；Alice 在联系人组内（排在基础候选之后）。
    let alice_pos = al_cands.iter().position(|(t, _)| t == "Alice");
    let ali_pos = al_cands.iter().position(|(t, _)| t == "阿里");
    if alice_ok && alice_pos.zip(ali_pos).is_some_and(|(a, b)| b < a) {
        runner.pass("英文名原文键命中并靠后位次（12.1-用例4）");
    } else {
        runner.fail(
            "英文名原文键命中并靠后位次（12.1-用例4）",
            &format!("alice={alice_pos:?} ali={ali_pos:?} 候选: {al_cands:?}"),
        );
    }

    // ---- 12.1-用例5：无命中不漂移（T-050 基线）----
    engine.handle_escape();
    type_text(&mut engine, "nihao");
    let boosted_texts: Vec<String> = engine.candidates().iter().map(|c| c.text.clone()).collect();
    let mut baseline =
        InputEngine::with_bigram(base_dict.clone(), Arc::new(InMemoryBigramModel::new()));
    baseline.handle_escape();
    type_text(&mut baseline, "nihao");
    let baseline_texts: Vec<String> = baseline
        .candidates()
        .iter()
        .map(|c| c.text.clone())
        .collect();
    let no_contact = engine
        .candidates()
        .iter()
        .all(|c| c.source != CandidateSource::Contact);
    if boosted_texts == baseline_texts && no_contact && boosted_texts.iter().any(|t| t == "你好")
    {
        runner.pass("无联系人命中时与无联系人引擎逐位一致（12.1-用例5）");
    } else {
        runner.fail(
            "无联系人命中时与无联系人引擎逐位一致（12.1-用例5）",
            &format!("boosted={boosted_texts:?} baseline={baseline_texts:?}"),
        );
    }

    // ---- 12.1-用例6：清除后恢复基线（FR-038）----
    engine.handle_escape();
    engine.clear_contacts();
    type_text(&mut engine, "zhang");
    let cleared = snapshot(&mut engine);
    if cleared.iter().all(|(_, s)| *s != CandidateSource::Contact) {
        runner.pass("清除联系人后不产出联系人候选（12.1-用例6）");
    } else {
        runner.fail(
            "清除联系人后不产出联系人候选（12.1-用例6）",
            &format!("候选: {cleared:?}"),
        );
    }

    // ---- 真实 vcf 导入复核（提供文件时）----
    if let Some(vcf) = path {
        let text =
            fs::read_to_string(vcf).map_err(|error| format!("读取 vcf {vcf:?} 失败: {error}"))?;
        let parsed =
            parse_vcard(&text).map_err(|error| format!("解析 vcf {vcf:?} 失败: {error:?}"))?;
        let real_index = build_contact_index(&parsed);
        let mut real =
            InputEngine::with_bigram(base_dict.clone(), Arc::new(InMemoryBigramModel::new()))
                .with_contacts(real_index);
        real.handle_escape();
        type_text(&mut real, "zhang");
        let real_cands: Vec<String> = real.candidates().iter().map(|c| c.text.clone()).collect();
        let hits_zhang_san = real_cands.iter().any(|t| t == "张三");
        if hits_zhang_san {
            runner.pass("真实 vcf 导入链路姓名可达（12.1-真实复核）");
        } else {
            runner.fail(
                "真实 vcf 导入链路姓名可达（12.1-真实复核）",
                &format!("候选: {real_cands:?}（vcf 需含「张三」姓名）"),
            );
        }
    } else {
        println!("[SKIP] 未提供 vcf 文件，跳过真实导入链路复核");
    }

    Ok(())
}

/// 第十一期候选覆盖验收入口（命令 `--m14 [<词典文件>]`）。
fn run_m14_checks(path: Option<&Path>) -> ExitCode {
    let mut runner = Runner {
        passed: 0,
        failed: 0,
    };
    if let Err(error) = m14_checks(path, &mut runner) {
        runner.fail("候选覆盖检查执行", &error);
    }
    runner.finish()
}

/// 第十一期候选覆盖断言组（FR-059，验收标准 16.1）。
///
/// 基础词表内存构造，全部可控确定：
///
/// - `shui → 说/睡/水`（主组 3 条 <9）+ 前缀更深组词 6 条（水稻/水果/水平/
///   睡觉/水面/水灾）→ 首屏补足到 9 条、展开组 `PrefixExpand` 独立组在主组之后
///   （16.1-1 低频音节首屏补足 + 排序）；
/// - `de → 的×9`（整词命中 ≥9）→ 零展开（D-70 常见拼音逐位不变）；
/// - `shuip`（不完整拼音）→ 走既有前缀补全路径（FR-023 互斥，展开组不出现）；
/// - `shui` 首屏无重复文本（去重链：主组优先 + 展开组内部去重）。
///
/// 提供真实词典文件时，追加机制一致性复核：主组条数 <9 → 出现展开组且条数
/// ≤ 9−主组；主组 ≥9 → 零展开（真实数据上验证触发口径，不写死具体词频）。
fn m14_checks(path: Option<&Path>, runner: &mut Runner) -> Result<(), String> {
    let base: Arc<dyn Dictionary> = Arc::new(InMemoryDictionary::from_entries(vec![
        DictionaryEntry::new("说", "shui", 413852),
        DictionaryEntry::new("睡", "shui", 80000),
        DictionaryEntry::new("水", "shui", 60000),
        DictionaryEntry::new("水稻", "shuidao", 120000),
        DictionaryEntry::new("水果", "shuiguo", 90000),
        DictionaryEntry::new("水平", "shuiping", 85000),
        DictionaryEntry::new("睡觉", "shuijiao", 70000),
        DictionaryEntry::new("水面", "shuimian", 65000),
        DictionaryEntry::new("水灾", "shuizai", 64000),
        DictionaryEntry::new("的", "de", 5000),
        DictionaryEntry::new("得", "de", 4900),
        DictionaryEntry::new("地", "de", 4800),
        DictionaryEntry::new("德", "de", 4700),
        DictionaryEntry::new("灯", "de", 4600),
        DictionaryEntry::new("等", "de", 4500),
        DictionaryEntry::new("低", "de", 4400),
        DictionaryEntry::new("滴", "de", 4300),
        DictionaryEntry::new("底", "de", 4200),
        DictionaryEntry::new("笛", "de", 4100),
    ]));

    let mut engine = InputEngine::with_bigram(base.clone(), Arc::new(InMemoryBigramModel::new()));
    engine.handle_escape();

    let snapshot = |engine: &mut InputEngine| {
        engine
            .candidates()
            .iter()
            .filter(|c| c.source != CandidateSource::Emoji)
            .map(|c| (c.text.clone(), c.source.clone()))
            .collect::<Vec<_>>()
    };

    // ---- 16.1-1：低频音节首屏补足到一页（D-70/D-71）----
    type_text(&mut engine, "shui");
    let cands = snapshot(&mut engine);
    let texts: Vec<&str> = cands.iter().map(|(text, _)| text.as_str()).collect();
    let expected = [
        "说", "睡", "水", "水稻", "水果", "水平", "睡觉", "水面", "水灾",
    ];
    if texts == expected {
        runner.pass("低频音节首屏补足到 9 条（16.1-1 展开）");
    } else {
        runner.fail(
            "低频音节首屏补足到 9 条（16.1-1 展开）",
            &format!("实际: {texts:?}，期望: {expected:?}"),
        );
    }
    let expand_sources_ok = cands[3..]
        .iter()
        .all(|(_, source)| *source == CandidateSource::PrefixExpand);
    if expand_sources_ok {
        runner.pass("展开组独立来源标注 PrefixExpand（16.1-1 来源）");
    } else {
        runner.fail(
            "展开组独立来源标注 PrefixExpand（16.1-1 来源）",
            &format!("候选: {cands:?}"),
        );
    }
    let mut seen = std::collections::HashSet::new();
    if texts.iter().all(|t| seen.insert(*t)) {
        runner.pass("展开后候选无重复文本（16.1-1 去重）");
    } else {
        runner.fail("展开后候选无重复文本（16.1-1 去重）", &format!("{texts:?}"));
    }

    // ---- 16.1-2：常见拼音候选已满一页不展开（D-70）----
    engine.handle_escape();
    type_text(&mut engine, "de");
    let de_cands = snapshot(&mut engine);
    let de_no_expand = de_cands
        .iter()
        .all(|(_, source)| *source != CandidateSource::PrefixExpand);
    if de_no_expand && de_cands.len() == 10 {
        runner.pass("常见拼音候选已满一页不展开（16.1-2）");
    } else {
        runner.fail(
            "常见拼音候选已满一页不展开（16.1-2）",
            &format!("de 候选: {de_cands:?}"),
        );
    }

    // ---- 16.1-3：不完整拼音与既有前缀补全互斥----
    engine.handle_escape();
    type_text(&mut engine, "shuip");
    let shuip_cands = snapshot(&mut engine);
    let no_expand = shuip_cands
        .iter()
        .all(|(_, source)| *source != CandidateSource::PrefixExpand);
    let has_level = shuip_cands.iter().any(|(text, _)| text == "水平");
    if no_expand && has_level {
        runner.pass("不完整拼音仍走前缀补全（16.1-3 互斥）");
    } else {
        runner.fail(
            "不完整拼音仍走前缀补全（16.1-3 互斥）",
            &format!("shuip 候选: {shuip_cands:?}"),
        );
    }

    // ---- 真实词典机制一致性复核（提供文件时）----
    if let Some(real) = path {
        let file = DictionaryFile::open(real)
            .map_err(|error| format!("打开真实词典 {real:?} 失败: {error}"))?;
        let real_dict: Arc<dyn Dictionary> = Arc::new(file.clone());
        let mut real_eng = InputEngine::with_bigram(real_dict.clone(), Arc::new(file.clone()));
        real_eng.handle_escape();
        type_text(&mut real_eng, "shui");
        let real_cands = snapshot(&mut real_eng);
        let main_count = real_dict.lookup("shui").len();
        let expand_count = real_cands
            .iter()
            .filter(|(_, source)| *source == CandidateSource::PrefixExpand)
            .count();
        let consistent = if main_count < 9 {
            expand_count > 0 && expand_count <= 9 - main_count
        } else {
            expand_count == 0
        };
        if consistent {
            runner.pass(&format!(
                "真实词典机制一致性（shui 主组 {main_count} 条 → 展开 {expand_count} 条）"
            ));
        } else {
            runner.fail(
                "真实词典机制一致性（shui 主组 <9 有展开、≥9 零展开）",
                &format!(
                    "main_count={main_count} expand_count={expand_count} 候选: {real_cands:?}"
                ),
            );
        }
    } else {
        println!("[SKIP] 未提供真实词典文件，跳过机制一致性复核");
    }

    Ok(())
}

fn seed_checks(path: &Path, runner: &mut Runner) -> Result<(), String> {
    let file = DictionaryFile::open(path)
        .map_err(|error| format!("打开词典文件 {path:?} 失败: {error}"))?;
    let dictionary: Arc<dyn Dictionary> = Arc::new(file.clone());
    runner.pass("词典加载与内容校验");

    let nihao = file.lookup("nihao");
    let first = nihao
        .first()
        .ok_or_else(|| "nihao 查询结果为空".to_owned())?;
    if first.word == "你好" && first.translation.as_deref() == Some("hello") {
        runner.pass("nihao 首候选与行内译文");
    } else {
        runner.fail("nihao 首候选与行内译文", &format!("实际: {first:?}"));
    }

    let mut engine = InputEngine::with_dictionary_file(path)
        .map_err(|error| format!("InputEngine 创建失败: {error}"))?;
    engine.handle_escape();
    type_text(&mut engine, "xian");
    let texts: Vec<String> = engine
        .candidates()
        .iter()
        .map(|candidate| candidate.text.clone())
        .collect();
    if texts.iter().any(|text| text == "先") && texts.iter().any(|text| text == "西安") {
        runner.pass("xian 歧义切分同时保留先与西安");
    } else {
        runner.fail("xian 歧义切分同时保留先与西安", &format!("实际: {texts:?}"));
    }

    engine.handle_escape();
    type_text(&mut engine, "nih");
    let prefix_texts: Vec<&str> = engine
        .candidates()
        .iter()
        .map(|candidate| candidate.text.as_str())
        .collect();
    if prefix_texts.starts_with(&["你好", "尼好", "你"]) {
        runner.pass("前缀候选补全组优先且含完成组");
    } else {
        runner.fail(
            "前缀候选补全组优先且含完成组",
            &format!("实际: {prefix_texts:?}"),
        );
    }
    engine.handle_escape();
    type_text(&mut engine, "zh");
    if engine.candidates().is_empty() {
        runner.pass("无完整音节开头无前缀候选");
    } else {
        runner.fail("无完整音节开头无前缀候选", "zh 不应出现候选");
    }
    engine.handle_escape();

    let mut expected: Vec<(String, Option<String>)> = Vec::new();
    type_text(&mut engine, "nihao");
    expected.extend(
        engine
            .candidates()
            .iter()
            .map(|c| (c.text.clone(), c.translation.clone())),
    );
    engine.handle_escape();
    type_text(&mut engine, "nihao");
    let actual: Vec<(String, Option<String>)> = engine
        .candidates()
        .iter()
        .map(|c| (c.text.clone(), c.translation.clone()))
        .collect();
    if actual == expected {
        runner.pass("候选顺序确定性");
    } else {
        runner.fail("候选顺序确定性", &format!("两次结果不一致: {actual:?}"));
    }

    engine.handle_escape();
    type_text(&mut engine, "nihao");
    if engine.select_index(1).as_deref() == Some("尼好")
        && engine.previous_word() == Some("尼好")
        && !engine.is_active()
    {
        runner.pass("数字键选择候选并更新前词");
    } else {
        runner.fail("数字键选择候选并更新前词", "选择结果与前词不符合预期");
    }

    let mut engine = InputEngine::with_dictionary_file(path)
        .map_err(|error| format!("InputEngine 创建失败: {error}"))?;
    type_text(&mut engine, "nihao");
    if engine.handle_space().as_deref() == Some("你好") && !engine.is_active() {
        runner.pass("空格上屏第一候选");
    } else {
        runner.fail("空格上屏第一候选", "未上屏你好");
    }

    type_text(&mut engine, "nihao");
    if engine.handle_enter().as_deref() == Some("nihao") && engine.previous_word().is_none() {
        runner.pass("回车提交拼音原文并清空前词");
    } else {
        runner.fail("回车提交拼音原文并清空前词", "提交结果不符合预期");
    }

    type_text(&mut engine, "nihao");
    if engine.handle_escape() && !engine.is_active() && engine.candidates().is_empty() {
        runner.pass("Esc 取消组合");
    } else {
        runner.fail("Esc 取消组合", "组合状态未清空");
    }

    let mut engine = InputEngine::with_dictionary_file(path)
        .map_err(|error| format!("InputEngine 创建失败: {error}"))?;
    engine.toggle_mode();
    if engine.mode() == InputMode::English && !engine.handle_letter('n') {
        engine.toggle_mode();
        if engine.handle_letter('n') {
            runner.pass("Shift 中英模式切换");
        } else {
            runner.fail("Shift 中英模式切换", "切回中文后字母仍被拒绝");
        }
    } else {
        runner.fail("Shift 中英模式切换", "英文模式未拒绝字母");
    }

    let many: Arc<dyn Dictionary> = Arc::new(InMemoryDictionary::from_entries(
        (0..20)
            .map(|i| DictionaryEntry::new(format!("候选{i:02}"), "houxuan", 10_000 - i))
            .collect(),
    ));
    let mut engine = InputEngine::new(many);
    type_text(&mut engine, "houxuan");
    let page0: Vec<String> = engine
        .visible_candidates()
        .iter()
        .map(|candidate| candidate.text.clone())
        .collect();
    engine.next_page();
    let page1: Vec<String> = engine
        .visible_candidates()
        .iter()
        .map(|candidate| candidate.text.clone())
        .collect();
    engine.previous_page();
    let page0_back: Vec<String> = engine
        .visible_candidates()
        .iter()
        .map(|candidate| candidate.text.clone())
        .collect();
    engine.next_page();
    engine.next_page();
    engine.next_page();
    let wrapped = engine.page() == 0;
    if page0.len() == 9 && page1.len() == 9 && page0 != page1 && page0_back == page0 && wrapped {
        runner.pass("候选翻页展示、回退与回卷");
    } else {
        runner.fail(
            "候选翻页展示、回退与回卷",
            &format!("page0={page0:?}, page1={page1:?}, back={page0_back:?}, wrapped={wrapped}"),
        );
    }

    let temp = TempDir::new("host-e2e-user");
    let store = UserDictStore::new(temp.path().join("user_words.json"));
    let mut engine = InputEngine::with_user_store(dictionary.clone(), store.clone());
    for _ in 0..3 {
        type_text(&mut engine, "nihao");
        engine.select_index(1);
    }
    type_text(&mut engine, "nihao");
    let promoted = engine.candidates().first().cloned();
    let store_frequency = store.load().unwrap_or_default().frequency("尼好", "nihao");
    engine.handle_escape();
    if promoted
        .as_ref()
        .is_some_and(|c| c.text == "尼好" && c.source == CandidateSource::User)
        && store_frequency == 3
    {
        runner.pass("用户词多次选择后排序提升并持久化");
    } else {
        runner.fail(
            "用户词多次选择后排序提升并持久化",
            &format!("首候选: {promoted:?}, 用户词频: {store_frequency}"),
        );
    }

    if engine.delete_user_word("尼好", "nihao").unwrap_or(false) {
        type_text(&mut engine, "nihao");
        if engine
            .candidates()
            .first()
            .is_some_and(|c| c.text == "你好")
        {
            runner.pass("用户词删除后恢复静态排序");
        } else {
            let first_text = engine
                .candidates()
                .first()
                .map(|c| c.text.clone())
                .unwrap_or_default();
            let residual = engine.user_dictionary().frequency("尼好", "nihao");
            runner.fail(
                "用户词删除后恢复静态排序",
                &format!("首候选: {first_text}, 残留用户词频: {residual}"),
            );
        }
    } else {
        runner.fail("用户词删除后恢复静态排序", "delete_user_word 返回 false");
    }
    if engine.reset_user_words().is_ok() && store.load().unwrap_or_default().is_empty() {
        runner.pass("用户词重置后词库为空");
    } else {
        runner.fail("用户词重置后词库为空", "重置或落盘失败");
    }

    let mut bigram = InMemoryBigramModel::new();
    bigram.insert("你好", "得", 100_000);
    let mut engine = InputEngine::with_bigram(input::m1_seed_dictionary(), Arc::new(bigram));
    type_text(&mut engine, "de");
    let first_without_context = engine.candidates().first().map(|c| c.text.clone());
    engine.handle_escape();
    type_text(&mut engine, "nihao");
    engine.handle_space();
    type_text(&mut engine, "de");
    let first_with_context = engine.candidates().first().map(|c| c.text.clone());
    if first_without_context.as_deref() == Some("的") && first_with_context.as_deref() == Some("得")
    {
        runner.pass("bigram 前词上下文参与排序");
    } else {
        runner.fail(
            "bigram 前词上下文参与排序",
            &format!("无上下文: {first_without_context:?}, 有上下文: {first_with_context:?}"),
        );
    }

    let mut engine = InputEngine::with_dictionary_file(path)
        .map_err(|error| format!("InputEngine 创建失败: {error}"))?;
    type_text(&mut engine, "nihao");
    if engine.toggle_translation_layer()
        && engine.layer() == CandidateLayer::Translation
        && engine
            .visible_candidates()
            .first()
            .is_some_and(|c| c.text == "你好" && c.translation.as_deref() == Some("hello"))
        && engine.handle_space().as_deref() == Some("hello")
        && engine.layer() == CandidateLayer::Chinese
    {
        runner.pass("译文层切换与英文上屏");
    } else {
        runner.fail("译文层切换与英文上屏", "译文层或上屏结果不符合预期");
    }

    let no_translation: Arc<dyn Dictionary> = Arc::new(InMemoryDictionary::from_entries(vec![
        DictionaryEntry::new("尼好", "nihao", 10),
    ]));
    let mut engine = InputEngine::new(no_translation);
    type_text(&mut engine, "nihao");
    if !engine.toggle_translation_layer()
        && engine.layer() == CandidateLayer::Chinese
        && !engine.visible_candidates().is_empty()
    {
        runner.pass("无译文候选不进入译文层");
    } else {
        runner.fail("无译文候选不进入译文层", "空页面或错误切换");
    }

    let zh = file.zh_to_en("你好");
    let en = file.en_to_zh("hello");
    if zh.as_deref() == Some("hello")
        && en.as_deref() == Some("你好")
        && file
            .translate("你好", TranslationDirection::ZhToEn)
            .as_deref()
            == Some("hello")
        && file.en_to_zh("HELLO").as_deref() == Some("你好")
    {
        runner.pass("本地翻译正查与英文反查");
    } else {
        runner.fail(
            &format!("本地翻译正查与英文反查: 你好->{zh:?}, hello->{en:?}"),
            "",
        );
    }

    Ok(())
}

fn real_smoke(path: &Path, runner: &mut Runner) -> Result<(), String> {
    let file = DictionaryFile::open(path)
        .map_err(|error| format!("打开真实词典 {path:?} 失败: {error}"))?;
    runner.pass("真实词典加载");

    let jiao: Vec<String> = file
        .lookup("jiao")
        .iter()
        .map(|entry| entry.word.clone())
        .collect();
    if jiao.iter().any(|word| word == "叫") {
        runner.pass("真实词典 jiao 保留叫");
    } else {
        runner.fail("真实词典 jiao 保留叫", &format!("实际: {jiao:?}"));
    }

    let xian: Vec<String> = file
        .lookup("xian")
        .iter()
        .map(|entry| entry.word.clone())
        .collect();
    if xian.iter().any(|word| word == "先") && !xian.iter().any(|word| word == "洗按") {
        runner.pass("真实词典 xian 无音节切分噪声");
    } else {
        runner.fail("真实词典 xian 无音节切分噪声", &format!("实际: {xian:?}"));
    }

    let zhidao: Vec<String> = file
        .lookup("zhidao")
        .iter()
        .map(|entry| entry.word.clone())
        .collect();
    if zhidao.first().map(String::as_str) == Some("知道") {
        runner.pass("真实词典整词 zhidao 首候选知道");
    } else {
        runner.fail(
            "真实词典整词 zhidao 首候选知道",
            &format!("实际: {zhidao:?}"),
        );
    }

    let mut real_engine = InputEngine::with_dictionary_file(path)
        .map_err(|error| format!("真实词典引擎创建失败: {error}"))?;
    type_text(&mut real_engine, "nih");
    let nih_texts: Vec<&str> = real_engine
        .candidates()
        .iter()
        .map(|candidate| candidate.text.as_str())
        .collect();
    if nih_texts.first() == Some(&"你好") && nih_texts.contains(&"你") {
        runner.pass("真实词典前缀候选补全组优先且含完成组");
    } else {
        runner.fail(
            "真实词典前缀候选补全组优先且含完成组",
            &format!("实际: {nih_texts:?}"),
        );
    }

    if file.zh_to_en("你好").is_some() && file.en_to_zh("good").is_some() {
        runner.pass("真实词典双向翻译可用");
    } else {
        runner.fail("真实词典双向翻译可用", "你好 或 good 未命中");
    }

    Ok(())
}

fn type_text(engine: &mut InputEngine, text: &str) {
    for c in text.chars() {
        assert!(
            engine.handle_letter(c),
            "字母 {c} 未被输入引擎接受（输入串 {text}）"
        );
    }
}

/// 混合输入串助手（T-066）：字母走 handle_letter、数字走 handle_digit、
/// 其余（`@`/`.`/`/`/`:`）走 handle_format_char，模拟 TSF 键路分派。
fn type_format(engine: &mut InputEngine, text: &str) {
    for c in text.chars() {
        let ok = match c {
            'a'..='z' => engine.handle_letter(c),
            '0'..='9' => engine.handle_digit(c),
            _ => engine.handle_format_char(c),
        };
        assert!(ok, "格式串 {text} 的字符 {c} 未被输入引擎接受");
    }
}

struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = env::temp_dir().join(format!(
            "zhu-ye-host-e2e-{label}-{}-{now}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
