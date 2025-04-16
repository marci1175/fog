use std::{collections::HashMap, path::PathBuf};

use crate::app::parser::{
    parser::{parse_code, parse_functions},
    types::FunctionDefinition,
};

use super::file_ingest::file_ingest;

pub fn compilation_process(path_to_file: PathBuf) -> anyhow::Result<()> {
    let formatted_file_contents = file_ingest(path_to_file)?;

    let tokens = parse_code(formatted_file_contents);

    dbg!(&tokens);

    let parsed_functions = parse_functions(tokens)?;

    dbg!(&parsed_functions);

    Ok(())
}
