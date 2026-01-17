use common::{
    DEFAULT_COMPILER_ADDRESS_SPACE_SIZE,
    anyhow::Result,
    codegen::{CustomType, LoopBodyBlocks, ty_to_llvm_ty},
    error::{codegen::CodeGenError, parser::ParserError},
    indexmap::IndexMap,
    inkwell::{
        AddressSpace,
        basic_block::BasicBlock,
        builder::Builder,
        context::Context,
        module::Module,
        types::{BasicMetadataTypeEnum, BasicTypeEnum, PointerType, StructType},
        values::{FunctionValue, PointerValue},
    },
    parser::{
        common::{ParsedToken, ParsedTokenInstance},
        function::FunctionDefinition,
        variable::{StructFieldRef, UniqueId, VariableReference},
    },
    ty::{Type, Value, ty_from_token},
};
use std::{
    collections::{HashMap, VecDeque},
    rc::Rc,
    slice::Iter,
    sync::Arc,
};

use crate::{allocate::create_new_variable, create_ir_from_parsed_token};

pub fn access_variable_ptr<'ctx>(
    ctx: &'ctx Context,
    module: &Module<'ctx>,
    fn_ret_ty: &Type,
    this_fn_block: BasicBlock<'ctx>,
    this_fn: FunctionValue<'ctx>,
    allocation_list: &HashMap<UniqueId, PointerValue<'ctx>>,
    is_loop_body: &Option<LoopBodyBlocks<'_>>,
    parsed_functions: Rc<IndexMap<String, FunctionDefinition>>,
    builder: &'ctx Builder,
    variable_reference: Box<VariableReference>,
    variable_map: &mut HashMap<
        String,
        (
            (PointerValue<'ctx>, BasicMetadataTypeEnum<'ctx>),
            (Type, UniqueId),
        ),
    >,
    custom_types: Rc<IndexMap<String, CustomType>>,
) -> Result<(PointerValue<'ctx>, BasicTypeEnum<'ctx>, Type)>
{
    match &*variable_reference {
        VariableReference::StructFieldReference(struct_field_ref) => {
            if let Some((idx, _, field_ty)) = struct_field_ref
                .struct_fields
                .get_full(&struct_field_ref.field_name)
            {
                // Get the ptr to the struct
                let (ptr, ptr_ty, _ty) = access_variable_ptr(
                    ctx,
                    module,
                    fn_ret_ty,
                    this_fn_block,
                    this_fn,
                    allocation_list,
                    is_loop_body,
                    parsed_functions,
                    builder,
                    struct_field_ref.variable_ref.clone(),
                    variable_map,
                    custom_types.clone(),
                )?;

                // Get the ptr to the field of the struct
                let field_ptr = builder.build_struct_gep(
                    ptr_ty,
                    ptr,
                    idx as u32,
                    &format!(
                        "get_{}_from_{}",
                        struct_field_ref.field_name, struct_field_ref.struct_name
                    ),
                )?;

                return Ok((
                    field_ptr,
                    ty_to_llvm_ty(ctx, field_ty, custom_types)?,
                    field_ty.clone(),
                ));
            }
            else {
                // if the struct doesnt have this field return an error
                return Err(CodeGenError::InternalStructFieldNotFound.into());
            }
        },
        VariableReference::BasicReference(variable_name, variable_id) => {
            let ((ptr, _ptr_ty), (variable_type, _)) =
                variable_map
                    .get(variable_name)
                    .ok_or(CodeGenError::InternalVariableNotFound(
                        variable_name.to_string(),
                    ))?;

            return Ok((
                ptr.clone(),
                ty_to_llvm_ty(ctx, &variable_type, custom_types)?,
                variable_type.clone(),
            ));
        },
        VariableReference::ArrayReference(array_indexing) => {
            // Get the underlying variable we are trying to index into
            let (ptr, ptr_ty, ty) = access_variable_ptr(
                ctx,
                module,
                fn_ret_ty,
                this_fn_block,
                this_fn,
                allocation_list,
                is_loop_body,
                parsed_functions.clone(),
                builder,
                array_indexing.variable_reference.clone(),
                variable_map,
                custom_types.clone(),
            )?;

            let (elem_ptr, _elem_ty, ty) = access_array_index(
                ctx,
                module,
                builder,
                variable_map,
                fn_ret_ty,
                this_fn_block,
                this_fn,
                allocation_list,
                is_loop_body,
                &parsed_functions,
                &custom_types,
                ((ptr, ptr_ty.into()), ty),
                array_indexing.idx.clone(),
            )?;

            return Ok((elem_ptr, ty_to_llvm_ty(ctx, &ty, custom_types)?, ty));
        },
    }
}

/// This function takes in the variable pointer which is dereferenced to set the variable's value.
/// Ensure that we are setting variable type `T` with value `T`
pub fn set_value_of_ptr<'ctx>(
    ctx: &'ctx Context,
    builder: &'ctx Builder,
    module: &Module<'ctx>,
    value: Value,
    v_ptr: PointerValue<'_>,
    custom_types: Rc<IndexMap<String, CustomType>>,
    variable_map: &mut HashMap<
        String,
        (
            (PointerValue<'ctx>, BasicMetadataTypeEnum<'ctx>),
            (Type, UniqueId),
        ),
    >,
    fn_ret_ty: &Type,
    this_fn_block: BasicBlock<'ctx>,
    this_fn: FunctionValue<'ctx>,
    allocation_table: &HashMap<UniqueId, PointerValue<'ctx>>,
    is_loop_body: &Option<LoopBodyBlocks<'_>>,
    parsed_functions: Rc<IndexMap<String, FunctionDefinition>>,
) -> Result<()>
{
    let bool_type = ctx.bool_type();
    let i8_type = ctx.i8_type();
    let i32_type = ctx.i32_type();
    let f32_type = ctx.f32_type();
    let f64_type = ctx.f64_type();
    let i64_type = ctx.i64_type();
    let i16_type = ctx.i16_type();
    let f16_type = ctx.f16_type();
    let ptr_type = ctx.ptr_type(AddressSpace::from(DEFAULT_COMPILER_ADDRESS_SPACE_SIZE));

    match value {
        Value::I64(inner) => {
            // Initialize const value
            let init_val = i64_type.const_int(inner as u64, true);

            // Store const
            builder.build_store(v_ptr, init_val)?;
        },
        Value::F64(inner) => {
            // Initialize const value
            let init_val = f64_type.const_float(*inner);

            // Store const
            builder.build_store(v_ptr, init_val)?;
        },
        Value::U64(inner) => {
            // Initialize const value
            let init_val = i64_type.const_int(inner, false);

            // Store const
            builder.build_store(v_ptr, init_val)?;
        },
        Value::I16(inner) => {
            // Initialize const value
            let init_val = i16_type.const_int(inner as u64, true);

            // Store const
            builder.build_store(v_ptr, init_val)?;
        },
        Value::F16(inner) => {
            // Initialize const value
            let init_val = f16_type.const_float(*inner as f64);

            // Store const
            builder.build_store(v_ptr, init_val)?;
        },
        Value::U16(inner) => {
            // Initialize const value
            let init_val = i16_type.const_int(inner as u64, false);

            // Store const
            builder.build_store(v_ptr, init_val)?;
        },
        Value::I32(inner) => {
            // Initialize const value
            let init_val = i32_type.const_int(inner as u64, true);

            // Store const
            builder.build_store(v_ptr, init_val)?;
        },
        Value::F32(inner) => {
            // Initialize const value
            let init_val = f32_type.const_float(*inner as f64);

            // Store const
            builder.build_store(v_ptr, init_val)?;
        },
        Value::U32(inner) => {
            // Initialize const value
            let init_val = i32_type.const_int(inner as u64, false);

            // Store const
            builder.build_store(v_ptr, init_val)?;
        },
        Value::U8(inner) => {
            // Initialize const value
            let init_val = i8_type.const_int(inner as u64, false);

            // Store const
            builder.build_store(v_ptr, init_val)?;
        },
        Value::String(inner) => {
            let string_bytes = inner.as_bytes();

            let char_array =
                ctx.const_string(string_bytes, Some(0) != string_bytes.last().copied());

            let global_string_handle = if let Some(global_string) = module.get_global(&inner) {
                global_string
            }
            else {
                let handle = module.add_global(
                    char_array.get_type(),
                    Some(AddressSpace::from(DEFAULT_COMPILER_ADDRESS_SPACE_SIZE)),
                    &inner,
                );

                handle.set_initializer(&char_array);
                handle.set_constant(true);

                handle
            };

            let buffer_ptr = global_string_handle.as_pointer_value();

            let input_ptr = unsafe {
                builder.build_gep(
                    char_array.get_type(),
                    buffer_ptr,
                    &[ctx.i32_type().const_zero(), ctx.i32_type().const_zero()],
                    "buf_ptr",
                )
            }?;

            // Store const
            builder.build_store(v_ptr, input_ptr)?;
        },
        Value::Boolean(inner) => {
            // Initialize const value
            let init_val = bool_type.const_int(inner as u64, false);

            // Store const
            builder.build_store(v_ptr, init_val)?;
        },
        Value::Void => {
            unreachable!()
        },
        Value::Struct((struct_name, struct_fields, struct_values)) => {
            // Get the struct pointer's ty
            let pointee_struct_ty = ty_to_llvm_ty(
                &ctx,
                &Type::Struct((struct_name, struct_fields.clone())),
                custom_types.clone(),
            )?
            .into_struct_type();

            // Pre-Allocate a struct so that it can be accessed later
            let allocate_struct = builder.build_alloca(pointee_struct_ty, "strct_init")?;

            // Iterate over the struct's fields
            for (field_idx, (field_name, field_ty)) in struct_fields.iter().enumerate() {
                // Convert to llvm type
                let llvm_ty = ty_to_llvm_ty(ctx, field_ty, custom_types.clone())?;

                // Create a new temp variable according to the struct's field type
                let (ptr, ty) = create_new_variable(
                    ctx,
                    builder,
                    field_name,
                    field_ty,
                    None,
                    allocation_table,
                    custom_types.clone(),
                )?;

                // Parse the value for the temp var
                create_ir_from_parsed_token(
                    ctx,
                    module,
                    builder,
                    *(struct_values.get(field_name).unwrap().clone()),
                    variable_map,
                    Some((field_name.to_string(), (ptr, ty), field_ty.clone())),
                    fn_ret_ty.clone(),
                    this_fn_block,
                    this_fn,
                    allocation_table,
                    is_loop_body.clone(),
                    parsed_functions.clone(),
                    custom_types.clone(),
                )?;

                // Load the temp value to memory and store it
                let temp_val = builder.build_load(llvm_ty, ptr, field_name)?;

                // Get the struct's field gep
                let struct_field_ptr = builder.build_struct_gep(
                    pointee_struct_ty,
                    allocate_struct,
                    field_idx as u32,
                    "field_gep",
                )?;

                // Store the temp value in the struct through the struct's field gep
                builder.build_store(struct_field_ptr, temp_val)?;
            }

            // Load the allocated struct into memory
            let constructed_struct = builder
                .build_load(pointee_struct_ty, allocate_struct, "constructed_struct")?
                .into_struct_value();

            // Store the struct in the main variable
            builder.build_store(v_ptr, constructed_struct)?;
        },
        Value::Array(inner_ty) => unimplemented!(),
        Value::Pointer((inner, _)) => {
            // Cast the integer to be a pointer since we cannot inherently create a pointer with a pre-determined destination
            let ptr = builder.build_int_to_ptr(i64_type.const_int(inner as u64, false), ptr_type, "raw_address_pointer")?;

            // LLVM does let us initalize a pointer type with a pre-determined address
            let store = builder.build_store(v_ptr, ptr)?;

            // Do not let llvm optimize it, cuz it can optimize out writes / reads
            // store.set_volatile(true)?;
        },
        Value::Enum((_ty, body, val)) => {
            set_value_of_ptr(
                &ctx,
                &builder,
                &module,
                body.get(&val)
                    .ok_or(ParserError::EnumVariantNotFound(val))?
                    .inner
                    .clone()
                    .try_as_literal()
                    .unwrap()
                    .clone(),
                v_ptr,
                custom_types.clone(),
                variable_map,
                fn_ret_ty,
                this_fn_block,
                this_fn,
                allocation_table,
                is_loop_body,
                parsed_functions,
            )?;
        },
    }

    Ok(())
}

pub fn access_array_index<'main, 'ctx>(
    ctx: &'main Context,
    module: &Module<'ctx>,
    builder: &'ctx Builder<'ctx>,
    variable_map: &mut HashMap<
        String,
        (
            (PointerValue<'ctx>, BasicMetadataTypeEnum<'ctx>),
            (Type, UniqueId),
        ),
    >,
    fn_ret_ty: &Type,
    this_fn_block: BasicBlock<'ctx>,
    this_fn: FunctionValue<'ctx>,
    allocation_list: &HashMap<UniqueId, PointerValue<'ctx>>,
    is_loop_body: &Option<LoopBodyBlocks<'_>>,
    parsed_functions: &Rc<IndexMap<String, FunctionDefinition>>,
    custom_types: &Rc<IndexMap<String, CustomType>>,
    ((array_ptr, _ptr_ty), ty_disc): ((PointerValue<'ctx>, BasicMetadataTypeEnum<'ctx>), Type),
    index: Box<ParsedTokenInstance>,
) -> Result<(PointerValue<'ctx>, BasicMetadataTypeEnum<'ctx>, Type)>
where
    'main: 'ctx,
{
    let index_val = create_ir_from_parsed_token(
        ctx,
        module,
        builder,
        *index.clone(),
        variable_map,
        None,
        fn_ret_ty.clone(),
        this_fn_block,
        this_fn,
        allocation_list,
        is_loop_body.clone(),
        parsed_functions.clone(),
        custom_types.clone(),
    )?;

    if let Some((idx_ptr, ptr_ty, idx_ty_disc)) = index_val {
        let idx = builder.build_load(
            ty_to_llvm_ty(ctx, &idx_ty_disc, custom_types.clone())?,
            idx_ptr,
            "array_idx_val",
        )?;

        let pointee_ty = ty_disc
            .clone()
            .to_basic_type_enum(ctx, custom_types.clone())?;

        let gep_ptr = unsafe {
            builder.build_gep(
                pointee_ty,
                array_ptr,
                &[ctx.i32_type().const_int(0, false), idx.into_int_value()],
                "array_idx_elem_ptr",
            )?
        };

        let (inner_ty_token, _len) = ty_disc.try_as_array().unwrap();
        let inner_ty = ty_from_token(&*inner_ty_token, custom_types)?;

        Ok((
            gep_ptr,
            inner_ty
                .clone()
                .to_basic_type_enum(ctx, custom_types.clone())?
                .into(),
            inner_ty.clone(),
        ))
    }
    else {
        Err(CodeGenError::InvalidIndexValue(index.inner.clone()).into())
    }
}
