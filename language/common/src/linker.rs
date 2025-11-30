use std::{
    path::PathBuf,
    process::{Command, ExitStatus, Stdio},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildManifest
{
    pub build_output_paths: Vec<PathBuf>,
    pub additional_linking_material: Vec<PathBuf>,
    pub output_path: PathBuf,
}

impl BuildManifest
{
    pub fn run_build_output(
        &self,
        project_root: PathBuf,
        args: Vec<String>,
    ) -> anyhow::Result<ExitStatus>
    {
        Ok(Command::new(self.output_path.clone())
            .args(args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .stdin(Stdio::inherit())
            .current_dir(project_root)
            .status()?)
    }
}
