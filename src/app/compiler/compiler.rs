use std::{path::PathBuf, sync::Arc};

use crate::app::parser::parser::{parse_code, parse_functions, parse_tokens};

use super::file_ingest::file_ingest;

pub fn compilation_process(path_to_file: PathBuf) -> anyhow::Result<()> {
    let formatted_file_contents = file_ingest(path_to_file)?;

    let tokens = parse_code(formatted_file_contents);

    dbg!(&tokens);

    let unparsed_functions = parse_tokens(tokens)?;

    let parsed_functions = parse_functions(Arc::new(unparsed_functions))?;

    dbg!(parsed_functions);

    Ok(())
}
