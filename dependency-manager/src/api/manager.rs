use std::{fs::{self, ReadDir}, io::{Cursor, Seek, Write}, path::PathBuf};

use crate::{
    ServerState, models::{Dependency, DependencyInformation}, schema::dependencies::{self, dependency_name, dependency_version}
};
use axum::{Json, body::Bytes, extract::State, http::StatusCode};
use common::{anyhow, dependency::DependencyRequest};
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use flate2::Compression;
use zip::{ZipWriter, write::{FileOptions, SimpleFileOptions}};

pub async fn fetch_dependency_information(
    State(state): State<ServerState>,
    Json(information): Json<DependencyRequest>,
) -> Result<Json<()>, StatusCode>
{
    Ok(Json(()))
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

    let query_result = dependency_filter.select(DependencyInformation::as_select()).first(&mut pg_connection).map_err(|err| {
        eprintln!("An error occured while fetching dependencies from db: {}", err);

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

    let dependency = Dependency { info: query_result, source: zip.into_inner() };

    let rmp_serialized = rmp_serde::to_vec(&dependency).unwrap();
    
    let mut compress = flate2::Compress::new(Compression::best(), false);

    let mut compressed_files = Vec::new();

    compress.compress(&rmp_serialized, &mut compressed_files, flate2::FlushCompress::Sync).unwrap();

    Ok(Bytes::from(compressed_files))
}

pub fn write_folder_items<T: Write + Seek>(writer: &mut ZipWriter<T>, read_dir: ReadDir, current_path: &mut PathBuf, options: SimpleFileOptions) -> anyhow::Result<()> {
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