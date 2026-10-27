use std::{collections::HashMap, fs, path::PathBuf, time::SystemTime};

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

/// This function verifies all the dependencies that they are present in the dependency folder and returns a Path for each.
/// This does not verify if config file and whatever need is in there, only check the presence of the folder inside the dependencies folder.
pub fn verify_dependencies_fs(
    mut dependency_root: PathBuf,
    dependencies: &HashMap<String, DependencyInfo>,
) -> anyhow::Result<Vec<(&str, PathBuf, SystemTime, &DependencyInfo)>>
{
    let mut dependency_paths = Vec::new();

    for (name, info) in dependencies {
        dependency_root.push(name);

        let metadata = fs::metadata(&dependency_root)?;

        dependency_paths.push((name.as_str(), dependency_root.clone(), metadata.modified()?, info));

        dependency_root.pop();
    }

    Ok(dependency_paths)
}
