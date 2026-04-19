use std::{
    collections::{HashMap, HashSet}, path::PathBuf, rc::Rc
};

use common::{
    anyhow::{self, Result},
    codegen::CustomItem,
    compiler::ProjectConfig,
    dashmap::DashMap,
    error::{SpanInfo, Spanned, codegen::CodeGenError, parser::ParserError},
    indexmap::IndexMap,
    parser::{common::{ItemVisibility, TokenStream}, function::{
        FunctionDefinition, FunctionSignature, PathMap,
        UnparsedFunctionDefinition, parse_signature_argument_tokens,
    }},
    tokenizer::{Token, TokenDiscriminants},
    ty::OrdSet,
};

#[derive(Clone, Debug)]
pub struct Context
{
    pub functions: PathMap<Vec<String>, String, UnparsedFunctionDefinition>,
    pub items: PathMap<Vec<String>, String, CustomItem>,
    pub external_decls: PathMap<Vec<String>, String, FunctionSignature>,
}

impl Context
{
    pub fn new() -> Self
    {
        Self {
            functions: PathMap::new(),
            items: PathMap::new(),
            external_decls: PathMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Settings
{
    // Project settings
    pub config: ProjectConfig,
    pub enabled_features: OrdSet<String>,
    /// The path to the root of this project.
    /// This is important when we are parsing libraries.
    pub module_path: Vec<String>,
    pub root_path: PathBuf,
}

impl Settings
{
    /*
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

    /*
        Internal notes:
        imma change some of the syntax for example imma make it so that i can do `pub import "blabla.f", so that i can bring path into scope.`
    */

    pub fn parse(&self, tokens: &mut TokenStream<Spanned<Token>>) -> Result<()>
    {
        // The first step should be parsing the top level items, such as structs, functions, enums.
        // We will store all the items present, and parse the inner contents of the function later.
        // By doing this, the compiler wont be single pass anymore and the sequence of function declarations wont be important.
        // Im gonna first parse the entire main file and then work out/parse all the other files which were linked.
        let mut ctx = Context::new();

        // Parse the actual tokens
        while let Some(tkn) = tokens.consume().cloned() {
            match tkn.inner() {
                Token::ItemVisibility(vis) => {
                    // Type of the item
                    let item_tkn = tokens.try_consume_match(ParserError::ItemTypeExpected, &TokenDiscriminants::TypeDefinition)?;

                    // Match the type of the item
                    match item_tkn.inner() {
                        Token::TypeDefinition(item_type) => {
                            match item_type {
                                common::tokenizer::TypeToken::Enum => parse_enum(&mut ctx, vis, tokens),
                                common::tokenizer::TypeToken::Struct => parse_struct(&mut ctx, vis, tokens),
                                common::tokenizer::TypeToken::Function => parse_function(&mut ctx, vis, tokens)?,
                                _ => return Err(ParserError::ItemTypeExpected.into())
                            }
                        }
                        
                        _ => return Err(ParserError::ItemTypeExpected.into())
                    }
                }

                // If the token was not recognized, return an error.
                _ => return Err(ParserError::ItemRequiresExplicitVisibility.into())
            }
        }

        Ok(())
    }

    pub fn new(
        config: ProjectConfig,
        module_path: Vec<String>,
        enabled_features: OrdSet<String>,
        root_path: PathBuf,
    ) -> Self
    {
        Self {
            enabled_features,
            config,
            module_path,
            root_path,
        }
    }
}

///
pub fn parse_function(ctx: &mut Context, vis: &ItemVisibility, tokens: &mut TokenStream<Spanned<Token>>) -> anyhow::Result<()> {
    // Get the function name token
    let function_name_tkn = tokens.try_consume_match(ParserError::SyntaxError(common::error::syntax::SyntaxError::InvalidFunctionName), &TokenDiscriminants::Identifier)?;
    // Parse function name, its safe to unwrap here
    let function_name = function_name_tkn.try_as_identifier_ref().unwrap();

    //Parse the arguments of the function
    // If the first token is a '|' that means the function has generics defined
    // If the first token is a '(' that means that its just a normal function
    if let Some(first_token) = tokens.consume() {
        match first_token.inner() {
            // Parse generics before arguments
            Token::BitOr => {
                
            },
            // Parse arguments
            Token::OpenParentheses => {
                
            },
            _ => return Err(ParserError::InvalidFunctionArgumentDefinition.into())
        }
    }

    Ok(())
}

pub fn parse_enum(ctx: &mut Context, vis: &ItemVisibility, tokens: &mut TokenStream<Spanned<Token>>) {

}

pub fn parse_struct(ctx: &mut Context, vis: &ItemVisibility, tokens: &mut TokenStream<Spanned<Token>>) {

}