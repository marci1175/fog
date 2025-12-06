use crate::{
    ServerState,
    models::{Dependency, DependencyInformation, DependencyUpload, DependencyUploadReply},
    schema::dependencies::{self, dependency_name, dependency_version},
};
use base64::Engine;
use common::{
    anyhow,
    axum::{Json, body::Bytes, extract::State, http::StatusCode},
    chrono::Utc,
    compiler::ProjectConfig,
    dependency::DependencyRequest,
    dependency_manager::write_folder_items,
    error::dependency_manager::DependencyManagerError,
    flate2::{Compression, read::ZlibDecoder},
    rmp_serde::*,
    zip::{
        CompressionMethod, ZipArchive, ZipWriter,
        write::{FileOptions, SimpleFileOptions},
    },
};
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use rand::{RngCore, TryRngCore};
use std::{
    fs::{self, ReadDir},
    io::{self, Cursor, Read, Seek, Write},
    path::PathBuf,
};

pub async fn fetch_dependency_information(
    State(state): State<ServerState>,
    Json(information): Json<DependencyRequest>,
) -> Result<Json<()>, StatusCode>
{
    Ok(Json(()))
}

pub async fn publish_dependency(
    State(state): State<ServerState>,
    // This should be the serialized bytes of type `DependencyUpload`
    serialized_bytes: Bytes,
) -> Result<Json<DependencyUploadReply>, DependencyManagerError>
{
    let mut pg_connection = state.db_connection.get().map_err(|err| {
        eprintln!(
            "An error occured while uploading a dependency to db: {}",
            err
        );

        DependencyManagerError::GenericDatabaseError
    })?;

    let mut decoder = ZlibDecoder::new(Cursor::new(serialized_bytes.to_vec()));

    let mut decompressed_bytes: Vec<u8> = Vec::new();

    decoder
        .read_to_end(&mut decompressed_bytes)
        .map_err(|err| {
            eprintln!("An error occured reading decompressed bytes: {}", err);

            DependencyManagerError::DecompressionError
        })?;

    match from_slice::<DependencyUpload>(&decompressed_bytes) {
        Ok(dependency_upload) => {
            // Decompress dependency, write to fs
            let mut dependency_bytes = Cursor::new(dependency_upload.source_files);

            match ZipArchive::new(&mut dependency_bytes) {
                Ok(mut archive) => {
                    let mut archive_idx = 0;
                    while let Ok(mut archived_file) = archive.by_index(archive_idx) {
                        if let Some(file_path) = archived_file.enclosed_name()
                            && archived_file.is_file()
                        {
                            let mut fs_file_path = state.deps_path.clone();
                            fs_file_path.push(dependency_upload.dependency_name.clone());
                            fs_file_path.push(file_path.clone());

                            let mut file_folder_path = fs_file_path.clone();
                            file_folder_path.pop();

                            // Create the directory for the file in the deps folder, if it fails the folder has prolly been created already.
                            let _ = fs::create_dir_all(file_folder_path);

                            if let Ok(mut file_handle) = fs::File::create(fs_file_path) {
                                io::copy(&mut archived_file, &mut file_handle).map_err(|_| {
                                    DependencyManagerError::FailedToWriteToFile(file_path)
                                })?;
                            }
                            else {
                                return Err(DependencyManagerError::FailedToCreateFile(file_path));
                            }
                        }
                        else {
                            // Invalid Zip archive path
                            return Err(DependencyManagerError::InvalidZipArchiveFilePath);
                        }

                        // Increment idx
                        archive_idx += 1;
                    }

                    let mut bytes = [0u8; 32];

                    rand::rngs::OsRng.try_fill_bytes(&mut bytes).unwrap();

                    let secret = state.base64_engine.encode(bytes);

                    // Store in db
                    diesel::insert_into(dependencies::table)
                        .values(DependencyInformation {
                            dependency_name: dependency_upload.dependency_name.clone(),
                            // Dependency name and uploaded folder name must match
                            dependency_source_path: format!(
                                "{}/{}",
                                state.deps_path.display(),
                                dependency_upload.dependency_name.clone()
                            ),
                            dependency_version: dependency_upload.dependency_version,
                            author: dependency_upload.author,
                            date_added: Utc::now().date_naive(),
                            secret: secret.clone(),
                        })
                        .get_result::<DependencyInformation>(&mut pg_connection)
                        .map_err(|err| {
                            eprintln!(
                                "Failed to store dependency `{}` to db: {}",
                                dependency_upload.dependency_name,
                                err.to_string()
                            );

                            DependencyManagerError::DependencyAlreadyExists
                        })?;

                    return Ok(Json(DependencyUploadReply {
                        secret_to_dep: secret,
                    }));
                },
                Err(err) => {
                    eprintln!("Error while extracting zip archive: {}", err.to_string());
                    return Err(DependencyManagerError::BadRequest);
                },
            }
        },
        Err(error) => {
            eprintln!(
                "Error while deserializing request body: {}",
                error.to_string()
            );
            return Err(DependencyManagerError::BadRequest);
        },
    }
}

pub async fn fetch_dependency_source(
    State(state): State<ServerState>,
    Json(request): Json<DependencyRequest>,
) -> Result<Bytes, StatusCode>
{
    let mut pg_connection = state.db_connection.get().map_err(|err| {
        eprintln!(
            "An error occured while fetching login information from db: {}",
            err
        );

        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let dependency_filter = dependencies::table.filter(
        dependency_name
            .eq(request.name)
            .and(dependency_version.eq(request.version)),
    );

    let query_result = dependency_filter
        .select(DependencyInformation::as_select())
        .first(&mut pg_connection)
        .map_err(|err| {
            eprintln!(
                "An error occured while fetching dependencies from db: {}",
                err
            );

            StatusCode::NOT_FOUND
        })?;

    let path = PathBuf::from(query_result.dependency_source_path.clone());

    // Read the dep on the server
    let metadata = fs::metadata(&path).map_err(|err| {
        eprintln!("An error occured while reading path db: {}", err);

        StatusCode::NOT_FOUND
    })?;

    if !metadata.is_dir() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let mut zip = Cursor::new(Vec::new());

    let mut writer = ZipWriter::new(&mut zip);

    // Walk through the dependency directory
    let read_dir = fs::read_dir(&path).map_err(|err| {
        eprintln!("An error occured while reading path db: {}", err);

        StatusCode::NOT_FOUND
    })?;

    let fop = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

    write_folder_items(&mut writer, read_dir, PathBuf::new(), fop, None).unwrap();

    writer.finish().unwrap();

    let dependency = Dependency {
        info: query_result,
        source: zip.into_inner(),
    };

    let rmp_serialized = to_vec(&dependency).unwrap();

    let mut compress = common::flate2::Compress::new(Compression::best(), false);

    let mut compressed_files = Vec::new();

    compress
        .compress(
            &rmp_serialized,
            &mut compressed_files,
            common::flate2::FlushCompress::None,
        )
        .unwrap();

    Ok(Bytes::from(compressed_files))
}
