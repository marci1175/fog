use std::str::FromStr;

use num::cast::AsPrimitive;
use strum::IntoDiscriminant;
use strum_macros::Display;

use crate::app::parser::error::ParserError;

#[derive(Debug, strum_macros::EnumDiscriminants, Clone, PartialEq, Display)]
#[strum_discriminants(derive(strum_macros::Display, strum_macros::VariantArray))]
pub enum Type {
    I32(i32),
    F32(f32),
    U32(u32),
    U8(u8),

    String(String),
    Boolean(bool),

    Void,
}

impl From<TypeDiscriminants> for Type {
    fn from(value: TypeDiscriminants) -> Self {
        match value {
            TypeDiscriminants::I32 => Self::I32(0),
            TypeDiscriminants::F32 => Self::F32(0.0),
            TypeDiscriminants::U32 => Self::U32(0),
            TypeDiscriminants::U8 => Self::U8(0),
            TypeDiscriminants::String => Self::String(String::new()),
            TypeDiscriminants::Boolean => Self::Boolean(false),
            TypeDiscriminants::Void => Self::Void,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StringReference {
    pub ref_idx: usize,
}

impl Default for StringReference {
    fn default() -> Self {
        Self::new()
    }
}

impl StringReference {
    pub fn new() -> Self {
        Self { ref_idx: 0 }
    }
}

pub fn unparsed_const_to_typed_literal_unsafe(
    raw_string: String,
    dest_type: TypeDiscriminants,
) -> Result<Type, ParserError> {
    let parsed_num = raw_string
        .parse::<f64>()
        .map_err(|_| ParserError::ConstTypeUndetermined(raw_string, dest_type))?;

    let casted_var = match dest_type {
        TypeDiscriminants::I32 => Type::I32(parsed_num as i32),
        TypeDiscriminants::F32 => Type::F32(parsed_num as f32),
        TypeDiscriminants::U32 => Type::U32(parsed_num as u32),
        TypeDiscriminants::U8 => Type::U8(parsed_num as u8),
        TypeDiscriminants::String => Type::String(parsed_num.to_string()),
        TypeDiscriminants::Boolean => {
            if parsed_num == 1.0 {
                Type::Boolean(true)
            } else {
                Type::Boolean(false)
            }
        }
        TypeDiscriminants::Void => Type::Void,
    };

    Ok(casted_var)
}
pub fn unparsed_const_to_typed_literal(
    raw_string: String,
    dest_type: TypeDiscriminants,
) -> Result<Type, ParserError> {
    let typed_var = match dest_type {
        TypeDiscriminants::I32 => Type::I32(
            raw_string
                .parse::<i32>()
                .map_err(|_| ParserError::ConstTypeUndetermined(raw_string, dest_type))?,
        ),
        TypeDiscriminants::F32 => Type::F32(
            raw_string
                .parse::<f32>()
                .map_err(|_| ParserError::ConstTypeUndetermined(raw_string, dest_type))?,
        ),
        TypeDiscriminants::U32 => Type::U32(
            raw_string
                .parse::<u32>()
                .map_err(|_| ParserError::ConstTypeUndetermined(raw_string, dest_type))?,
        ),
        TypeDiscriminants::U8 => Type::U8(
            raw_string
                .parse::<u8>()
                .map_err(|_| ParserError::ConstTypeUndetermined(raw_string, dest_type))?,
        ),
        TypeDiscriminants::String => {
            return Err(ParserError::ConstTypeUndetermined(raw_string, dest_type));
        }
        TypeDiscriminants::Boolean => {
            if raw_string == "false" {
                Type::Boolean(false)
            } else if raw_string == "true" {
                Type::Boolean(true)
            } else {
                return Err(ParserError::ConstTypeUndetermined(raw_string, dest_type));
            }
        }
        TypeDiscriminants::Void => {
            return Err(ParserError::ConstTypeUndetermined(raw_string, dest_type));
        }
    };

    Ok(typed_var)
}

pub fn convert_as(value: Type, dest_type: TypeDiscriminants) -> anyhow::Result<Type> {
    if value.discriminant() == dest_type {
        return Ok(value);
    }

    if dest_type == TypeDiscriminants::Void {
        return Ok(Type::Void);
    }

    let return_val = match value {
        Type::I32(inner) => match dest_type {
            TypeDiscriminants::F32 => Type::F32(inner as f32),
            TypeDiscriminants::U32 => Type::U32(inner as u32),
            TypeDiscriminants::U8 => Type::U8(inner as u8),
            TypeDiscriminants::String => Type::String(inner.to_string()),
            TypeDiscriminants::Boolean => {
                if inner == 1 {
                    Type::Boolean(true)
                } else {
                    Type::Boolean(false)
                }
            }

            TypeDiscriminants::I32 | TypeDiscriminants::Void => unreachable!(),
        },
        Type::F32(inner) => match dest_type {
            TypeDiscriminants::I32 => Type::I32(inner as i32),
            TypeDiscriminants::U32 => Type::U32(inner as u32),
            TypeDiscriminants::U8 => Type::U8(inner as u8),
            TypeDiscriminants::String => Type::String(inner.to_string()),
            TypeDiscriminants::Boolean => {
                if inner == 1.0 {
                    Type::Boolean(true)
                } else {
                    Type::Boolean(false)
                }
            }

            TypeDiscriminants::F32 | TypeDiscriminants::Void => unreachable!(),
        },
        Type::U32(inner) => match dest_type {
            TypeDiscriminants::F32 => Type::F32(inner as f32),
            TypeDiscriminants::I32 => Type::I32(inner as i32),
            TypeDiscriminants::U8 => Type::U8(inner as u8),
            TypeDiscriminants::String => Type::String(inner.to_string()),
            TypeDiscriminants::Boolean => {
                if inner == 1 {
                    Type::Boolean(true)
                } else {
                    Type::Boolean(false)
                }
            }

            TypeDiscriminants::U32 | TypeDiscriminants::Void => unreachable!(),
        },
        Type::U8(inner) => match dest_type {
            TypeDiscriminants::F32 => Type::F32(inner as f32),
            TypeDiscriminants::I32 => Type::I32(inner as i32),
            TypeDiscriminants::U32 => Type::U32(inner as u32),
            TypeDiscriminants::String => Type::String(inner.to_string()),
            TypeDiscriminants::Boolean => {
                if inner == 1 {
                    Type::Boolean(true)
                } else {
                    Type::Boolean(false)
                }
            }

            TypeDiscriminants::U8 | TypeDiscriminants::Void => unreachable!(),
        },
        Type::String(inner) => match dest_type {
            TypeDiscriminants::I32 => Type::I32(inner.parse::<i32>()?),
            TypeDiscriminants::F32 => Type::F32(inner.parse::<f32>()?),
            TypeDiscriminants::U32 => Type::U32(inner.parse::<u32>()?),
            TypeDiscriminants::U8 => Type::U8(inner.parse::<u8>()?),
            TypeDiscriminants::Boolean => Type::Boolean(inner.parse::<bool>()?),

            TypeDiscriminants::String | TypeDiscriminants::Void => unreachable!(),
        },

        Type::Boolean(inner) => match dest_type {
            TypeDiscriminants::I32 => Type::I32(inner as i32),
            TypeDiscriminants::F32 => Type::F32(inner as i32 as f32),
            TypeDiscriminants::U32 => Type::U32(inner as u32),
            TypeDiscriminants::U8 => Type::U8(inner as u8),
            TypeDiscriminants::String => Type::String(inner.to_string()),

            TypeDiscriminants::Boolean | TypeDiscriminants::Void => unreachable!(),
        },
        Type::Void => unreachable!(),
    };

    Ok(return_val)
}
