use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CodeGenError {
    #[error("Output path `{0}` is unavailable.")]
    InvalidOutPath(PathBuf),
}
