use crate::{
    error::{Spanned, syntax::SyntaxError},
    parser::{
        common::{StatementVariant, StreamChild, Streamable},
        statements::{
            conditionals::{conditional_else, conditional_elseif, conditional_if},
            function_call::function_call,
            loops::{loop_for, loop_while},
            variable_declaration::var_decl,
        },
    },
    tokenizer::{Token, TokenDiscriminants},
};

#[derive(Debug, Clone, Copy)]
pub enum Expr
{
    FunctionCall,
    VariableDeclaration,
    If,
    Else,
    Elseif,
    While,
    For,
}

/// A map of all of the valid and invalid expression patterns.
/// This should serve as a "fastpath" for expressions so that the main expressions can be easily updated later.
/// This way code is easier to maintain and update.
///
/// **********************
/// IMPORTANT:
///     No patterns should partially contain one another, since that will cause the parser to take the first match's path.
/// **********************
///
pub const EXPR_PAT: &[(&[&[TokenDiscriminants]], Result<Expr, SyntaxError>)] = &[
    // Function calls should look more or less like this.
    // <name> "(" [{<args>}] ")"
    (
        &[&[
            TokenDiscriminants::Identifier,
            TokenDiscriminants::OpenParentheses,
        ]],
        Ok(Expr::FunctionCall),
    ),
    // Varaible declarations should look like this:
    //
    // <ty> <name> "=" <val>
    // <ident (for custom types)> <name> "=" <val>
    //
    (
        &[
            // <ty> <name> "=" <val>
            &[
                TokenDiscriminants::TypeDefinition,
                TokenDiscriminants::Identifier,
                TokenDiscriminants::SetValue,
            ],
            // <ident (for custom types)> <name> "=" <val>
            &[
                TokenDiscriminants::Identifier,
                TokenDiscriminants::Identifier,
                TokenDiscriminants::SetValue,
            ],
        ],
        Ok(Expr::VariableDeclaration),
    ),
    // Lets return an error for a common pattern:
    // All function must be initialized with a value before creating them.
    // Null variables are invalid.
    (
        &[
            // <ty> <name> ";"
            &[
                TokenDiscriminants::TypeDefinition,
                TokenDiscriminants::Identifier,
                TokenDiscriminants::SemiColon,
            ],
            // <ident (for custom types)> <name> ";"
            &[
                TokenDiscriminants::Identifier,
                TokenDiscriminants::Identifier,
                TokenDiscriminants::SemiColon,
            ],
        ],
        Err(SyntaxError::VariableRequiresInitialization),
    ),
    (
        &[&[TokenDiscriminants::If, TokenDiscriminants::OpenParentheses]],
        Ok(Expr::If),
    ),
    (
        &[&[
            TokenDiscriminants::ElseIf,
            TokenDiscriminants::OpenParentheses,
        ]],
        Ok(Expr::Elseif),
    ),
    (
        &[&[TokenDiscriminants::Else, TokenDiscriminants::OpenBraces]],
        Ok(Expr::Else),
    ),
    (
        &[&[
            TokenDiscriminants::For,
            TokenDiscriminants::Identifier,
            TokenDiscriminants::In,
        ]],
        Ok(Expr::For),
    ),
    (
        &[&[
            TokenDiscriminants::While,
            TokenDiscriminants::OpenParentheses,
        ]],
        Ok(Expr::While),
    ),
];

/// Matches and returns the first match of the EXPR_PAT list from a given tokenstream.
fn match_expr_pattern<'a>(
    tkns: &'a StreamChild<'_, Spanned<Token>>,
) -> Option<&'a Result<Expr, SyntaxError>>
{
    let matched_pattern = EXPR_PAT
        .iter()
        .find(|(patterns, expr_res)| patterns.iter().any(|pat| tkns.try_match_pattern(*pat)))
        .map(|(_, expr_res)| expr_res);

    matched_pattern
}

pub fn parse_statement(
    tkns: &mut StreamChild<'_, Spanned<Token>>,
) -> anyhow::Result<StatementVariant>
{
    // Try matching with the pre-defined expression patterns
    // If the pattern starts with a variable reference or and identifier which is not a function we will parse that manually.
    if let Some(matched) = match_expr_pattern(tkns).cloned() {
        let expr = matched?;

        let stmt = match expr {
            Expr::FunctionCall => function_call(tkns),
            Expr::VariableDeclaration => var_decl(tkns),
            Expr::If => conditional_if(tkns),
            Expr::Elseif => conditional_elseif(tkns),
            Expr::Else => conditional_else(tkns),
            Expr::While => loop_while(tkns),
            Expr::For => loop_for(tkns),
        }?;

        return Ok(todo!())
    }

    Ok(todo!())
}
