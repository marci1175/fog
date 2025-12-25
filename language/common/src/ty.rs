use std::{
    fmt::{Debug, Display},
    hash::Hash,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use indexmap::{IndexMap, IndexSet};
use inkwell::{
    AddressSpace,
    context::Context,
    types::{BasicType, BasicTypeEnum},
};
use num::Float;
use strum::EnumTryAs;
use strum_macros::Display;

use crate::{
    DEFAULT_COMPILER_ADDRESS_SPACE_SIZE,
    codegen::{CustomType, struct_field_to_ty_list},
    error::{DebugInformation, parser::ParserError},
    tokenizer::Token,
};

#[derive(Debug, Clone, Display, Default, PartialEq, Eq, Hash)]
pub enum Value
{
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

    Struct((String, OrdMap<String, Value>)),

    /// First item is the type of the array
    /// Second item is the length
    Array((Box<Token>, usize)),

    Pointer((usize, Option<Box<Token>>)),
}

#[derive(Debug, Clone)]
pub struct NotNan<T>(T);

impl<T: Float> NotNan<T>
{
    pub fn new(inner: T) -> Result<Self, ParserError>
    {
        if inner.is_nan() {
            return Err(ParserError::FloatIsNAN);
        }

        Ok(Self(inner))
    }
}

impl NotNan<f16>
{
    pub fn new_f16(inner: f16) -> Result<Self, ParserError>
    {
        if inner.is_nan() {
            return Err(ParserError::FloatIsNAN);
        }

        Ok(Self(inner))
    }
}

impl<T: Debug + PartialEq> PartialEq for NotNan<T>
{
    fn eq(&self, other: &Self) -> bool
    {
        self.0 == other.0
    }
}

impl<T> Deref for NotNan<T>
{
    type Target = T;

    fn deref(&self) -> &Self::Target
    {
        &self.0
    }
}

impl<T> DerefMut for NotNan<T>
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.0
    }
}

impl From<f64> for NotNan<f64>
{
    fn from(value: f64) -> Self
    {
        Self(value)
    }
}

impl From<f32> for NotNan<f32>
{
    fn from(value: f32) -> Self
    {
        Self(value)
    }
}

impl From<f16> for NotNan<f16>
{
    fn from(value: f16) -> Self
    {
        Self(value)
    }
}

impl<T: FloatBits> Hash for NotNan<T>
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H)
    {
        state.write_u64(self.0.to_bits());
    }
}

trait FloatBits
{
    fn to_bits(&self) -> u64;
}

impl FloatBits for f16
{
    fn to_bits(&self) -> u64
    {
        f16::to_bits(*self) as u64
    }
}

impl FloatBits for f32
{
    fn to_bits(&self) -> u64
    {
        f32::to_bits(*self) as u64
    }
}

impl FloatBits for f64
{
    fn to_bits(&self) -> u64
    {
        f64::to_bits(*self)
    }
}

impl<T: PartialEq + Debug> Eq for NotNan<T> {}

impl Value
{
    pub fn discriminant(&self) -> Type
    {
        match self {
            Value::I64(_) => Type::I64,
            Value::F64(_) => Type::F64,
            Value::U64(_) => Type::U64,
            Value::I32(_) => Type::I32,
            Value::F32(_) => Type::F32,
            Value::U32(_) => Type::U32,
            Value::I16(_) => Type::I16,
            Value::F16(_) => Type::F16,
            Value::U16(_) => Type::U16,
            Value::U8(_) => Type::U8,
            Value::String(_) => Type::String,
            Value::Boolean(_) => Type::Boolean,
            Value::Void => Type::Void,
            Value::Struct((struct_name, struct_fields)) => {
                let mut struct_field_ty_list = OrdMap::new();

                for (name, ty) in struct_fields.iter() {
                    struct_field_ty_list.insert(name.clone(), ty.discriminant());
                }

                Type::Struct((struct_name.clone(), struct_field_ty_list))
            },
            Value::Array(inner) => Type::Array(inner.clone()),
            Value::Pointer((_, inner_ty)) => Type::Pointer(inner_ty.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Eq, Hash, EnumTryAs)]
pub enum Type
{
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

    /// Automatic type casting is not implemented for enum variants due to it being ineffecient and difficult with the current codebase. (aka im too lazy)
    Enum((Box<Type>, OrdMap<String, (Value, DebugInformation)>)),

    Struct((String, OrdMap<String, Type>)),
    Array((Box<Token>, usize)),
    Pointer(Option<Box<Token>>),
}

impl Type
{
    pub fn is_float(&self) -> bool
    {
        matches!(self, Self::F64 | Self::F32 | Self::F16)
    }

    pub fn is_int(&self) -> bool
    {
        matches!(
            self,
            Self::I64 | Self::I32 | Self::I16 | Self::U64 | Self::U32 | Self::U16 | Self::U8
        )
    }

    /// Returns DWARF encoding for a type. For more complex types see: [`generate_debug_type_from_type_disc`].
    /// Reference arcticle: https://dwarfstd.org/doc/DWARF5.pdf
    pub fn get_dwarf_encoding(&self) -> u32
    {
        match self {
            Self::I64 | Self::I32 | Self::I16 => 5,
            Self::U64 | Self::U32 | Self::U16 | Self::U8 => 7,
            Self::F64 | Self::F32 | Self::F16 => 4,
            Self::Boolean => 2,
            Self::String => 12,
            Self::Struct(_) => 13,
            Self::Pointer(_) => 15,
            Self::Array(_) => 1,
            Self::Enum(_) => 4,
            _ => panic!("DWARF identifier requested on invalid type."),
        }
    }

    pub fn sizeof(&self, custom_types: Arc<IndexMap<String, CustomType>>) -> usize
    {
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
            Self::Struct((_, fields)) => {
                fields
                    .iter()
                    .map(|(_, ty)| ty.sizeof(custom_types.clone()))
                    .sum()
            },
            Self::Enum((inner_ty, _)) => inner_ty.sizeof(custom_types.clone()),
            Self::Array((inner, _)) => {
                token_to_ty(inner, &custom_types)
                    .unwrap()
                    .sizeof(custom_types.clone())
            },
            Self::Pointer(_) => std::mem::size_of::<usize>(),
        }
    }

    pub fn to_basic_type_enum(
        self,
        ctx: &Context,
        custom_types: Arc<IndexMap<String, CustomType>>,
    ) -> anyhow::Result<BasicTypeEnum<'_>>
    {
        let basic_ty = match self {
            Type::I64 => BasicTypeEnum::IntType(ctx.i64_type()),
            Type::F64 => BasicTypeEnum::FloatType(ctx.f64_type()),
            Type::U64 => BasicTypeEnum::IntType(ctx.i64_type()),
            Type::I32 => BasicTypeEnum::IntType(ctx.i32_type()),
            Type::F32 => BasicTypeEnum::FloatType(ctx.f32_type()),
            Type::U32 => BasicTypeEnum::IntType(ctx.i32_type()),
            Type::I16 => BasicTypeEnum::IntType(ctx.i16_type()),
            Type::F16 => BasicTypeEnum::FloatType(ctx.f16_type()),
            Type::U16 => BasicTypeEnum::IntType(ctx.i16_type()),
            Type::U8 => BasicTypeEnum::IntType(ctx.i8_type()),
            Type::String => {
                BasicTypeEnum::PointerType(
                    ctx.ptr_type(AddressSpace::from(DEFAULT_COMPILER_ADDRESS_SPACE_SIZE)),
                )
            },
            Type::Boolean => BasicTypeEnum::IntType(ctx.bool_type()),
            Type::Void => unimplemented!("A BasicTypeEnum cannot be a `Void` type."),
            Type::Struct((_struct_name, fields)) => {
                BasicTypeEnum::StructType(ctx.struct_type(
                    &struct_field_to_ty_list(ctx, &fields, custom_types.clone())?,
                    false,
                ))
            },
            Type::Array((array_ty, len)) => {
                BasicTypeEnum::ArrayType(
                    token_to_ty(&array_ty, &custom_types)?
                        .to_basic_type_enum(ctx, custom_types.clone())?
                        .array_type(len as u32),
                )
            },
            Type::Enum((ty, _)) => ty.to_basic_type_enum(ctx, custom_types.clone())?,
            Type::Pointer(_) => {
                BasicTypeEnum::PointerType(
                    ctx.ptr_type(AddressSpace::from(size_of::<usize>() as u16)),
                )
            },
        };

        Ok(basic_ty)
    }
}

impl From<Type> for Value
{
    fn from(value: Type) -> Self
    {
        match value {
            Type::I64 => Self::I64(0),
            Type::F64 => Self::F64(NotNan::new(0.0).unwrap()),
            Type::U64 => Self::U64(0),
            Type::I32 => Self::I32(0),
            Type::F32 => Self::F32(NotNan::new(0.0).unwrap()),
            Type::U32 => Self::U32(0),
            Type::I16 => Self::I16(0),
            Type::F16 => Self::F16(NotNan::new_f16(0.0).unwrap()),
            Type::U16 => Self::U16(0),
            Type::U8 => Self::U8(0),
            Type::String => Self::String(String::new()),
            Type::Boolean => Self::Boolean(false),
            Type::Void => Self::Void,
            Type::Struct(_) => {
                unimplemented!("Cannot create a Custom type from a `TypeDiscriminant`.")
            },
            Type::Enum(_) => {
                unimplemented!("Cannot create a Custom type from a `TypeDiscriminant`.")
            },
            Type::Array(array) => Self::Array(array),
            Type::Pointer(_) => Self::Pointer((0, None)),
        }
    }
}

impl Display for Type
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_str(&match self {
            Type::I64 => "I64".to_string(),
            Type::F64 => "F64".to_string(),
            Type::U64 => "U64".to_string(),
            Type::I16 => "I16".to_string(),
            Type::F16 => "F16".to_string(),
            Type::U16 => "U16".to_string(),
            Type::I32 => "I32".to_string(),
            Type::F32 => "F32".to_string(),
            Type::U32 => "U32".to_string(),
            Type::U8 => "U8".to_string(),
            Type::String => "String".to_string(),
            Type::Boolean => "Boolean".to_string(),
            Type::Void => "Void".to_string(),
            Type::Struct((struct_name, _)) => format!("Struct({struct_name})"),
            Type::Array((inner_ty, len)) => {
                format!("Array(ty: {inner_ty}, len:{len})")
            },
            Type::Pointer(inner_ty) => format!("Ptr<{:?}>", inner_ty),
            Type::Enum((ty, _)) => format!("Enum<{ty}>"),
        })
    }
}

// TODO: Rework this
pub fn unparsed_const_to_typed_literal_unsafe(
    raw_string: String,
    dest_type: Option<Type>,
) -> Result<Value, ParserError>
{
    let parsed_val = if let Some(dest_type) = dest_type {
        let parsed_num = raw_string
            .parse::<f64>()
            .map_err(|_| ParserError::InvalidTypeCast(raw_string.clone(), dest_type.clone()))?;

        match dest_type {
            Type::I64 => {
                if parsed_num.floor() != parsed_num {
                    return Err(ParserError::InvalidTypeCast(
                        parsed_num.to_string(),
                        Type::I64,
                    ));
                }
                else {
                    Value::I64(parsed_num as i64)
                }
            },
            Type::F64 => Value::F64(parsed_num.into()),
            Type::U64 => {
                if parsed_num.floor() != parsed_num {
                    return Err(ParserError::InvalidTypeCast(
                        parsed_num.to_string(),
                        Type::U64,
                    ));
                }
                else {
                    Value::U64(parsed_num as u64)
                }
            },
            Type::I16 => {
                if parsed_num.floor() != parsed_num {
                    return Err(ParserError::InvalidTypeCast(
                        parsed_num.to_string(),
                        Type::I16,
                    ));
                }
                else {
                    Value::I16(parsed_num as i16)
                }
            },
            Type::F16 => Value::F16(NotNan::new_f16(parsed_num as f16)?),
            Type::U16 => {
                if parsed_num.floor() != parsed_num {
                    return Err(ParserError::InvalidTypeCast(
                        parsed_num.to_string(),
                        Type::U16,
                    ));
                }
                else {
                    Value::U16(parsed_num as u16)
                }
            },
            Type::I32 => {
                if parsed_num.floor() != parsed_num {
                    return Err(ParserError::InvalidTypeCast(
                        parsed_num.to_string(),
                        Type::I32,
                    ));
                }
                else {
                    Value::I32(parsed_num as i32)
                }
            },
            Type::F32 => Value::F32(NotNan::new(parsed_num as f32)?),
            Type::U32 => {
                if parsed_num.floor() != parsed_num {
                    return Err(ParserError::InvalidTypeCast(
                        parsed_num.to_string(),
                        Type::U32,
                    ));
                }
                else {
                    Value::U32(parsed_num as u32)
                }
            },
            Type::U8 => {
                if parsed_num.floor() != parsed_num {
                    return Err(ParserError::InvalidTypeCast(
                        parsed_num.to_string(),
                        Type::U32,
                    ));
                }
                else {
                    Value::U8(parsed_num as u8)
                }
            },
            Type::String => {
                return Err(ParserError::InvalidTypeCast(
                    parsed_num.to_string(),
                    Type::String,
                ));
            },
            Type::Boolean => {
                if parsed_num == 1.0 {
                    Value::Boolean(true)
                }
                else if parsed_num == 0.0 {
                    Value::Boolean(false)
                }
                else {
                    return Err(ParserError::InvalidTypeCast(
                        raw_string.clone(),
                        Type::Boolean,
                    ));
                }
            },
            Type::Void => Value::Void,
            Type::Struct(inner) => {
                return Err(ParserError::InvalidTypeCast(
                    raw_string,
                    Type::Struct(inner),
                ));
            },
            Type::Array(inner) => {
                return Err(ParserError::InvalidTypeCast(raw_string, Type::Array(inner)));
            },
            Type::Pointer(_) => {
                if parsed_num.floor() != parsed_num {
                    return Err(ParserError::InvalidTypeCast(
                        parsed_num.to_string(),
                        Type::I16,
                    ));
                }
                else {
                    Value::Pointer((parsed_num as usize, None))
                }
            },
            Type::Enum(inner) => {
                return Err(ParserError::InvalidTypeCast(raw_string, Type::Enum(inner)));
            },
        }
    }
    else {
        let parsed_num = raw_string
            .parse::<f64>()
            .map_err(|_| ParserError::ValueTypeUnknown(raw_string.clone()))?;

        if raw_string.contains('.') {
            Value::F64(parsed_num.into())
        }
        else {
            Value::I64(parsed_num as i64)
        }
    };

    Ok(parsed_val)
}

/// This custom wrapper type is for implementing [`Hash`] for [`IndexMap`].
/// The type implements its own custom [`PartialEq`] in which the order of the items matter. Therefor, two maps with the same items with a different order will not be equal.
#[derive(Debug, Clone, Default)]
pub struct OrdMap<K, V>(IndexMap<K, V>);

impl<K, V> Deref for OrdMap<K, V>
{
    type Target = IndexMap<K, V>;

    fn deref(&self) -> &Self::Target
    {
        &self.0
    }
}

impl<K, V> DerefMut for OrdMap<K, V>
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.0
    }
}

/// Implement PartialEq for the wrapper type so that it can be used in the hash implementation later.
impl<K: PartialEq + Hash, V: PartialEq> PartialEq for OrdMap<K, V>
{
    fn eq(&self, other: &Self) -> bool
    {
        self.iter()
            .enumerate()
            .all(|(idx, (k, v))| other.get_index(idx) == Some((k, v)))
            && other
                .iter()
                .enumerate()
                .all(|(idx, (k, v))| self.get_index(idx) == Some((k, v)))
    }
}

/// Implement hashing for the wrapper type.
impl<K: Hash, V: Hash> Hash for OrdMap<K, V>
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H)
    {
        for (k, v) in &self.0 {
            k.hash(state);
            v.hash(state);
        }
    }
}

impl<K: PartialEq + Hash, V: PartialEq> Eq for OrdMap<K, V> {}

impl<K, V> OrdMap<K, V>
{
    pub fn new() -> Self
    {
        OrdMap(IndexMap::new())
    }
}

impl<K: Hash + Eq + Clone, V: Clone> OrdMap<K, V>
{
    pub fn extend_clone(&self, rhs: Self) -> Self
    {
        let mut self_clone = self.clone();

        self_clone.extend(rhs.iter().map(|(k, v)| (k.clone(), v.clone())));

        self_clone
    }
}

impl<K, V> From<IndexMap<K, V>> for OrdMap<K, V>
{
    fn from(value: IndexMap<K, V>) -> Self
    {
        Self(value)
    }
}

#[derive(Debug, Clone)]
pub struct OrdSet<T>(IndexSet<T>);

impl<T> Deref for OrdSet<T>
{
    type Target = IndexSet<T>;

    fn deref(&self) -> &Self::Target
    {
        &self.0
    }
}

impl<T> DerefMut for OrdSet<T>
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.0
    }
}

impl<T: Eq> PartialEq for OrdSet<T>
{
    fn eq(&self, other: &Self) -> bool
    {
        if self.0.len() != other.0.len() {
            return false;
        }

        self.0.iter().zip(other.0.iter()).all(|(a, b)| a == b)
    }
}

/// Blanket implementation of the `Eq` trait.
impl<T: Eq> Eq for OrdSet<T> {}

impl<T> Default for OrdSet<T>
{
    fn default() -> Self
    {
        Self(IndexSet::default())
    }
}

impl<T: Hash> Hash for OrdSet<T>
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H)
    {
        for elem in self.0.iter() {
            elem.hash(state);
        }
    }
}

impl<T: Hash + Eq + Clone> OrdSet<T>
{
    pub fn new() -> Self
    {
        Self::default()
    }

    pub fn wrap(inner: IndexSet<T>) -> Self
    {
        Self(inner)
    }

    pub fn from_vec(vec: Vec<T>) -> Self
    {
        let mut set = Self::new();

        vec.iter().for_each(|item| {
            set.0.insert(item.clone());
        });

        set
    }
}

pub fn token_to_ty(
    token: &Token,
    custom_types: &IndexMap<String, CustomType>,
) -> anyhow::Result<Type>
{
    match &token {
        Token::Identifier(ident) => {
            if let Some(custom_type) = custom_types.get(ident) {
                match custom_type {
                    CustomType::Struct(struct_def) => Ok(Type::Struct(struct_def.clone())),
                    CustomType::Enum(_ord_map) => unimplemented!(),
                }
            }
            else {
                Err(ParserError::InvalidType(vec![token.clone()]).into())
            }
        },
        Token::TypeDefinition(type_def) => Ok(type_def.clone()),

        _ => Err(ParserError::InvalidType(vec![token.clone()]).into()),
    }
}
