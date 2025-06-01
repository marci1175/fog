use std::fmt::Display;

use indexmap::IndexMap;
use strum_macros::Display;

use crate::app::parser::error::ParserError;

#[derive(Debug, Clone, PartialEq, Display, Default)]
pub enum Type {
    I32(i32),
    F32(f32),
    U32(u32),
    U8(u8),

    String(String),
    Boolean(bool),

    #[default]
    Void,

    Struct((String, IndexMap<String, Type>)),
}

impl Type {
    pub fn discriminant(&self) -> TypeDiscriminant {
        match self {
            Type::I32(_) => TypeDiscriminant::I32,
            Type::F32(_) => TypeDiscriminant::F32,
            Type::U32(_) => TypeDiscriminant::U32,
            Type::U8(_) => TypeDiscriminant::U8,
            Type::String(_) => TypeDiscriminant::String,
            Type::Boolean(_) => TypeDiscriminant::Boolean,
            Type::Void => TypeDiscriminant::Void,
            Type::Struct((struct_name, struct_fields)) => {
                let mut struct_field_ty_list = IndexMap::new();

                for (name, ty) in struct_fields.iter() {
                    struct_field_ty_list.insert(name.clone(), ty.discriminant());
                }

                TypeDiscriminant::Struct((struct_name.clone(), struct_field_ty_list))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Eq)]
pub enum TypeDiscriminant {
    I32,
    F32,
    U32,
    U8,

    String,
    Boolean,

    #[default]
    Void,

    Struct((String, IndexMap<String, TypeDiscriminant>)),
}

impl From<TypeDiscriminant> for Type {
    fn from(value: TypeDiscriminant) -> Self {
        match value {
            TypeDiscriminant::I32 => Self::I32(0),
            TypeDiscriminant::F32 => Self::F32(0.0),
            TypeDiscriminant::U32 => Self::U32(0),
            TypeDiscriminant::U8 => Self::U8(0),
            TypeDiscriminant::String => Self::String(String::new()),
            TypeDiscriminant::Boolean => Self::Boolean(false),
            TypeDiscriminant::Void => Self::Void,
            TypeDiscriminant::Struct(_) => {
                unimplemented!("Cannot create a Custom type from a `TypeDiscriminant`.")
            }
        }
    }
}

impl Display for TypeDiscriminant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match self {
            TypeDiscriminant::I32 => "I32".to_string(),
            TypeDiscriminant::F32 => "F32".to_string(),
            TypeDiscriminant::U32 => "U32".to_string(),
            TypeDiscriminant::U8 => "U8".to_string(),
            TypeDiscriminant::String => format!("String"),
            TypeDiscriminant::Boolean => "Boolean".to_string(),
            TypeDiscriminant::Void => "Void".to_string(),
            TypeDiscriminant::Struct((struct_name, _)) => format!("Struct({struct_name})"),
        })
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
    dest_type: TypeDiscriminant,
) -> Result<Type, ParserError> {
    let parsed_num = raw_string
        .parse::<f64>()
        .map_err(|_| ParserError::InvalidTypeCast(raw_string.clone(), dest_type.clone()))?;

    let casted_var = match dest_type {
        TypeDiscriminant::I32 => {
            if parsed_num.floor() != parsed_num {
                return Err(ParserError::InvalidTypeCast(parsed_num.to_string(), TypeDiscriminant::I32));
            }
            else {
                Type::I32(parsed_num as i32)
            }
        },
        TypeDiscriminant::F32 => Type::F32(parsed_num as f32),
        TypeDiscriminant::U32 => {
            if parsed_num.floor() != parsed_num {
                return Err(ParserError::InvalidTypeCast(parsed_num.to_string(), TypeDiscriminant::U32));
            }
            else {
                Type::U32(parsed_num as u32)
            }
        },
        TypeDiscriminant::U8 => {
            if parsed_num.floor() != parsed_num {
                return Err(ParserError::InvalidTypeCast(parsed_num.to_string(), TypeDiscriminant::U32));
            }
            else {
                Type::U8(parsed_num as u8)
            }
        },
        TypeDiscriminant::String => {
            return Err(ParserError::InvalidTypeCast(
                parsed_num.to_string(),
                TypeDiscriminant::String,
            ));
        }
        TypeDiscriminant::Boolean => {
            if parsed_num == 1.0 {
                Type::Boolean(true)
            } else if parsed_num == 0.0 {
                Type::Boolean(false)
            } else {
                return Err(ParserError::InvalidTypeCast(
                    raw_string.clone(),
                    TypeDiscriminant::Boolean,
                ));
            }
        }
        TypeDiscriminant::Void => Type::Void,
        TypeDiscriminant::Struct(inner) => {
            return Err(ParserError::InvalidTypeCast(
                raw_string,
                TypeDiscriminant::Struct(inner),
            ));
        }
    };

    Ok(casted_var)
}