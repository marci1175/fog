use std::path::PathBuf;

use axum::response::IntoResponse;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum DependencyManagerError
{
    #[error("Dependency already exists on remote")]
    DependencyAlreadyExists,
    #[error("Failed to decompress files")]
    DecompressionError,
    #[error("Failed to compress files")]
    CompressionError,
    #[error("Remote could not create a file with name: `{0}`")]
    FailedToCreateFile(PathBuf),
    #[error("Remote could not write to file `{0}`")]
    FailedToWriteToFile(PathBuf),
    #[error("Invalid request body")]
    BadRequest,
    #[error("Could not acquire database pool")]
    GenericDatabaseError,
    #[error("An invalid Zip path was detected")]
    InvalidZipArchiveFilePath,
    #[error("Invalid Zip archive")]
    InvalidZipArchive,
    #[error("Requested dependency was not found")]
    DependencyNotFound,
    #[error("Invalid path linked to dependency in database")]
    InvalidFileError,
}

impl IntoResponse for DependencyManagerError
{
    fn into_response(self) -> axum::response::Response
    {
        let body = self.to_string();

        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}
