use crate::{
    codegen::{StructAttributes, StructDefinition},
    error::{Spanned, parser::ParserError, syntax::SyntaxError},
    parser::{
        common::{Context, ItemVisibility, Streamable, TokenStream},
        function::{CompilerInstruction, parse_generics},
    },
    tokenizer::{self, Token, TokenDiscriminants},
    ty::{OrdMap, OrdSet, Type},
};

pub fn parse_enum(
    _ctx: &mut Context,
    _vis: &ItemVisibility,
    _tokens: &mut TokenStream<Spanned<Token>>,
    _compiler_instructions: OrdSet<CompilerInstruction>,
)
{
}

pub fn parse_struct(
    ctx: &mut Context,
    vis: &ItemVisibility,
    tokens: &mut TokenStream<Spanned<Token>>,
    compiler_instructions: OrdSet<CompilerInstruction>,
) -> anyhow::Result<StructDefinition>
{
    let mut fields: OrdMap<String, Type> = OrdMap::new();
    let mut generics: OrdMap<String, OrdSet<String>> = OrdMap::new();

    // The first token after the keyword should be the struct's name
    let struct_name_tkn = tokens.try_consume_match(
        ParserError::SyntaxError(SyntaxError::InvalidStructDefinition),
        &TokenDiscriminants::Identifier,
    )?;
    let struct_name = struct_name_tkn
        .get_inner()
        .try_as_identifier_ref()
        .unwrap()
        .to_owned();

    // The next token could be a "|" or a "{" since the struct body should start here but the user can define generics for the struct.
    if let Some(tkn) = tokens.consume() {
        match tkn.get_inner() {
            Token::BitOr => {
                // Parse the generics of the struct
                generics = parse_generics(tokens)?;

                // This token should be the `Token::OpenBraces` due to syntax.
                tokens.try_consume_match(
                    ParserError::SyntaxError(SyntaxError::InvalidStructDefinition),
                    &TokenDiscriminants::OpenBraces,
                )?;
            },
            // We have consumed the OpenBraces token we can move onto the main struct body parsing loop
            Token::OpenBraces => (),
            _ => return Err(ParserError::SyntaxError(SyntaxError::InvalidStructDefinition).into()),
        }
    }

    // Lets consume the fields inside the struct
    'main_loop: while let Some(tkn) = tokens.consume() {
        // Consume the first token, this should be the field's name on the first iteration
        // This could also be the closing braces if there are trailing commas
        match tkn.get_inner() {
            Token::Identifier(name) => {
                // Field name
                let name = name.clone();

                // The next token should be a ":" cuz of syntax
                tokens.try_consume_match(
                    ParserError::SyntaxError(SyntaxError::InvalidStructDefinition),
                    &TokenDiscriminants::Colon,
                )?;

                // After the colon the field's type should follow
                if let Some(ty) = tokens.consume() {
                    // Fetch the type of the field
                    let field_ty = create_ty_token(ty)?;

                    // Store the field of the struct
                    fields.insert(name.clone(), field_ty);

                    // Field closing token
                    if let Some(closing_tkn) = tokens.consume() {
                        match closing_tkn.get_inner() {
                            // If we have reached the end of the struct definition
                            Token::CloseBraces => break 'main_loop,
                            // If there is a trailing comma or more fields
                            Token::Comma => continue 'main_loop,
                            _ => {
                                return Err(ParserError::SyntaxError(
                                    SyntaxError::InvalidStructDefinition,
                                )
                                .into());
                            },
                        }
                    }
                }
                // If the struct is cut in half
                else {
                    return Err(ParserError::EOF.into());
                }
            },
            Token::CloseBraces => break 'main_loop,
            _ => return Err(ParserError::SyntaxError(SyntaxError::InvalidStructDefinition).into()),
        }
    }

    // Create a new struct definition and return it
    Ok(ctx.create_struct(
        vis.clone(),
        struct_name,
        fields,
        generics,
        StructAttributes::new(compiler_instructions, OrdSet::new(), OrdMap::new()),
    ))
}

pub fn parse_type(tokens: &mut TokenStream<Spanned<Token>>) -> anyhow::Result<Type>
{
    if let Some(tkn) = tokens.consume() {
        return match tkn.get_inner() {
            Token::TypeDefinition(ty) => {
                match ty {
                    tokenizer::TypeToken::String
                    | tokenizer::TypeToken::Boolean
                    | tokenizer::TypeToken::Void
                    | tokenizer::TypeToken::I64
                    | tokenizer::TypeToken::F64
                    | tokenizer::TypeToken::U64
                    | tokenizer::TypeToken::I32
                    | tokenizer::TypeToken::F32
                    | tokenizer::TypeToken::U32
                    | tokenizer::TypeToken::I16
                    | tokenizer::TypeToken::F16
                    | tokenizer::TypeToken::U16
                    | tokenizer::TypeToken::U8 => Ok((ty.to_owned()).try_into()?),

                    tokenizer::TypeToken::Array => {
                        // Array syntax
                        // "Array" "<" <type> "," <len> ">"

                        // The next token should be a "<"
                        tokens.try_consume_match(
                            ParserError::SyntaxError(SyntaxError::InvalidTypeGenericDefinition),
                            &TokenDiscriminants::OpenAngledBrackets,
                        )?;

                        // Resolve the base type of the array
                        let ty = parse_type(tokens)?;

                        // Ensure syntax correctness
                        tokens.try_consume_match(
                            ParserError::SyntaxError(SyntaxError::InvalidTypeGenericDefinition),
                            &TokenDiscriminants::Comma,
                        )?;

                        // Parse the length of the array
                        let len_val = tokens
                            .try_consume_match(
                                ParserError::SyntaxError(SyntaxError::InvalidTypeGenericDefinition),
                                &TokenDiscriminants::Literal,
                            )?
                            .try_as_literal_ref()
                            .unwrap()
                            .to_owned();

                        // Get the raw value of the array's length
                        let len = len_val
                            .try_as_u_32()
                            .ok_or(ParserError::SyntaxError(SyntaxError::InvalidArrayLenType))?;

                        // Ensure syntax correctness
                        tokens.try_consume_match(
                            ParserError::SyntaxError(SyntaxError::InvalidTypeGenericDefinition),
                            &TokenDiscriminants::CloseAngledBrackets,
                        )?;

                        Ok(Type::Array((Box::new(ty), len as usize)))
                    },
                    tokenizer::TypeToken::Pointer => {
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
                            tokens.try_consume_match(
                                ParserError::SyntaxError(SyntaxError::InvalidTypeGenericDefinition),
                                &TokenDiscriminants::CloseAngledBrackets,
                            )?;

                            Ok(Type::Pointer(Some(Box::new(ty))))
                        }
                        // We can assume that the inner type is not specified
                        else {
                            Ok(Type::Pointer(None))
                        }
                    },

                    tokenizer::TypeToken::Enum
                    | tokenizer::TypeToken::Struct
                    | tokenizer::TypeToken::Function => {
                        return Err(ParserError::InvalidType.into());
                    },
                }
            },
            Token::Identifier(ident) => Ok(Type::Unresolved(ident.to_owned())),
            _ => {
                return Err(ParserError::SyntaxError(SyntaxError::FunctionRequiresReturn).into());
            },
        };
    }

    Err(ParserError::ExpectedTypeReference.into())
}

pub fn create_ty_token(ty: &Spanned<Token>) -> Result<Type, anyhow::Error>
{
    let arg_ty = match ty.get_inner() {
        Token::Identifier(ty_name) => {
            // Store the type as unresolved, this will be resolved later at the semantic checking process
            Type::Unresolved(ty_name.clone())
        },
        Token::TypeDefinition(ty) => {
            // Turn the concrete typetoken into a type
            (ty.clone()).try_into()?
        },

        // Invalid syntax, return an error
        _ => return Err(ParserError::InvalidArgumentType.into()),
    };
    Ok(arg_ty)
}
