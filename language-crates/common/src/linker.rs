use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildManifest
{
    pub build_output_paths: Vec<PathBuf>,
    pub output_path: PathBuf,
}
