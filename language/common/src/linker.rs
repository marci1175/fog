use std::{
    fs,
    path::PathBuf,
    process::{Command, ExitStatus, Stdio},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildManifest
{
    /// The list of build artifacts needed to compile and link the project.
    pub build_arctifact_paths: Vec<PathBuf>,

    /// Files requried to be linked in order for the build to function
    pub additional_linking_material: Vec<PathBuf>,

    /// When linking a binary from a [`BuildManifest`], this path is where the binary is written to.
    pub build_path: PathBuf,

    /// Type of binary this [`BuildManifest`] should help link.
    pub build_type: BuildType,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BuildType
{
    /// .exe or any other
    Executable,
    /// .lib
    Library,
}

impl BuildManifest
{
    pub fn run_build_output(
        &self,
        project_root: PathBuf,
        args: Vec<String>,
    ) -> anyhow::Result<ExitStatus>
    {
        Ok(Command::new(self.build_path.clone())
            .args(args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .stdin(Stdio::inherit())
            .current_dir(project_root)
            .status()?)
    }

    pub fn localize_paths(self, root: PathBuf) -> Self
    {
        Self {
            build_arctifact_paths: self
                .build_arctifact_paths
                .iter()
                .map(|p| {
                    p.strip_prefix(fs::canonicalize(&root).unwrap())
                        .unwrap()
                        .to_path_buf()
                })
                .collect::<Vec<PathBuf>>(),
            additional_linking_material: self
                .additional_linking_material
                .iter()
                .map(|p| {
                    p.strip_prefix(fs::canonicalize(&root).unwrap())
                        .unwrap()
                        .to_path_buf()
                })
                .collect::<Vec<PathBuf>>(),
            build_path: self.build_path.strip_prefix(&root).unwrap().to_path_buf(),
            build_type: self.build_type,
        }
    }
}
