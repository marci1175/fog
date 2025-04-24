use std::{collections::HashMap, path::PathBuf};

use inkwell::{builder::Builder, context::Context, execution_engine::ExecutionEngine, module::Module, values::{BasicValueEnum, FunctionValue, GlobalValue}, AddressSpace, OptimizationLevel};
use anyhow::Result;

use crate::app::{parser::types::FunctionDefinition, standard_lib::lib::print};

use super::error::CodeGenError;

pub fn codegen_main(parsed_functions: &HashMap<String, FunctionDefinition>, path_to_output: PathBuf) -> Result<()> {

    let ctx = Context::create();
    
    let module = ctx.create_module("main");

    let builder = ctx.create_builder();
    
    let i8_ptr_type = ctx.ptr_type(AddressSpace::default());

    
    let main_fn_type = ctx.i32_type().fn_type(&[], false);
    let main_fn = module.add_function("main", main_fn_type, None);
    let entry = ctx.append_basic_block(main_fn, "entry");
    builder.position_at_end(entry);
    
    let hello_str = builder.build_global_string_ptr("Csá!", "msg")?;
    
    let return_type = ctx.i32_type().const_int(0, false);
    builder.build_return(Some(&BasicValueEnum::IntValue(return_type)))?;

    module.print_to_file(&path_to_output).map_err(|_| CodeGenError::InvalidOutPath(path_to_output))?;

    Ok(())
}

#[derive(Debug, Clone)]
pub struct GlobalCodeGenState {
    pub global_functions: HashMap<String, FunctionValue<'static>>,
    pub global_values: HashMap<String, GlobalValue<'static>>,
}

impl GlobalCodeGenState {
    pub fn new() -> Self {
        Self { global_functions: HashMap::new(), global_values: HashMap::new() }
    }

    pub fn from_hashmap(global_functions: HashMap<String, FunctionValue<'static>>, global_values: HashMap<String, GlobalValue<'static>>) -> Self {
        Self { global_functions, global_values }
    }
}