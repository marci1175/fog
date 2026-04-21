use std::path::PathBuf;

use common::{
    anyhow::{self, Result}, codegen::CustomItem, combine_path, compiler::ProjectConfig, error::{Spanned, parser::ParserError}, parser::{
        common::{ItemVisibility, ParsedToken, Streamable, TokenStream},
        function::{
            CompilerInstruction, CompilerInstructionDiscriminants, FunctionArguments, FunctionDefinition, FunctionSignature, PathMap, UnparsedFunctionDefinition
        },
    }, tokenizer::{Token, TokenDiscriminants}, ty::{OrdMap, OrdSet, Type}
};

#[derive(Clone, Debug)]
pub struct Context
{
    pub functions: PathMap<Vec<String>, String, FunctionDefinition>,
    pub items: PathMap<Vec<String>, String, CustomItem>,
    pub external_decls: PathMap<Vec<String>, String, FunctionSignature>,
    pub path: Vec<String>,
}

impl Context
{
    pub fn new(path: Vec<String>) -> Self
    {
        Self {
            functions: PathMap::new(),
            items: PathMap::new(),
            external_decls: PathMap::new(),
            path,
        }
    }

    pub fn create_function(&self, vis: ItemVisibility, name: String, arguments: FunctionArguments, return_type: Type, compiler_instructions: OrdSet<CompilerInstruction>, body: Vec<Spanned<ParsedToken>>) -> FunctionDefinition {
        FunctionDefinition { signature: FunctionSignature { name: name, args: arguments, return_type, module_path: self.path.clone(), visibility: vis, compiler_instructions }, body }
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

    pub fn parse(&self, tokens: &mut TokenStream<Spanned<Token>>) -> Result<Context>
    {
        // The first step should be parsing the top level items, such as structs, functions, enums.
        // We will store all the items present, and parse the inner contents of the function later.
        // By doing this, the compiler wont be single pass anymore and the sequence of function declarations wont be important.
        // Im gonna first parse the entire main file and then work out/parse all the other files which were linked.
        let mut ctx = Context::new(self.module_path.clone());
        
        // Collect the compiler instructions in a list and we can move the instructions to the next item we are parsing.
        let mut item_compiler_instruction: OrdSet<CompilerInstruction> = OrdSet::new();

        // Parse the actual tokens
        while let Some(tkn) = tokens.consume().cloned() {
            match tkn.inner() {
                Token::CompilerHintSymbol => {
                    parse_compiler_instruction(&mut item_compiler_instruction, tokens)?;
                },
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
                                    parse_enum(
                                        &mut ctx,
                                        vis,
                                        tokens,
                                        std::mem::take(&mut item_compiler_instruction),
                                    )
                                },
                                common::tokenizer::TypeToken::Struct => {
                                    parse_struct(
                                        &mut ctx,
                                        vis,
                                        tokens,
                                        std::mem::take(&mut item_compiler_instruction),
                                    )
                                },
                                common::tokenizer::TypeToken::Function => {
                                    let function = parse_function(
                                        &mut ctx,
                                        vis,
                                        tokens,
                                        std::mem::take(&mut item_compiler_instruction),
                                    )?;

                                    ctx.functions.insert(combine_path(function.signature.module_path.clone(), function.signature.name.clone()), function.signature.name.clone().into(), function);
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

        Ok(ctx)
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

/// The function parses the entire function, but does not validate the function's body.
/// Syntax of a function:
/// <vis> "function" <name> "(" [{<arg>: <type>}] ")" ":" <return type> "{" [{<expr>}] "}"
pub fn parse_function(
    ctx: &Context,
    vis: &ItemVisibility,
    tokens: &mut TokenStream<Spanned<Token>>,
    compiler_instructions: OrdSet<CompilerInstruction>,
) -> anyhow::Result<FunctionDefinition>
{
    // Get the function name token
    let function_name_tkn = tokens.try_consume_match(
        ParserError::SyntaxError(common::error::syntax::SyntaxError::InvalidFunctionName),
        &TokenDiscriminants::Identifier,
    )?;

    // Parse function name, its safe to unwrap here
    let function_name = function_name_tkn.try_as_identifier_ref().unwrap().to_owned();

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

    // This should be the ":" character singaling the return type
    tokens.try_consume_match(
        ParserError::SyntaxError(common::error::syntax::SyntaxError::FunctionRequiresReturn),
        &TokenDiscriminants::Colon,
    )?;

    // Parse the return type of the function
    let return_type = parse_type(tokens)?;

    // The TokenStream should now point to `Token::OpenBraces`
    tokens.try_consume_match(
        ParserError::SyntaxError(common::error::syntax::SyntaxError::InvalidFunctionBodyStart),
        &TokenDiscriminants::OpenBraces,
    )?;

    // Fetch the function body and increment the tokenstream accordingly.
    let fn_body = parse_fn_body(tokens)?;

    // This should never return an error since we are already checking the closing brace when fetching the fn body.
    tokens.try_consume_match(
        ParserError::SyntaxError(common::error::syntax::SyntaxError::LeftOpenBraces),
        &TokenDiscriminants::CloseBraces,
    )?;

    Ok(ctx.create_function(vis.clone(), function_name, arguments, return_type, compiler_instructions, fn_body))
}

pub fn parse_type(tokens: &mut TokenStream<Spanned<Token>>) -> anyhow::Result<Type>
{
    if let Some(tkn) = tokens.consume() {
        return match tkn.inner() {
            Token::TypeDefinition(ty) => {
                match ty {
                    common::tokenizer::TypeToken::String
                    | common::tokenizer::TypeToken::Boolean
                    | common::tokenizer::TypeToken::Void
                    | common::tokenizer::TypeToken::I64
                    | common::tokenizer::TypeToken::F64
                    | common::tokenizer::TypeToken::U64
                    | common::tokenizer::TypeToken::I32
                    | common::tokenizer::TypeToken::F32
                    | common::tokenizer::TypeToken::U32
                    | common::tokenizer::TypeToken::I16
                    | common::tokenizer::TypeToken::F16
                    | common::tokenizer::TypeToken::U16
                    | common::tokenizer::TypeToken::U8 => Ok((ty.to_owned()).try_into()?),

                    common::tokenizer::TypeToken::Array => {
                        // Array syntax
                        // "Array" "<" <type> "," <len> ">"

                        // The next token should be a "<"
                        tokens.try_consume_match(
                            ParserError::SyntaxError(
                                common::error::syntax::SyntaxError::InvalidTypeGenericDefinition,
                            ),
                            &TokenDiscriminants::OpenAngledBrackets,
                        )?;

                        // Resolve the base type of the array
                        let ty = parse_type(tokens)?;

                        // Ensure syntax correctness
                        tokens.try_consume_match(
                            ParserError::SyntaxError(
                                common::error::syntax::SyntaxError::InvalidTypeGenericDefinition,
                            ),
                            &TokenDiscriminants::Comma,
                        )?;

                        // Parse the length of the array
                        let len_val = tokens.try_consume_match(ParserError::SyntaxError(common::error::syntax::SyntaxError::InvalidTypeGenericDefinition), &TokenDiscriminants::Literal)?.try_as_literal_ref().unwrap().to_owned();

                        // Get the raw value of the array's length
                        let len = len_val.try_as_u_32().ok_or(ParserError::SyntaxError(
                            common::error::syntax::SyntaxError::InvalidArrayLenType,
                        ))?;

                        // Ensure syntax correctness
                        tokens.try_consume_match(
                            ParserError::SyntaxError(
                                common::error::syntax::SyntaxError::InvalidTypeGenericDefinition,
                            ),
                            &TokenDiscriminants::CloseAngledBrackets,
                        )?;

                        Ok(Type::Array((Box::new(ty), len as usize)))
                    },
                    common::tokenizer::TypeToken::Pointer => {
                        // Pointer syntax
                        // "ptr" [ "<" <type> ">" ]
                        // If the underlying type is not specified with the pointer, the underlying data can be transmuted.
                        // If the the underlying type is explicitly indicated the pointer can only be dereferenced to that specific type.
                        // ptr<T> = ptr
                        // ptr != ptr<T>

                        // Check if the next token matches the syntax for specifying the inner type.
                        if let Some(Spanned {
                            inner: Token::OpenAngledBrackets,
                            ..
                        }) = tokens.consume()
                        {
                            // Resolve the base type of the pointer
                            let ty = parse_type(tokens)?;

                            // Ensure syntax correctness
                            tokens.try_consume_match(ParserError::SyntaxError(common::error::syntax::SyntaxError::InvalidTypeGenericDefinition), &TokenDiscriminants::CloseAngledBrackets)?;

                            Ok(Type::Pointer(Some(Box::new(ty))))
                        }
                        // We can assume that the inner type is not specified
                        else {
                            Ok(Type::Pointer(None))
                        }
                    },

                    common::tokenizer::TypeToken::Enum
                    | common::tokenizer::TypeToken::Struct
                    | common::tokenizer::TypeToken::Function => {
                        return Err(ParserError::InvalidType.into());
                    },
                }
            },
            Token::Identifier(ident) => Ok(Type::Unresolved(ident.to_owned())),
            _ => {
                return Err(ParserError::SyntaxError(
                    common::error::syntax::SyntaxError::FunctionRequiresReturn,
                )
                .into());
            },
        };
    }

    Err(ParserError::InternalTypeParsingTokenMissing.into())
}

/// The function assumes the first token to be the first token in the `|`s.
pub fn parse_fn_generics(
    ctx: &Context,
    arguments: &mut FunctionArguments,
    tokens: &mut TokenStream<Spanned<Token>>,
) -> anyhow::Result<()>
{
    let _generics: OrdMap<String, OrdSet<Vec<String>>> = OrdMap::new();

    /*
        Syntax definition:

        {
            <generic> ":" { { <trait> ["+"] } [","] } [","]
        }
    */
    // Lets loop through all the generics
    while let Some(tkn) = tokens.consume() {
        match tkn.inner() {
            Token::Identifier(_generic_name) => {},
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
    ctx: &Context,
    arguments: &mut FunctionArguments,
    tokens: &mut TokenStream<Spanned<Token>>,
) -> anyhow::Result<()>
{
    tokens.consume();
    Ok(())
}

/// This function will parse the tokens in the body of the function, but it will not check the validness of the tokens themselves.
/// 
/// The function parses the tokens but does not evaluate them.
pub fn parse_fn_body(
    tokens: &mut TokenStream<Spanned<Token>>,
) -> anyhow::Result<Vec<Spanned<ParsedToken>>>
{
    // Get the index of the closing brace token
    let body_closing_tkn = find_closing_braces(&*tokens).ok_or(ParserError::SyntaxError(
        common::error::syntax::SyntaxError::LeftOpenBraces,
    ))?;

    // It is safe to unwrap here, since we have already checked if the closing braces would be in the TokenStream
    let mut _fn_body = tokens.child_iterator_bulk(body_closing_tkn).unwrap();

    // Store the parsed tokens somewhere
    let parsed_tokens = Vec::new();

    // parse_tokens(&mut fn_body, &mut parsed_tokens)?;

    Ok(parsed_tokens)
}

pub fn find_closing_braces(tokens: &TokenStream<Spanned<Token>>) -> Option<usize>
{
    tokens
        .peek_remainder()
        .and_then(|tkns| {
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
}

pub fn parse_compiler_instruction(
    instr_buf: &mut OrdSet<CompilerInstruction>,
    tokens: &mut TokenStream<Spanned<Token>>,
) -> anyhow::Result<()>
{
    if let Some(tkn) = tokens.consume() {
        match tkn.inner() {
            Token::CompilerInstruction(instr) => {
                // If this is a feature that means the next token should be a string referencing the feature name.
                if instr == &CompilerInstructionDiscriminants::Feature {
                    // Its safe to unwrap since we are already checking inside the try consume
                    let feature_name = tokens
                        .try_consume_match(
                            ParserError::InvalidFunctionFeature,
                            &TokenDiscriminants::Identifier,
                        )?
                        .try_as_identifier_ref()
                        .unwrap();

                    instr_buf.insert(CompilerInstruction::Feature(feature_name.clone()));
                }
                // If its not a feature we can just store the instruction as is.
                else {
                    instr_buf.insert((*instr).into());
                }
            },
            _ => {
                return Err(ParserError::SyntaxError(
                    common::error::syntax::SyntaxError::CompilerInstructionRequiredAfterSymbol,
                )
                .into());
            },
        }
    }
    else {
        return Err(ParserError::SyntaxError(
            common::error::syntax::SyntaxError::CompilerInstructionRequiredAfterSymbol,
        )
        .into());
    }

    Ok(())
}

pub fn parse_enum(
    _ctx: &mut Context,
    _vis: &ItemVisibility,
    _tokens: &mut TokenStream<Spanned<Token>>,
    _compiler_instructions: OrdSet<CompilerInstruction>,
)
{
}

pub fn parse_struct(
    _ctx: &mut Context,
    _vis: &ItemVisibility,
    _tokens: &mut TokenStream<Spanned<Token>>,
    _compiler_instructions: OrdSet<CompilerInstruction>,
)
{
}
