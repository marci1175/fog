use anyhow::Error;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};
use thiserror::Error;

use crate::dependency::DependencyInfo;

#[derive(Debug, Error)]
pub enum DependencyError
{
    #[error("Dependency at `{0}` is missing a config file.")]
    DependencyMissingConfig(PathBuf),
    #[error("Dependency `{0}` is not a library. A project can only depend on libraries.")]
    InvalidDependencyType(String),
    #[error("Dependency `{0}` is imported with version `{1}` but the library has version `{2}`.")]
    MismatchedVersionNumber(String, String, String),
    #[error(
        "The dependency folder was not found. It is usually located at the project root as `deps/`."
    )]
    DependencyFolderNotFound,
    #[error("An unknown error occured while managing dependencies: {0}")]
    FileError(Error),
    #[error("1 or more missing dependencies: `{0:#?}`")]
    MissingDependencies(HashMap<String, DependencyInfo>),
    #[error("Failed linking the libraries' module: `{0}`")]
    ModuleLinkingFailed(String),

    /// The first argument is the dependency, the second is the available features, the third is the enabled features.
    #[error("Dependency `{0}` has features `{1:?}`, but features {2:?} were enabled.")]
    InvalidDependencyFeature(String, Vec<String>, Vec<String>),

    #[error("Dependency could not be compressed when publishing it.")]
    PublishFailedToCompress,
}
