use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
use inkwell::values::{FunctionValue, GlobalValue};

use crate::app::parser::tokens::FunctionDefinition;


pub fn codegen_main(
    parsed_functions: &HashMap<String, FunctionDefinition>,
    path_to_output: PathBuf,
) -> Result<()> {
    Ok(())
}

#[derive(Debug, Clone)]
pub struct GlobalCodeGenState {
    pub global_functions: HashMap<String, FunctionValue<'static>>,
    pub global_values: HashMap<String, GlobalValue<'static>>,
}

impl Default for GlobalCodeGenState {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalCodeGenState {
    pub fn new() -> Self {
        Self {
            global_functions: HashMap::new(),
            global_values: HashMap::new(),
        }
    }

    pub fn from_hashmap(
        global_functions: HashMap<String, FunctionValue<'static>>,
        global_values: HashMap<String, GlobalValue<'static>>,
    ) -> Self {
        Self {
            global_functions,
            global_values,
        }
    }
}
