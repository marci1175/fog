use std::path::PathBuf;

use crate::{
    ApplicationError,
    app::{
        codegen::codegen::codegen_main,
        parser::{parser::ParserState, tokenizer::tokenize},
    },
};

use super::file_ingest::file_ingest;

pub fn compilation_process(
    path_to_file: PathBuf,
    target_path: PathBuf,
    optimization: bool,
) -> anyhow::Result<()> {
    println!("Reading file...");
    let formatted_file_contents = file_ingest(path_to_file)?;

    println!("Tokenizing...");
    let tokens = tokenize(formatted_file_contents)?;

    let mut parser_state = ParserState::new(tokens);

    println!("Parsing Tokens...");
    parser_state.parse_tokens()?;

    let function_table = parser_state.function_table();
    let imported_functions = parser_state.imported_functions();

    println!("LLVM-IR generation...");
    codegen_main(
        function_table,
        target_path,
        optimization,
        imported_functions,
    )
    .map_err(ApplicationError::CodeGenError)?;

    Ok(())
}
