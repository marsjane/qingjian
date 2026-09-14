//! 多输入框共享词频，避免各会话用旧快照覆盖同一个用户文件。

use qingjian_core::{Candidate, Learner};
use qingjian_learning::FrequencyLearner;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct SharedLearner {
    /// 所有会话共用同一张词频及选词记录表。
    pub inner: Arc<Mutex<FrequencyLearner>>,
}

impl SharedLearner {
    fn with<T>(&self, f: impl FnOnce(&mut FrequencyLearner) -> T) -> T {
        f(&mut self.inner.lock().unwrap_or_else(|e| e.into_inner()))
    }
}

impl Learner for SharedLearner {
    fn record(&mut self, c: &Candidate) {
        self.with(|l| l.record(c));
    }
    fn weight(&self, text: &str) -> u32 {
        self.with(|l| l.weight(text))
    }
    fn record_choice(&mut self, input: &str, text: &str) {
        self.with(|l| l.record_choice(input, text));
    }
    fn choice_weight(&self, input: &str, text: &str) -> u32 {
        self.with(|l| l.choice_weight(input, text))
    }
    fn record_raw(&mut self, input: &str) {
        self.with(|l| l.record_raw(input));
    }
    fn raw_count(&self, input: &str) -> u32 {
        self.with(|l| l.raw_count(input))
    }
    fn unrecord(&mut self, text: &str) {
        self.with(|l| l.unrecord(text));
    }
    fn unrecord_choice(&mut self, input: &str, text: &str) {
        self.with(|l| l.unrecord_choice(input, text));
    }
    fn record_typo(&mut self, typed: &str, intended: &str) {
        self.with(|l| l.record_typo(typed, intended));
    }
    fn unrecord_typo(&mut self, typed: &str, intended: &str) {
        self.with(|l| l.unrecord_typo(typed, intended));
    }
    fn typo_count(&self, typed: &str, intended: &str) -> u32 {
        self.with(|l| l.typo_count(typed, intended))
    }
    fn flush(&mut self) {
        self.with(|l| l.flush());
    }
}
