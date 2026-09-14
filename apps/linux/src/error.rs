//! Linux 资源加载错误。

#[derive(Debug, thiserror::Error)]
pub enum LinuxError {
    #[error("required resource missing: {0}")]
    Resource(String),
    #[error("invalid configuration: {0}")]
    Config(String),
    #[error("failed to load resource: {0}")]
    Load(String),
}
