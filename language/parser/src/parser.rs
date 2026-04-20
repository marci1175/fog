use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    rc::Rc,
};

use common::{
    anyhow::{self, Result},
    codegen::CustomItem,
    compiler::ProjectConfig,
    dashmap::DashMap,
    error::{SpanInfo, Spanned, codegen::CodeGenError, parser::ParserError},
    indexmap::IndexMap,
    parser::{
        common::{ItemVisibility, TokenStream},
        function::{
            CompilerInstruction, FunctionArguments, FunctionDefinition, FunctionSignature, PathMap, UnparsedFunctionDefinition, parse_signature_argument_tokens
        },
    },
    tokenizer::{Token, TokenDiscriminants},
    ty::{OrdMap, OrdSet},
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

        // Collect the compiler instructions in a list and we can move the instructions to the next item we are parsing.
        let mut item_compiler_instruction: OrdSet<CompilerInstruction> = OrdSet::new();

        // Parse the actual tokens
        while let Some(tkn) = tokens.consume().cloned() {
            match tkn.inner() {
                Token::CompilerHintSymbol => {
                    parse_compiler_instruction(&mut item_compiler_instruction, tokens)?;
                }
                Token::ItemVisibility(vis) => {
                    // Type of the item
                    let item_tkn = tokens.try_consume_match(
                        ParserError::ItemTypeExpected,
                        &TokenDiscriminants::TypeDefinition,
                    )?;

                    // Match the type of the item
                    match item_tkn.inner() {
                        Token::TypeDefinition(item_type) => {
                            match item_type {
                                common::tokenizer::TypeToken::Enum => {
                                    parse_enum(&mut ctx, vis, tokens)
                                },
                                common::tokenizer::TypeToken::Struct => {
                                    parse_struct(&mut ctx, vis, tokens)
                                },
                                common::tokenizer::TypeToken::Function => {
                                    parse_function(&mut ctx, vis, tokens)?
                                },
                                _ => return Err(ParserError::ItemTypeExpected.into()),
                            }
                        },

                        _ => return Err(ParserError::ItemTypeExpected.into()),
                    }
                },

                // If the token was not recognized, return an error.
                _ => return Err(ParserError::ItemRequiresExplicitVisibility.into()),
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

/*

    All of these functions should be moved to the `common` library.

*/

///
pub fn parse_function(
    ctx: &mut Context,
    vis: &ItemVisibility,
    tokens: &mut TokenStream<Spanned<Token>>,
) -> anyhow::Result<()>
{
    // Get the function name token
    let function_name_tkn = tokens.try_consume_match(
        ParserError::SyntaxError(common::error::syntax::SyntaxError::InvalidFunctionName),
        &TokenDiscriminants::Identifier,
    )?;

    // Parse function name, its safe to unwrap here
    let function_name = function_name_tkn.try_as_identifier_ref().unwrap();

    // This will hold the function's arguments. This variable will get modified later.
    let mut arguments = FunctionArguments::new();

    //Parse the arguments of the function
    // If the first token is a '|' that means the function has generics defined
    // If the first token is a '(' that means that its just a normal function
    if let Some(tkn) = tokens.consume() {
        match tkn.inner() {
            // Parse generics before arguments
            Token::BitOr => {
                parse_fn_generics(ctx, &mut arguments, tokens)?;
                parse_fn_arguments(ctx, &mut arguments, tokens)?;
            },
            // Parse arguments
            Token::OpenParentheses => parse_fn_arguments(ctx, &mut arguments, tokens)?,
            _ => return Err(ParserError::InvalidFunctionArgumentDefinition.into()),
        }
    }

    // The TokenStream should now point to `Token::OpenBraces`
    tokens.try_consume_match(
        ParserError::SyntaxError(common::error::syntax::SyntaxError::InvalidFunctionBodyStart),
        &TokenDiscriminants::OpenBraces,
    )?;

    // Fetch the function body and increment the tokenstream accordingly.
    let fn_body = fetch_fn_body(tokens)?;

    // This should never return an error since we are already checking the closing brace when fetching the fn body.
    tokens.try_consume_match(
        ParserError::SyntaxError(common::error::syntax::SyntaxError::LeftOpenBraces),
        &TokenDiscriminants::CloseBraces,
    )?;


    
    Ok(())
}

/// The function assumes the first token to be the first token in the `|`s.
pub fn parse_fn_generics(
    ctx: &mut Context,
    arguments: &mut FunctionArguments,
    tokens: &mut TokenStream<Spanned<Token>>,
) -> anyhow::Result<()>
{
    let mut generics: OrdMap<String, OrdSet<Vec<String>>> = OrdMap::new();

    /*
        Syntax definition:

        {
            <generic> ":" { { <trait> ["+"] } [","] } [","]
        }
    */
    // Lets loop through all the generics
    while let Some(tkn) = tokens.consume() {
        match tkn.inner() {
            Token::Identifier(generic_name) => {},
            // If we encounter the closing `|` break the loop
            Token::BitOr => break,

            _ => {
                return Err(ParserError::SyntaxError(
                    common::error::syntax::SyntaxError::InvalidFunctionGenericsDefinition,
                )
                .into());
            },
        }
    }

    Ok(())
}

/// The function assumes the first token to be the first token in the parentheses.
pub fn parse_fn_arguments(
    ctx: &mut Context,
    arguments: &mut FunctionArguments,
    tokens: &mut TokenStream<Spanned<Token>>,
) -> anyhow::Result<()>
{
    Ok(())
}

/// This function will not parse the tokens present in the function body. It will only fetch them but not evaluate them further.
pub fn fetch_fn_body<'a>(
    tokens: &'a mut TokenStream<Spanned<Token>>,
) -> anyhow::Result<&'a [Spanned<Token>]>
{
    // Get the index of the closing brace token
    let body_closing_tkn = find_closing_braces(&*tokens).ok_or(ParserError::SyntaxError(common::error::syntax::SyntaxError::LeftOpenBraces))?;

    // It is safe to unwrap here, since we have already checked if the closing braces would be in the TokenStream
    Ok(tokens.consume_bulk(body_closing_tkn).unwrap())
}

pub fn find_closing_braces(tokens: &TokenStream<Spanned<Token>>) -> Option<usize>
{
    tokens
        .peek_remainder()
        .map(|tkns| {
            let mut braces_counter: usize = 1;

            for (idx, token) in tkns.iter().enumerate() {
                if token.inner() == &Token::OpenBraces {
                    braces_counter += 1;
                }
                else if token.inner() == &Token::CloseBraces {
                    braces_counter -= 1;
                }

                if braces_counter == 0 {
                    return Some(idx);
                }
            }

            None
        })
        .flatten()
}

pub fn parse_compiler_instruction(instr_buf: &mut OrdSet<CompilerInstruction>, tokens: &mut TokenStream<Spanned<Token>>) -> anyhow::Result<()> {
    if let Some(tkn) = tokens.consume() {
        match tkn.inner() {
            Token::CompilerInstruction(instr) => {
                // If this is a feature that means the next token should be a string referencing the feature name.
                if instr == CompilerInstruction::Feature {

                }
                // If its not a feature we can just store the instruction as is.
                else {

                }
            }
            _ => {}
        }
    }
    else {
        return Err(ParserError::SyntaxError(common::error::syntax::SyntaxError::CompilerInstructionRequiredAfterSymbol).into());
    }

    Ok(())
}

pub fn parse_enum(ctx: &mut Context, vis: &ItemVisibility, tokens: &mut TokenStream<Spanned<Token>>)
{
}

pub fn parse_struct(
    ctx: &mut Context,
    vis: &ItemVisibility,
    tokens: &mut TokenStream<Spanned<Token>>,
)
{
}
