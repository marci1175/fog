use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct LibraryImport
{
    pub name: String,
    pub version: String,
}

pub enum ImportItem
{
    Module(Box<ImportItem>),
    Function(String),
}
