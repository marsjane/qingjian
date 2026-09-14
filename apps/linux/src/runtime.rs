//! 资源与配置装配；只在创建输入会话时读取磁盘。

use crate::{error::LinuxError, learning::SharedLearner, session::Session};
use qingjian_core::{Engine, Language};
use qingjian_dictionary::Dictionary;
use qingjian_learning::FrequencyLearner;
use qingjian_lm::BigramModel;
use qingjian_platform::{Config, extra_dictionaries};
use qingjian_translate::Glossary;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub struct Runtime {
    /// 资源根目录，含 data/generated 和 assets。
    root: PathBuf,

    /// 配置文件路径。
    config: PathBuf,

    /// 所有输入框共享的学习记录。
    learner: SharedLearner,
}

impl Runtime {
    pub fn new(root: PathBuf, config: PathBuf, user: PathBuf) -> Result<Self, LinuxError> {
        resource(&root, "dict", "lexicon")?;
        let learner = FrequencyLearner::from_path(user.join("user-frequency.tsv"))
            .map_err(|e| LinuxError::Load(e.to_string()))?;
        Ok(Self {
            root,
            config,
            learner: SharedLearner {
                inner: Arc::new(Mutex::new(learner)),
            },
        })
    }

    pub fn session(&self) -> Result<Session, LinuxError> {
        let config_path = if self.config.exists() {
            self.config.clone()
        } else {
            self.root.join("config.toml")
        };
        let config = Config::load(&config_path).map_err(|e| LinuxError::Config(e.to_string()))?;
        let language = match config.general.learning_language.as_str() {
            "en" => Language::English,
            "ja" => Language::Japanese,
            other => {
                return Err(LinuxError::Config(format!(
                    "unsupported learning language: {other}"
                )));
            }
        };
        if config.general.zhuyin {
            return Err(LinuxError::Config(
                "Linux alpha does not support zhuyin yet".into(),
            ));
        }
        let dict = Dictionary::from_path(resource(&self.root, "dict", "lexicon")?)
            .map_err(|e| LinuxError::Load(e.to_string()))?;
        let glossary = Glossary::from_path(
            language,
            resource(
                &self.root,
                &format!("glossary-{}", language.code()),
                "glossary",
            )?,
        )
        .map_err(|e| LinuxError::Load(e.to_string()))?;
        let mut engine = Engine::new(dict)
            .with_translator(Box::new(glossary))
            .with_learner(Box::new(self.learner.clone()));
        let lm = self.root.join("data/generated/lm.qj");
        if lm.is_file() {
            engine = engine.with_language_model(Box::new(
                BigramModel::from_path(&lm).map_err(|e| LinuxError::Load(e.to_string()))?,
            ));
        }
        let bundled = self.root.join("data/generated/dicts");
        let user = self.config.parent().map(|p| p.join("dicts"));
        engine.set_extra_dictionaries(extra_dictionaries::load(
            Some(&bundled),
            user.as_deref(),
            &config.dictionaries,
        ));
        engine.set_fuzzy(config.fuzzy);
        engine.set_shuangpin(config.general.shuangpin());
        engine.set_mode_keys(config.shortcut.mode);
        engine.set_full_width_punctuation(config.general.full_width_punctuation);
        engine
            .set_custom_phrases(config.custom_phrases.clone())
            .map_err(LinuxError::Config)?;
        Ok(Session::new(engine, config))
    }
}

fn resource(root: &Path, name: &str, assets: &str) -> Result<PathBuf, LinuxError> {
    for p in [
        root.join(format!("data/generated/{name}.qj")),
        root.join(format!("assets/{assets}/{name}.tsv")),
        root.join(format!("assets/sample/{name}.tsv")),
    ] {
        if p.is_file() {
            return Ok(p);
        }
    }
    Err(LinuxError::Resource(name.into()))
}
