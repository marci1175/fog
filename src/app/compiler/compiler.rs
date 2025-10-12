use std::{path::PathBuf, rc::Rc};

use inkwell::{context::Context, llvm_sys::target::{
    LLVM_InitializeAllAsmParsers, LLVM_InitializeAllAsmPrinters, LLVM_InitializeAllTargetInfos,
    LLVM_InitializeAllTargetMCs, LLVM_InitializeAllTargets,
}};
use serde::{Deserialize, Serialize};

use crate::{app::{
    codegen::{codegen::codegen_main, error::CodeGenError}, linking::linker::link_llvm_to_target, parser::{parser::ParserState, tokenizer::tokenize}, type_system::type_system::TypeDiscriminant
}, ApplicationError};

#[derive(Deserialize, Serialize, Clone)]
pub struct LibraryImport {
    pub name: String,
    pub version: i32,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct CompilerConfig {
    pub name: String,
    pub is_library: bool,
    pub version: i32,
    pub imports: Vec<LibraryImport>,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            name: "project".to_string(),
            is_library: false,
            version: 0,
            imports: Vec::new()
        }
    }
}

impl CompilerConfig {
    pub fn new(name: String, is_library: bool, version: i32, imports: Vec<LibraryImport>) -> Self {
        Self { name, is_library, version, imports }
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
        target_ir_path: PathBuf,
        target_o_path: PathBuf,
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

        println!("Initializing LLVM environment...");
        unsafe {
            LLVM_InitializeAllTargetInfos();
            LLVM_InitializeAllTargets();
            LLVM_InitializeAllTargetMCs();
            LLVM_InitializeAllAsmParsers();
            LLVM_InitializeAllAsmPrinters();
        }

        println!("LLVM-IR generation...");

        // Create LLVM context
        let context = Context::create();
        let builder = context.create_builder();
        let module = context.create_module("main");

        let target = codegen_main(
            &context,
            &builder,
            &module,
            Rc::new(function_table.clone()),
            target_ir_path,
            target_o_path.clone(),
            optimization,
            imported_functions,
            parser_state.custom_types(),
            // We should make it so that this argument will contain all of the flags the user has passed in
            "",
        )?;
        
        // Linking the object file
        // link_llvm_to_target(&module, target, target_o_path)?;

        Ok(())
    }
}
