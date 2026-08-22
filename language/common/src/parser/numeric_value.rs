use crate::{
    error::{Spanned, parser::ParserError},
    parser::{
        common::Streamable,
        dbg::combine_span_info,
        statement::{parse_expr, parse_statement, parse_variable_expression},
    },
    tokenizer::Token,
    ty::{NotNan, TypeDiscriminants, Value},
};
use anyhow::Result;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MathematicalSymbol
{
    Addition,
    Subtraction,
    Division,
    Multiplication,
    Modulo,
    Power,
}

use crate::{error::syntax::SyntaxError, parser::common::StatementVariant};

///
/// Bits	Signed	Unsigned	Float
/// 8-bit	-	    uintsmall	-
/// 16-bit	inthalf	uinthalf	floathalf
/// 32-bit	int	    uint	    float
/// 64-bit	intlong	uintlong	floatlong
///
/// Numeric suffixes will also be supported
/// The list of suffixes are found here:
///

// The list of the numeric suffiexes
pub const NUMERIC_SUFFIX: &[(&str, TypeDiscriminants)] = &[
    ("uintsmall", TypeDiscriminants::U8),
    ("uinthalf", TypeDiscriminants::U16),
    ("uint", TypeDiscriminants::U32),
    ("uintlong", TypeDiscriminants::U64),
    ("inthalf", TypeDiscriminants::I16),
    ("int", TypeDiscriminants::I32),
    ("intlong", TypeDiscriminants::I64),
    ("floathalf", TypeDiscriminants::F16),
    ("float", TypeDiscriminants::F32),
    ("floatlong", TypeDiscriminants::F64),
];

// Matches the suffix of a number and returns the unparsed literal without the suffix.
fn try_match_suffix(str: &str) -> Option<(&str, TypeDiscriminants)>
{
    for (suf, ty) in NUMERIC_SUFFIX {
        if str.ends_with(*suf) {
            return Some((str.trim_suffix(*suf), *ty));
        }
    }

    None
}

fn fit_unsigned(digits: &str) -> Result<Value, ParserError>
{
    let n = digits
        .parse::<u64>()
        .map_err(|_| ParserError::LiteralOutOfRange(digits.to_string()))?;

    if n <= u8::MAX as u64 {
        return Ok(Value::U8(n as u8));
    }
    if n <= u16::MAX as u64 {
        return Ok(Value::U16(n as u16));
    }
    if n <= u32::MAX as u64 {
        return Ok(Value::U32(n as u32));
    }
    Ok(Value::U64(n))
}

// Future me: The reason why this is unused (here) is because, normally integers get parsed as either a negated usize (basically -usize, which is converted to an integer in analyzer::type_inference), usize or a float.
fn fit_signed(digits: &str) -> Result<Value, ParserError>
{
    let n = digits
        .parse::<i64>()
        .map_err(|_| ParserError::LiteralOutOfRange(digits.to_string()))?;

    if n >= i16::MIN as i64 && n <= i16::MAX as i64 {
        return Ok(Value::I16(n as i16));
    }
    if n >= i32::MIN as i64 && n <= i32::MAX as i64 {
        return Ok(Value::I32(n as i32));
    }
    Ok(Value::I64(n))
}

fn fit_float(digits: &str) -> Result<Value, ParserError>
{
    // Parse once into f64 as the widest type
    let n = digits
        .parse::<f64>()
        .map_err(|_| ParserError::LiteralOutOfRange(digits.to_string()))?;

    if n.is_nan() {
        return Err(ParserError::LiteralIsNan);
    }

    // Try f16 first — check if it round-trips without significant precision loss
    let as_f16 = n as f32;
    if (as_f16 as f64 - n).abs() < f64::EPSILON {
        return NotNan::new(as_f16)
            .map(Value::F32)
            .map_err(|_| ParserError::LiteralIsNan);
    }

    // Try f32
    let as_f32 = n as f32;
    if (as_f32 as f64 - n).abs() < f64::EPSILON {
        return NotNan::new(as_f32)
            .map(Value::F32)
            .map_err(|_| ParserError::LiteralIsNan);
    }

    // Fall back to f64
    NotNan::new(n)
        .map(Value::F64)
        .map_err(|_| ParserError::LiteralIsNan)
}

/// The name is a bit inaccurate, please check definition before use.
/// This functions tries to parse a value related to a mathematical equation.
pub fn parse_numeric_value<S: Streamable<Spanned<Token>> + std::fmt::Debug>(
    first_token: Spanned<Token>,
    tkns: &mut S,
) -> anyhow::Result<Spanned<StatementVariant>>
{
    // Fetch the span of the token consumed
    let current_token_span = *first_token.get_span();

    // Fetch the lhs of the expression
    let val = match first_token.get_inner() {
        // Check if the first token is a negation/subtraction sign.
        Token::MathSym(MathematicalSymbol::Subtraction) => {
            Spanned {
                inner: StatementVariant::NegateValue(Box::new(parse_expr(tkns)?)),
                span: current_token_span,
            }
        },
        // I defined this so its a bit easier to read since subtraction is a different path too
        Token::MathSym(MathematicalSymbol::Addition) => parse_expr(tkns)?,
        // Parse the number present
        Token::UnparsedLiteral(unparsed_literal) => {
            Spanned {
                inner: {
                    if let Some((lit, ty)) = try_match_suffix(unparsed_literal) {
                        match ty {
                            TypeDiscriminants::I64 => {
                                StatementVariant::Value(Value::I64(lit.parse::<i64>()?))
                            },
                            TypeDiscriminants::F64 => {
                                StatementVariant::Value(Value::F64(NotNan::new(
                                    lit.parse::<f64>()?,
                                )?))
                            },
                            TypeDiscriminants::U64 => {
                                StatementVariant::Value(Value::U64(lit.parse::<u64>()?))
                            },
                            TypeDiscriminants::I32 => {
                                StatementVariant::Value(Value::I32(lit.parse::<i32>()?))
                            },
                            TypeDiscriminants::F32 => {
                                StatementVariant::Value(Value::F32(NotNan::new(
                                    lit.parse::<f32>()?,
                                )?))
                            },
                            TypeDiscriminants::U32 => {
                                StatementVariant::Value(Value::U32(lit.parse::<u32>()?))
                            },
                            TypeDiscriminants::I16 => {
                                StatementVariant::Value(Value::I16(lit.parse::<i16>()?))
                            },
                            TypeDiscriminants::F16 => {
                                StatementVariant::Value(Value::F32(NotNan::new(
                                    lit.parse::<f16>()? as f32,
                                )?))
                            },
                            TypeDiscriminants::U16 => {
                                StatementVariant::Value(Value::U16(lit.parse::<u16>()?))
                            },
                            TypeDiscriminants::U8 => {
                                StatementVariant::Value(Value::U8(lit.parse::<u8>()?))
                            },

                            _ => {
                                unreachable!(
                                    "An unreachable type suffix was implemented. Check numerical suffix match."
                                )
                            },
                        }
                    }
                    else {
                        // If the literal contains a '.' then its a float, the automatic sizing is inside the function
                        // If it doesnt, then it is an unsigned since subtractions are handled elsewhere. (Semantic analysis)
                        StatementVariant::Value({
                            if unparsed_literal.contains('.') {
                                fit_float(unparsed_literal)?
                            }
                            else {
                                fit_unsigned(unparsed_literal)?
                            }
                        })
                    }
                },
                span: current_token_span,
            }
        },

        // Try to parse the value regardless
        _ => parse_expr(tkns)?,
    };

    // We should accept any of these:
    // 200 + 100
    // foo;
    // -(foo)
    // -200
    // -foo
    // -bar()

    return Ok(parse_variable_expression(tkns, val)?);
}
