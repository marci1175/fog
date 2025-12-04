use crate::{
    ServerState,
    models::{Dependency, DependencyInformation, DependencyUpload, DependencyUploadReply},
    schema::dependencies::{self, dependency_name, dependency_version},
};
use axum::{Json, body::Bytes, extract::State, http::StatusCode};
use base64::Engine;
use chrono::Utc;
use common::{anyhow, dependency::DependencyRequest};
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use flate2::Compression;
use rand::{RngCore, TryRngCore};
use std::{
    fs::{self, ReadDir},
    io::{self, Cursor, Seek, Write},
    path::PathBuf,
};
use zip::{
    ZipWriter,
    write::{FileOptions, SimpleFileOptions},
};

pub async fn fetch_dependency_information(
    State(state): State<ServerState>,
    Json(information): Json<DependencyRequest>,
) -> Result<Json<()>, StatusCode>
{
    Ok(Json(()))
}

pub async fn upload_dependency(
    State(state): State<ServerState>,
    // This should be the serialized bytes of type `DependencyUpload`
    serialized_bytes: Bytes,
) -> Result<Json<DependencyUploadReply>, StatusCode>
{
    let mut pg_connection = state.db_connection.get().map_err(|err| {
        eprintln!(
            "An error occured while uploading a dependency to db: {}",
            err
        );

        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut decompressor = flate2::Decompress::new(false);
    let mut decompressed_bytes = Vec::new();

    if let Err(err) = decompressor.decompress(
        &serialized_bytes,
        &mut decompressed_bytes,
        flate2::FlushDecompress::None,
    ) {
        eprintln!("Error while decompressing request body: {err}");
        return Err(StatusCode::BAD_REQUEST);
    }

    match rmp_serde::from_slice::<DependencyUpload>(&decompressed_bytes) {
        Ok(dependency_upload) => {
            // Decompress dependency, write to fs
            let mut dependency_bytes = Cursor::new(dependency_upload.source_files);

            match zip::ZipArchive::new(&mut dependency_bytes) {
                Ok(mut archive) => {
                    let mut archive_idx = 0;
                    while let Ok(mut archived_file) = archive.by_index(archive_idx) {
                        if let Some(file_path) = archived_file.enclosed_name()
                            && archived_file.is_file()
                        {
                            // If the path doesnt start with the dep's name that means the folder is not named correctly
                            if !file_path
                                .starts_with(format!("{}", dependency_upload.dependency_name))
                            {
                                return Err(StatusCode::BAD_REQUEST);
                            }

                            let mut file_folder_path = file_path.clone();
                            file_folder_path.pop();

                            let mut fs_file_path = state.deps_path.clone();
                            fs_file_path.push(file_path);

                            // Create the directory for the file in the deps folder, if it fails the folder has prolly been created already.
                            let _ = fs::create_dir_all(file_folder_path);

                            if let Ok(mut file_handle) = fs::File::create(fs_file_path) {
                                io::copy(&mut archived_file, &mut file_handle)
                                    .map_err(|_| StatusCode::BAD_REQUEST)?;
                            }
                            else {
                                return Err(StatusCode::BAD_REQUEST);
                            }
                        }
                        else {
                            // Invalid Zip archive path
                            return Err(StatusCode::BAD_REQUEST);
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
                                "{}\\{}",
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
                                "An storing a dependency `{}` to db: {}",
                                dependency_upload.dependency_name,
                                err.to_string()
                            );

                            StatusCode::INTERNAL_SERVER_ERROR
                        })?;

                    return Ok(Json(DependencyUploadReply {
                        secret_to_dep: secret,
                    }));
                },
                Err(err) => {
                    eprintln!("Error while extracting zip archive: {}", err.to_string());
                    return Err(StatusCode::BAD_REQUEST);
                },
            }
        },
        Err(error) => {
            eprintln!(
                "Error while deserializing request body: {}",
                error.to_string()
            );
            return Err(StatusCode::BAD_REQUEST);
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

    let mut writer = zip::ZipWriter::new(&mut zip);

    // Walk through the dependency directory
    let read_dir = fs::read_dir(&path).map_err(|err| {
        eprintln!("An error occured while reading path db: {}", err);

        StatusCode::NOT_FOUND
    })?;

    let fop = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    write_folder_items(&mut writer, read_dir, &mut PathBuf::new(), fop).unwrap();

    writer.finish().unwrap();

    let dependency = Dependency {
        info: query_result,
        source: zip.into_inner(),
    };

    let rmp_serialized = rmp_serde::to_vec(&dependency).unwrap();

    let mut compress = flate2::Compress::new(Compression::best(), false);

    let mut compressed_files = Vec::new();

    compress
        .compress(
            &rmp_serialized,
            &mut compressed_files,
            flate2::FlushCompress::None,
        )
        .unwrap();

    Ok(Bytes::from(compressed_files))
}

pub fn write_folder_items<T: Write + Seek>(
    writer: &mut ZipWriter<T>,
    read_dir: ReadDir,
    current_path: &mut PathBuf,
    options: SimpleFileOptions,
) -> anyhow::Result<()>
{
    for item in read_dir {
        let item = item?;

        let file_type = item.file_type()?;

        if file_type.is_dir() {
            current_path.push(item.path().file_name().unwrap_or_default());

            write_folder_items(writer, fs::read_dir(item.path())?, current_path, options)?;
        }
        // If file type is a file
        else {
            let mut file_path = current_path.clone();

            file_path.push(item.path().file_name().unwrap_or_default());

            writer.start_file(file_path.to_string_lossy().to_string(), options)?;
            writer.write_all(&fs::read(item.path())?)?;
        }
    }

    Ok(())
}
