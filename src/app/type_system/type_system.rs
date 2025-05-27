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
    pub fn discriminant(&self) -> TypeDiscriminants {
        match self {
            Type::I32(_) => TypeDiscriminants::I32,
            Type::F32(_) => TypeDiscriminants::F32,
            Type::U32(_) => TypeDiscriminants::U32,
            Type::U8(_) => TypeDiscriminants::U8,
            Type::String(_) => TypeDiscriminants::String,
            Type::Boolean(_) => TypeDiscriminants::Boolean,
            Type::Void => TypeDiscriminants::Void,
            Type::Struct((struct_name, struct_fields)) => {
                let mut struct_field_ty_list = IndexMap::new();

                for (name, ty) in struct_fields.iter() {
                    struct_field_ty_list.insert(name.clone(), ty.discriminant());
                }

                TypeDiscriminants::Struct((struct_name.clone(), struct_field_ty_list))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Eq)]
pub enum TypeDiscriminants {
    I32,
    F32,
    U32,
    U8,

    String,
    Boolean,

    #[default]
    Void,

    Struct((String, IndexMap<String, TypeDiscriminants>)),
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
            TypeDiscriminants::Struct(_) => {
                unimplemented!("Cannot create a Custom type from a `TypeDiscriminant`.")
            }
        }
    }
}

impl Display for TypeDiscriminants {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match self {
            TypeDiscriminants::I32 => "I32".to_string(),
            TypeDiscriminants::F32 => "F32".to_string(),
            TypeDiscriminants::U32 => "U32".to_string(),
            TypeDiscriminants::U8 => "U8".to_string(),
            TypeDiscriminants::String => "String".to_string(),
            TypeDiscriminants::Boolean => "Boolean".to_string(),
            TypeDiscriminants::Void => "Void".to_string(),
            TypeDiscriminants::Struct((struct_name, _)) => format!("Struct({struct_name})"),
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
    dest_type: TypeDiscriminants,
) -> Result<Type, ParserError> {
    let parsed_num = raw_string
        .parse::<f64>()
        .map_err(|_| ParserError::InvalidTypeCast(raw_string.clone(), dest_type.clone()))?;

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
        TypeDiscriminants::Struct(inner) => {
            return Err(ParserError::InvalidTypeCast(
                raw_string,
                TypeDiscriminants::Struct(inner),
            ));
        }
    };

    Ok(casted_var)
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
            TypeDiscriminants::I32 | TypeDiscriminants::Void | TypeDiscriminants::Struct(_) => {
                unreachable!()
            }
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

            TypeDiscriminants::F32 | TypeDiscriminants::Void | TypeDiscriminants::Struct(_) => {
                unreachable!()
            }
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

            TypeDiscriminants::U32 | TypeDiscriminants::Void | TypeDiscriminants::Struct(_) => {
                unreachable!()
            }
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

            TypeDiscriminants::U8 | TypeDiscriminants::Void | TypeDiscriminants::Struct(_) => {
                unreachable!()
            }
        },
        Type::String(inner) => match dest_type {
            TypeDiscriminants::I32 => Type::I32(inner.parse::<i32>()?),
            TypeDiscriminants::F32 => Type::F32(inner.parse::<f32>()?),
            TypeDiscriminants::U32 => Type::U32(inner.parse::<u32>()?),
            TypeDiscriminants::U8 => Type::U8(inner.parse::<u8>()?),
            TypeDiscriminants::Boolean => Type::Boolean(inner.parse::<bool>()?),

            TypeDiscriminants::String | TypeDiscriminants::Void | TypeDiscriminants::Struct(_) => {
                unreachable!()
            }
        },
        Type::Boolean(inner) => match dest_type {
            TypeDiscriminants::I32 => Type::I32(inner as i32),
            TypeDiscriminants::F32 => Type::F32(inner as i32 as f32),
            TypeDiscriminants::U32 => Type::U32(inner as u32),
            TypeDiscriminants::U8 => Type::U8(inner as u8),
            TypeDiscriminants::String => Type::String(inner.to_string()),

            TypeDiscriminants::Boolean | TypeDiscriminants::Void | TypeDiscriminants::Struct(_) => {
                unreachable!()
            }
        },
        Type::Void | Type::Struct(_) => unreachable!(),
    };

    Ok(return_val)
}
