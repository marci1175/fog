use std::{path::PathBuf, sync::Arc};

use crate::app::{codegen::codegen::codegen_main, parser::{parser::ParserState, tokenizer::tokenize}};

use super::file_ingest::file_ingest;

pub fn compilation_process(path_to_file: PathBuf) -> anyhow::Result<()> {
    let formatted_file_contents = file_ingest(path_to_file)?;

    let tokens = tokenize(formatted_file_contents)?;

    let mut parser_state = ParserState::new(tokens);

    parser_state.parse_tokens()?;

    dbg!(parser_state.function_table());

    codegen_main(parser_state.function_table().clone())?;

    Ok(())
}
