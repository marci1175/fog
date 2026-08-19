use crate::{
    error::{Spanned, parser::ParserError, syntax::SyntaxError},
    parser::{
        common::{StatementVariant, Streamable, child_iterator_until},
        dbg::combine_span_info,
        numeric_value::{MathematicalSymbol, parse_numeric_literal},
        statements::{
            conditionals::{conditional_else, conditional_elseif, conditional_if},
            loops::{loop_for, loop_while},
            variables::{mod_variable, var_decl},
        },
    },
    tokenizer::{Token, TokenDiscriminants},
    ty::OrdMap,
};

#[derive(Debug, Clone, Copy)]
pub enum Expr
{
    // FunctionCall,
    VariableDeclaration,
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
    let value = 
    // Try matching with the pre-defined expression patterns
    // If the pattern starts with a variable reference or and identifier which is not a function we will parse that manually.
    if let Some(matched) = match_expr_pattern(tkns).cloned() {
        // The reason the matched expression could be an error is because common syntactical mistakes are also recognized, thus these can be returned through a fastpath
        let expr = matched?;

        // These are complete statements that do not create a new value. These statements introduce loop and logic to the language, but these do not create new values.
        let stmt = match expr {
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

            // TODO: This should not be fixed as it would mean that id have to double the implementation of the variable parsing.
            // (Just move it further down)
            // Please note that this is not only for the simple ```<ident> "="``` statement but rather any expression that directly modifies the value of the variable. ("/=", "+=", ....)
            // This path does not accept more complex uses of mathmeatical expressions, such as `foo() - 3 + var2`.
            // Expr::ModifyVariable => {
            //     mod_variable(&mut child_iterator_until(
            //         tkns,
            //         &TokenDiscriminants::SemiColon,
            //         ParserError::SyntaxError(SyntaxError::MissingSemiColon),
            //     )?)
            // },
        }?;

        // Return the expression matched by the fastpaths
        stmt
    }
    else {
        // Parse the value we might have here (most of the times this will be useless in this function, except the function call which may have a side effect on the code.)
        // Even though we separate expression based on semicolons, if the user leaves out a semicolon the code is stil going to break so we have to add eadditional checks later.
        let mut expr_tkns = child_iterator_until(
            tkns,
            &TokenDiscriminants::SemiColon,
            ParserError::SyntaxError(SyntaxError::MissingSemiColon),
        )?;
    
        parse_value(&mut expr_tkns)?
    };

    Ok(value)
}

pub fn parse_variable_expression<S: Streamable<Spanned<Token>> + std::fmt::Debug>(
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
                        // After the opening parentheses if something other than the closing parentheses is following it means that the function has arguments
                        // We should consume the token straight away since if the function has an argument we wouldnt be able to provide athe whole token buffer for the arguments
                        let next_tkn = tkns.peek_next().cloned();

                        // The function has no arguments
                        if let Some(Spanned {
                            inner: Token::CloseParentheses,
                            span,
                        }) = next_tkn
                        {
                            // Consume token here since we are sure what this is
                            tkns.consume();

                            parse_variable_expression(
                                tkns,
                                Spanned {
                                    inner: StatementVariant::FunctionCall {
                                        identifier: Box::new(stmt),
                                        arguments: OrdMap::new(),
                                    },
                                    span: combine_span_info(&[*tkn.get_span(), span], true),
                                },
                            )?
                        }
                        // Function has arguments
                        else {
                            // Select everything until the closing brace
                            let closing_paren_pos = tkns
                                .map_next_pos({
                                    let mut currently_open = 1;

                                    move |tkn| {
                                        match tkn.get_inner() {
                                            Token::OpenParentheses => currently_open += 1,
                                            Token::CloseParentheses => currently_open -= 1,
                                            _ => {},
                                        }

                                        currently_open == 0
                                    }
                                })
                                .ok_or(ParserError::SyntaxError(
                                    SyntaxError::LeftOpenParentheses,
                                ))?;

                            // Tokens for the arguments inside the function call
                            let mut argument_tkns = tkns
                                .child_iterator_bulk(closing_paren_pos)
                                .ok_or(ParserError::EOF)?;

                            // Create a way to store the parsed arguments
                            let mut arguments = OrdMap::new();

                            // Create a variable for tracking the indexed arguments' index.
                            let mut argument_idx: usize = 0;

                            // Parse the values in the arguments buffer until the buffer is exhausted.
                            while argument_tkns.peek_next().is_some() {
                                // Try peeking the next two tokens to see if it is a named argument
                                let next_tokens = argument_tkns.peek_bulk(2);

                                // Check if the current arguments is a named argument
                                // Pattern match the two next tokens
                                if matches!(
                                    next_tokens,
                                    Some([
                                        Spanned {
                                            inner: Token::Identifier(_),
                                            span: _
                                        },
                                        Spanned {
                                            inner: Token::Equal,
                                            span: _
                                        }
                                    ])
                                ) {
                                    // We can safely assume that the next token is an identifier due to the check above
                                    let named_arg = argument_tkns
                                        .consume()
                                        .unwrap()
                                        .try_as_identifier_ref()
                                        .unwrap()
                                        .clone();

                                    // Consume the equals sign
                                    argument_tkns.consume();

                                    arguments.insert(
                                        crate::codegen::FunctionArgumentIdentifier::Identifier(
                                            named_arg,
                                        ),
                                        parse_value(&mut argument_tkns)?,
                                    );
                                }
                                else {
                                    arguments.insert(
                                        crate::codegen::FunctionArgumentIdentifier::Index(
                                            argument_idx,
                                        ),
                                        parse_value(&mut argument_tkns)?,
                                    );

                                    // Increment argument index, but only after other indexed arguments
                                    argument_idx += 1;
                                }
                            }

                            // Explicitly drop child iterator
                            drop(argument_tkns);

                            // The next token should be the function call's closing parentheses. Fetch the span of this token and create a span for the whole function call.
                            let closing_paren_span = *tkns
                                .try_consume_match(
                                    ParserError::SyntaxError(SyntaxError::LeftOpenParentheses),
                                    &TokenDiscriminants::CloseParentheses,
                                )?
                                .get_span();

                            parse_variable_expression(
                                tkns,
                                Spanned {
                                    inner: StatementVariant::FunctionCall {
                                        identifier: Box::new(stmt),
                                        arguments,
                                    },
                                    span: combine_span_info(
                                        &[*tkn.get_span(), closing_paren_span],
                                        true,
                                    ),
                                },
                            )?
                        }
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
                    Token::MathSym(sym) => {
                        // THe right hand side of the mathematical operation
                        let rhs = parse_value(tkns)?;

                        parse_variable_expression(
                            tkns,
                            Spanned {
                                inner: StatementVariant::MathematicalExpression(
                                    Box::new(stmt),
                                    *sym,
                                    Box::new(rhs),
                                ),
                                span: combine_span_info(
                                    &[
                                        *tkn.get_span(),
                                        *tkns
                                            .get_last_consumed()
                                            .ok_or(ParserError::EOF)?
                                            .get_span(),
                                    ],
                                    true,
                                ),
                            },
                        )?
                    },

                    Token::SetValueMathSym(_sym) => {
                        todo!()
                    },

                    // If the next token after a statement is a comma, then we should assume that the comme is used as a separator of some sorts and return the original stmt.
                    // Example `fun(a, b)`, parse value returns a, since we are returning if a comma is present
                    Token::Comma => stmt,

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
pub fn parse_value<S: Streamable<Spanned<Token>> + std::fmt::Debug>(
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
            Token::UnparsedLiteral(_)
            | Token::MathSym(MathematicalSymbol::Addition)
            | Token::MathSym(MathematicalSymbol::Subtraction) => parse_numeric_literal(tkns)?,

            _ => todo!(),
        };

        return Ok(value);
    }

    Err(ParserError::UnknownValueExpression.into())
}
