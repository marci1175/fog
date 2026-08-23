use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct LibraryImport
{
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone)]
pub enum ImportType
{
    Path(PathBuf),
    Dependency(Vec<String>),
    Aliased(String, Box<Self>),
}
