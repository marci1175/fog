use std::{
    collections::{HashMap, HashSet},
    fs, mem,
    path::PathBuf,
    rc::Rc,
};

use common::{
    anyhow::{self, Result},
    codegen::{CustomItem, FunctionArgumentIdentifier, If, ParsedState, StructAttributes},
    compiler::ProjectConfig,
    dashmap::DashMap,
    error::{SpanInfo, parser::ParserError, syntax::SyntaxError},
    indexmap::IndexMap,
    parser::{
        common::{ItemVisibility, ParsedToken, ParsedTokenInstance, find_closing_braces, find_closing_paren},
        dbg::fetch_and_merge_debug_information,
        function::{
            self, CompilerHint, FunctionArguments, FunctionDefinition, FunctionSignature,
            PathMap, UnparsedFunctionDefinition, parse_fn_generics,
            parse_function_call_args, parse_signature_argument_tokens,
        },
        import::parse_import_path,
        value::parse_value,
        variable::{
            ControlFlowType, UniqueId, VARIABLE_ID_SOURCE, VariableReference,
            resolve_variable_expression,
        },
    },
    strum::IntoDiscriminant,
    tokenizer::Token,
    tracing::{info, warn},
    ty::{OrdMap, OrdSet, Type, Value, ty_from_token},
};

use crate::{parser::Settings, tokenizer::tokenize};

// /// This function parses all of the functions found in the Token slice.
// /// The returned functions still need to be parsed.
// pub fn parse_functions(
//     tokens: &[Token],
//     // Enabled features for the current project
//     enabled_features: &OrdSet<String>,
//     // Module path for the current struct we are implmenting the functions for
//     module_path: Vec<String>,
//     // Custom types created by the user above this implementation body
//     custom_types: &IndexMap<String, CustomItem>,
//     // This argument basically sets whether functions are allowed to use `this` in their arguments.
//     // Functions implemented for struct can use this to reference themselves.
//     is_struct_implementation: bool,
// ) -> Result<IndexMap<String, UnparsedFunctionDefinition>, anyhow::Error>
// {
//     let mut function_list: IndexMap<String, UnparsedFunctionDefinition> = IndexMap::new();
//     let mut function_compiler_hint_buffer: OrdSet<CompilerHint> = OrdSet::new();
//     let mut function_enabling_feature: OrdSet<String> = OrdSet::new();
//     let mut token_idx = 0;

//     while token_idx < tokens.len() {
//         let current_token = tokens[token_idx].clone();

//         if current_token == Token::ItemVisibility(common::parser::common::ItemVisibility::Private)
//             || current_token == Token::ItemVisibility(common::parser::common::ItemVisibility::Public)
//             || current_token == Token::ItemVisibility(common::parser::common::ItemVisibility::PublicLibrary)
//         {
//             token_idx += 1;

//             if tokens[token_idx] == Token::Function {
//                 if let Token::Identifier(function_name) = tokens[token_idx + 1].clone() {
//                     // Try to collect the function generics specified next to the args
//                     let mut function_generics: OrdMap<String, OrdSet<Vec<String>>> = OrdMap::new();

//                     token_idx += 2;

//                     // Check if there are any generics defined
//                     if tokens[token_idx] == Token::BitOr {
//                         // Increment idx
//                         token_idx += 1;

//                         let jumped_idx = parse_fn_generics(
//                             dbg!(&tokens[token_idx..]),
//                             custom_types,
//                             &mut function_generics,
//                         )?;

//                         token_idx += jumped_idx + 1;
//                     }

//                     if tokens[token_idx] == Token::OpenParentheses {
//                         // This function also stores the generics of the function signature
//                         let (bracket_close_idx, args) = parse_signature_argument_tokens(
//                             &tokens[token_idx + 1..],
//                             custom_types,
//                             is_struct_implementation,
//                             function_generics,
//                         )?;

//                         token_idx += bracket_close_idx + 1;

//                         // Fetch the returned type of the function
//                         if tokens[token_idx + 1] == Token::Colon
//                             || tokens[token_idx + 1] == Token::Returns
//                         {
//                             let return_type = ty_from_token(&tokens[token_idx + 2], custom_types)?;

//                             if tokens[token_idx + 3] == Token::OpenBraces {
//                                 // Create a variable which stores the level of braces we are in
//                                 let mut brace_layer_counter = 1;

//                                 // Get the slice of the list which may contain the braces' scope
//                                 let tokens_slice = &tokens[token_idx + 4..];

//                                 // Create an index which indexes the tokens slice
//                                 let mut token_braces_idx = 0;

//                                 // Create a list which contains all the tokens inside the two braces
//                                 let mut braces_contains: Vec<Token> = vec![];

//                                 // Find the scope of this function
//                                 loop {
//                                     // We have itered through the whole function and its still not found, it may be an open brace.
//                                     if tokens_slice.len() == token_braces_idx {
//                                         return Err(ParserError::SyntaxError(
//                                             SyntaxError::LeftOpenParentheses,
//                                         )
//                                         .into());
//                                     }

//                                     // If a bracket is closed the layer counter should be incremented
//                                     if tokens_slice[token_braces_idx] == Token::OpenBraces {
//                                         brace_layer_counter += 1;
//                                     }
//                                     // If a bracket is closed the layer counter should be decreased
//                                     else if tokens_slice[token_braces_idx] == Token::CloseBraces {
//                                         brace_layer_counter -= 1;
//                                     }

//                                     // If we have arrived at the end of the braces this is when we know that this is the end of the function's scope
//                                     if brace_layer_counter == 0 {
//                                         break;
//                                     }

//                                     // Store the current item in the token buffer
//                                     braces_contains.push(tokens_slice[token_braces_idx].clone());

//                                     // Increment the index
//                                     token_braces_idx += 1;
//                                 }

//                                 let braces_contains_len = braces_contains.len();

//                                 // Extract the compiler hints for the function
//                                 let compiler_hints: OrdSet<function::CompilerHint> =
//                                     mem::take(&mut function_compiler_hint_buffer);

//                                 let function_enabling_features =
//                                     mem::take(&mut function_enabling_feature);

//                                 if !function_enabling_features.is_disjoint(enabled_features)
//                                     || function_enabling_features.is_empty()
//                                 {
//                                     // Store the function
//                                     let insertion = function_list.insert(
//                                         function_name.clone(),
//                                         UnparsedFunctionDefinition {
//                                             inner: braces_contains.clone(),
//                                             token_offset: token_idx + 4,
//                                             signature: FunctionSignature {
//                                                 name: function_name.clone(),
//                                                 args: args.clone(),
//                                                 return_type: return_type.clone(),
//                                                 // To be honest I dont really think this matters what we set it, since im not planning to make a disctinction between public and private functions
//                                                 // For now ;)
//                                                 visibility: current_token.try_as_item_visibility().ok_or(ParserError::InvalidSignatureDefinition)?,
//                                                 module_path: module_path.clone(),
//                                                 compiler_hints: compiler_hints.clone(),
//                                                 enabling_features: function_enabling_features
//                                                     .clone(),
//                                             },
//                                         },
//                                     );

//                                     // If a function with a similar name exists throw an error as there is no function overloading an excpetion is when they are covered under different features
//                                     if let Some(overwritten_function) = insertion {
//                                         return Err(ParserError::SyntaxError(
//                                             SyntaxError::DuplicateFunctions(
//                                                 function_name,
//                                                 overwritten_function.signature,
//                                             ),
//                                         )
//                                         .into());
//                                     }
//                                 }

//                                 // Set the iterator index
//                                 token_idx += braces_contains_len + 5;

//                                 // Countinue with the loop
//                                 continue;
//                             }
//                         }

//                         return Err(ParserError::InvalidSignatureDefinition.into());
//                     }
//                     else {
//                         return Err(ParserError::InvalidSignatureDefinition.into());
//                     }
//                 }
//                 else {
//                     return Err(ParserError::SyntaxError(SyntaxError::InvalidFunctionName).into());
//                 }
//             }
//         }
//         else if current_token == Token::Function {
//             return Err(ParserError::ItemRequiresExplicitVisibility.into());
//         }
//         else {
//             return Err(ParserError::InvalidImplItem.into());
//         }

//         token_idx += 1;
//     }

//     Ok(function_list)
// }

impl Settings
{
    pub fn parse_functions(
        &self,
        unparsed_functions: &mut PathMap<Vec<String>, String, UnparsedFunctionDefinition>,
        function_imports: Rc<HashMap<String, FunctionSignature>>,
        custom_items: &mut IndexMap<String, CustomItem>,
    ) -> Result<IndexMap<String, FunctionDefinition>>
    {
        return Ok(IndexMap::new());
    }

    pub fn parse_function_block(
        &self,
        // tokens: Vec<Token>,
        // function_token_offset: usize,
        // unparsed_functions: &mut PathMap<Vec<String>, String, UnparsedFunctionDefinition>,
        // parsed_functions: &mut IndexMap<String, FunctionDefinition>,
        // this_function_signature: FunctionSignature,
        // function_imports: Rc<HashMap<String, FunctionSignature>>,
        // custom_items: Rc<IndexMap<String, CustomItem>>,
        // this_fn_args: FunctionArguments,
        // additional_variables: OrdMap<String, (Type, UniqueId)>,
        // receiver_type: Option<(Type, usize)>,
    ) -> Result<Vec<ParsedTokenInstance>>
    {
        return Ok(vec![]);
    }
}

/// This function is only used to parse function signatures for imports.
pub fn parse_function_signature(
    tokens: &[Token],
    token_idx: &mut usize,
    custom_types: &IndexMap<String, CustomItem>,
    module_path: Vec<String>,
    function_name: String,
    is_struct_implementation: bool,
    function_generics: OrdMap<String, OrdSet<Vec<String>>>,
) -> anyhow::Result<FunctionSignature>
{
    let (bracket_close_idx, args) = parse_signature_argument_tokens(
        &tokens[*token_idx..],
        custom_types,
        is_struct_implementation,
        function_generics,
    )?;

    *token_idx += bracket_close_idx;

    if tokens[*token_idx + 1] == Token::Colon {
        // Check for SemiColon for shits and giggles
        if tokens[*token_idx + 3] != Token::SemiColon {
            return Err(ParserError::SyntaxError(SyntaxError::MissingSemiColon).into());
        }

        // Get return type for function
        let return_ty = ty_from_token(&tokens[*token_idx + 2], custom_types)?;

        // Increment idx
        *token_idx += 3;

        Ok(FunctionSignature {
            name: function_name,
            args,
            return_type: return_ty,
            module_path,
            // Imported functions can only be accessed at the source file they were imported at
            // I might change this later to smth like pub import similar to pub mod in rust
            visibility: ItemVisibility::Private,
            compiler_hints: OrdSet::new(),
            enabling_features: OrdSet::new(),
        })
    }
    else {
        Err(SyntaxError::FunctionSignatureReturnTypeRequired.into())
    }
}

// This is a blanket function will need to expand it if i want primitives to implement traits
pub fn get_type_traits(ty: &Type) -> &OrdSet<Vec<String>>
{
    match ty {
        Type::I64 => todo!(),
        Type::F64 => todo!(),
        Type::U64 => todo!(),
        Type::I32 => todo!(),
        Type::F32 => todo!(),
        Type::U32 => todo!(),
        Type::I16 => todo!(),
        Type::F16 => todo!(),
        Type::U16 => todo!(),
        Type::U8 => todo!(),
        Type::String => todo!(),
        Type::Boolean => todo!(),
        Type::Void => todo!(),
        Type::Enum(_) => todo!(),
        Type::Struct((_name, _fields, attributes)) => &attributes.traits_implemented,
        Type::Array(_) => todo!(),
        Type::Pointer(_token) => todo!(),
        Type::Trait {
            name: _,
            access_path: _,
            functions: _,
        } => todo!(),
        Type::TraitObject(_ord_set) => todo!(),
    }
}
