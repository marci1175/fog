use std::{collections::HashMap, sync::Arc};

use fog_common::anyhow::Result;
use fog_common::indexmap::IndexMap;
use fog_common::{
    codegen::CustomType,
    parser::{FunctionDefinition, FunctionSignature},
    tokenizer::Token,
};

use crate::parser::function::{create_signature_table, parse_functions};

#[derive(Debug, Clone)]
pub struct ParserState {
    tokens: Vec<Token>,

    function_table: IndexMap<String, FunctionDefinition>,

    custom_types: Arc<IndexMap<String, CustomType>>,

    imported_functions: Arc<HashMap<String, FunctionSignature>>,
}

impl ParserState {
    pub fn parse_tokens(&mut self) -> Result<()> {
        println!("Creating signature table...");
        // Create user defined signature table
        // Create an import table which can be used later by other functions
        let (unparsed_functions, source_imports, mut external_imports, custom_types) =
            create_signature_table(self.tokens.clone())?;

        let custom_types: Arc<IndexMap<String, CustomType>> = Arc::new(custom_types);

        // Extend the list of external imports with source imports aka imports from Fog source files.
        external_imports.extend(
            source_imports
                .iter()
                .map(|(fn_name, fn_def)| (fn_name.clone(), fn_def.function_sig.clone())),
        );

        let imports = Arc::new(external_imports);

        // Copy the the HashMap to this field
        self.imported_functions = imports.clone();

        println!("Parsing functions...");
        // Set the function table field of this struct
        self.function_table = parse_functions(
            Arc::new(unparsed_functions),
            imports.clone(),
            custom_types.clone(),
        )?;

        self.custom_types = custom_types.clone();

        Ok(())
    }

    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            function_table: IndexMap::new(),
            imported_functions: Arc::new(HashMap::new()),
            custom_types: Arc::new(IndexMap::new()),
        }
    }

    pub fn function_table(&self) -> &IndexMap<String, FunctionDefinition> {
        &self.function_table
    }

    pub fn imported_functions(&self) -> &HashMap<String, FunctionSignature> {
        &self.imported_functions
    }

    pub fn custom_types(&self) -> Arc<IndexMap<String, CustomType>> {
        self.custom_types.clone()
    }
}
