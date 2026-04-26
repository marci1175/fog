use crate::{
    error::{Spanned, syntax::SyntaxError},
    parser::{
        common::{StatementVariant, StreamChild, Streamable},
        statements::{
            conditionals::{conditional_else, conditional_elseif, conditional_if},
            function_call::function_call,
            loops::{loop_for, loop_while},
            variables::{mod_variable, var_decl},
        },
    },
    tokenizer::{Token, TokenDiscriminants},
};

#[derive(Debug, Clone, Copy)]
pub enum Expr
{
    FunctionCall,
    VariableDeclaration,
    ModifyVariable,
    If,
    Else,
    Elseif,
    While,
    For,
}

const fn discriminants_eq(a: TokenDiscriminants, b: TokenDiscriminants) -> bool
{
    a as u32 == b as u32
}

const fn is_prefix_of(shorter: &[TokenDiscriminants], longer: &[TokenDiscriminants]) -> bool
{
    if shorter.len() > longer.len() {
        return false;
    }
    let mut k = 0;
    while k < shorter.len() {
        if !discriminants_eq(shorter[k], longer[k]) {
            return false;
        }
        k += 1;
    }
    true
}

/// Checks if two patterns collide (one is a prefix of the other)
const fn patterns_collide(a: &[TokenDiscriminants], b: &[TokenDiscriminants]) -> bool
{
    is_prefix_of(a, b) || is_prefix_of(b, a)
}

/*
    May the lord bless this macro and all versions of the future me trying to modify it.
*/
macro_rules! expr_pat {
    ($(
        (
            &[ $( &[ $( $tok:tt )* ] ),* $(,)? ],
            $result:expr $(,)?
        )
    ),* $(,)?) => {{
        const ALL_PATTERNS: &[&[&[TokenDiscriminants]]] = &[
            $(
                {
                    const GROUP: &[&[TokenDiscriminants]] = &[ $( &[ $( $tok )* ] ),* ];
                    GROUP
                },
            )*
        ];

        const _: () = {
            let groups = ALL_PATTERNS;
            let mut i = 0;
            while i < groups.len() {
                let pats_a = groups[i];
                let mut pi = 0;
                while pi < pats_a.len() {
                    let pat_a = pats_a[pi];
                    let mut j = 0;
                    while j < groups.len() {
                        let pats_b = groups[j];
                        let mut pj = 0;
                        while pj < pats_b.len() {
                            let pat_b = pats_b[pj];

                            if !(i == j && pi == pj) && patterns_collide(pat_a, pat_b) {
                                panic!("EXPR_PAT collision: a pattern is a prefix of another");
                            }
                            pj += 1;
                        }
                        j += 1;
                    }
                    pi += 1;
                }
                i += 1;
            }
        };

        {
            const RESULT: &[(&[&[TokenDiscriminants]], Result<Expr, SyntaxError>)] = &[
                $(
                    (
                        { const GROUP: &[&[TokenDiscriminants]] = &[ $( &[ $( $tok )* ] ),* ]; GROUP },
                        $result,
                    ),
                )*
            ];
            RESULT
        }
    }};
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
pub const EXPR_PAT: &[(&[&[TokenDiscriminants]], Result<Expr, SyntaxError>)] = expr_pat!(
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
    (
        &[
            // <ident> "="
            &[TokenDiscriminants::Identifier, TokenDiscriminants::SetValue],
            // <ident> "+="
            &[
                TokenDiscriminants::Identifier,
                TokenDiscriminants::SetValueAddition,
            ],
            // <ident> "/="
            &[
                TokenDiscriminants::Identifier,
                TokenDiscriminants::SetValueDivision,
            ],
            // <ident> "%="
            &[
                TokenDiscriminants::Identifier,
                TokenDiscriminants::SetValueModulo,
            ],
            // <ident> "*="
            &[
                TokenDiscriminants::Identifier,
                TokenDiscriminants::SetValueMultiplication,
            ],
            // <ident> "-="
            &[
                TokenDiscriminants::Identifier,
                TokenDiscriminants::SetValueSubtraction,
            ],
        ],
        Ok(Expr::ModifyVariable),
    ),
);

/// Matches and returns the first match of the EXPR_PAT list from a given tokenstream.
fn match_expr_pattern<'a>(
    tkns: &'a StreamChild<'_, Spanned<Token>>,
) -> Option<&'a Result<Expr, SyntaxError>>
{
    let matched_pattern = EXPR_PAT
        .iter()
        .find(|(patterns, _)| patterns.iter().any(|pat| tkns.try_match_pattern(*pat)))
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
            // Please note that this is not only for the simple ```<ident> "="``` statement but rather any expression that directly modifies the value of the variable. ("/=", "+=", ....)
            Expr::ModifyVariable => mod_variable(tkns),
            Expr::If => conditional_if(tkns),
            Expr::Elseif => conditional_elseif(tkns),
            Expr::Else => conditional_else(tkns),
            Expr::While => loop_while(tkns),
            Expr::For => loop_for(tkns),
        }?;

        // Return the expression matched by the fastpaths
        return Ok(todo!());
    }

    // If we couldnt parse it by the fastpath try parsing the lhs of the statement
    // Saying "lhs" is kinda inaccurate cuz it implies we have a "rhs" but we dont know yet so....
    // Consume the first token
    if let Some(tkn) = tkns.consume() {
        match tkn.get_inner() {
            Token::Identifier(ident) => {},
            _ => todo!(),
        }
    }

    Ok(todo!())
}
