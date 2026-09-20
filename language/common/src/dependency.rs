use std::{collections::HashMap, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

/// Contains all the important information about a dependency in a config file.
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct DependencyInfo
{
    pub version: String,
    pub features: Vec<String>,
    // Remove this for the time being
    // pub remote: Option<String>,
}

/// Can be used to fetch a dependency from a remote dependency manager.
/// This struct does not account features as when fetching a dependency we want to download all of the source files.
/// Please check [`crate::distributed_compiler::DependencyRequest`] when wanting to request a remote distributed compiler to send pre-compiled dependencies.
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct DependencyRequest
{
    pub name: String,
    pub version: String,
}

pub fn construct_dependency_path(
    deps_path: PathBuf,
    dependency_name: String,
    dependency_version: String,
) -> PathBuf
{
    let mut dependency_path = deps_path.clone();

    dependency_path.push(format!(
        "{}({})",
        dependency_name.clone(),
        dependency_version.clone()
    ));

    dependency_path
}

/// This function verifies all the dependencies that they are present in the dependency folder and returns a Path for each.
/// This does not verify if config file and whatever need is in there, only check the presence of the folder inside the dependencies folder.
pub fn verify_dependencies_fs<'a>(
    mut project_root: PathBuf,
    dependencies: &'a HashMap<String, DependencyInfo>,
) -> anyhow::Result<Vec<(&'a str, PathBuf)>>
{
    let mut dependency_paths = Vec::new();

    for (name, _) in dependencies.iter() {
        project_root.push(name.to_string());

        if fs::exists(&project_root)? {
            dependency_paths.push((name.as_str(), project_root.clone()));
        }

        project_root.pop();
    }

    Ok(dependency_paths)
}
