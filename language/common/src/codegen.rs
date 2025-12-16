use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use crate::{
    DEFAULT_COMPILER_ADDRESS_SPACE_SIZE,
    error::{codegen::CodeGenError, parser::ParserError, syntax::SyntaxError},
    parser::{FunctionSignature, ParsedToken, ParsedTokenInstance},
    tokenizer::Token,
    ty::{OrdMap, TypeDiscriminant, token_to_ty},
};
use anyhow::Result;
use indexmap::IndexMap;
use inkwell::{
    AddressSpace, FloatPredicate, IntPredicate,
    basic_block::BasicBlock,
    context::Context,
    types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FunctionType},
    values::PointerValue,
};
use strum::Display;

/// All of the custom types implemented by the User are defined here
#[derive(Debug, Clone, PartialEq, Display)]
pub enum CustomType
{
    Struct((String, OrdMap<String, TypeDiscriminant>)),
    Enum(OrdMap<String, TypeDiscriminant>),
    // First argument is the struct's name which the Extend extends
    // The second argument is the list of functions the stuct is being extended with
    // Extend(String, IndexMap<String, FunctionDefinition>),
}

/// These are used to define Imports.
/// Function symbols are manually defined to be imported.
#[derive(Debug, Clone, Default)]
pub struct Imports(HashMap<String, FunctionSignature>);

impl DerefMut for Imports
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.0
    }
}

impl Deref for Imports
{
    type Target = HashMap<String, FunctionSignature>;

    fn deref(&self) -> &Self::Target
    {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct If
{
    pub condition: Box<ParsedTokenInstance>,

    pub complete_body: Vec<ParsedTokenInstance>,
    pub incomplete_body: Vec<ParsedTokenInstance>,
}

#[derive(Debug, Clone, Display, PartialEq, Eq, Hash)]
pub enum Order
{
    Equal,
    NotEqual,
    Bigger,
    EqBigger,
    Smaller,
    EqSmaller,
}

impl Order
{
    pub fn from_token(token: &Token) -> anyhow::Result<Self>
    {
        match token {
            Token::Equal => Ok(Self::Equal),
            Token::NotEqual => Ok(Self::NotEqual),
            Token::Bigger => Ok(Self::Bigger),
            Token::EqBigger => Ok(Self::EqBigger),
            Token::Smaller => Ok(Self::Smaller),
            Token::EqSmaller => Ok(Self::EqSmaller),

            _ => {
                Err(
                    ParserError::SyntaxError(SyntaxError::InvalidTokenComparisonUsage(
                        token.clone(),
                    ))
                    .into(),
                )
            },
        }
    }
    pub fn into_int_predicate(&self, signed: bool) -> IntPredicate
    {
        if signed {
            match self {
                Order::Equal => IntPredicate::EQ,
                Order::NotEqual => IntPredicate::NE,
                Order::Bigger => IntPredicate::SGT,
                Order::EqBigger => IntPredicate::SGE,
                Order::Smaller => IntPredicate::SLT,
                Order::EqSmaller => IntPredicate::SLE,
            }
        }
        else {
            match self {
                Order::Equal => IntPredicate::EQ,
                Order::NotEqual => IntPredicate::NE,
                Order::Bigger => IntPredicate::UGT,
                Order::EqBigger => IntPredicate::UGE,
                Order::Smaller => IntPredicate::ULT,
                Order::EqSmaller => IntPredicate::ULE,
            }
        }
    }

    pub fn into_float_predicate(&self) -> FloatPredicate
    {
        match self {
            Order::Equal => FloatPredicate::OEQ,
            Order::NotEqual => FloatPredicate::ONE,
            Order::Bigger => FloatPredicate::OGT,
            Order::EqBigger => FloatPredicate::OGE,
            Order::Smaller => FloatPredicate::OLT,
            Order::EqSmaller => FloatPredicate::OLE,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PreAllocationEntry<'ctx>
{
    AllocationMap(HashMap<ParsedToken, PreAllocationEntry<'ctx>>),
    PreAllocationPtr(
        (
            PointerValue<'ctx>,
            BasicMetadataTypeEnum<'ctx>,
            TypeDiscriminant,
        ),
    ),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum FunctionArgumentIdentifier<IDENT, IDX>
{
    Identifier(IDENT),
    Index(IDX),
}

/// This function takes the field of a struct, and returns the fields' [`BasicTypeEnum`] variant.
/// The returned types are in order with the struct's fields
pub fn struct_field_to_ty_list<'a>(
    ctx: &'a Context,
    struct_inner: &IndexMap<String, TypeDiscriminant>,
    custom_types: Arc<IndexMap<String, CustomType>>,
) -> Result<Vec<BasicTypeEnum<'a>>>
{
    // Allocate a new list for storing the types
    let mut type_list = Vec::new();

    // Iterate over the struct's fields and convert the types into BasicTypeEnums
    for (_, ty) in struct_inner.iter() {
        // Convert the ty
        let basic_ty = ty_to_llvm_ty(ctx, ty, custom_types.clone())?;

        // Store the ty
        type_list.push(basic_ty);
    }

    Ok(type_list)
}

/// Converts a `TypeDiscriminant` into a `BasicTypeEnum` which can be used by inkwell.
pub fn ty_to_llvm_ty<'a>(
    ctx: &'a Context,
    ty: &TypeDiscriminant,
    custom_types: Arc<IndexMap<String, CustomType>>,
) -> Result<BasicTypeEnum<'a>>
{
    let bool_type = ctx.bool_type();
    let i8_type = ctx.i8_type();
    let i16_type = ctx.i16_type();
    let i32_type = ctx.i32_type();
    let f16_type = ctx.f16_type();
    let f32_type = ctx.f32_type();
    let i64_type = ctx.i64_type();
    let f64_type = ctx.f64_type();
    let ptr_type = ctx.ptr_type(AddressSpace::from(DEFAULT_COMPILER_ADDRESS_SPACE_SIZE));

    // Pattern match the type
    let field_ty = match ty {
        TypeDiscriminant::I32 => BasicTypeEnum::IntType(i32_type),
        TypeDiscriminant::F32 => BasicTypeEnum::FloatType(f32_type),
        TypeDiscriminant::U32 => BasicTypeEnum::IntType(i32_type),
        TypeDiscriminant::U8 => BasicTypeEnum::IntType(i8_type),
        TypeDiscriminant::String => BasicTypeEnum::PointerType(ptr_type),
        TypeDiscriminant::Boolean => BasicTypeEnum::IntType(bool_type),
        TypeDiscriminant::Void => {
            return Err(CodeGenError::InvalidVoidValue.into());
        },
        TypeDiscriminant::Struct((struct_name, struct_inner)) => {
            // If we are creating a new struct based on the TypeDiscriminant, we should first check if there is a struct created with the name
            let struct_type = if let Some(struct_type) = ctx.get_struct_type(struct_name) {
                // If we have already created a struct with this name, return the struct type
                struct_type
            }
            // If there are no existing struct with this name, create a new named struct
            else {
                // Create a named struct
                let op_struct_type = ctx.opaque_struct_type(struct_name);

                // Set the body of the struct
                op_struct_type.set_body(
                    &struct_field_to_ty_list(ctx, struct_inner, custom_types.clone())?,
                    false,
                );

                // Return the type of the struct
                op_struct_type
            };

            BasicTypeEnum::StructType(struct_type)
        },
        TypeDiscriminant::I64 => BasicTypeEnum::IntType(i64_type),
        TypeDiscriminant::F64 => BasicTypeEnum::FloatType(f64_type),
        TypeDiscriminant::U64 => BasicTypeEnum::IntType(i64_type),
        TypeDiscriminant::I16 => BasicTypeEnum::IntType(i16_type),
        TypeDiscriminant::F16 => BasicTypeEnum::FloatType(f16_type),
        TypeDiscriminant::U16 => BasicTypeEnum::IntType(i16_type),
        TypeDiscriminant::Array((token_ty, len)) => {
            let llvm_ty = ty_to_llvm_ty(
                ctx,
                &token_to_ty(&(*token_ty).clone(), &custom_types)?,
                custom_types.clone(),
            )?;

            let array_ty = llvm_ty.array_type(*len as u32);

            inkwell::types::BasicTypeEnum::ArrayType(array_ty)
        },
        TypeDiscriminant::Pointer => BasicTypeEnum::PointerType(ptr_type),
    };

    Ok(field_ty)
}

pub fn ty_enum_to_metadata_ty_enum(ty_enum: BasicTypeEnum<'_>) -> BasicMetadataTypeEnum<'_>
{
    match ty_enum {
        BasicTypeEnum::ArrayType(array_type) => BasicMetadataTypeEnum::ArrayType(array_type),
        BasicTypeEnum::FloatType(float_type) => BasicMetadataTypeEnum::FloatType(float_type),
        BasicTypeEnum::IntType(int_type) => BasicMetadataTypeEnum::IntType(int_type),
        BasicTypeEnum::PointerType(pointer_type) => {
            BasicMetadataTypeEnum::PointerType(pointer_type)
        },
        BasicTypeEnum::StructType(struct_type) => BasicMetadataTypeEnum::StructType(struct_type),
        BasicTypeEnum::VectorType(vector_type) => BasicMetadataTypeEnum::VectorType(vector_type),
        BasicTypeEnum::ScalableVectorType(_scalable_vector_type) => todo!(),
    }
}

pub fn fn_arg_to_string(fn_name: &str, fn_arg: &FunctionArgumentIdentifier<String, usize>)
-> String
{
    match fn_arg {
        FunctionArgumentIdentifier::Identifier(ident) => ident.to_string(),
        FunctionArgumentIdentifier::Index(idx) => {
            format!("{fn_name}_idx_{idx}_arg")
        },
    }
}

/// Serves as a way to store information about the current loop body we are currently in.
#[derive(Debug, Clone)]
pub struct LoopBodyBlocks<'ctx>
{
    /// The BasicBlock of the loop's body
    pub loop_body: BasicBlock<'ctx>,

    /// The BasicBlock of the code's continuation. This gets executed when we break out of the `loop_body`.
    pub loop_body_exit: BasicBlock<'ctx>,
}

impl<'ctx> LoopBodyBlocks<'ctx>
{
    pub fn new(loop_body: BasicBlock<'ctx>, loop_body_exit: BasicBlock<'ctx>) -> Self
    {
        Self {
            loop_body,
            loop_body_exit,
        }
    }
}

/// Creates a function type from a FunctionSignature.
/// It uses the Function's return type and arguments to create a `FunctionType` which can be used later in llvm context.
pub fn create_fn_type_from_ty_disc(
    ctx: &Context,
    fn_sig: FunctionSignature,
    custom_types: Arc<IndexMap<String, CustomType>>,
) -> Result<FunctionType<'_>>
{
    // Make an exception if the return type is Void
    if fn_sig.return_type == TypeDiscriminant::Void {
        return Ok(ctx.void_type().fn_type(
            &get_args_from_sig(ctx, fn_sig.clone(), custom_types.clone())?,
            false,
        ));
    }

    // Create an LLVM type
    let llvm_ty = ty_to_llvm_ty(ctx, &fn_sig.return_type, custom_types.clone())?;

    // Create the actual function type and parse the function's arguments
    Ok(llvm_ty.fn_type(
        &get_args_from_sig(ctx, fn_sig.clone(), custom_types.clone())?,
        false, /* Variable arguments can not be used on source code defined functions */
    ))
}

/// Fetches the arguments (and converts it into an LLVM type) from the function's signature
pub fn get_args_from_sig(
    ctx: &Context,
    fn_sig: FunctionSignature,
    custom_types: Arc<IndexMap<String, CustomType>>,
) -> Result<Vec<BasicMetadataTypeEnum<'_>>>
{
    // Create an iterator over the function's arguments
    let fn_args = fn_sig.args.arguments_list.iter();

    // Create a list for all the arguments
    let mut arg_list: Vec<BasicMetadataTypeEnum> = vec![];

    // Iter over all the arguments and store the converted variants of the argument types
    for (_arg_name, arg_ty) in fn_args {
        // Create an llvm ty
        let argument_sig = ty_to_llvm_ty(ctx, arg_ty, custom_types.clone())?;

        // Convert the type and store it
        arg_list.push(argument_sig.into());
    }

    // Return the list
    Ok(arg_list)
}
