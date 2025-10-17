use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct LibraryImport {
    pub name: String,
    pub version: i32,
}