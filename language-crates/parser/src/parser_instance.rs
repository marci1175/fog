use std::{collections::HashMap, sync::Arc};

use fog_common::{
    anyhow::Result,
    codegen::CustomType,
    compiler::ProjectConfig,
    error::parser::ParserError,
    indexmap::IndexMap,
    parser::{FunctionDefinition, FunctionSignature, FunctionVisibility},
    tokenizer::Token,
    ty::OrdSet,
};

use crate::parser::function::{create_signature_table, parse_functions};

#[derive(Debug, Clone)]
pub struct Parser
{
    tokens: Vec<Token>,

    function_table: IndexMap<String, FunctionDefinition>,

    library_public_function_table: IndexMap<Vec<String>, FunctionSignature>,

    custom_types: Arc<IndexMap<String, CustomType>>,

    imported_functions: Arc<HashMap<String, FunctionSignature>>,

    config: ProjectConfig,

    enabled_features: OrdSet<String>,

    module_path: Vec<String>,
}

impl Parser
{
    pub fn parse(&mut self, dep_fn_list: IndexMap<Vec<String>, FunctionSignature>) -> Result<()>
    {
        // Create user defined signature table
        // Create an import table which can be used later by other functions
        let (unparsed_functions, dep_imports, mut imports, custom_types) = create_signature_table(
            self.tokens.clone(),
            self.module_path.clone(),
            self.enabled_features.clone(),
            self.config.clone(),
        )?;

        let custom_types: Arc<IndexMap<String, CustomType>> = Arc::new(custom_types);

        // Only import the functions which have been specifically imported by the user too
        for import in dep_imports.iter() {
            if let Some(imported_fn_sig) = dep_fn_list.get(import) {
                if let Some(reimported_function) =
                    imports.insert(imported_fn_sig.name.clone(), imported_fn_sig.clone())
                {
                    return Err(
                        ParserError::DuplicateSignatureImports(reimported_function.name).into(),
                    );
                }
            }
            else {
                return Err(ParserError::FunctionDependencyNotFound(import.clone()).into());
            }
        }

        // Extend the list of external imports with source imports aka imports from Fog source files.
        // imports.extend(
        //     source_imports
        //         .iter()
        //         .map(|(fn_name, fn_def)| (fn_name.clone(), fn_def.function_sig.clone())),
        // );

        let imports = Arc::new(imports);

        // Copy the the HashMap to this field
        self.imported_functions = imports.clone();

        self.library_public_function_table = IndexMap::from_iter(
            unparsed_functions
                .iter()
                .filter(|(_fn_name, unparsed_fn)| {
                    unparsed_fn.function_sig.visibility == FunctionVisibility::PublicLibrary
                })
                .map(|(fn_name, unparsed_fn)| {
                    (
                        unparsed_fn.function_sig.module_path.clone(),
                        unparsed_fn.function_sig.clone(),
                    )
                }),
        );

        // Set the function table field of this struct
        self.function_table = parse_functions(
            self.config.clone(),
            Arc::new(unparsed_functions),
            imports.clone(),
            custom_types.clone(),
            self.module_path.clone(),
        )?;

        self.custom_types = custom_types.clone();

        Ok(())
    }

    pub fn new(
        tokens: Vec<Token>,
        config: ProjectConfig,
        module_path: Vec<String>,
        enabled_features: OrdSet<String>,
    ) -> Self
    {
        Self {
            tokens,
            function_table: IndexMap::new(),
            imported_functions: Arc::new(HashMap::new()),
            library_public_function_table: IndexMap::new(),
            enabled_features,
            custom_types: Arc::new(IndexMap::new()),
            config,
            module_path,
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

    pub fn library_public_function_table(&self) -> &IndexMap<Vec<String>, FunctionSignature>
    {
        &self.library_public_function_table
    }

    pub fn config(&self) -> &ProjectConfig
    {
        &self.config
    }

    pub fn enabled_features(&self) -> &OrdSet<String>
    {
        &self.enabled_features
    }
}
