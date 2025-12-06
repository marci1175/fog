use std::{
    ffi::OsStr,
    fs::ReadDir,
    io::{Seek, Write},
    path::PathBuf,
};

#[cfg(feature = "dependency_manager")]
use base64::Engine;
use chrono::NaiveDate;
#[cfg(feature = "dependency_manager")]
use diesel::{PgConnection, r2d2::ConnectionManager};
#[cfg(feature = "dependency_manager")]
use rand::TryRngCore;

/// ****
/// PLEASE NOTE THAT THESE TYPES ARE COPIES OF THE TYPES FOUND IN THE DEPENDENCY MANAGER WORKSPACE. (It was easier to just copies of the type definitions due to dependencies and diesel.)
/// ANY TYPE DEFINITION CHANGES MUST BE COPIED TO THE DEPENDENCY MANAGER'S DEFINITIONS
/// ****
use serde;
use zip::{ZipWriter, write::SimpleFileOptions};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct DependencyInformation
{
    pub dependency_name: String,
    pub dependency_source_path: String,
    pub dependency_version: String,
    pub author: String,
    pub date_added: NaiveDate,
    pub secret: String,
}

/// Contains both the raw compressed bytes and the information
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Dependency
{
    pub info: DependencyInformation,
    pub source: Vec<u8>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct DependencyUpdateRequest
{
    pub dependency_name: String,
    // pub author: String,
    pub secret: String,
    pub updated_source: Vec<u8>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct DependencyUploadReply
{
    /// Updates can ONLY be uploaded with the use of this secret
    pub secret_to_dep: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct DependencyUpload
{
    pub dependency_name: String,
    pub dependency_version: String,
    pub author: String,
    /// Compressed ZIP source files
    pub source_files: Vec<u8>,
}

impl DependencyUpload
{
    pub fn new(
        dependency_name: String,
        dependency_version: String,
        author: String,
        source_files: Vec<u8>,
    ) -> Self
    {
        Self {
            dependency_name,
            dependency_version,
            author,
            source_files,
        }
    }
}

pub fn write_folder_items<T: Write + Seek>(
    writer: &mut ZipWriter<T>,
    read_dir: ReadDir,
    current_path: PathBuf,
    options: SimpleFileOptions,
    folder_filter: Option<String>,
) -> anyhow::Result<()>
{
    for item in read_dir {
        let item = item?;

        let file_type = item.file_type()?;

        if file_type.is_dir() {
            if let Some(forbidden_folder_name) = folder_filter.clone()
                && item.path().file_name() == Some(OsStr::new(&forbidden_folder_name))
            {
                continue;
            }

            let mut path_clone = current_path.clone();

            path_clone.push(item.path().file_name().unwrap_or_default());

            write_folder_items(
                writer,
                std::fs::read_dir(item.path())?,
                path_clone.clone(),
                options,
                folder_filter.clone(),
            )?;
        }
        // If file type is a file
        else {
            let mut file_path = current_path.clone();

            file_path.push(item.path().file_name().unwrap_or_default());

            writer.start_file(file_path.to_string_lossy().to_string(), options)?;

            writer.write_all(&std::fs::read(item.path())?)?;
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
#[cfg(feature = "dependency_manager")]

pub struct ServerState
{
    pub db_connection: r2d2::Pool<ConnectionManager<PgConnection>>,
    pub deps_path: PathBuf,
    pub base64_engine: base64::engine::GeneralPurpose,
}

#[cfg(feature = "dependency_manager")]
pub fn generate_secret<const LEN: usize>(state: &ServerState) -> String
{
    let mut bytes = [0u8; LEN];

    rand::rngs::OsRng.try_fill_bytes(&mut bytes).unwrap();

    state.base64_engine.encode(bytes)
}
