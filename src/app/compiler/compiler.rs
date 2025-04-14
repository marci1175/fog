use std::path::PathBuf;

use super::file_ingest::file_ingest;

pub fn compilation_process(path_to_file: PathBuf) -> anyhow::Result<()> {
    let file_contents = file_ingest(path_to_file)?;
    
    Ok(())
}
