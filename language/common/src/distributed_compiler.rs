use std::{net::SocketAddr, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{compiler::ProjectConfig, dependency_manager::DependencyInformation, linker::BuildManifest, ty::OrdSet};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct DistributedCompilerWorker
{
    /// The user needs to name their remote workers.
    pub name: String,

    /// Remote address the client is connecting to.
    pub address: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DependencyRequest
{
    pub name: String,
    pub version: String,
    pub features: Vec<String>,
    pub target_triple: String,
    pub cpu_features: Option<String>,
    pub cpu_name: Option<String>,
    pub flags_passed_in: String,
}

#[derive(Debug, Clone)]
pub struct CompileJob
{
    pub remote_address: SocketAddr,
    pub target_triple: String,
    pub features: OrdSet<String>,
    pub depdendency_path: PathBuf,
    pub cpu_features: Option<String>,
    pub cpu_name: Option<String>,
    pub flags_passed_in: String,
    pub dependency_information: DependencyInformation,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FinishedJob
{
    pub info: DependencyInformation,
    pub artifacts_zip_bytes: Vec<u8>,
    pub dependency_config: ProjectConfig,
    pub build_manifest: BuildManifest
}
