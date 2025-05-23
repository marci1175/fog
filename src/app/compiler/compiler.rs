use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{
    ApplicationError,
    app::{
        codegen::codegen::codegen_main,
        parser::{parser::ParserState, tokenizer::tokenize},
    },
};


#[derive(Deserialize, Serialize)]
pub struct CompilerConfig {
    pub name: String,
    pub is_library: bool,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            name: "project".to_string(),
            is_library: false,
        }
    }
}

pub struct CompilerState {
    config: CompilerConfig,
}

impl CompilerState {
    pub fn new(config: CompilerConfig) -> Self {
        Self { config }
    }

    pub fn compilation_process(
        &self,
        file_contents: String,
        target_path: PathBuf,
        optimization: bool,
    ) -> anyhow::Result<()> {
        println!("Tokenizing...");
        let tokens = tokenize(file_contents)?;

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
}
