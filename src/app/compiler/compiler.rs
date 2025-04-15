use std::path::PathBuf;

use crate::app::code_parser::code_parser::parse_code;

use super::file_ingest::file_ingest;

pub fn compilation_process(path_to_file: PathBuf) -> anyhow::Result<()> {
    let formatted_file_contents = file_ingest(path_to_file)?;

    let tokens = parse_code(formatted_file_contents);

    dbg!(tokens);

    Ok(())
}
