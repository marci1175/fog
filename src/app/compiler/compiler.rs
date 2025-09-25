use std::{path::PathBuf, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::app::{
    codegen::{codegen::codegen_main, error::CodeGenError},
    parser::{parser::ParserState, tokenizer::tokenize},
    type_system::type_system::TypeDiscriminant,
};

#[derive(Deserialize, Serialize, Clone)]
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

impl CompilerConfig {
    pub fn new(name: String, is_library: bool) -> Self {
        Self { name, is_library }
    }
}

pub struct CompilerState {
    pub config: CompilerConfig,
}

impl CompilerState {
    pub fn new(config: CompilerConfig) -> Self {
        Self { config }
    }

    pub fn compilation_process(
        &self,
        file_contents: &str,
        target_path: PathBuf,
        optimization: bool,
        is_lib: bool,
    ) -> anyhow::Result<()> {
        println!("Tokenizing...");
        let tokens = tokenize(file_contents)?;

        let mut parser_state = ParserState::new(tokens);

        println!("Parsing Tokens...");
        parser_state.parse_tokens()?;

        let function_table = parser_state.function_table();
        let imported_functions = parser_state.imported_functions();

        if !is_lib {
            if let Some(fn_sig) = function_table.get("main") {
                if fn_sig.function_sig.return_type != TypeDiscriminant::I32
                    || !fn_sig.function_sig.args.arguments_list.is_empty()
                {
                    return Err(CodeGenError::InvalidMain.into());
                }
            } else {
                return Err(CodeGenError::NoMain.into());
            }
        } else if function_table.contains_key("main") {
            println!("A `main` function has been found, but the library flag is set to `true`.");
        }

        println!("LLVM-IR generation...");
        codegen_main(
            Rc::new(function_table.clone()),
            target_path,
            optimization,
            imported_functions,
            parser_state.custom_types(),
        )?;

        Ok(())
    }
}
