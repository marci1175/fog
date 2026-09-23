use std::{
    collections::HashMap,
    rc::Rc,
    sync::Arc,
};

use crate::{
    DEFAULT_COMPILER_ADDRESS_SPACE_SIZE,
    error::codegen::CodeGenError,
    parser::{
        common::{CustomItem, StatementVariant},
        function::FunctionSignature,
    },
    ty::Type,
};
use anyhow::Result;
use indexmap::IndexMap;
use inkwell::{
    AddressSpace,
    basic_block::BasicBlock,
    context::Context,
    types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FunctionType},
    values::PointerValue,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PreAllocationEntry<'ctx>
{
    AllocationMap(HashMap<StatementVariant, PreAllocationEntry<'ctx>>),
    PreAllocationPtr((PointerValue<'ctx>, BasicMetadataTypeEnum<'ctx>, Type)),
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
    struct_inner: &IndexMap<String, Type>,
    custom_types: Rc<IndexMap<String, CustomItem>>,
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
    ty: &Type,
    custom_types: Rc<IndexMap<String, CustomItem>>,
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
        Type::I32 => BasicTypeEnum::IntType(i32_type),
        Type::F32 => BasicTypeEnum::FloatType(f32_type),
        Type::U32 => BasicTypeEnum::IntType(i32_type),
        Type::U8 => BasicTypeEnum::IntType(i8_type),
        Type::String => BasicTypeEnum::PointerType(ptr_type),
        Type::Boolean => BasicTypeEnum::IntType(bool_type),
        Type::Void => {
            return Err(CodeGenError::InvalidVoidValue.into());
        },
        Type::Struct((struct_name, struct_inner, _struct_attributes)) => {
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
        Type::Enum((ty, _)) => ty_to_llvm_ty(ctx, ty, custom_types.clone())?,
        Type::I64 => BasicTypeEnum::IntType(i64_type),
        Type::F64 => BasicTypeEnum::FloatType(f64_type),
        Type::U64 => BasicTypeEnum::IntType(i64_type),
        Type::I16 => BasicTypeEnum::IntType(i16_type),
        Type::F16 => BasicTypeEnum::FloatType(f16_type),
        Type::U16 => BasicTypeEnum::IntType(i16_type),
        Type::Array((token_ty, len)) => {
            let llvm_ty = ty_to_llvm_ty(ctx, token_ty, custom_types.clone())?;

            let array_ty = llvm_ty.array_type(*len as u32);

            inkwell::types::BasicTypeEnum::ArrayType(array_ty)
        },
        Type::Pointer(_) => BasicTypeEnum::PointerType(ptr_type),
        Type::Trait { .. } => {
            return Err(CodeGenError::TraitIsNotType.into());
        },
        Type::TraitObject { .. } => {
            return Err(CodeGenError::TraitIsNotType.into());
        },
        Type::Unresolved(_) => todo!(),
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
        BasicTypeEnum::ScalableVectorType(_scalable_vector_type) => {
            BasicMetadataTypeEnum::ScalableVectorType(_scalable_vector_type)
        },
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
    custom_types: Rc<IndexMap<String, CustomItem>>,
) -> Result<FunctionType<'_>>
{
    // Make an exception if the return type is Void
    if fn_sig.return_type == Type::Void {
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
    custom_types: Rc<IndexMap<String, CustomItem>>,
) -> Result<Vec<BasicMetadataTypeEnum<'_>>>
{
    // Create an iterator over the function's arguments
    let fn_args = fn_sig.args.arguments.iter();

    // Create a list for all the arguments
    let mut arg_list: Vec<BasicMetadataTypeEnum> = vec![];

    // Iter over all the arguments and store the converted variants of the argument types
    for (_arg_name, (arg_ty, _)) in fn_args {
        // Create an llvm ty
        let argument_sig = ty_to_llvm_ty(ctx, arg_ty, custom_types.clone())?;

        // Convert the type and store it
        arg_list.push(argument_sig.into());
    }

    // Return the list
    Ok(arg_list)
}

pub fn fetch_nested_pointer_ty(
    custom_types: &Arc<IndexMap<String, CustomItem>>,
    pointer_ty: Type,
) -> Result<Type>
{
    match pointer_ty.clone().try_as_pointer() {
        Some(pointer_inner) => {
            match pointer_inner {
                Some(inner_token) => Ok(fetch_nested_pointer_ty(custom_types, *inner_token)?),
                None => Ok(pointer_ty),
            }
        },
        None => Ok(pointer_ty),
    }
}
