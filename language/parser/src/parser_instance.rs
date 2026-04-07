use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use common::{
    anyhow::Result,
    codegen::CustomItem,
    compiler::ProjectConfig,
    dashmap::DashMap,
    error::{DbgInfo, parser::ParserError},
    indexmap::IndexMap,
    parser::function::{
        FunctionDefinition, FunctionMap, FunctionSignature, FunctionVisibility, UnparsedFunctionDefinition
    },
    tokenizer::Token,
    ty::OrdSet,
};

#[derive(Debug, Clone)]
pub struct SigTable
{
    pub function_list: IndexMap<String, UnparsedFunctionDefinition>,
    pub dependency_imports: HashSet<Vec<String>>,
    pub external_imports: HashMap<String, FunctionSignature>,
    pub custom_types: IndexMap<String, CustomItem>,
    pub imported_file_list: HashMap<Vec<String>, FunctionDefinition>,
}

#[derive(Debug, Clone)]
pub struct ParserSettings
{
    // Project settings
    pub config: ProjectConfig,
    pub enabled_features: OrdSet<String>,
    /// The path to the root of this project.
    /// This is important when we are parsing libraries.
    pub module_path: Vec<String>,
}

impl ParserSettings
{
    /*
        REEEEEEEEEEEEEEEEEEEECOOOOOOOOODE

        ---------Fuck all this---------
        TODO: recode importing stuff

        First of all, remove the extra logic from here relating to dependencies
        Also, when parsing the deps make a dependency tree, with the value of `HashMap<&[&str], Dependency>`
        Implement parsing for `foo::bar::x()` type expressions, this will allow us to use functions with the same name on different paths

        Modify the type resolving function to look up dependency items
        Create the `namespace` keyword rework how the dependency paths work
        ```
        namespace backend {
            struct request {};
        }

        use backend::request;
        ```
    */
    pub fn parse(&self, tokens: Vec<Token>, dependency_functions_map: &FunctionMap<Vec<String>, String, FunctionSignature>)
    -> Result<()>
    {
        // Create user defined signature table
        // Create an import table which can be used later by other functions
        // let (
        //     mut unparsed_functions,
        //     dep_imports,
        //     mut external_imports,
        //     mut custom_types,
        //     file_imported_functions,
        // ) = self.create_signature_table(dependency_functions_map.clone())?;

        // // Only import the functions which have been specifically imported by the user too
        // for import in dep_imports.iter() {
        //     let import_result = if let Some(imported_fn_sig) = dependency_functions_map.get(import) {
        //         external_imports.insert(imported_fn_sig.name.clone(), imported_fn_sig.clone())
        //     }
        //     else if let Some(file_imported_fn) = file_imported_functions.get(import) {
        //         self.function_table
        //             .insert(import.clone(), file_imported_fn.clone(), file_imported_fn.signature.name.clone().into());

        //         external_imports.insert(
        //             file_imported_fn.signature.name.clone(),
        //             file_imported_fn.signature.clone(),
        //         )
        //     }
        //     else {
        //         return Err(ParserError::FunctionDependencyNotFound(import.clone()).into());
        //     };

        //     if let Some(reimported_function) = import_result {
        //         return Err(
        //             ParserError::DuplicateSignatureImports(reimported_function.name).into(),
        //         );
        //     }
        // }

        // let imports = Rc::new(external_imports);

        // // Copy the the HashMap to this field
        // self.imported_functions = imports.clone();

        // self.library_public_function_table = IndexMap::from_iter(
        //     unparsed_functions
        //         .iter()
        //         .filter(|(_fn_path, _fn_name, unparsed_fn)| {
        //             unparsed_fn.signature.visibility == FunctionVisibility::PublicLibrary
        //         })
        //         .map(|(_, _, unparsed_fn)| {
        //             (
        //                 unparsed_fn.signature.module_path.clone(),
        //                 unparsed_fn.signature.clone(),
        //             )
        //         }),
        // );

        // Set the function table field of this struct
        
        //
        // TODO: Do not start by implementing extend for Functionmap, instead recode the importing code.
        //

        // self.function_table.extend(self.parse_functions(
        //     &mut unparsed_functions,
        //     imports.clone(),
        //     &mut custom_types,
        // )?);

        // self.custom_types = Rc::new(custom_types);

        Ok(())
    }

    pub fn new(
        config: ProjectConfig,
        module_path: Vec<String>,
        enabled_features: OrdSet<String>,
    ) -> Self
    {
        Self {
            enabled_features,
            config,
            module_path,
        }
    }
}
