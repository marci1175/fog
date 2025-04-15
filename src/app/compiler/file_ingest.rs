use std::{fs, path::PathBuf};

pub fn file_ingest(path: PathBuf) -> anyhow::Result<String> {
    let formatted_string = fs::read_to_string(path)?;

    Ok(formatted_string)
}
