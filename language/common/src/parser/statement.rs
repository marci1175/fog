use crate::{
    codegen::ty::OrdMap,
    error::{Spanned, parser::ParserError, syntax::SyntaxError},
    parser::{
        common::{
            StatementVariant, Streamable, child_iterator_until, find_closing_braces,
            find_closing_paren,
        },
        dbg::combine_span_info,
        numeric_value::{MathematicalSymbol, parse_numeric_value},
        statements::{
            conditionals::conditional_expr,
            loops::{loop_for, loop_while},
            variables::var_decl,
        },
    },
    tokenizer::{Token, TokenDiscriminants},
};

pub fn parse_statement<S: Streamable<Spanned<Token>> + std::fmt::Debug>(
    tkns: &mut S,
) -> anyhow::Result<Spanned<StatementVariant>>
{
    let value = match tkns.peek_next().map(|tkn| tkn.get_inner()) {
        Some(Token::If) => conditional_expr(tkns)?,
        Some(Token::While) => loop_while(tkns)?,
        Some(Token::For) => loop_for(tkns)?,
        // These expression should end at the `;` terminator since they are set size expressions.
        Some(Token::Variable) | Some(Token::Const) => {
            let return_val = var_decl(&mut child_iterator_until(
                tkns,
                &TokenDiscriminants::SemiColon,
                ParserError::SyntaxError(SyntaxError::MissingSemiColon),
            )?)?;

            // Consume semicolon after variable declaration
            tkns.try_consume_match(
                ParserError::ExpressionSemicolonMissing,
                &TokenDiscriminants::SemiColon,
            )?;

            return_val
        },
        None => {
            return Err(ParserError::EOF.into());
        },
        Some(_) => {
            // Parse the value we might have here (most of the times this will be useless in this function, except the function call which may have a side effect on the code.)
            // Even though we separate expression based on semicolons, if the user leaves out a semicolon the code is stil going to break so we have to add eadditional checks later.
            let mut expr_tkns = child_iterator_until(
                tkns,
                &TokenDiscriminants::SemiColon,
                ParserError::SyntaxError(SyntaxError::MissingSemiColon),
            )?;

            let return_val = parse_expr(&mut expr_tkns, None)?;

            // Drop child buffer
            drop(expr_tkns);

            // Consume semicolon after expression
            tkns.try_consume_match(
                ParserError::ExpressionSemicolonMissing,
                &TokenDiscriminants::SemiColon,
            )?;

            return_val
        },
    };

    Ok(value)
}

/// This function is called when we want to parse a part of the expression which might change or repeat an undefined times.
/// Examples include parsing mathematical expressions: ```10 + foo + bar```
/// Parsing struct field accessing: ```foo.bar[3].baz.asd[0]```
pub fn parse_expr<S: Streamable<Spanned<Token>> + std::fmt::Debug>(
    tkns: &mut S,
    base_statement: Option<Spanned<StatementVariant>>,
) -> anyhow::Result<Spanned<StatementVariant>>
{
    let mut stmt = fetch_base_expression(tkns, base_statement)?;

    while let Some(tkn) = tkns.consume().cloned() {
        // Consume the next token in the stream
        stmt = match tkn.get_inner() {
            Token::OpenParentheses => {
                // If the last statement parsed was an identifier, we should interpret the parentheses as the arguments of a function call
                if matches!(stmt.get_inner(), StatementVariant::BasicReference { .. })
                    || matches!(
                        stmt.get_inner(),
                        StatementVariant::StructPropertyAccess { .. }
                    )
                {
                    parse_function_call(tkns, stmt, &tkn)?
                }
                // Interpret it as a grouped expression
                else {
                    parse_grouped_expr(tkns, tkn)?
                }
            },

            // Indexing
            Token::OpenSquareBrackets => parse_indexing(tkns, stmt, &tkn)?,

            // Struct access
            Token::Dot => parse_field_access(tkns, stmt, &tkn)?,

            // Implement math expressions here
            Token::MathSym(sym) => parse_math_symbol(tkns, stmt, sym, &tkn)?,

            // If the next token after a statement is a comma, then we should assume that the comme is used as a separator of some sorts and return the original stmt.
            // Example `fun(a, b)`, parse value returns `a` since we are returning if a comma is present
            Token::Comma => stmt,

            Token::SetValueMathSym(sym) => parse_set_value_math_symbol(tkns, stmt, sym)?,

            Token::SetValue => parse_set_value(tkns, stmt)?,

            Token::LogicalOperator(log_op) => parse_logical_operator(tkns, stmt, log_op)?,

            Token::Comparison(_) | Token::OpenAngledBrackets | Token::CloseAngledBrackets => {
                parse_comparison(tkns, stmt, &tkn)?
            },

            Token::OpenBraces => parse_array_init(tkns, &tkn)?,

            _ => {
                return Err(
                    ParserError::SyntaxError(SyntaxError::InvalidVariableExpression(
                        tkn.get_inner().clone(),
                    ))
                    .into(),
                );
            },
        };
    }

    Ok(stmt)
}

fn parse_set_value<S: Streamable<Spanned<Token>> + std::fmt::Debug>(
    tkns: &mut S,
    stmt: Spanned<StatementVariant>,
) -> Result<Spanned<StatementVariant>, anyhow::Error>
{
    let value = Box::new(parse_expr(tkns, None)?);
    let span = combine_span_info(&[*stmt.get_span(), *value.get_span()], true);

    Ok(Spanned {
        inner: StatementVariant::SetValue {
            receiver: Box::new(stmt),
            value,
        },
        span,
    })
}

#[inline(never)]
fn parse_logical_operator<S: Streamable<Spanned<Token>> + std::fmt::Debug>(
    tkns: &mut S,
    stmt: Spanned<StatementVariant>,
    log_op: &super::common::LogicalOperator,
) -> Result<Spanned<StatementVariant>, anyhow::Error>
{
    let lhs = stmt;
    let rhs = parse_expr(tkns, None)?;

    let span = combine_span_info(&[*lhs.get_span(), *rhs.get_span()], true);

    Ok(Spanned {
        inner: StatementVariant::LogicalOperation {
            lhs: Box::new(lhs),
            op: *log_op,
            rhs: Box::new(rhs),
        },
        span,
    })
}

#[inline(never)]
fn parse_set_value_math_symbol<S: Streamable<Spanned<Token>> + std::fmt::Debug>(
    tkns: &mut S,
    stmt: Spanned<StatementVariant>,
    sym: &MathematicalSymbol,
) -> Result<Spanned<StatementVariant>, anyhow::Error>
{
    let value = Box::new(parse_expr(tkns, None)?);

    let span = combine_span_info(&[*stmt.get_span(), *value.get_span()], true);

    Ok(Spanned {
        inner: StatementVariant::ModifyValueArithmetic {
            receiver: Box::new(stmt),
            value,
            symbol: *sym,
        },
        span,
    })
}

#[inline(never)]
fn parse_math_symbol<S: Streamable<Spanned<Token>> + std::fmt::Debug>(
    tkns: &mut S,
    stmt: Spanned<StatementVariant>,
    sym: &MathematicalSymbol,
    tkn: &Spanned<Token>,
) -> Result<Spanned<StatementVariant>, anyhow::Error>
{
    let rhs = parse_expr(tkns, None)?;

    Ok(Spanned {
        inner: StatementVariant::MathematicalExpression {
            lhs: Box::new(stmt),
            symbol: *sym,
            rhs: Box::new(rhs),
        },
        span: combine_span_info(
            &[
                *tkn.get_span(),
                *tkns.get_last_consumed().ok_or_else(|| ParserError::EOF)?.get_span(),
            ],
            true,
        ),
    })
}

#[inline(never)]
fn parse_field_access<S: Streamable<Spanned<Token>> + std::fmt::Debug>(
    tkns: &mut S,
    stmt: Spanned<StatementVariant>,
    tkn: &Spanned<Token>,
) -> Result<Spanned<StatementVariant>, anyhow::Error>
{
    let field_name = tkns
        .try_consume_match(
            ParserError::SyntaxError(SyntaxError::InvalidStructFieldReference),
            &TokenDiscriminants::Identifier,
        )?
        .try_as_identifier_ref()
        .unwrap()
        .clone();

    Ok(Spanned {
        inner: StatementVariant::StructPropertyAccess {
            variable_reference: Box::new(stmt),
            field_name,
        },
        span: *tkn.get_span(),
    })
}

#[inline(never)]
fn parse_comparison<S: Streamable<Spanned<Token>> + std::fmt::Debug>(
    tkns: &mut S,
    stmt: Spanned<StatementVariant>,
    tkn: &Spanned<Token>,
) -> Result<Spanned<StatementVariant>, anyhow::Error>
{
    let ord = match tkn.get_inner() {
        Token::Comparison(ord) => *ord,
        Token::OpenAngledBrackets => crate::parser::common::Order::Bigger,
        Token::CloseAngledBrackets => crate::parser::common::Order::Smaller,
        _ => unreachable!(),
    };

    let rhs = parse_expr(tkns, None)?;

    let span = combine_span_info(&[*stmt.get_span(), *rhs.get_span()], true);

    Ok(Spanned {
        inner: StatementVariant::Comparison {
            lhs: Box::new(stmt),
            ord,
            rhs: Box::new(rhs),
        },
        span,
    })
}

#[inline(never)]
fn fetch_base_expression<S: Streamable<Spanned<Token>> + std::fmt::Debug>(
    tkns: &mut S,
    base_statement: Option<Spanned<StatementVariant>>,
) -> Result<Spanned<StatementVariant>, anyhow::Error>
{
    let stmt: Spanned<StatementVariant> = if let Some(stmt) = base_statement {
        stmt
    }
    else {
        if let Some(tkn) = tkns.consume().cloned() {
            match tkn.get_inner() {
                Token::Identifier(ident) => {
                    // Conume the next token
                    // Call a recursive function here which will resolve the expression after the variable's name.
                    // This function is the legacy eq of "fetch_variable_expr", parsing the statement after an identifier - such as foo[0] | foo.asd
                    Spanned {
                        inner: StatementVariant::BasicReference {
                            identifier: ident.to_owned(),
                        },
                        span: *tkn.get_span(),
                    }
                },
                Token::Literal(val) => {
                    // Consume the tokens after that since this may be a math expression or anything like that.
                    Spanned {
                        inner: StatementVariant::Value(Box::new(*val.clone())),
                        span: *tkn.get_span(),
                    }
                },
                Token::Reference => {
                    Spanned {
                        inner: StatementVariant::GetPointerTo(Box::new(parse_expr(tkns, None)?)),
                        span: *tkn.get_span(),
                    }
                },
                Token::Dereference => {
                    Spanned {
                        inner: StatementVariant::DerefPointer(Box::new(parse_expr(tkns, None)?)),
                        span: *tkn.get_span(),
                    }
                },

                Token::UnparsedLiteral(_)
                | Token::MathSym(MathematicalSymbol::Addition)
                | Token::MathSym(MathematicalSymbol::Subtraction) => {
                    parse_numeric_value(tkn, tkns)?
                },

                // Return token indicates the end of the function on that specific path
                Token::Return => {
                    // Get the returned value from the tokens
                    let returned_value = parse_expr(tkns, None)?;

                    // Create a new span for this return statement
                    let span =
                        combine_span_info(&[*tkn.get_span(), *returned_value.get_span()], true);

                    Spanned {
                        inner: StatementVariant::ReturnValue {
                            value: Box::new(returned_value),
                        },
                        span,
                    }
                },

                Token::OpenParentheses => parse_grouped_expr(tkns, tkn)?,

                _ => return Err(ParserError::UnknownExpression.into()),
            }
        }
        else {
            return Err(ParserError::EOF.into());
        }
    };
    Ok(stmt)
}

#[inline(never)]
fn parse_indexing<S: Streamable<Spanned<Token>> + std::fmt::Debug>(
    tkns: &mut S,
    stmt: Spanned<StatementVariant>,
    tkn: &Spanned<Token>,
) -> Result<Spanned<StatementVariant>, anyhow::Error>
{
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
        .ok_or_else(|| ParserError::SyntaxError(
            SyntaxError::LeftOpenSquareBrackets,
        ))?;
    
    let mut index_value_tkns = tkns
        .child_iterator_bulk(closing_pos)
        .ok_or_else(|| ParserError::EOF)?;
    
    let index_value = parse_expr(&mut index_value_tkns, None)?;
    
    drop(index_value_tkns);

    let closing_bracket_span = *tkns
        .try_consume_match(
            ParserError::SyntaxError(SyntaxError::LeftOpenSquareBrackets),
            &TokenDiscriminants::CloseSquareBrackets,
        )?
        .get_span();
    
    Ok(Spanned {
        inner: StatementVariant::ArrayReference {
            variable_reference: Box::new(stmt),
            index: Box::new(index_value),
        },
        // Combine the spans of the opening and the closing brackets so that the span will contain the whole array reference.
        span: combine_span_info(&[*tkn.get_span(), closing_bracket_span], true),
    })
}

#[inline(never)]
fn parse_array_init<S: Streamable<Spanned<Token>> + std::fmt::Debug>(
    tkns: &mut S,
    tkn: &Spanned<Token>,
) -> Result<Spanned<StatementVariant>, anyhow::Error>
{
    let mut values = Vec::new();

    let mut array_init_tokens = tkns
        .child_iterator_bulk(
            find_closing_braces(&*tkns)
                .ok_or_else(|| ParserError::SyntaxError(SyntaxError::LeftOpenBraces))?,
        )
        .ok_or_else(|| ParserError::EOF)?;

    while array_init_tokens.peek_next().is_some() {
        // This function returns at the "," so all of the values can be parsed
        let value = parse_expr(&mut array_init_tokens, None)?;

        // Store value
        values.push(value);
    }

    drop(array_init_tokens);

    let closing_brace = tkns.try_consume_match(
        ParserError::SyntaxError(SyntaxError::LeftOpenBraces),
        &TokenDiscriminants::CloseBraces,
    )?;

    let span = combine_span_info(&[*tkn.get_span(), *closing_brace.get_span()], true);

    Ok(Spanned {
        inner: StatementVariant::ArrayInitialization { values },
        span,
    })
}

#[inline(never)]
fn parse_function_call<S: Streamable<Spanned<Token>> + std::fmt::Debug>(
    tkns: &mut S,
    stmt: Spanned<StatementVariant>,
    tkn: &Spanned<Token>,
) -> Result<Spanned<StatementVariant>, anyhow::Error>
{
    let next_tkn = tkns.peek_next().cloned();

    let call = if let Some(Spanned {
        inner: Token::CloseParentheses,
        span,
    }) = next_tkn
    {
        // Consume token here since we are sure what this is
        tkns.consume();

        Spanned {
            inner: StatementVariant::FunctionCall {
                identifier: Box::new(stmt),
                arguments: OrdMap::new(),
            },
            span: combine_span_info(&[*tkn.get_span(), span], true),
        }
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
            .ok_or_else(|| ParserError::SyntaxError(SyntaxError::LeftOpenParentheses))?;

        // Tokens for the arguments inside the function call
        let mut argument_tkns = tkns
            .child_iterator_bulk(closing_paren_pos)
            .ok_or_else(|| ParserError::EOF)?;

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
                        inner: Token::SetValue,
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
                    crate::codegen::common::FunctionArgumentIdentifier::Identifier(named_arg),
                    Box::new(parse_expr(&mut argument_tkns, None)?),
                );
            }
            else {
                arguments.insert(
                    crate::codegen::common::FunctionArgumentIdentifier::Index(argument_idx),
                    Box::new(parse_expr(&mut argument_tkns, None)?),
                );
            }

            // Increment index regardless whether this was an indexed argument or not
            // When looking up a function argument we need the absolute position/index of the argument
            argument_idx += 1;
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

        Spanned {
            inner: StatementVariant::FunctionCall {
                identifier: Box::new(stmt),
                arguments,
            },
            span: combine_span_info(&[*tkn.get_span(), closing_paren_span], true),
        }
    };

    Ok(call)
}

#[inline(never)]
fn parse_grouped_expr<S: Streamable<Spanned<Token>> + std::fmt::Debug>(
    tkns: &mut S,
    tkn: Spanned<Token>,
) -> Result<Spanned<StatementVariant>, anyhow::Error>
{
    let mut code_block_tokens = tkns
        .child_iterator_bulk(
            find_closing_paren(&*tkns)
                .ok_or_else(|| ParserError::SyntaxError(SyntaxError::LeftOpenBraces))?,
        )
        .ok_or_else(|| ParserError::EOF)?;

    let grouped_expression = parse_expr(&mut code_block_tokens, None)?;

    drop(code_block_tokens);

    let grouping_close = tkns.try_consume_match(
        ParserError::SyntaxError(SyntaxError::LeftOpenParentheses),
        &TokenDiscriminants::CloseParentheses,
    )?;

    let grouped_expr_span = combine_span_info(&[*tkn.get_span(), *grouping_close.get_span()], true);

    Ok(Spanned {
        inner: StatementVariant::Grouping {
            inner_expr: Box::new(grouped_expression),
        },
        span: grouped_expr_span,
    })
}
