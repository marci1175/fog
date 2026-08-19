use crate::{
    error::{Spanned, parser::ParserError},
    parser::{
        common::{StatementVariant, Streamable},
        dbg::combine_span_info,
        statement::parse_value,
        ty::parse_type,
        variable::VARIABLE_ID_SOURCE,
    },
    tokenizer::{Token, TokenDiscriminants},
};

pub fn var_decl<S: Streamable<Spanned<Token>> + std::fmt::Debug>(
    tkns: &mut S,
) -> anyhow::Result<Spanned<StatementVariant>>
{
    // Fetch the very first token's span
    let span_start = *tkns
        .peek_next()
        .map(|tkn| tkn.get_span())
        .ok_or(ParserError::EOF)?;

    // If the first token is a const token then that means that this variable is a constant and cannot be modified later.
    let is_constant = matches!(
        tkns.peek_next(),
        Some(&Spanned {
            inner: Token::Const,
            span: _
        })
    );

    // If the variable is a constant we should consume the constant token 
    if is_constant {
        tkns.consume();
    }

    // The first token should be a type of some sorts
    let variable_type = parse_type(tkns)?;

    // Fetch the variable's name
    let variable_name = tkns
        .try_consume_match(
            ParserError::SyntaxError(crate::error::syntax::SyntaxError::InvalidVariableDefinition),
            &TokenDiscriminants::Identifier,
        )?
        .try_as_identifier_ref()
        .unwrap()
        .clone();

    // Next token should be the equals sign in all cases as a variable cannot be uninitalized.
    tkns.try_consume_match(
        ParserError::SyntaxError(crate::error::syntax::SyntaxError::VariableRequiresInitialization),
        &TokenDiscriminants::SetValue,
    )?;

    // Get the value this variable was initalized with
    let variable_value = Box::new(parse_value(tkns)?);
    let span_end = *variable_value.get_span();

    // Return a valid variable decleration statement
    Ok(Spanned {
        inner: StatementVariant::NewVariable {
            variable_name,
            variable_type,
            variable_value,
            variable_id: VARIABLE_ID_SOURCE.get_unique_id(),
            is_mutable: !is_constant,
        },
        span: combine_span_info(&[span_start, span_end], true),
    })
}

pub fn mod_variable<S: Streamable<Spanned<Token>>>(
    tkns: &mut S,
) -> anyhow::Result<Spanned<StatementVariant>>
{
    Ok(todo!())
}
