use crate::{
    error::{Spanned, parser::ParserError, syntax::SyntaxError},
    parser::{
        common::{Context, ItemVisibility, Streamable, TokenStream},
        function::CompilerInstruction,
    },
    tokenizer::{self, Token, TokenDiscriminants},
    ty::{OrdSet, Type},
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
    _ctx: &mut Context,
    _vis: &ItemVisibility,
    _tokens: &mut TokenStream<Spanned<Token>>,
    _compiler_instructions: OrdSet<CompilerInstruction>,
)
{
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
