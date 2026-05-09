use crate::{
    error::{Spanned, parser::ParserError, syntax::SyntaxError},
    parser::{
        common::{StatementVariant, Streamable, child_iterator_until},
        dbg::combine_span_info,
        numeric_value::parse_numeric_literal,
        statements::{
            conditionals::{conditional_else, conditional_elseif, conditional_if},
            loops::{loop_for, loop_while},
            variables::{mod_variable, var_decl},
        },
    },
    tokenizer::{Token, TokenDiscriminants},
};

#[derive(Debug, Clone, Copy)]
pub enum Expr
{
    // FunctionCall,
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
    // (
    //     &[&[
    //         TokenDiscriminants::Identifier,
    //         TokenDiscriminants::OpenParentheses,
    //     ]],
    //     Ok(Expr::FunctionCall),
    // ),
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
            // const <ty> <name> "=" <val>
            &[
                TokenDiscriminants::Const,
                TokenDiscriminants::TypeDefinition,
                TokenDiscriminants::Identifier,
                TokenDiscriminants::SetValue,
            ],
            // const <ident (for custom types)> <name> "=" <val>
            &[
                TokenDiscriminants::Const,
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
fn match_expr_pattern<'a, S: Streamable<Spanned<Token>>>(
    tkns: &mut S,
) -> Option<&'a Result<Expr, SyntaxError>>
{
    

    EXPR_PAT
        .iter()
        .find(|(patterns, _)| patterns.iter().any(|pat| tkns.try_match_pattern(pat)))
        .map(|(_, expr_res)| expr_res)
}

pub fn parse_statement<S: Streamable<Spanned<Token>>>(
    tkns: &mut S,
) -> anyhow::Result<Spanned<StatementVariant>>
{
    // Try matching with the pre-defined expression patterns
    // If the pattern starts with a variable reference or and identifier which is not a function we will parse that manually.
    if let Some(matched) = match_expr_pattern(tkns).cloned() {
        let expr = matched?;

        // These are complete statements that do not create a new value. These statements introduce loop and logic to the language, but these do not create new values.
        match expr {
            // These are complete expressions, these do not need the ';' terminator.
            Expr::If => conditional_if(tkns),
            Expr::Elseif => conditional_elseif(tkns),
            Expr::Else => conditional_else(tkns),
            Expr::While => loop_while(tkns),
            Expr::For => loop_for(tkns),

            // These expression should end at the `;` terminator since they are set size expressions.
            Expr::VariableDeclaration => {
                var_decl(&mut child_iterator_until(
                    tkns,
                    &TokenDiscriminants::SemiColon,
                    ParserError::SyntaxError(SyntaxError::MissingSemiColon),
                )?)
            },
            // Please note that this is not only for the simple ```<ident> "="``` statement but rather any expression that directly modifies the value of the variable. ("/=", "+=", ....)
            Expr::ModifyVariable => {
                mod_variable(&mut child_iterator_until(
                    tkns,
                    &TokenDiscriminants::SemiColon,
                    ParserError::SyntaxError(SyntaxError::MissingSemiColon),
                )?)
            },
        }?;

        // Return the expression matched by the fastpaths
        return Ok(/*stmt*/ todo!());
    }

    // Parse the value we might have here (most of the times this will be useless in this function, except the function call which may have a side effect on the code.)
    // Even though we separate expression based on semicolons, if the user leaves out a semicolon the code is stil going to break so we have to add eadditional checks later.
    let mut expr_tkns = child_iterator_until(
        tkns,
        &TokenDiscriminants::SemiColon,
        ParserError::SyntaxError(SyntaxError::MissingSemiColon),
    )?;

    let value = parse_value(&mut expr_tkns)?;

    Ok(value)
}

fn parse_variable_expression<S: Streamable<Spanned<Token>>>(
    tkns: &mut S,
    stmt: Spanned<StatementVariant>,
) -> anyhow::Result<Spanned<StatementVariant>>
{
    Ok(
        // Consume the next token in the stream
        match tkns.consume().cloned() {
            Some(tkn) => {
                match tkn.get_inner() {
                    // Match a function call
                    Token::OpenParentheses => {
                        todo!()
                    },
                    // Indexing
                    Token::OpenSquareBrackets => {
                        // The next few tokens should be the index referencing the position of the value in the array
                        // Capture the tokens until the closing "]"
                        let closing_pos = tkns
                            .map_next_pos({
                                let mut currently_open = 1;

                                move |tkn| {
                                    match tkn.get_inner() {
                                        Token::OpenSquareBrackets => currently_open += 1,
                                        Token::CloseSquareBrackets => currently_open -= 1,
                                        _ => {},
                                    }

                                    currently_open == 0
                                }
                            })
                            .ok_or(ParserError::SyntaxError(
                                SyntaxError::LeftOpenSquareBrackets,
                            ))?;

                        let mut index_value_tkns = tkns
                            .child_iterator_bulk(closing_pos)
                            .ok_or(ParserError::EOF)?;

                        let index_value = parse_value(&mut index_value_tkns)?;
                        
                        // Drop the child buffer explicitly
                        drop(index_value_tkns);

                        // The next token should be the closing "]", consume it for syntax purposes
                        let closing_bracket_span = *tkns
                            .try_consume_match(
                                ParserError::SyntaxError(SyntaxError::InvalidVariableExpression),
                                &TokenDiscriminants::CloseSquareBrackets,
                            )?
                            .get_span();

                        parse_variable_expression(
                            tkns,
                            Spanned {
                                inner: StatementVariant::ArrayReference {
                                    variable_reference: Box::new(stmt),
                                    index: Box::new(index_value),
                                },
                                // Combine the spans of the opening and the closing brackets so that the span will contain the whole array reference.
                                span: combine_span_info(
                                    &[*tkn.get_span(), closing_bracket_span],
                                    true,
                                ),
                            },
                        )?
                    },
                    // Struct access
                    Token::Dot => {
                        // The next item should be an identifer referencing the struct name
                        let field_name = tkns
                            .try_consume_match(
                                ParserError::SyntaxError(SyntaxError::InvalidStructFieldReference),
                                &TokenDiscriminants::Identifier,
                            )?
                            .try_as_identifier_ref()
                            .unwrap()
                            .clone();

                        // Call the function recursively to see if there are any more tokens left in the stream
                        parse_variable_expression(
                            tkns,
                            Spanned {
                                inner: StatementVariant::StructFieldReference {
                                    variable_reference: Box::new(stmt),
                                    field_name,
                                },
                                span: *tkn.get_span(),
                            },
                        )?
                    },

                    // Implement math expressions here
                    

                    _ => {
                        return Err(ParserError::SyntaxError(
                            SyntaxError::InvalidVariableExpression,
                        )
                        .into());
                    },
                }
            },
            None => stmt,
        },
    )
}

/// This function should already be receiving a slice of tokens until the next semicolon.
fn parse_value<S: Streamable<Spanned<Token>>>(
    tkns: &mut S,
) -> Result<Spanned<StatementVariant>, anyhow::Error>
{
    if let Some(tkn) = tkns.peek_next().cloned() {
        let value = match tkn.get_inner() {
            Token::Identifier(ident) => {
                // Consume the token from the stream after peeking it.
                tkns.consume();

                // Own the first identifier
                let ident = ident.to_owned();

                // Conume the next token
                // Call a recursive function here which will resolve the expression after the variable's name.
                // This function is the legacy eq of "fetch_variable_expr", parsing the statement after an identifier - such as foo[0] | foo.asd
                

                parse_variable_expression(
                    tkns,
                    Spanned {
                        inner: StatementVariant::BasicReference {
                            variable_name: ident,
                        },
                        span: *tkn.get_span(),
                    },
                )?
            },
            Token::Literal(val) => {
                // Consume the token from the stream after peeking it.
                tkns.consume();

                // Return the span with the statement
                Spanned {
                    inner: StatementVariant::Value(val.clone()),
                    span: *tkn.get_span(),
                }
            },
            // Parse the numeric value
            Token::Addition | Token::Subtraction | Token::UnparsedLiteral(_) => {
                parse_numeric_literal(tkns)?
            },
            _ => todo!(),
        };

        return Ok(value);
    }

    Err(ParserError::UnknownValueExpression.into())
}
