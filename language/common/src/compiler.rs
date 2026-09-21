use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{dependency::DependencyInfo, distributed_compiler::DistributedCompilerWorker};

/// This contains the project's `config.toml`.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProjectConfig
{
    /// Name of the project
    pub name: String,

    /// Whether project is a library
    pub is_library: bool,

    /// This is only enabled if its a library
    pub features: Option<Vec<String>>,

    /// This allows the user to use the remote compiler worker feature.
    pub remote_compiler_workers: Option<Vec<DistributedCompilerWorker>>,

    /// The actual version of the project
    pub version: String,

    /// The default directory the compiler puts the emitted arctifacts
    pub build_path: String,

    /// Extra object files or linkable files which are additionally linked during the linking process
    pub additional_linking_material: Vec<PathBuf>,

    /// The dependencies present in this map must be present in the `dependencies` folder in the project root.
    pub dependencies: HashMap<String, DependencyInfo>,

    #[serde(skip)]
    /// This field should be initalized by the code.
    pub root_path: PathBuf,
}

impl Default for ProjectConfig
{
    fn default() -> Self
    {
        Self {
            name: "project".to_string(),
            is_library: false,
            features: None,
            remote_compiler_workers: None,
            version: "0.0.1".to_string(),
            build_path: "out".to_string(),
            additional_linking_material: Vec::new(),
            dependencies: HashMap::new(),
            root_path: PathBuf::new(),
        }
    }
}

impl ProjectConfig
{
    pub fn new(name: String, root_path: PathBuf) -> Self
    {
        Self {
            name,
            root_path,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone)]
pub struct HostInformation
{
    pub cpu_features: Option<String>,
    pub cpu_name: Option<String>,
    pub flags_passed_in: Option<String>,
    pub target_triple: String,
}

impl HostInformation
{
    pub fn new(
        cpu_features: Option<String>,
        cpu_name: Option<String>,
        flags_passed_in: Option<String>,
        target_triple: String,
    ) -> Self
    {
        Self {
            cpu_features,
            cpu_name,
            flags_passed_in,
            target_triple,
        }
    }
}
