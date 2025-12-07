use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{dependency::DependencyInfo, distributed_compiler::DistributedCompilerWorker};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProjectConfig
{
    pub name: String,
    pub is_library: bool,

    /// This is only enabled if its a library
    pub features: Option<Vec<String>>,

    /// This allows the user to use the remote compiler worker feature.
    pub remote_compiler_workers: Option<Vec<DistributedCompilerWorker>>,

    pub version: String,
    pub build_path: String,
    pub additional_linking_material: Vec<PathBuf>,
    pub dependencies: HashMap<String, DependencyInfo>,
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
        }
    }
}

impl ProjectConfig
{
    pub fn new_from_name(name: String) -> Self
    {
        Self {
            name,
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
