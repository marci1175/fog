use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DependencyInfo
{
    pub version: String,
    pub features: Vec<String>,
}