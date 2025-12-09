use std::{
    fs::{self, ReadDir},
    io::{self, Cursor, Read, Seek, Write},
    path::PathBuf,
};

use flate2::{
    Compression,
    write::{ZlibDecoder, ZlibEncoder},
};
use tokio::io::BufReader;
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

use crate::{
    dependency_manager::write_folder_items, error::dependency_manager::DependencyManagerError,
};

pub fn compress_bytes(bytes: &[u8]) -> anyhow::Result<Vec<u8>>
{
    let mut compressor = ZlibEncoder::new(Vec::new(), Compression::best());

    compressor.write_all(bytes)?;

    Ok(compressor.finish()?)
}

pub fn decompress_bytes(compressed_bytes: &[u8]) -> anyhow::Result<Vec<u8>>
{
    let mut decompressor = ZlibDecoder::new(Vec::new());

    decompressor.write_all(compressed_bytes)?;

    Ok(decompressor.finish()?)
}

pub fn zip_folder(
    dir: ReadDir,
    path_filter: Option<String>,
) -> Result<ZipWriter<Cursor<Vec<u8>>>, anyhow::Error>
{
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));

    write_folder_items(
        &mut zip,
        dir,
        PathBuf::new(),
        SimpleFileOptions::default(),
        path_filter,
    )?;

    Ok(zip)
}

pub fn unzip_from_bytes<T: Read + Seek>(inner: T) -> anyhow::Result<ZipArchive<T>>
{
    Ok(ZipArchive::new(inner)?)
}

pub fn write_zip_to_fs<T: Seek + Read>(
    dependency_path: &PathBuf,
    mut archive: ZipArchive<T>,
) -> anyhow::Result<Vec<PathBuf>, DependencyManagerError>
{
    let mut archive_idx = 0;

    // Paths we have written to
    let mut paths_written_to = Vec::new();

    while let Ok(mut archived_file) = archive.by_index(archive_idx) {
        if let Some(file_path) = archived_file.enclosed_name()
            && archived_file.is_file()
        {
            let mut fs_file_path = dependency_path.clone();
            fs_file_path.push(file_path.clone());

            if archived_file.is_file() {
                let mut file_folder_path = fs_file_path.clone();
                file_folder_path.pop();

                // Create the directory for the file in the deps folder, if it fails the folder has prolly been created already.
                let _ = fs::create_dir_all(file_folder_path);

                if let Ok(mut file_handle) = fs::File::create(&fs_file_path) {
                    io::copy(&mut archived_file, &mut file_handle)
                        .map_err(|_| DependencyManagerError::FailedToWriteToFile(file_path))?;
                }
                else {
                    return Err(DependencyManagerError::FailedToCreateFile(file_path));
                }

                paths_written_to.push(fs_file_path);
            }
            else {
                let _ = fs::create_dir_all(fs_file_path);
            }
        }
        else {
            // Invalid Zip archive path
            return Err(DependencyManagerError::InvalidZipArchiveFilePath);
        }

        // Increment idx
        archive_idx += 1;
    }

    Ok(paths_written_to)
}

pub async fn write_zip_to_fs_async<T: Seek + Read>(
    dependency_path: PathBuf,
    mut archive: ZipArchive<T>,
) -> anyhow::Result<PathBuf, DependencyManagerError>
{
    let mut archive_idx = 0;
    while let Ok(archived_file) = archive.by_index(archive_idx) {
        if let Some(file_path) = archived_file.enclosed_name()
            && archived_file.is_file()
        {
            let mut fs_file_path = dependency_path.clone();
            fs_file_path.push(file_path.clone());

            let mut file_folder_path = fs_file_path.clone();
            file_folder_path.pop();

            // Create the directory for the file in the deps folder, if it fails the folder has prolly been created already.
            let _ = tokio::fs::create_dir_all(file_folder_path).await;

            if let Ok(mut file_handle) = tokio::fs::File::create(fs_file_path).await {
                let inner = archived_file
                    .bytes()
                    .map(|bytes| bytes.unwrap())
                    .collect::<Vec<u8>>();

                tokio::io::copy(&mut BufReader::new(inner.as_slice()), &mut file_handle)
                    .await
                    .map_err(|_| DependencyManagerError::FailedToWriteToFile(file_path))?;
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

    Ok(dependency_path)
}
