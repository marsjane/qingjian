//! 回归实际按键序列、前缀候选消费与输入框隔离。
use super::*;
use qingjian_core::Language;
use qingjian_dictionary::Dictionary;
use qingjian_translate::Glossary;
use std::path::Path;

fn session() -> Session {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/sample");
    let dict = Dictionary::from_path(root.join("dict.tsv")).unwrap();
    let glossary = Glossary::from_path(Language::English, root.join("glossary-en.tsv")).unwrap();
    Session::new(
        Engine::new(dict).with_translator(Box::new(glossary)),
        Config::default(),
    )
}
fn type_text(s: &mut Session, text: &str) {
    for c in text.chars() {
        assert!(s.key(c as u32, c as u32, 0, 0));
    }
}
fn index(s: &Session, text: &str) -> usize {
    s.candidates.iter().position(|c| c.text == text).unwrap()
}
#[test]
fn pinyin_translation_and_space_commit() {
    let mut s = session();
    type_text(&mut s, "kaifa");
    let i = index(&s, "开发");
    assert!(!s.annotations[i].to_bytes().is_empty());
    assert!(s.select(i, None));
    assert_eq!(s.committed.to_str().unwrap(), "开发");
    assert!(s.engine.composition().is_empty());
    type_text(&mut s, "kaifa");
    assert!(s.key(32, 32, 0, index(&s, "开发")));
    assert_eq!(s.committed.to_str().unwrap(), "开发");
}
#[test]
fn translation_commits_gloss_not_chinese() {
    let mut s = session();
    type_text(&mut s, "kaifa");
    let i = index(&s, "开发");
    let expected = s.candidates[i].translation.as_ref().unwrap().senses()[0]
        .text
        .clone();
    assert!(s.select(i, Some(0)));
    assert_eq!(s.committed.to_str().unwrap(), expected);
}
#[test]
fn prefix_selection_keeps_remaining_pinyin() {
    let mut s = session();
    type_text(&mut s, "kaifazhe");
    assert!(s.select(index(&s, "开发"), None));
    assert_eq!(s.engine.composition().text(), "zhe");
}
#[test]
fn contexts_never_share_composition() {
    let mut a = session();
    let mut b = session();
    type_text(&mut a, "kaifa");
    type_text(&mut b, "biancheng");
    a.reset();
    assert!(a.engine.composition().is_empty());
    assert_eq!(b.engine.composition().text(), "biancheng");
}
#[test]
fn backspace_cursor_return_escape_and_shortcuts() {
    let mut s = session();
    type_text(&mut s, "kaifa");
    assert!(!s.key('c' as u32, 'c' as u32, 2, 0));
    assert_eq!(s.engine.composition().text(), "kaifa");
    s.key(keys::LEFT, 0, 0, 0);
    s.key(keys::BACKSPACE, 0, 0, 0);
    assert_eq!(s.engine.composition().text(), "kaia");
    s.key(keys::RETURN, 0, 0, 0);
    assert_eq!(s.committed.to_str().unwrap(), "kaia");
    type_text(&mut s, "kaifa");
    s.key(keys::ESCAPE, 0, 0, 0);
    assert!(s.engine.composition().is_empty());
    assert!(s.committed.to_bytes().is_empty());
}
#[test]
fn punctuation_and_decimal_passthrough() {
    let mut s = session();
    assert!(s.key(',' as u32, ',' as u32, 0, 0));
    assert_eq!(s.committed.to_str().unwrap(), "，");
    assert!(!s.key('3' as u32, '3' as u32, 0, 0));
    assert!(!s.key('.' as u32, '.' as u32, 0, 0));
    assert!(s.committed.to_bytes().is_empty());
}
#[test]
fn shuangpin_uses_core_scheme() {
    let mut s = session();
    s.engine
        .set_shuangpin(Some(qingjian_core::ShuangpinScheme::Xiaohe));
    type_text(&mut s, "kd fa".replace(' ', "").as_str());
    assert!(s.candidates.iter().any(|c| c.text == "开发"));
}
#[test]
fn invalid_candidate_and_private_input_do_not_learn() {
    use crate::learning::SharedLearner;
    use qingjian_core::Learner;
    use qingjian_learning::FrequencyLearner;
    use std::sync::{Arc, Mutex};
    let shared = SharedLearner {
        inner: Arc::new(Mutex::new(FrequencyLearner::default())),
    };
    let mut s = session();
    let engine = std::mem::replace(&mut s.engine, Engine::new(Dictionary::default()));
    s.engine = engine.with_learner(Box::new(shared.clone()));
    type_text(&mut s, "kaifa");
    assert!(!s.select(usize::MAX, None));
    s.engine.set_private(true);
    s.select(index(&s, "开发"), None);
    assert_eq!(shared.weight("开发"), 0);
    s.engine.set_private(false);
    type_text(&mut s, "kaifa");
    s.select(index(&s, "开发"), None);
    assert!(shared.weight("开发") > 0);
}

#[test]
fn expression_space_accepts_result() {
    let mut s = session();
    type_text(&mut s, "v1+2");
    assert!(s.key(32, 32, 0, 0));
    assert_eq!(s.committed.to_str().unwrap(), "3");
    assert!(s.engine.composition().is_empty());
}

#[test]
fn zero_after_composition_preserves_order() {
    let mut s = session();
    type_text(&mut s, "kaifa");
    assert!(s.key('0' as u32, '0' as u32, 0, index(&s, "开发")));
    assert_eq!(s.committed.to_str().unwrap(), "开发0");
    assert!(s.engine.composition().is_empty());
}
