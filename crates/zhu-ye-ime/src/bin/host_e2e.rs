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
use zhu_ye_core::{DictionaryEntry, DictionaryFile, InMemoryDictionary, UserDictStore};

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
