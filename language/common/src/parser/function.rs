use std::{fmt::Display, hash::Hash};

use indexmap::IndexSet;
use strum::{EnumDiscriminants, EnumTryAs};

use crate::{
    anyhow::{self},
    codegen::ty::{OrdMap, OrdSet, Type},
    error::{Spanned, parser::ParserError, syntax::SyntaxError},
    parser::{
        common::{
            Context, ItemVisibility, StatementVariant, Stream, Streamable, find_closing_braces,
        },
        numeric_value::MathematicalSymbol,
        statement::parse_statement,
        ty::{create_ty_token, parse_type},
        variable::{UniqueId, VARIABLE_ID_SOURCE},
    },
    tokenizer::{Token, TokenDiscriminants},
};

#[derive(Clone, Debug, Default, PartialEq, Hash)]
pub struct UnparsedFunctionDefinition
{
    pub signature: FunctionSignature,
    pub inner: Vec<Token>,

    /// This is used to offset the index when fetching [`DebugInformation`] about [`ParsedToken`]s inside the function.
    pub token_offset: usize,
}

#[derive(Debug, Clone, PartialEq, Hash, Default, Eq)]
pub struct FunctionDefinition
{
    /// Raw signature of the function.
    pub signature: FunctionSignature,

    /// The actual body of the function.
    pub body: Vec<Spanned<StatementVariant>>,

    /// The visibility of this function in the given [`Context`] (scope).
    pub visibility: ItemVisibility,

    /// Compiler instructions for this specific function.
    pub compiler_instructions: OrdSet<CompilerInstruction>,

    /// Features required to be enabled for this function.
    pub enabling_features: OrdSet<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct FunctionSignature
{
    /// Name of the function.
    pub name: String,
    /// Required arguments of the function.
    pub args: FunctionArguments,
    /// Return type of the function.
    pub return_type: Type,
}

impl Display for FunctionSignature
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_str(&format!("[Function Signature]:\n{:#?}", self))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct FunctionArguments
{
    /// Even though [`UniqueId`]s are truly unique, we still dont want to use them (for now) as a key because strings are unique in this context.
    pub arguments: OrdMap<String, (Type, UniqueId)>,
    /// Made for backwards compatibility with FFI functions. (`...`)
    pub ellipsis_present: bool,
    /// The map consists of the generic types and their traits.
    /// ie: { "T": {"trait1", "trait2"} }
    /// Functions containing generics are generated when a function call is made to the specific function.
    /// Basically, when a function with generics is called, it will check the arguments and if the arguments have the traits needed it will create a new function instance which will have the concrete types as its arguments.
    pub generics: OrdMap<String, OrdSet<String>>,
    /// If the function is being implemented for a valid type, this field will contain the type we are implementing for.
    pub referenced_receiver: Option<Type>,
}

impl FunctionArguments
{
    /// We need to implement a Custom eq check on [`FunctionArguments`]s because the use of the [`UniqueId`] they both contain.
    /// If the argument's order and name match this returns `true` otherwise `false`.
    pub fn check_arg_eq(&self, rhs: &Self) -> bool
    {
        self.arguments
            .iter()
            .map(|(name, (ty, _))| (name, ty))
            .collect::<Vec<_>>()
            == rhs
                .arguments
                .iter()
                .map(|(name, (ty, _))| (name, ty))
                .collect::<Vec<_>>()
    }
}

impl FunctionArguments
{
    pub fn new() -> Self
    {
        Self {
            arguments: OrdMap::new(),
            generics: OrdMap::new(),
            ellipsis_present: false,
            referenced_receiver: None,
        }
    }
}

#[derive(
    Debug, Clone, PartialEq, strum_macros::Display, Eq, Hash, EnumDiscriminants, EnumTryAs,
)]
#[strum_discriminants(derive(Hash))]
pub enum CompilerInstruction
{
    /// See llvm function attributes
    Cold,
    NoFree,
    Inline,
    NoUnWind,

    /// Feature flag to only enable compilation of the function if a certain function is enabled
    Feature(String),
}

impl From<CompilerInstructionDiscriminants> for CompilerInstruction
{
    fn from(val: CompilerInstructionDiscriminants) -> Self
    {
        match val {
            CompilerInstructionDiscriminants::Cold => CompilerInstruction::Cold,
            CompilerInstructionDiscriminants::NoFree => CompilerInstruction::NoFree,
            CompilerInstructionDiscriminants::Inline => CompilerInstruction::Inline,
            CompilerInstructionDiscriminants::NoUnWind => CompilerInstruction::NoUnWind,
            CompilerInstructionDiscriminants::Feature => {
                CompilerInstruction::Feature(String::new())
            },
        }
    }
}

/// The function parses the entire function, but does not validate the function's body.
/// Syntax of a function:
/// ```
/// <vis> "function" <name> ["|" {<generic>: <trait> [{+ <trait>]}} "|"] "(" [{<arg>: <type>}] ")" ":" <return type> "{" [{<expr>}] "}"
/// ```
pub fn parse_function(
    ctx: &Context,
    vis: &ItemVisibility,
    tokens: &mut Stream<Spanned<Token>>,
    mut compiler_instructions: OrdSet<CompilerInstruction>,
    implementing_for_type: Option<Type>,
) -> anyhow::Result<FunctionDefinition>
{
    // Get the function name token
    let function_name_tkn = tokens.try_consume_match(
        ParserError::SyntaxError(SyntaxError::InvalidFunctionName),
        &TokenDiscriminants::Identifier,
    )?;

    // Parse function name, its safe to unwrap here
    let function_name = function_name_tkn
        .try_as_identifier_ref()
        .unwrap()
        .to_owned();

    // This will hold the function's arguments. This variable will get modified later.
    let mut arguments = FunctionArguments::new();

    //Parse the arguments of the function
    // If the first token is a '|' that means the function has generics defined
    // If the first token is a '(' that means that its just a normal function
    if let Some(tkn) = tokens.consume() {
        match tkn.get_inner() {
            // Parse generics before arguments
            Token::BitOr => {
                // Fetch the generics of the function
                arguments.generics = parse_generics(tokens)?;

                // The next token should be a "(" due to the syntax.
                tokens.try_consume_match(
                    ParserError::InvalidFunctionSignatureDefinition,
                    &TokenDiscriminants::OpenParentheses,
                )?;

                // Parse the arguments of the function
                parse_function_signature(tokens, &mut arguments, implementing_for_type)?;
            },
            // Parse arguments
            Token::OpenParentheses => {
                parse_function_signature(tokens, &mut arguments, implementing_for_type)?
            },
            _ => return Err(ParserError::InvalidFunctionArgumentDefinition.into()),
        }
    }

    // This should be the ":" character singaling the return type
    tokens.try_consume_match(
        ParserError::SyntaxError(SyntaxError::FunctionRequiresReturn),
        &TokenDiscriminants::Colon,
    )?;

    // Parse the return type of the function
    let return_type = parse_type(tokens)?;

    // The TokenStream should now point to `Token::OpenBraces`
    tokens.try_consume_match(
        ParserError::SyntaxError(SyntaxError::InvalidFunctionBodyStart),
        &TokenDiscriminants::OpenBraces,
    )?;

    // Fetch the function body and increment the tokenstream accordingly.
    let fn_body = parse_body(tokens)?;

    // This should never return an error since we are already checking the closing brace when fetching the fn body.
    tokens.try_consume_match(
        ParserError::SyntaxError(SyntaxError::LeftOpenBraces),
        &TokenDiscriminants::CloseBraces,
    )?;

    // Extract those compiler instructions which are a feature requirement for the function
    // Store those in a different place so that its easier to access.
    let enabling_features = compiler_instructions
        .extract_if(.., |instr| {
            // Extract the compiler instruction if its a required feature.
            matches!(instr, CompilerInstruction::Feature(_))
        })
        .map(|instruction| {
            instruction
                .try_as_feature()
                .expect("CompilerInstruction::Feature assert failed.")
        })
        .collect::<IndexSet<String>>();

    Ok(ctx.create_function(
        *vis,
        function_name,
        arguments,
        return_type,
        compiler_instructions,
        fn_body,
        OrdSet::wrap(enabling_features),
    ))
}

/// The function assumes the first token to be the first token in the `|`s.
/// The function does not check or evaluate anything it parses besides syntax checking.
pub fn parse_generics(
    tokens: &mut Stream<Spanned<Token>>,
) -> anyhow::Result<OrdMap<String, OrdSet<String>>>
{
    let mut generics: OrdMap<String, OrdSet<String>> = OrdMap::new();

    /*
        Syntax definition:

        {
            <generic> ":" { { <trait> ["+"] } [","] } [","]
        }
    */
    // Lets loop through all the generics
    'main_loop: while let Some(tkn) = tokens.consume() {
        match tkn.get_inner() {
            Token::Identifier(generic_name) => {
                let generic_name = generic_name.clone();

                // Store a checkpoint of the stream so we can load it back later if we need to
                let generic_name_checkpoint = tokens.create_checkpoint();

                // The next token should be a ":" due to syntax
                tokens.try_consume_match(
                    ParserError::SyntaxError(SyntaxError::InvalidFunctionGenericsDefinition),
                    &TokenDiscriminants::Colon,
                )?;

                // Create a new entry for the current generic
                generics.insert(generic_name.clone(), OrdSet::new());
                // We can safely unwrap here because the field is present in the map and return a mutable handle
                let generic_handle = generics.get_mut(&generic_name).unwrap();

                // Loop over the traits
                'trait_loop: while let Some(tkn) = tokens.consume() {
                    if let Token::Identifier(trait_name) = tkn.get_inner() {
                        // Store the trait's name which the user entered for the generic
                        // The ordset for the generic should already be present in the map.
                        generic_handle.insert(trait_name.clone());

                        // Check the next token
                        let next = tokens.consume().ok_or(ParserError::EOF)?;

                        // Match the next token
                        match next.get_inner() {
                            Token::BitOr => break 'main_loop,
                            Token::MathSym(MathematicalSymbol::Addition) => continue 'trait_loop,
                            // If we have reached the comma that means that the current trait bound has ended.
                            Token::Comma => break 'trait_loop,
                            _ => {
                                return Err(ParserError::SyntaxError(
                                    SyntaxError::InvalidFunctionGenericsDefinition,
                                )
                                .into());
                            },
                        }
                    }
                    else {
                        return Err(ParserError::SyntaxError(
                            SyntaxError::InvalidFunctionGenericsDefinition,
                        )
                        .into());
                    }
                }

                // Ensure that the trait bound is not empty (although i dont think its possible, but code may change later)
                // If this returned an error load the checkpoint back so that the error will point at the correct generic
                if generic_handle.is_empty() {
                    // Load the position of the cursor at the generic name
                    tokens.load_checkpoint(generic_name_checkpoint);
                    return Err(ParserError::GenericMustHaveAtleastOneTrait.into());
                }
            },
            // If we encounter the closing `|` break the loop
            Token::BitOr => break,

            _ => {
                return Err(ParserError::SyntaxError(
                    SyntaxError::InvalidFunctionGenericsDefinition,
                )
                .into());
            },
        }
    }

    Ok(generics)
}

/// The function assumes the first token to be the first token in the parentheses.
/// Please note that the function does not evaluate anything it parses.
pub fn parse_function_signature<S: Streamable<Spanned<Token>>>(
    tokens: &mut S,
    function_args: &mut FunctionArguments,

    // This is used when we are specifically implementing a function for a struct or an enum.
    // When the `this` keyword is referenced this argument is checked and the type we are implementing for will be stored.
    implementing_for_type: Option<Type>,
) -> anyhow::Result<()>
{
    /*
        Arguments are defined like so:
        "(" [{<arg_name> ":" <type>, }] ")"
        The function will be called after the first "(" therefor the function should start parsing from the first arguments name or the closing ")".
    */

    // Loop thorugh all the arguments
    'main_loop: while let Some(tkn) = tokens.consume() {
        // Get the name of the variable
        match tkn.get_inner() {
            Token::Identifier(arg_name) => {
                let arg_name = arg_name.clone();

                // The next token should be a ":" due to syntax
                tokens.try_consume_match(
                    ParserError::InvalidFunctionArgumentDefinition,
                    &TokenDiscriminants::Colon,
                )?;

                // The next token should be a concrete type or an identifier.
                if let Some(ty) = tokens.consume() {
                    // Get the function argument's type
                    let arg_ty = create_ty_token(ty)?;

                    // Store the argument
                    let insertion_result = function_args.arguments.insert(
                        arg_name.clone(),
                        (arg_ty, VARIABLE_ID_SOURCE.get_unique_id()),
                    );

                    // Check if there are duplicate argument names
                    if insertion_result.is_some() {
                        return Err(ParserError::DuplicateArguments(arg_name.clone()).into());
                    }

                    // Check the next token
                    // If it is a "," that means that there are more arguments or the user just left it in.
                    // If it s a ")" that shows that the all the function arguments have been parsed
                    if let Some(tkn) = tokens.consume() {
                        match tkn.get_inner() {
                            Token::Comma => continue 'main_loop,
                            Token::CloseParentheses => break 'main_loop,
                            _ => return Err(ParserError::InvalidFunctionArgumentDefinition.into()),
                        }
                    }
                }

                // If we didnt break continue or return an error that means that there werent any more tokens left in the stream therefor we can do an EOF.
                return Err(ParserError::EOF.into());
            },
            // The receiver doesnt have to be the first argument in the function.
            Token::This => {
                // Check if we are implementing this function for a type
                if implementing_for_type.is_none() {
                    return Err(ParserError::NotImplementingForAny.into());
                }

                // Check if we have already referenced `this`
                if function_args.referenced_receiver.is_some() {
                    return Err(ParserError::SyntaxError(SyntaxError::ThisRereferenced).into());
                }

                function_args.referenced_receiver = implementing_for_type.clone();

                // If the receiver is present, indicate that in the FunctionSignature instance
                // The next token should be a comma
                // Check the next token
                // If it is a "," that means that there are more arguments or the user just left it in.
                // If it s a ")" that shows that the all the function arguments have been parsed
                if let Some(tkn) = tokens.consume() {
                    match tkn.get_inner() {
                        Token::Comma => continue 'main_loop,
                        Token::CloseParentheses => break 'main_loop,
                        _ => return Err(ParserError::InvalidFunctionArgumentDefinition.into()),
                    }
                }
                // If we didnt break continue or return an error that means that there werent any more tokens left in the stream therefor we can do an EOF.
                return Err(ParserError::EOF.into());
            },
            // The usage of ellpsises would only be valid in a FFI declaration
            Token::Ellipsis => {
                function_args.ellipsis_present = true;

                // The ellipsis does have to be the last argument present unlike all the other
                if let Some(tkn) = tokens.consume() {
                    match tkn.get_inner() {
                        Token::CloseParentheses => break 'main_loop,
                        _ => return Err(ParserError::InvalidFunctionArgumentDefinition.into()),
                    }
                }
                // If we didnt break continue or return an error that means that there werent any more tokens left in the stream therefor we can do an EOF.
                return Err(ParserError::EOF.into());
            },
            Token::CloseParentheses => break 'main_loop,
            _ => return Err(ParserError::InvalidFunctionArgumentDefinition.into()),
        }
    }

    Ok(())
}

/// This function will parse the tokens in the body of the function, but it will not check the validness of the tokens themselves.
///
/// The function parses the tokens but does not evaluate them.
pub fn parse_body<S: Streamable<Spanned<Token>> + std::fmt::Debug>(
    tokens: &mut S,
) -> anyhow::Result<Vec<Spanned<StatementVariant>>>
{
    // Get the index of the closing brace token
    let body_closing_tkn = find_closing_braces(&*tokens)
        .ok_or(ParserError::SyntaxError(SyntaxError::LeftOpenBraces))?;

    // It is safe to unwrap here, since we have already checked if the closing braces would be in the TokenStream
    let mut fn_body = tokens.child_iterator_bulk(body_closing_tkn).unwrap();

    // Store the parsed tokens somewhere
    let mut parsed_tokens = Vec::new();

    // Iterate over the whole body matching "chunks" of tokens.
    // I dont want to consume the token from the token stream every iteration, since i want to match "patterns" of tokens.
    while fn_body.peek_next().is_some() {
        // The line expression closing semi colon is not consumed so it must be matched here.
        // The semicolons are consumed in the statement parser function
        let stmt = parse_statement(&mut fn_body)?;

        // Store parsed statement
        parsed_tokens.push(stmt);
    }

    Ok(parsed_tokens)
}
