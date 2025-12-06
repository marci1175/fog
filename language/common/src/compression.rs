use std::{
    fs::{self, ReadDir},
    io::{self, Cursor, Read, Seek, Write},
    path::PathBuf,
};

use flate2::{
    Compression,
    write::{ZlibDecoder, ZlibEncoder},
};
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

pub fn write_zip_to_fs<T: Seek + Read>(
    deps_path: PathBuf,
    dependency_name: String,
    mut archive: ZipArchive<T>,
) -> anyhow::Result<(), DependencyManagerError>
{
    let mut archive_idx = 0;
    while let Ok(mut archived_file) = archive.by_index(archive_idx) {
        if let Some(file_path) = archived_file.enclosed_name()
            && archived_file.is_file()
        {
            let mut fs_file_path = deps_path.clone();
            fs_file_path.push(dependency_name.clone());
            fs_file_path.push(file_path.clone());

            let mut file_folder_path = fs_file_path.clone();
            file_folder_path.pop();

            // Create the directory for the file in the deps folder, if it fails the folder has prolly been created already.
            let _ = fs::create_dir_all(file_folder_path);

            if let Ok(mut file_handle) = fs::File::create(fs_file_path) {
                io::copy(&mut archived_file, &mut file_handle)
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

    Ok(())
}
