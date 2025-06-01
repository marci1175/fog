use std::fmt::Display;

use indexmap::IndexMap;
use strum_macros::Display;

use crate::app::parser::error::ParserError;

#[derive(Debug, Clone, PartialEq, Display, Default)]
pub enum Type {
    I64(i64),
    F64(f64),
    U64(u64),

    I32(i32),
    F32(f32),
    U32(u32),

    I16(i16),
    F16(f16),
    U16(u16),

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
            Type::I64(_) => TypeDiscriminant::I64,
            Type::F64(_) => TypeDiscriminant::F64,
            Type::U64(_) => TypeDiscriminant::U64,
            Type::I32(_) => TypeDiscriminant::I32,
            Type::F32(_) => TypeDiscriminant::F32,
            Type::U32(_) => TypeDiscriminant::U32,
            Type::I16(_) => TypeDiscriminant::I16,
            Type::F16(_) => TypeDiscriminant::F16,
            Type::U16(_) => TypeDiscriminant::U16,
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
    I64,
    F64,
    U64,

    I32,
    F32,
    U32,

    I16,
    F16,
    U16,

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
            TypeDiscriminant::I64 => Self::I64(0),
            TypeDiscriminant::F64 => Self::F64(0.0),
            TypeDiscriminant::U64 => Self::U64(0),
            TypeDiscriminant::I32 => Self::I32(0),
            TypeDiscriminant::F32 => Self::F32(0.0),
            TypeDiscriminant::U32 => Self::U32(0),
            TypeDiscriminant::I16 => Self::I16(0),
            TypeDiscriminant::F16 => Self::F16(0.0),
            TypeDiscriminant::U16 => Self::U16(0),
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
            TypeDiscriminant::I64 => "I64".to_string(),
            TypeDiscriminant::F64 => "F64".to_string(),
            TypeDiscriminant::U64 => "U64".to_string(),
            TypeDiscriminant::I16 => "I16".to_string(),
            TypeDiscriminant::F16 => "F16".to_string(),
            TypeDiscriminant::U16 => "U16".to_string(),
            TypeDiscriminant::I32 => "I32".to_string(),
            TypeDiscriminant::F32 => "F32".to_string(),
            TypeDiscriminant::U32 => "U32".to_string(),
            TypeDiscriminant::U8 => "U8".to_string(),
            TypeDiscriminant::String => "String".to_string(),
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
        TypeDiscriminant::I64 => {
            if parsed_num.floor() != parsed_num {
                return Err(ParserError::InvalidTypeCast(
                    parsed_num.to_string(),
                    TypeDiscriminant::I64,
                ));
            } else {
                Type::I64(parsed_num as i64)
            }
        }
        TypeDiscriminant::F64 => Type::F64(parsed_num),
        TypeDiscriminant::U64 => {
            if parsed_num.floor() != parsed_num {
                return Err(ParserError::InvalidTypeCast(
                    parsed_num.to_string(),
                    TypeDiscriminant::U64,
                ));
            } else {
                Type::U64(parsed_num as u64)
            }
        }
        TypeDiscriminant::I16 => {
            if parsed_num.floor() != parsed_num {
                return Err(ParserError::InvalidTypeCast(
                    parsed_num.to_string(),
                    TypeDiscriminant::I16,
                ));
            } else {
                Type::I16(parsed_num as i16)
            }
        }
        TypeDiscriminant::F16 => Type::F16(parsed_num as f16),
        TypeDiscriminant::U16 => {
            if parsed_num.floor() != parsed_num {
                return Err(ParserError::InvalidTypeCast(
                    parsed_num.to_string(),
                    TypeDiscriminant::U16,
                ));
            } else {
                Type::U16(parsed_num as u16)
            }
        }
        TypeDiscriminant::I32 => {
            if parsed_num.floor() != parsed_num {
                return Err(ParserError::InvalidTypeCast(
                    parsed_num.to_string(),
                    TypeDiscriminant::I32,
                ));
            } else {
                Type::I32(parsed_num as i32)
            }
        }
        TypeDiscriminant::F32 => Type::F32(parsed_num as f32),
        TypeDiscriminant::U32 => {
            if parsed_num.floor() != parsed_num {
                return Err(ParserError::InvalidTypeCast(
                    parsed_num.to_string(),
                    TypeDiscriminant::U32,
                ));
            } else {
                Type::U32(parsed_num as u32)
            }
        }
        TypeDiscriminant::U8 => {
            if parsed_num.floor() != parsed_num {
                return Err(ParserError::InvalidTypeCast(
                    parsed_num.to_string(),
                    TypeDiscriminant::U32,
                ));
            } else {
                Type::U8(parsed_num as u8)
            }
        }
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
