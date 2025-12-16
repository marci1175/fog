use crate::{
    models::{Dependency, DependencyInformation, DependencyUpload, DependencyUploadReply},
    schema::dependencies::{self, dependency_name, dependency_version},
};
use common::{
    axum::{Json, body::Bytes, extract::State},
    chrono::Utc,
    compression::{compress_bytes, decompress_bytes, write_zip_to_fs, zip_folder},
    dependency::{DependencyRequest, construct_dependency_path},
    dependency_manager::{ServerState, generate_secret},
    error::dependency_manager::DependencyManagerError,
    rmp_serde::*,
    zip::ZipArchive,
};
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use std::{
    fs::{self},
    io::Cursor,
    path::PathBuf,
};

pub async fn fetch_dependency_information(
    State(state): State<ServerState>,
    Json(request): Json<DependencyRequest>,
) -> Result<Json<DependencyInformation>, DependencyManagerError>
{
    let mut pg_connection = state.db_connection.get().map_err(|err| {
        eprintln!(
            "An error occured while fetching login information from db: {}",
            err
        );

        DependencyManagerError::GenericDatabaseError
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

            DependencyManagerError::DependencyNotFound
        })?;

    Ok(Json(query_result))
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

    let decompressed_bytes = decompress_bytes(&serialized_bytes)
        .map_err(|_| DependencyManagerError::DecompressionError)?;

    match from_slice::<DependencyUpload>(&decompressed_bytes) {
        Ok(dependency_upload) => {
            // Decompress dependency, write to fs
            let mut dependency_bytes = Cursor::new(dependency_upload.source_files.clone());

            let dep_path = construct_dependency_path(
                state.deps_path.clone(),
                dependency_upload.dependency_name.clone(),
                dependency_upload.dependency_version.clone(),
            );

            write_zip_to_fs(
                &dep_path,
                ZipArchive::new(&mut dependency_bytes)
                    .map_err(|_| DependencyManagerError::InvalidZipArchive)?,
            )?;

            let secret = generate_secret::<32>(&state);

            // Store in db
            diesel::insert_into(dependencies::table)
                .values(DependencyInformation {
                    dependency_name: dependency_upload.dependency_name.clone(),
                    // Dependency name and uploaded folder name must match
                    dependency_source_path: dep_path.to_string_lossy().to_string(),
                    dependency_version: dependency_upload.dependency_version.clone(),
                    author: dependency_upload.author.clone(),
                    date_added: Utc::now().date_naive(),
                    secret: secret.clone(),
                })
                .get_result::<DependencyInformation>(&mut pg_connection)
                .map_err(|err| {
                    eprintln!(
                        "Failed to store dependency `{}` to db: {}",
                        dependency_upload.dependency_name.clone(),
                        err
                    );

                    DependencyManagerError::DependencyAlreadyExists
                })?;

            Ok(Json(DependencyUploadReply {
                secret_to_dep: secret,
            }))
        },
        Err(error) => {
            eprintln!("Error while deserializing request body: {}", error);
            Err(DependencyManagerError::BadRequest)
        },
    }
}

pub async fn fetch_dependency_source(
    State(state): State<ServerState>,
    Json(request): Json<DependencyRequest>,
) -> Result<Bytes, DependencyManagerError>
{
    let mut pg_connection = state.db_connection.get().map_err(|err| {
        eprintln!(
            "An error occured while fetching login information from db: {}",
            err
        );

        DependencyManagerError::GenericDatabaseError
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

            DependencyManagerError::DependencyNotFound
        })?;

    let path = PathBuf::from(query_result.dependency_source_path.clone());

    // Read the dep on the server
    let metadata = fs::metadata(&path).map_err(|err| {
        eprintln!("An error occured while reading path db: {}", err);

        DependencyManagerError::InvalidFileError
    })?;

    if !metadata.is_dir() {
        return Err(DependencyManagerError::InvalidFileError);
    }

    // Walk through the dependency directory
    let read_dir = fs::read_dir(&path).map_err(|err| {
        eprintln!("An error occured while reading path db: {}", err);

        DependencyManagerError::InvalidFileError
    })?;

    let zip = zip_folder(read_dir, None)
        .and_then(|zip| Ok(zip.finish()?))
        .map_err(|_| DependencyManagerError::CompressionError)?;

    let dependency = Dependency {
        info: query_result,
        source: zip.into_inner(),
    };

    let rmp_serialized = common::rmp_serde::to_vec(&dependency).unwrap();

    dbg!(rmp_serialized.len());

    let compressed_files =
        compress_bytes(&rmp_serialized).map_err(|_err| DependencyManagerError::CompressionError)?;

    Ok(Bytes::from(compressed_files))
}
