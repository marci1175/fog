use crate::{
    codegen::{Branch, If},
    error::{SpanInfo, Spanned, parser::ParserError, syntax::SyntaxError},
    parser::{
        common::{StatementVariant, Streamable, find_closing_braces, find_closing_paren},
        dbg::combine_span_info,
        function::parse_body,
        statement::{Expr, parse_expr},
    },
    tokenizer::{Token, TokenDiscriminants},
};

pub fn conditional_expr<S: Streamable<Spanned<Token>> + std::fmt::Debug>(
    expr: Expr,
    tkns: &mut S,
) -> anyhow::Result<Spanned<StatementVariant>>
{
    // The first token must be an `if` token
    tkns.try_consume_match(
        ParserError::InternalFastpathMatchingError(expr),
        &TokenDiscriminants::If,
    )?;

    let condition = Box::new(parse_condition(expr, tkns)?);
    let if_branch = parse_expr_body(tkns)?;

    // This if instance can be modified later on by else if chains
    let mut if_instance = If {
        condition,
        // This will contain the original, first body of code which will get ran if the if's condition returns true
        true_branch: if_branch,
        false_branch: None,
    };

    let mut current_false_branch: &mut Option<Branch> = &mut if_instance.false_branch;

    // We should now parse the else if chain of the if expression
    while matches!(
        tkns.peek_next(),
        Some(Spanned {
            inner: Token::ElseIf,
            span: _
        })
    ) {
        // Consume the actual elseif token
        tkns.consume().unwrap();

        let condition = Box::new(parse_condition(expr, tkns)?);
        let chained_branch = parse_expr_body(tkns)?;

        let false_branch_snapshot = std::mem::take(current_false_branch);

        let branch_span = chained_branch.span;

        // Insert a nested if and move the current false branch into the nested if's false branch.
        // Realistically we will never really need this to be stored since we are parsing the else at the last place which is the only thing that can overwrite the actual false branch at the very end.
        *current_false_branch = Some(Branch {
            body: vec![Spanned {
                inner: StatementVariant::If(If {
                    condition,
                    true_branch: chained_branch,
                    false_branch: false_branch_snapshot,
                }),

                // The actal false branch is not included in this span (correctly) because this if statement is chained into the false branch
                span: branch_span,
            }],
            // Same thing here
            span: branch_span,
        });

        current_false_branch = &mut current_false_branch.as_mut().unwrap().body[0]
            .inner
            .try_as_if_mut()
            .unwrap()
            .false_branch;
    }

    // If there is an else branch, parse that, the `current_false_branch` should already point to the correct location
    if matches!(
        tkns.peek_next(),
        Some(Spanned {
            inner: Token::Else,
            span: _
        })
    ) {
        // Consume else token
        tkns.consume();

        // Parse body of the else statement
        let else_branch = parse_expr_body(tkns)?;

        *current_false_branch = Some(else_branch);
    }

    // Combine spans but if there are no false branches, the false branch span will be unset, lets handle it as the body span as a safe default.
    let span = combine_span_info(
        &[
            if_instance.true_branch.span,
            current_false_branch
                .as_ref()
                .map(|branch| branch.span)
                .unwrap_or(if_instance.true_branch.span),
        ],
        true,
    );

    Ok(Spanned {
        inner: StatementVariant::If(if_instance),
        span,
    })
}

fn parse_expr_body<S: Streamable<Spanned<Token>> + std::fmt::Debug>(
    tkns: &mut S,
) -> Result<Branch, anyhow::Error>
{
    let body_start = *tkns
        .try_consume_match(
            ParserError::SyntaxError(SyntaxError::ConditionalExpressionRequiresBody),
            &TokenDiscriminants::OpenBraces,
        )?
        .get_span();

    let body = parse_body(tkns)?;

    let body_end = *tkns
        .try_consume_match(
            ParserError::SyntaxError(SyntaxError::LeftOpenBraces),
            &TokenDiscriminants::CloseBraces,
        )?
        .get_span();

    Ok(Branch {
        body,
        span: combine_span_info(&[body_start, body_end], true),
    })
}

fn parse_condition<S: Streamable<Spanned<Token>>>(
    expr: Expr,
    tkns: &mut S,
) -> Result<Spanned<StatementVariant>, anyhow::Error>
{
    tkns.try_consume_match(
        ParserError::InternalFastpathMatchingError(expr),
        &TokenDiscriminants::OpenParentheses,
    )?;

    let mut condition_tokens = tkns
        .child_iterator_bulk(
            find_closing_paren(tkns)
                .ok_or(ParserError::SyntaxError(SyntaxError::LeftOpenParentheses))?,
        )
        .ok_or(ParserError::EOF)?;

    let condition = parse_expr(&mut condition_tokens)?;

    drop(condition_tokens);

    tkns.try_consume_match(
        ParserError::SyntaxError(SyntaxError::LeftOpenParentheses),
        &TokenDiscriminants::CloseParentheses,
    )?;

    Ok(condition)
}
