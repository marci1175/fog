use std::{
    fmt::{Debug, Display},
    hash::Hash,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use crate::{
    DEFAULT_COMPILER_ADDRESS_SPACE_SIZE,
    codegen::{CustomItem, StructAttributes, struct_field_to_ty_list},
    error::{codegen::CodeGenError, parser::ParserError},
    parser::{
        common::ParsedTokenInstance,
        function::{FunctionDefinition, FunctionSignature},
    },
    tokenizer::Token,
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

    Struct(
        (
            String,
            OrdMap<String, Type>,
            OrdMap<String, Box<ParsedTokenInstance>>,
            StructAttributes,
        ),
    ),

    /// First item is the type of the array
    /// Second item is the length
    Array((Box<Token>, usize)),
    Enum((Type, OrdMap<String, ParsedTokenInstance>, String)),
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
            Value::Struct((struct_name, struct_fields, _struct_values, attr)) => {
                let mut struct_field_ty_list = OrdMap::new();

                for (name, ty) in struct_fields.iter() {
                    struct_field_ty_list.insert(name.clone(), ty.clone());
                }

                Type::Struct((struct_name.clone(), struct_field_ty_list, attr.clone()))
            },
            Value::Array(inner) => Type::Array(inner.clone()),
            Value::Enum((ty, body, _)) => Type::Enum((Box::new(ty.clone()), body.clone())),
            Value::Pointer((_, inner_ty)) => Type::Pointer(inner_ty.clone()),
        }
    }
}

#[derive(Debug, Clone, Default, Eq, Hash, EnumTryAs)]
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
    Enum((Box<Type>, OrdMap<String, ParsedTokenInstance>)),

    Struct((String, OrdMap<String, Type>, StructAttributes)),
    Array((Box<Token>, usize)),
    Pointer(Option<Box<Token>>),
    TraitGeneric
    {
        name: String,
        functions: OrdMap<String, FunctionSignature>,
    },
}

impl PartialEq for Type
{
    fn eq(&self, other: &Self) -> bool
    {
        match (self, other) {
            (Self::Enum(l0), Self::Enum(r0)) => l0 == r0,
            (Self::Struct(l0), Self::Struct(r0)) => l0 == r0,
            (Self::Array(l0), Self::Array(r0)) => l0 == r0,
            (Self::Pointer(l0), Self::Pointer(r0)) => l0 == r0,
            (
                Self::TraitGeneric {
                    name: l_name,
                    functions: l_functions,
                },
                Self::TraitGeneric {
                    name: r_name,
                    functions: r_functions,
                },
            ) => l_name == r_name && l_functions == r_functions,
            // Implement specific logic cmp for Traits
            (
                Self::TraitGeneric {
                    name: trait_name, ..
                },
                Self::Struct((_, _, attr)),
            ) => attr.traits.contains_key(trait_name),
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }

    fn ne(&self, other: &Self) -> bool
    {
        !self.eq(other)
    }
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
    /// Reference arcticle: <https://dwarfstd.org/doc/DWARF5.pdf>
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

    pub fn sizeof(&self, custom_types: Rc<IndexMap<String, CustomItem>>) -> usize
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
            Self::Struct((_, fields, _)) => {
                fields
                    .iter()
                    .map(|(_, ty)| ty.sizeof(custom_types.clone()))
                    .sum()
            },
            Self::Enum((inner_ty, _)) => inner_ty.sizeof(custom_types.clone()),
            Self::Array((inner, _)) => {
                ty_from_token(inner, &custom_types)
                    .unwrap()
                    .sizeof(custom_types.clone())
            },
            Self::Pointer(_) => std::mem::size_of::<usize>(),
            Self::TraitGeneric {
                functions: inner_type,
                ..
            } => 0,
        }
    }

    pub fn to_basic_type_enum<'a>(
        &self,
        ctx: &'a Context,
        custom_types: Rc<IndexMap<String, CustomItem>>,
    ) -> anyhow::Result<BasicTypeEnum<'a>>
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
            Type::Void => return Err(CodeGenError::InvalidVoidValue.into()),
            Type::Struct((_struct_name, fields, _)) => {
                BasicTypeEnum::StructType(ctx.struct_type(
                    &struct_field_to_ty_list(ctx, fields, custom_types.clone())?,
                    false,
                ))
            },
            Type::Array((array_ty, len)) => {
                BasicTypeEnum::ArrayType(
                    ty_from_token(array_ty, &custom_types)?
                        .to_basic_type_enum(ctx, custom_types.clone())?
                        .array_type(*len as u32),
                )
            },
            Type::Enum((ty, _)) => ty.to_basic_type_enum(ctx, custom_types.clone())?,
            Type::Pointer(_) => {
                BasicTypeEnum::PointerType(
                    ctx.ptr_type(AddressSpace::from(size_of::<usize>() as u16)),
                )
            },
            Type::TraitGeneric { .. } => return Err(CodeGenError::TraitGenericIsNotType.into()),
        };

        Ok(basic_ty)
    }

    /// Returns the inner type of an enum, if it is an enum.
    /// This function is made so that code can be shortened
    /// Be cautious when using this function to ensure correctness in the codebase.
    pub fn try_get_enum_inner(self) -> Self
    {
        if let Self::Enum((inner_ty, _)) = self {
            return *inner_ty;
        }

        self
    }
}

impl Type
{
    pub fn into_value_default(&self) -> Value
    {
        match self {
            Self::I64 => Value::I64(0),
            Self::F64 => Value::F64(NotNan::new(0.0).unwrap()),
            Self::U64 => Value::U64(0),
            Self::I32 => Value::I32(0),
            Self::F32 => Value::F32(NotNan::new(0.0).unwrap()),
            Self::U32 => Value::U32(0),
            Self::I16 => Value::I16(0),
            Self::F16 => Value::F16(NotNan::new_f16(0.0).unwrap()),
            Self::U16 => Value::U16(0),
            Self::U8 => Value::U8(0),
            Self::String => Value::String(String::new()),
            Self::Boolean => Value::Boolean(false),
            Self::Void => Value::Void,
            Self::Struct(_) => {
                unimplemented!("Cannot create a Custom type from a `TypeDiscriminant`.")
            },
            Self::Enum(_) => {
                unimplemented!("Cannot create a Custom type from a `TypeDiscriminant`.")
            },
            Self::Array(array) => Value::Array(array.to_owned()),
            Self::Pointer(_) => Value::Pointer((0, None)),
            Self::TraitGeneric { .. } => {
                unimplemented!("Cannot create a Custom type from a `TypeDiscriminant`.")
            },
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
            Type::Struct((struct_name, _, _)) => format!("Struct({struct_name})"),
            Type::Array((inner_ty, len)) => {
                format!("Array(ty: {inner_ty}, len:{len})")
            },
            Type::Pointer(inner_ty) => format!("Ptr<{:?}>", inner_ty),
            Type::Enum((ty, _)) => format!("Enum<{ty}>"),
            Type::TraitGeneric {
                functions: inner_type,
                name: trait_name,
            } => format!("TraitGeneric({trait_name})<{inner_type:#?}>"),
        })
    }
}

/// If None is passed in as a destination type try to guess the value of the literal.
/// That means that we want to cast the value produced by the function to a pre-determined Type.
pub fn unparsed_const_to_typed_literal_unsafe(
    raw_string: &str,
    dest_type: Option<Type>,
) -> Result<Value, ParserError>
{
    let val = match dest_type.clone() {
        Some(Type::I64) => {
            Value::I64(raw_string.parse::<i64>().map_err(|_| {
                ParserError::InvalidTypeCast(raw_string.to_string(), dest_type.unwrap())
            })?)
        },
        Some(Type::F64) => {
            Value::F64(NotNan(raw_string.parse::<f64>().map_err(|_| {
                ParserError::InvalidTypeCast(raw_string.to_string(), dest_type.unwrap())
            })?))
        },
        Some(Type::U64) => {
            Value::U64(raw_string.parse::<u64>().map_err(|_| {
                ParserError::InvalidTypeCast(raw_string.to_string(), dest_type.unwrap())
            })?)
        },
        Some(Type::I16) => {
            Value::I16(raw_string.parse::<i16>().map_err(|_| {
                ParserError::InvalidTypeCast(raw_string.to_string(), dest_type.unwrap())
            })?)
        },
        Some(Type::F16) => {
            Value::F16(NotNan(raw_string.parse::<f16>().map_err(|_| {
                ParserError::InvalidTypeCast(raw_string.to_string(), dest_type.unwrap())
            })?))
        },
        Some(Type::U16) => {
            Value::U16(raw_string.parse::<u16>().map_err(|_| {
                ParserError::InvalidTypeCast(raw_string.to_string(), dest_type.unwrap())
            })?)
        },
        Some(Type::I32) => {
            Value::I32(raw_string.parse::<i32>().map_err(|_| {
                ParserError::InvalidTypeCast(raw_string.to_string(), dest_type.unwrap())
            })?)
        },
        Some(Type::F32) => {
            Value::F32(NotNan(raw_string.parse::<f32>().map_err(|_| {
                ParserError::InvalidTypeCast(raw_string.to_string(), dest_type.unwrap())
            })?))
        },
        Some(Type::U32) => {
            Value::U32(raw_string.parse::<u32>().map_err(|_| {
                ParserError::InvalidTypeCast(raw_string.to_string(), dest_type.unwrap())
            })?)
        },
        Some(Type::U8) => {
            Value::U8(raw_string.parse::<u8>().map_err(|_| {
                ParserError::InvalidTypeCast(raw_string.to_string(), dest_type.unwrap())
            })?)
        },
        Some(Type::String) => {
            return Err(ParserError::InvalidTypeCast(
                raw_string.to_string(),
                Type::String,
            ));
        },
        Some(Type::Boolean) => {
            Value::Boolean(raw_string.parse::<bool>().map_err(|_| {
                ParserError::InvalidTypeCast(raw_string.to_string(), dest_type.unwrap())
            })?)
        },
        Some(Type::Void) => Value::Void,
        Some(Type::Struct(inner)) => {
            return Err(ParserError::InvalidTypeCast(
                raw_string.to_string(),
                Type::Struct(inner),
            ));
        },
        Some(Type::Array(inner)) => {
            return Err(ParserError::InvalidTypeCast(
                raw_string.to_string(),
                Type::Array(inner),
            ));
        },
        Some(Type::Pointer(ref ptr_ty)) => {
            Value::Pointer((
                raw_string.parse::<usize>().map_err(|_| {
                    ParserError::InvalidTypeCast(raw_string.to_string(), dest_type.unwrap())
                })?,
                ptr_ty.clone(),
            ))
        },
        Some(Type::Enum(inner)) => {
            return Err(ParserError::InvalidTypeCast(
                raw_string.to_string(),
                Type::Enum(inner),
            ));
        },
        Some(Type::TraitGeneric {
            functions: inner_type,
            name: trait_name,
        }) => {
            return Err(ParserError::InvalidTypeCast(
                raw_string.to_string(),
                Type::TraitGeneric {
                    functions: inner_type,
                    name: trait_name,
                },
            ));
        },
        None => {
            let negative_flag = raw_string.get(0..1) == Some("-");
            let float_flag = raw_string.as_bytes().contains(&b'.');

            if raw_string == "true" {
                Value::Boolean(true)
            }
            else if raw_string == "false" {
                Value::Boolean(false)
            }
            else if float_flag {
                Value::F64(NotNan(raw_string.parse::<f64>().map_err(|_| {
                    ParserError::InvalidTypeCast(raw_string.to_string(), dest_type.unwrap())
                })?))
            }
            else if negative_flag {
                Value::I64(raw_string.parse::<i64>().map_err(|_| {
                    ParserError::InvalidTypeCast(raw_string.to_string(), dest_type.unwrap())
                })?)
            }
            else {
                Value::U64(raw_string.parse::<u64>().map_err(|_| {
                    ParserError::InvalidTypeCast(raw_string.to_string(), dest_type.unwrap())
                })?)
            }
        },
    };
    Ok(val)
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

pub fn ty_from_token(
    token: &Token,
    custom_types: &IndexMap<String, CustomItem>,
) -> anyhow::Result<Type>
{
    match &token {
        Token::Identifier(ident) => {
            if let Some(custom_type) = custom_types.get(ident) {
                match custom_type.clone() {
                    CustomItem::Struct(struct_def) => Ok(Type::Struct(struct_def)),
                    CustomItem::Enum((ty, body)) => Ok(Type::Enum((Box::new(ty), body))),
                    // TODO: Make it so that Trait types exist. It will basically mean that any struct can be passed in to this arg which implements this trait
                    // This is a type interface, this isnt a concrete type
                    CustomItem::Trait { name, functions } => {
                        Ok(Type::TraitGeneric { name, functions })
                    },
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
