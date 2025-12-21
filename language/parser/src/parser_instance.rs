use std::{collections::HashMap, sync::Arc};

use common::{
    anyhow::Result,
    codegen::CustomType,
    compiler::ProjectConfig,
    dashmap::DashMap,
    error::{DebugInformation, parser::ParserError},
    indexmap::IndexMap,
    parser::{FunctionDefinition, FunctionSignature, FunctionVisibility},
    tokenizer::Token,
    ty::OrdSet,
};

#[derive(Debug, Clone)]
pub struct Parser
{
    pub tokens: Vec<Token>,
    pub tokens_debug_info: Vec<DebugInformation>,
    pub function_table: IndexMap<String, FunctionDefinition>,
    pub library_public_function_table: IndexMap<Vec<String>, FunctionSignature>,
    pub custom_types: Arc<IndexMap<String, CustomType>>,
    pub imported_functions: Arc<HashMap<String, FunctionSignature>>,
    pub config: ProjectConfig,
    pub enabled_features: OrdSet<String>,
    pub module_path: Vec<String>,
}

impl Parser
{
    pub fn parse(&mut self, dep_fn_list: Arc<DashMap<Vec<String>, FunctionSignature>>)
    -> Result<()>
    {
        // Create user defined signature table
        // Create an import table which can be used later by other functions
        let (
            unparsed_functions,
            dep_imports,
            mut external_imports,
            custom_types,
            file_imported_functions,
        ) = self.create_signature_table(dep_fn_list.clone())?;

        let custom_types: Arc<IndexMap<String, CustomType>> = Arc::new(custom_types);

        // Only import the functions which have been specifically imported by the user too
        for import in dep_imports.iter() {
            let import_result = if let Some(imported_fn_sig) = dep_fn_list.get(import) {
                external_imports.insert(imported_fn_sig.name.clone(), imported_fn_sig.clone())
            }
            else if let Some(file_imported_fn) = file_imported_functions.get(import) {
                self.function_table.insert(
                    file_imported_fn.function_sig.name.clone(),
                    file_imported_fn.clone(),
                );

                external_imports.insert(
                    file_imported_fn.function_sig.name.clone(),
                    file_imported_fn.function_sig.clone(),
                )
            }
            else {
                return Err(ParserError::FunctionDependencyNotFound(import.clone()).into());
            };

            if let Some(reimported_function) = import_result {
                return Err(
                    ParserError::DuplicateSignatureImports(reimported_function.name).into(),
                );
            }
        }

        let imports = Arc::new(external_imports);

        // Copy the the HashMap to this field
        self.imported_functions = imports.clone();

        self.library_public_function_table = IndexMap::from_iter(
            unparsed_functions
                .iter()
                .filter(|(_fn_name, unparsed_fn)| {
                    unparsed_fn.function_sig.visibility == FunctionVisibility::PublicLibrary
                })
                .map(|(_fn_name, unparsed_fn)| {
                    (
                        unparsed_fn.function_sig.module_path.clone(),
                        unparsed_fn.function_sig.clone(),
                    )
                }),
        );

        // Set the function table field of this struct
        self.function_table.extend(self.parse_functions(
            Arc::new(unparsed_functions),
            imports.clone(),
            custom_types.clone(),
        )?);

        self.custom_types = custom_types.clone();

        Ok(())
    }

    pub fn new(
        tokens: Vec<Token>,
        token_ranges: Vec<DebugInformation>,
        config: ProjectConfig,
        module_path: Vec<String>,
        enabled_features: OrdSet<String>,
    ) -> Self
    {
        Self {
            tokens,
            tokens_debug_info: token_ranges,
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
