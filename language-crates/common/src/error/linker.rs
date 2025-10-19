use thiserror::Error;

#[derive(Debug, Error)]
pub enum LinkerError
{
    #[error("An error occured while linking: {0}")]
    Other(Box<dyn std::error::Error + Send + Sync>),
    #[error("Clang not found at PATH. Clang must be added to PATH inorder to use the linker.")]
    ClangNotFound,
    #[error("Manifest must be TOML-serialized. Please check file validity.")]
    InvalidManifestFormat,
    #[error("Linking with Clang failed: `{0}`")]
    ClangError(Box<dyn std::error::Error + Send + Sync>),
}
