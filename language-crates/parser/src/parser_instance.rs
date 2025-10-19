use std::{collections::HashMap, sync::Arc};

use fog_common::{
    anyhow::Result, codegen::CustomType, compiler::ProjectConfig, indexmap::IndexMap, parser::{FunctionDefinition, FunctionSignature}, tokenizer::Token
};

use crate::parser::function::{create_signature_table, parse_functions};

#[derive(Debug, Clone)]
pub struct Parser
{
    tokens: Vec<Token>,

    function_table: IndexMap<String, FunctionDefinition>,

    library_public_function_table: IndexMap<String, FunctionSignature>,

    custom_types: Arc<IndexMap<String, CustomType>>,

    imported_functions: Arc<HashMap<String, FunctionSignature>>,

    config: ProjectConfig,
}

impl Parser
{
    pub fn parse(
        &mut self,
        dep_fn_list: HashMap<String, IndexMap<String, FunctionSignature>>,
    ) -> Result<()>
    {
        // Create user defined signature table
        // Create an import table which can be used later by other functions
        let (
            library_public_function_table,
            unparsed_functions,
            source_imports,
            mut external_imports,
            custom_types,
        ) = create_signature_table(self.tokens.clone())?;

        let custom_types: Arc<IndexMap<String, CustomType>> = Arc::new(custom_types);

        let external_import_clone = external_imports.clone();

        // Only import the functions which have been specifically import by the user too
        external_imports.extend(dep_fn_list.values().flat_map(|v| {
            v.iter()
                .filter(|(fn_name, fn_sig)| {
                    external_import_clone
                        .get(*fn_name)
                        .is_some_and(|import_sig| {
                            **fn_sig == *import_sig
                        })
                })
                .map(|(k, v)| (k.clone(), v.clone()))
        }));

        // Extend the list of external imports with source imports aka imports from Fog source files.
        external_imports.extend(
            source_imports
                .iter()
                .map(|(fn_name, fn_def)| (fn_name.clone(), fn_def.function_sig.clone())),
        );

        let imports = Arc::new(external_imports);

        // Copy the the HashMap to this field
        self.imported_functions = imports.clone();
        self.library_public_function_table = library_public_function_table.clone();

        // Set the function table field of this struct
        self.function_table = parse_functions(
            self.config.clone(),
            Arc::new(unparsed_functions),
            imports.clone(),
            custom_types.clone(),
        )?;

        self.custom_types = custom_types.clone();

        Ok(())
    }

    pub fn new(tokens: Vec<Token>, config: ProjectConfig) -> Self
    {
        Self {
            tokens,
            function_table: IndexMap::new(),
            imported_functions: Arc::new(HashMap::new()),
            library_public_function_table: IndexMap::new(),
            custom_types: Arc::new(IndexMap::new()),
            config,
        }
    }

    pub fn function_table(&self) -> &IndexMap<String, FunctionDefinition>
    {
        &self.function_table
    }

    pub fn imported_functions(&self) -> &HashMap<String, FunctionSignature>
    {
        &self.imported_functions
    }

    pub fn custom_types(&self) -> Arc<IndexMap<String, CustomType>>
    {
        self.custom_types.clone()
    }

    pub fn library_public_function_table(&self) -> &IndexMap<String, FunctionSignature>
    {
        &self.library_public_function_table
    }
}
