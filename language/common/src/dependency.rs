use serde::{Deserialize, Serialize};

/// Contains all the important information about a dependency in a config file.
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct DependencyInfo
{
    pub version: String,
    pub features: Vec<String>,
    pub remote_compile_with: Option<String>,
    pub remote: Option<String>,
}

/// Can be used to fetch a dependency from a remote.
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct DependencyRequest {
    pub name: String,
    pub version: String,
}