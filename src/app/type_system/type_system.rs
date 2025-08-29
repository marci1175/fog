use std::{
    fmt::{Debug, Display},
    hash::Hash,
    ops::{Deref, DerefMut},
};

use indexmap::IndexMap;
use num::Float;
use strum_macros::Display;

use crate::app::parser::error::ParserError;

#[derive(Debug, Clone, Display, Default, PartialEq, Eq, Hash)]
pub enum Type {
    I64(i64),
    F64(NotNan<f64>),
    U64(u64),

    I32(i32),
    F32(NotNan<f32>),
    U32(u32),

    I16(i16),
    F16(NotNan<f16>),
    U16(u16),

    U8(u8),

    String(String),
    Boolean(bool),

    #[default]
    Void,

    Struct((String, OrdMap<String, Type>)),
    Array((Box<TypeDiscriminant>, usize)),
}

#[derive(Debug, Clone)]
pub struct NotNan<T>(T);

impl<T: Float> NotNan<T> {
    pub fn new(inner: T) -> Result<Self, ParserError> {
        if inner.is_nan() {
            return Err(ParserError::FloatIsNAN);
        }

        Ok(Self(inner))
    }
}

impl NotNan<f16> {
    pub fn new_f16(inner: f16) -> Result<Self, ParserError> {
        if inner.is_nan() {
            return Err(ParserError::FloatIsNAN);
        }

        Ok(Self(inner))
    }
}

impl<T: Debug + PartialEq> PartialEq for NotNan<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Deref for NotNan<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for NotNan<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<f64> for NotNan<f64> {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl From<f32> for NotNan<f32> {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl From<f16> for NotNan<f16> {
    fn from(value: f16) -> Self {
        Self(value)
    }
}

impl<T: FloatBits> Hash for NotNan<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.0.to_bits());
    }
}

trait FloatBits {
    fn to_bits(&self) -> u64;
}

impl FloatBits for f16 {
    fn to_bits(&self) -> u64 {
        f16::to_bits(*self) as u64
    }
}

impl FloatBits for f32 {
    fn to_bits(&self) -> u64 {
        f32::to_bits(*self) as u64
    }
}

impl FloatBits for f64 {
    fn to_bits(&self) -> u64 {
        f64::to_bits(*self)
    }
}

impl<T: PartialEq + Debug> Eq for NotNan<T> {}

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
                let mut struct_field_ty_list = OrdMap::new();

                for (name, ty) in struct_fields.iter() {
                    struct_field_ty_list.insert(name.clone(), ty.discriminant());
                }

                TypeDiscriminant::Struct((struct_name.clone(), struct_field_ty_list))
            }
            Type::Array(inner) => TypeDiscriminant::Array(inner.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Eq, Hash)]
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

    Struct((String, OrdMap<String, TypeDiscriminant>)),
    Array((Box<TypeDiscriminant>, usize)),
}

impl TypeDiscriminant {
    pub fn is_float(&self) -> bool {
        matches!(self, Self::F64 | Self::F32 | Self::F16)
    }

    pub fn is_int(&self) -> bool {
        matches!(
            self,
            Self::I64 | Self::I32 | Self::I16 | Self::U64 | Self::U32 | Self::U16 | Self::U8
        )
    }

    pub fn sizeof(&self) -> usize {
        match self {
            Self::I64 => std::mem::size_of::<i64>(),
            Self::F64 => std::mem::size_of::<f64>(),
            Self::U64 => std::mem::size_of::<u64>(),
            Self::I32 => std::mem::size_of::<i32>(),
            Self::F32 => std::mem::size_of::<f32>(),
            Self::U32 => std::mem::size_of::<u32>(),
            Self::I16 => std::mem::size_of::<i16>(),
            Self::F16 => std::mem::size_of::<f16>(),
            Self::U16 => std::mem::size_of::<u16>(),
            Self::U8 => std::mem::size_of::<u8>(),
            Self::String => std::mem::size_of::<String>(),
            Self::Boolean => std::mem::size_of::<bool>(),
            Self::Void => 0,
            Self::Struct((_, fields)) => fields.iter().map(|(_, ty)| ty.sizeof()).sum(),
            Self::Array((inner, _)) => inner.sizeof(),
        }
    }
}

impl From<TypeDiscriminant> for Type {
    fn from(value: TypeDiscriminant) -> Self {
        match value {
            TypeDiscriminant::I64 => Self::I64(0),
            TypeDiscriminant::F64 => Self::F64(NotNan::new(0.0).unwrap()),
            TypeDiscriminant::U64 => Self::U64(0),
            TypeDiscriminant::I32 => Self::I32(0),
            TypeDiscriminant::F32 => Self::F32(NotNan::new(0.0).unwrap()),
            TypeDiscriminant::U32 => Self::U32(0),
            TypeDiscriminant::I16 => Self::I16(0),
            TypeDiscriminant::F16 => Self::F16(NotNan::new_f16(0.0).unwrap()),
            TypeDiscriminant::U16 => Self::U16(0),
            TypeDiscriminant::U8 => Self::U8(0),
            TypeDiscriminant::String => Self::String(String::new()),
            TypeDiscriminant::Boolean => Self::Boolean(false),
            TypeDiscriminant::Void => Self::Void,
            TypeDiscriminant::Struct(_) => {
                unimplemented!("Cannot create a Custom type from a `TypeDiscriminant`.")
            }
            TypeDiscriminant::Array(_) => todo!(),
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
            TypeDiscriminant::Array((inner_ty, len)) => {
                format!("Vector(ty: {inner_ty}, len:{len})")
            }
        })
    }
}

pub fn unparsed_const_to_typed_literal_unsafe(
    raw_string: String,
    dest_type: Option<TypeDiscriminant>,
) -> Result<Type, ParserError> {
    let parsed_val = if let Some(dest_type) = dest_type {
        let parsed_num = raw_string
            .parse::<f64>()
            .map_err(|_| ParserError::InvalidTypeCast(raw_string.clone(), dest_type.clone()))?;

        match dest_type {
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
            TypeDiscriminant::F64 => Type::F64(parsed_num.into()),
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
            TypeDiscriminant::F16 => Type::F16(NotNan::new_f16(parsed_num as f16)?),
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
            TypeDiscriminant::F32 => Type::F32(NotNan::new(parsed_num as f32)?),
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
            TypeDiscriminant::Array(inner) => {
                return Err(ParserError::InvalidTypeCast(
                    raw_string,
                    TypeDiscriminant::Array(inner),
                ));
            }
        }
    } else {
        let parsed_num = raw_string
            .parse::<f64>()
            .map_err(|_| ParserError::ValueTypeUnknown(raw_string.clone()))?;

        if raw_string.contains('.') {
            Type::F64(parsed_num.into())
        } else {
            Type::I64(parsed_num as i64)
        }
    };

    Ok(parsed_val)
}

/// This custom wrapper type is for implementing [`Hash`] for [`IndexMap`].
/// The type implements its own custom [`PartialEq`] in which the order of the items matter. Therefor, two maps with the same items with a different order will not be equal.
#[derive(Debug, Clone, Default)]
pub struct OrdMap<K, V>(IndexMap<K, V>);

impl<K, V> Deref for OrdMap<K, V> {
    type Target = IndexMap<K, V>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K, V> DerefMut for OrdMap<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Implement PartialEq for the wrapper type so that it can be used in the hash implementation later.
impl<K: PartialEq + Hash, V: PartialEq> PartialEq for OrdMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.iter()
            .enumerate()
            .all(|(idx, (k, v))| other.get_index(idx) == Some((k, v)))
    }
}

/// Implement hashing for the wrapper type.
impl<K: Hash, V: Hash> Hash for OrdMap<K, V> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for (k, v) in &self.0 {
            k.hash(state);
            v.hash(state);
        }
    }
}

impl<K: PartialEq + Hash, V: PartialEq> Eq for OrdMap<K, V> {}

impl<K, V> OrdMap<K, V> {
    pub fn new() -> Self {
        OrdMap(IndexMap::new())
    }
}

impl<K: Hash + Eq + Clone, V: Clone> OrdMap<K, V> {
    pub fn extend_clone(&self, rhs: Self) -> Self {
        let mut self_clone = self.clone();

        self_clone.extend(rhs.iter().map(|(k, v)| (k.clone(), v.clone())));

        self_clone
    }
}

impl<K, V> From<IndexMap<K, V>> for OrdMap<K, V> {
    fn from(value: IndexMap<K, V>) -> Self {
        Self(value)
    }
}
