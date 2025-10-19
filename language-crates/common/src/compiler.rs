use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::dependency::DependencyInfo;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProjectConfig
{
    pub name: String,
    pub is_library: bool,
    pub version: String,
    pub dependencies: HashMap<String, DependencyInfo>,
}

impl Default for ProjectConfig
{
    fn default() -> Self
    {
        Self {
            name: "project".to_string(),
            is_library: false,
            version: "0.0.1".to_string(),
            dependencies: HashMap::new(),
        }
    }
}

impl ProjectConfig
{
    pub fn new(
        name: String,
        is_library: bool,
        version: String,
        imports: HashMap<String, DependencyInfo>,
    ) -> Self
    {
        Self {
            name,
            is_library,
            version,
            dependencies: imports,
        }
    }
}
