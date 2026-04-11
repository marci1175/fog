use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use common::{
    anyhow::Result,
    codegen::CustomItem,
    compiler::ProjectConfig,
    dashmap::DashMap,
    error::{SpanInfo, parser::ParserError},
    indexmap::IndexMap,
    parser::function::{
        FunctionDefinition, PathMap, FunctionSignature, FunctionVisibility,
        UnparsedFunctionDefinition,
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

    pub fn parse(&self, tokens: Vec<Token>) -> Result<()>
    {
        // The first step should be parsing the top level items, such as structs, functions, enums.
        // We will store all the items present, and parse the inner contents of the function later.
        // By doing this, the compiler wont be single pass anymore and the sequence of function declarations wont be important.

        

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

    fn link_files(&self) {
        
    }

    fn create_context(&self) -> Result<Context> {
        let mut ctx = Context::new();



        Ok(ctx)
    }
}

pub struct Context {
    pub functions: PathMap<Vec<String>, String, UnparsedFunctionDefinition>,
    pub items: PathMap<Vec<String>, String, CustomItem>,
}

impl Context {
    pub fn new() -> Self {
        Self { functions: PathMap::new(), items: PathMap::new() }
    }
}