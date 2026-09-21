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

use zhu_ye_core::bigram::InMemoryBigramModel;
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
        Some(path) => run_seed_checks(Path::new(path)),
        None => run_seed_checks(Path::new("data/artifacts/seed.zyct")),
    }
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
