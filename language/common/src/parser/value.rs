use crate::{error::parser::ParserError, tokenizer::Token};
use anyhow::Result;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MathematicalSymbol
{
    Addition,
    Subtraction,
    Division,
    Multiplication,
    Modulo,
}

impl TryInto<MathematicalSymbol> for Token
{
    type Error = ParserError;

    fn try_into(self) -> Result<MathematicalSymbol, Self::Error>
    {
        let expr = match self {
            Self::Addition => MathematicalSymbol::Addition,
            Self::Subtraction => MathematicalSymbol::Subtraction,
            Self::Division => MathematicalSymbol::Division,
            Self::Multiplication => MathematicalSymbol::Multiplication,
            Self::Modulo => MathematicalSymbol::Modulo,

            _ => return Err(ParserError::InternalVariableError),
        };

        Ok(expr)
    }
}
