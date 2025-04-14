use std::{fs, path::PathBuf};

pub fn file_ingest(path: PathBuf) -> anyhow::Result<String> {
    let file_content = fs::read_to_string(path)?;

    let formatted_string = file_content.trim().replace(" ", "");

    Ok(formatted_string)
}