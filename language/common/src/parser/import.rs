use std::{collections::HashMap, path::PathBuf};

use crate::{
    error::{Spanned, parser::ParserError, syntax::SyntaxError::InvalidImportDefinition},
    imports::{FFIDeclType, ImportType},
    parser::{
        common::Streamable,
        function::{FunctionArguments, FunctionSignature, parse_function_signature},
        ty::parse_type,
    },
    tokenizer::{Token, TokenDiscriminants},
    ty,
};

/// All item imports must point to concrete items, such as a function or enum, they cannot point to a module.
/// Both raw paths and dependencies can be imported via this keyword.
/// For declaring external function token `Token::External` must be used.  
/// When a file is imported via its raw path, the modules are accessible via its file name.
///
/// ```fog
/// import "foo.f";
/// import foo::bleble;
/// import foo::bar::baz;
/// ```
///
/// "Foreign" (imported) items cannot have implementations given later.
/// When importing dependencies all dependency paths are defined from root.
/// Given that we have a dependency named `helper`.
/// ```fog
/// import helper::hello;                        
/// ```
///
/// Imported items (aswell as files) can be aliased via the `as` keyword.
/// ```fog
/// import foo::bar as "hello";
/// hello();
/// bar(); # Not found
/// ```
pub fn parse_import_statement<S: Streamable<Spanned<Token>>>(
    tkns: &mut S,
    imports: &mut HashMap<String, ImportType>,
) -> anyhow::Result<()>
{
    // Peek the next token
    // The two accepted paths right now would be a string literal or an identifier.
    let peek_next = tkns.consume();

    if let Some(next) = peek_next {
        let next_token = next.get_inner();

        let (identifier, import): (String, ImportType) = match next_token {
            Token::Literal(ty::Value::String(path)) => {
                let path = PathBuf::from(path.clone());

                // Get the file name of the imported file
                let file_name = path
                    .file_prefix()
                    .map(|str| str.to_string_lossy().to_string())
                    .ok_or(ParserError::InvalidImportPath)?;

                (file_name, ImportType::Path(path))
            },
            Token::Identifier(ident) => {
                // Stores the elements of the import chain
                let mut path_chain: Vec<String> = vec![];

                // Store the very first chain item
                path_chain.push(ident.clone());

                // Create a loop which stores all the remaining identifiers but stops either at the end of the stream or at the `as` keyword
                while tkns.peek_next() != None
                    && !matches!(
                        tkns.peek_next(),
                        Some(Spanned {
                            inner: Token::As,
                            span: _
                        })
                    )
                {
                    // The next token should be a double colon
                    tkns.try_consume_match(
                        ParserError::SyntaxError(
                            crate::error::syntax::SyntaxError::InvalidImportDefinition,
                        ),
                        &TokenDiscriminants::DoubleColon,
                    )?;

                    // The next token should be an identifier
                    let path_item = tkns.try_consume_match(
                        ParserError::SyntaxError(
                            crate::error::syntax::SyntaxError::InvalidImportDefinition,
                        ),
                        &TokenDiscriminants::Identifier,
                    )?;

                    // Safe to unwrap due to the check above
                    path_chain.push(
                        path_item
                            .get_inner()
                            .try_as_identifier_ref()
                            .unwrap()
                            .clone(),
                    );
                }

                // Get the last item of the chain, as that will be the ident that is actually referenced in the code later on
                let item_name = path_chain.last().ok_or(ParserError::InvalidImportPath)?;

                (item_name.clone(), ImportType::Dependency(path_chain))
            },
            _ => return Err(ParserError::SyntaxError(InvalidImportDefinition).into()),
        };

        // If the import is aliased store it as aliased
        // Whether an import is aliased depends whether there is an `as` keyword after the import.
        let (stored_ident, store_result) = if matches!(
            tkns.peek_next(),
            Some(Spanned {
                inner: Token::As,
                span: _
            })
        ) || tkns.peek_next().is_some()
        {
            // Consume the as keyword
            tkns.consume();

            // The next token should be an identifier
            let alias = tkns
                .try_consume_match(
                    ParserError::InvalidImportAlias,
                    &TokenDiscriminants::Identifier,
                )?
                .get_inner()
                .try_as_identifier_ref()
                .ok_or(ParserError::InvalidImportAlias)?
                .clone();

            // Check if there are more tokens left in this import, if yes that means that the import syntax is invalid
            if tkns.peek_next().is_some() {
                return Err(ParserError::InvalidImportAlias.into());
            }

            // Store aliased import
            (alias.clone(), imports.insert(alias.clone(), import))
        }
        else {
            (
                identifier.clone(),
                imports.insert(identifier.clone(), import),
            )
        };

        // Check if there is a name collision in the imports
        if store_result.is_some() {
            return Err(ParserError::ImportNameCollision(stored_ident).into());
        }
    }
    else {
        return Err(ParserError::EOF.into());
    }

    Ok(())
}

pub fn parse_external_decl<S: Streamable<Spanned<Token>> + std::fmt::Debug>(
    tkns: &mut S,
    external_decls: &mut HashMap<String, FFIDeclType>,
) -> anyhow::Result<()>
{
    // The first token should be the type of the item (This can either be a static or a function)
    let decl_type = tkns.consume();

    if let Some(decl_type) = decl_type {
        let tkn = decl_type.get_inner();

        // Match the valid item variants
        if let Token::TypeDefinition(crate::tokenizer::TypeToken::Function) = tkn {
            // Next token is the name of the item
            let name = tkns
                .try_consume_match(
                    ParserError::ItemNameExpected,
                    &TokenDiscriminants::Identifier,
                )?
                .try_as_identifier_ref()
                .unwrap()
                .clone();

            // Consume the first opening parentheses
            tkns.try_consume_match(
                ParserError::InvalidFunctionSignatureDefinition,
                &TokenDiscriminants::OpenParentheses,
            )?;

            let mut args = FunctionArguments::new();
            dbg!(&tkns);
            // This function consumes the token until the closing parentheses
            parse_function_signature(tkns, &mut args)?;

            // All functions must have a return type
            // Consume colon for syntax
            tkns.try_consume_match(
                ParserError::FunctionReturnTypeRequired,
                &TokenDiscriminants::Colon,
            )?;

            // Consume the type this function returns
            let return_type = parse_type(tkns)?;

            external_decls.insert(
                name.clone(),
                FFIDeclType::Function(FunctionSignature {
                    name,
                    args,
                    return_type,
                }),
            );
        }
        else if let Token::Static = tkn {
            // Next token is the type of the static
            let ty = parse_type(tkns)?;

            // Next token is the name of the item
            let name = tkns
                .try_consume_match(
                    ParserError::ItemNameExpected,
                    &TokenDiscriminants::Identifier,
                )?
                .try_as_identifier_ref()
                .unwrap()
                .clone();

            external_decls.insert(name, FFIDeclType::Static(ty));
        }
        else {
            return Err(ParserError::InvalidFFIDecl.into());
        }
    }
    else {
        return Err(ParserError::EOF.into());
    }

    Ok(())
}
