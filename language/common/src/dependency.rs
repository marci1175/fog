use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct DependencyInfo
{
    pub version: String,
    pub features: Vec<String>,
    pub remote_compile_with: Option<String>,
    pub remote: Option<String>,
}
