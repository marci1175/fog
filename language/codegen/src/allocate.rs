use std::{
    collections::{HashMap, VecDeque},
    rc::Rc,
    sync::Arc,
};

use common::{
    anyhow::Result,
    codegen::{CustomType, ty_enum_to_metadata_ty_enum, ty_to_llvm_ty},
    error::codegen::CodeGenError,
    indexmap::IndexMap,
    inkwell::{
        basic_block::BasicBlock,
        builder::Builder,
        context::Context,
        module::Module,
        types::{ArrayType, BasicMetadataTypeEnum},
        values::{FunctionValue, IntValue, PointerValue},
    },
    parser::{FunctionDefinition, ParsedToken, ParsedTokenInstance},
    ty::Type,
};

use crate::{irgen::create_ir_from_parsed_token, pointer::access_nested_struct_field_ptr};

pub fn create_alloca_table<'main, 'ctx>(
    module: &Module<'ctx>,
    // Inkwell IR builder
    builder: &'ctx Builder<'ctx>,
    // Inkwell Context
    ctx: &'main Context,
    // The list of ParsedToken-s
    parsed_tokens: Vec<ParsedTokenInstance>,
    // Type returned type of the Function
    fn_ret_ty: Type,
    this_fn_block: BasicBlock<'ctx>,
    variable_map: &mut HashMap<String, ((PointerValue<'ctx>, BasicMetadataTypeEnum<'ctx>), Type)>,
    this_fn: FunctionValue<'ctx>,
    parsed_functions: Rc<IndexMap<String, FunctionDefinition>>,
    custom_items: Arc<IndexMap<String, CustomType>>,
) -> Result<
    VecDeque<(
        ParsedTokenInstance,
        PointerValue<'ctx>,
        BasicMetadataTypeEnum<'ctx>,
        Type,
    )>,
>
where
    'main: 'ctx,
{
    let mut alloc_list: VecDeque<(
        ParsedTokenInstance,
        PointerValue<'ctx>,
        BasicMetadataTypeEnum<'ctx>,
        Type,
    )> = VecDeque::new();

    for token in parsed_tokens {
        let allocations = fetch_alloca_ptr(
            ctx,
            module,
            builder,
            token.clone(),
            variable_map,
            fn_ret_ty.clone(),
            this_fn_block,
            this_fn,
            parsed_functions.clone(),
            custom_items.clone(),
        )?;

        alloc_list.extend(allocations);
    }

    Ok(dbg!(alloc_list))
}

/// This function returns a pointer to the allocation made by according to the specific [`ParsedToken`] which had been passed in.
/// It serves as a way to make allocations before entering a loop, to avoid stack overflows.
/// If no allocation had been made the function will return [`None`].
pub fn fetch_alloca_ptr<'main, 'ctx>(
    ctx: &'main Context,
    module: &Module<'ctx>,
    builder: &'ctx Builder<'ctx>,
    parsed_token_instance: ParsedTokenInstance,
    variable_map: &mut HashMap<String, ((PointerValue<'ctx>, BasicMetadataTypeEnum<'ctx>), Type)>,
    // Type returned type of the Function
    fn_ret_ty: Type,
    this_fn_block: BasicBlock<'ctx>,
    this_fn: FunctionValue<'ctx>,
    parsed_functions: Rc<IndexMap<String, FunctionDefinition>>,
    custom_types: Arc<IndexMap<String, CustomType>>,
) -> Result<
    Vec<(
        ParsedTokenInstance,
        PointerValue<'ctx>,
        BasicMetadataTypeEnum<'ctx>,
        Type,
    )>,
>
where
    'main: 'ctx,
{
    let mut pre_allocation_list: Vec<(
        ParsedTokenInstance,
        PointerValue<'_>,
        BasicMetadataTypeEnum<'_>,
        Type,
    )> = Vec::new();

    let parsed_token = parsed_token_instance.inner.clone();

    match parsed_token.clone() {
        ParsedToken::NewVariable(var_name, var_type, var_set_val) => {
            let (ptr, ty) = if let Some(((ptr, ty), _)) = variable_map.get(&var_name) {
                (*ptr, *ty)
            }
            else {
                let (ptr, ty) =
                    create_new_variable(ctx, builder, &var_name, &var_type, custom_types.clone())?;

                variable_map.insert(var_name.clone(), ((ptr, ty), var_type.clone()));

                (ptr, ty)
            };

            // We will pre-allocate the variable itself and we will also preallocate its value which will get loaded into this variable.
            pre_allocation_list.push((parsed_token_instance.clone(), ptr, ty, var_type.clone()));

            // We only set the value of the pre-allocated variable if its a constant, like if its a literal
            // This skips a step of setting the value in the loop, however this pre evaluation cannot be applied safely to all of the types
            // Check if the value is a literal
            // We also check if its a literal when we are checking for pre-allocated variables so that we dont set the value twice.
            if matches!(
                &*var_set_val,
                ParsedTokenInstance {
                    inner: ParsedToken::Literal(_),
                    debug_information: _
                }
            ) {
                // Set the value of the newly created variable
                create_ir_from_parsed_token(
                    ctx,
                    module,
                    builder,
                    *var_set_val.clone(),
                    variable_map,
                    Some((var_name.clone(), (ptr, ty), var_type.clone())),
                    fn_ret_ty.clone(),
                    this_fn_block,
                    this_fn,
                    &mut VecDeque::new(),
                    None,
                    parsed_functions.clone(),
                    custom_types.clone(),
                )?;

                pre_allocation_list.push(((*var_set_val).clone(), ptr, ty, var_type.clone()));
            }
            else {
                let allocas = fetch_alloca_ptr(
                    ctx,
                    module,
                    builder,
                    *var_set_val.clone(),
                    variable_map,
                    fn_ret_ty,
                    this_fn_block,
                    this_fn,
                    parsed_functions.clone(),
                    custom_types.clone(),
                )?;

                pre_allocation_list.extend(allocas);
            }
        },
        ParsedToken::VariableReference(var_ref) => {
            match var_ref {
                common::parser::VariableReference::StructFieldReference(
                    struct_field_stack,
                    (struct_name, struct_def),
                ) => {
                    let mut field_stack_iter = struct_field_stack.field_stack.iter();

                    if let Some(main_struct_var_name) = field_stack_iter.next() {
                        if let Some(((ptr, ty), _)) = variable_map.get(main_struct_var_name) {
                            let (f_ptr, f_ty, ty_disc) = access_nested_struct_field_ptr(
                                ctx,
                                builder,
                                &mut field_stack_iter,
                                &struct_def,
                                (*ptr, *ty),
                                custom_types.clone(),
                            )?;

                            pre_allocation_list.push((
                                parsed_token_instance.clone(),
                                f_ptr,
                                ty_enum_to_metadata_ty_enum(f_ty),
                                ty_disc,
                            ));
                        }
                        else {
                            return Err(CodeGenError::InternalVariableNotFound(
                                main_struct_var_name.clone(),
                            )
                            .into());
                        }
                    }
                    else {
                        return Err(CodeGenError::InternalInvalidStructReference.into());
                    }
                },
                common::parser::VariableReference::BasicReference(name) => {
                    if let Some(((ptr, ty), disc)) = variable_map.get(&name) {
                        pre_allocation_list.push((
                            parsed_token_instance.clone(),
                            *ptr,
                            *ty,
                            disc.clone(),
                        ));
                    }
                },
                common::parser::VariableReference::ArrayReference(_, parsed_tokens) => {
                    todo!()
                },
            }
        },
        ParsedToken::Literal(literal) => {
            let var_type = literal.discriminant();

            let (ptr, ty) = create_new_variable(ctx, builder, "", &var_type, custom_types.clone())?;

            pre_allocation_list.push((parsed_token_instance.clone(), ptr, ty, var_type));
        },
        ParsedToken::TypeCast(parsed_token, desired_type) => {
            let created_var = create_ir_from_parsed_token(
                ctx,
                module,
                builder,
                parsed_token_instance.clone(),
                variable_map,
                None,
                fn_ret_ty,
                this_fn_block,
                this_fn,
                &mut VecDeque::new(),
                None,
                parsed_functions.clone(),
                custom_types.clone(),
            )?;

            if let Some((var_ptr, var_ty, ty_disc)) = created_var {
                let returned_alloca = match ty_disc {
                    Type::I64 | Type::I32 | Type::I16 => {
                        match desired_type {
                            Type::I64 => Some((var_ptr, var_ty, Type::I64)),
                            Type::F64 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res = builder.build_signed_int_to_float(
                                    value,
                                    ctx.f64_type(),
                                    "casted_value",
                                )?;

                                let allocation =
                                    builder.build_alloca(cast_res.get_type(), "cast_result")?;

                                builder.build_store(allocation, cast_res)?;

                                Some((
                                    allocation,
                                    BasicMetadataTypeEnum::FloatType(cast_res.get_type()),
                                    desired_type,
                                ))
                            },
                            Type::U64 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res =
                                    builder.build_int_cast(value, ctx.i64_type(), "i64_to_u64")?;

                                let allocation =
                                    builder.build_alloca(cast_res.get_type(), "cast_result")?;

                                builder.build_store(allocation, cast_res)?;

                                Some((
                                    allocation,
                                    BasicMetadataTypeEnum::IntType(cast_res.get_type()),
                                    desired_type,
                                ))
                            },
                            Type::I32 | Type::U32 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res = builder.build_int_truncate(
                                    value,
                                    ctx.i32_type(),
                                    "i64_to_i32",
                                )?;

                                let allocation =
                                    builder.build_alloca(cast_res.get_type(), "cast_result")?;

                                builder.build_store(allocation, cast_res)?;

                                Some((
                                    allocation,
                                    BasicMetadataTypeEnum::IntType(cast_res.get_type()),
                                    desired_type,
                                ))
                            },
                            Type::F32 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res = builder.build_signed_int_to_float(
                                    value,
                                    ctx.f32_type(),
                                    "casted_value",
                                )?;

                                let allocation =
                                    builder.build_alloca(cast_res.get_type(), "cast_result")?;

                                builder.build_store(allocation, cast_res)?;

                                Some((
                                    allocation,
                                    BasicMetadataTypeEnum::FloatType(cast_res.get_type()),
                                    desired_type,
                                ))
                            },
                            Type::I16 | Type::U16 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res = builder.build_int_truncate(
                                    value,
                                    ctx.i16_type(),
                                    "i64_to_i32",
                                )?;

                                let allocation =
                                    builder.build_alloca(cast_res.get_type(), "cast_result")?;

                                builder.build_store(allocation, cast_res)?;

                                Some((
                                    allocation,
                                    BasicMetadataTypeEnum::IntType(cast_res.get_type()),
                                    desired_type,
                                ))
                            },
                            Type::F16 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res = builder.build_signed_int_to_float(
                                    value,
                                    ctx.f16_type(),
                                    "casted_value",
                                )?;

                                let allocation =
                                    builder.build_alloca(cast_res.get_type(), "cast_result")?;

                                builder.build_store(allocation, cast_res)?;

                                Some((
                                    allocation,
                                    BasicMetadataTypeEnum::FloatType(cast_res.get_type()),
                                    desired_type,
                                ))
                            },
                            Type::U8 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res = builder.build_int_truncate(
                                    value,
                                    ctx.i8_type(),
                                    "i64_to_i32",
                                )?;

                                let allocation =
                                    builder.build_alloca(cast_res.get_type(), "cast_result")?;

                                builder.build_store(allocation, cast_res)?;

                                Some((
                                    allocation,
                                    BasicMetadataTypeEnum::IntType(cast_res.get_type()),
                                    desired_type,
                                ))
                            },
                            Type::String => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let raw_val = value.get_sign_extended_constant().unwrap();

                                let int_string = raw_val.to_string();

                                let (buf_ptr, buf_ty) =
                                    allocate_string(builder, ctx.i8_type(), int_string)?;

                                Some((
                                    buf_ptr,
                                    BasicMetadataTypeEnum::ArrayType(buf_ty),
                                    desired_type,
                                ))
                            },
                            Type::Boolean => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let bool_ty = ctx.bool_type();

                                let bool_value = if value.get_sign_extended_constant().unwrap() == 0
                                {
                                    bool_ty.const_int(0, false)
                                }
                                else {
                                    bool_ty.const_int(1, false)
                                };

                                let allocation =
                                    builder.build_alloca(bool_value.get_type(), "cast_result")?;

                                builder.build_store(allocation, bool_value)?;

                                Some((
                                    allocation,
                                    BasicMetadataTypeEnum::IntType(bool_value.get_type()),
                                    desired_type,
                                ))
                            },
                            Type::Void => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            },
                            Type::Struct(_) => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            },
                            Type::Array(_) => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            },
                            Type::Pointer(_) => todo!(),
                            Type::Enum(_) => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            },
                        }
                    },
                    Type::F64 | Type::F32 | Type::F16 => {
                        match desired_type {
                            Type::I64 => {
                                let cast_res = builder.build_float_to_signed_int(
                                    builder
                                        .build_load(var_ty.into_float_type(), var_ptr, "")?
                                        .into_float_value(),
                                    ctx.i64_type(),
                                    "",
                                )?;

                                let allocation =
                                    builder.build_alloca(cast_res.get_type(), "cast_result")?;

                                builder.build_store(allocation, cast_res)?;

                                Some((
                                    allocation,
                                    BasicMetadataTypeEnum::IntType(cast_res.get_type()),
                                    desired_type,
                                ))
                            },
                            Type::F64 => Some((var_ptr, var_ty, Type::F64)),
                            Type::U64 => {
                                let cast_res = builder.build_float_to_unsigned_int(
                                    builder
                                        .build_load(var_ty.into_float_type(), var_ptr, "")?
                                        .into_float_value(),
                                    ctx.i64_type(),
                                    "",
                                )?;

                                let allocation =
                                    builder.build_alloca(cast_res.get_type(), "cast_result")?;

                                builder.build_store(allocation, cast_res)?;

                                Some((
                                    allocation,
                                    BasicMetadataTypeEnum::IntType(cast_res.get_type()),
                                    desired_type,
                                ))
                            },
                            Type::I32 => {
                                let cast_res = builder.build_float_to_signed_int(
                                    builder
                                        .build_load(var_ty.into_float_type(), var_ptr, "")?
                                        .into_float_value(),
                                    ctx.i32_type(),
                                    "",
                                )?;
                                let allocation =
                                    builder.build_alloca(cast_res.get_type(), "cast_result")?;

                                builder.build_store(allocation, cast_res)?;

                                Some((
                                    allocation,
                                    BasicMetadataTypeEnum::IntType(cast_res.get_type()),
                                    desired_type,
                                ))
                            },
                            Type::F32 => {
                                let value = builder
                                    .build_load(var_ty.into_float_type(), var_ptr, "")?
                                    .into_float_value();

                                let cast_res =
                                    ctx.f32_type().const_float(value.get_constant().unwrap().0);

                                let allocation =
                                    builder.build_alloca(cast_res.get_type(), "cast_result")?;

                                builder.build_store(allocation, cast_res)?;

                                Some((
                                    allocation,
                                    BasicMetadataTypeEnum::FloatType(cast_res.get_type()),
                                    desired_type,
                                ))
                            },
                            Type::U32 => {
                                let cast_res = builder.build_float_to_unsigned_int(
                                    builder
                                        .build_load(var_ty.into_float_type(), var_ptr, "")?
                                        .into_float_value(),
                                    ctx.i32_type(),
                                    "",
                                )?;
                                let allocation =
                                    builder.build_alloca(cast_res.get_type(), "cast_result")?;

                                builder.build_store(allocation, cast_res)?;

                                Some((
                                    allocation,
                                    BasicMetadataTypeEnum::IntType(cast_res.get_type()),
                                    desired_type,
                                ))
                            },
                            Type::I16 => {
                                let cast_res = builder.build_float_to_signed_int(
                                    builder
                                        .build_load(var_ty.into_float_type(), var_ptr, "")?
                                        .into_float_value(),
                                    ctx.i16_type(),
                                    "",
                                )?;
                                let allocation =
                                    builder.build_alloca(cast_res.get_type(), "cast_result")?;

                                builder.build_store(allocation, cast_res)?;

                                Some((
                                    allocation,
                                    BasicMetadataTypeEnum::IntType(cast_res.get_type()),
                                    desired_type,
                                ))
                            },
                            Type::F16 => {
                                let value = builder
                                    .build_load(var_ty.into_float_type(), var_ptr, "")?
                                    .into_float_value();

                                let cast_res =
                                    ctx.f16_type().const_float(value.get_constant().unwrap().0);

                                let allocation =
                                    builder.build_alloca(cast_res.get_type(), "cast_result")?;

                                builder.build_store(allocation, cast_res)?;

                                Some((
                                    allocation,
                                    BasicMetadataTypeEnum::FloatType(cast_res.get_type()),
                                    desired_type,
                                ))
                            },
                            Type::U16 => {
                                let cast_res = builder.build_float_to_unsigned_int(
                                    builder
                                        .build_load(var_ty.into_float_type(), var_ptr, "")?
                                        .into_float_value(),
                                    ctx.i16_type(),
                                    "",
                                )?;
                                let allocation =
                                    builder.build_alloca(cast_res.get_type(), "cast_result")?;

                                builder.build_store(allocation, cast_res)?;

                                Some((
                                    allocation,
                                    BasicMetadataTypeEnum::IntType(cast_res.get_type()),
                                    desired_type,
                                ))
                            },
                            Type::U8 => {
                                let cast_res = builder.build_float_to_unsigned_int(
                                    builder
                                        .build_load(var_ty.into_float_type(), var_ptr, "")?
                                        .into_float_value(),
                                    ctx.i8_type(),
                                    "",
                                )?;
                                let allocation =
                                    builder.build_alloca(cast_res.get_type(), "cast_result")?;

                                builder.build_store(allocation, cast_res)?;

                                Some((
                                    allocation,
                                    BasicMetadataTypeEnum::IntType(cast_res.get_type()),
                                    desired_type,
                                ))
                            },
                            Type::String => {
                                let value = builder
                                    .build_load(var_ty.into_float_type(), var_ptr, "")?
                                    .into_float_value();

                                let raw_val = value.get_constant().unwrap().0;

                                let int_string = raw_val.to_string();

                                let (buf_ptr, buf_ty) =
                                    allocate_string(builder, ctx.i8_type(), int_string)?;

                                Some((
                                    buf_ptr,
                                    BasicMetadataTypeEnum::ArrayType(buf_ty),
                                    desired_type,
                                ))
                            },
                            Type::Boolean => {
                                let value = builder
                                    .build_load(var_ty.into_float_type(), var_ptr, "")?
                                    .into_float_value();

                                let bool_ty = ctx.bool_type();

                                let bool_value = if value.get_constant().unwrap().0 == 0.0 {
                                    bool_ty.const_int(0, false)
                                }
                                else {
                                    bool_ty.const_int(1, false)
                                };

                                let allocation =
                                    builder.build_alloca(bool_value.get_type(), "cast_result")?;

                                builder.build_store(allocation, bool_value)?;

                                Some((
                                    allocation,
                                    BasicMetadataTypeEnum::IntType(bool_value.get_type()),
                                    desired_type,
                                ))
                            },
                            Type::Void => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            },
                            Type::Struct(_) => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            },
                            Type::Array(_) => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            },
                            Type::Pointer(_) => todo!(),
                            Type::Enum(_) => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            },
                        }
                    },
                    Type::U64 | Type::U32 | Type::U16 | Type::U8 => {
                        match desired_type {
                            Type::I64 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res = ctx.i64_type().const_int(
                                    value.get_sign_extended_constant().unwrap() as u64,
                                    true,
                                );

                                let allocation =
                                    builder.build_alloca(cast_res.get_type(), "cast_result")?;

                                builder.build_store(allocation, cast_res)?;

                                Some((
                                    allocation,
                                    BasicMetadataTypeEnum::IntType(cast_res.get_type()),
                                    desired_type,
                                ))
                            },
                            Type::F64 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res = builder.build_unsigned_int_to_float(
                                    value,
                                    ctx.f64_type(),
                                    "casted_value",
                                )?;

                                let allocation =
                                    builder.build_alloca(cast_res.get_type(), "cast_result")?;

                                builder.build_store(allocation, cast_res)?;

                                Some((
                                    allocation,
                                    BasicMetadataTypeEnum::FloatType(cast_res.get_type()),
                                    desired_type,
                                ))
                            },
                            Type::U64 => Some((var_ptr, var_ty, Type::U64)),
                            Type::I32 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res = ctx.i32_type().const_int(
                                    value.get_sign_extended_constant().unwrap() as u64,
                                    true,
                                );

                                let allocation =
                                    builder.build_alloca(cast_res.get_type(), "cast_result")?;

                                builder.build_store(allocation, cast_res)?;

                                Some((
                                    allocation,
                                    BasicMetadataTypeEnum::IntType(cast_res.get_type()),
                                    desired_type,
                                ))
                            },
                            Type::F32 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res = builder.build_unsigned_int_to_float(
                                    value,
                                    ctx.f32_type(),
                                    "casted_value",
                                )?;

                                let allocation =
                                    builder.build_alloca(cast_res.get_type(), "cast_result")?;

                                builder.build_store(allocation, cast_res)?;

                                Some((
                                    allocation,
                                    BasicMetadataTypeEnum::FloatType(cast_res.get_type()),
                                    desired_type,
                                ))
                            },
                            Type::U32 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res = ctx.i32_type().const_int(
                                    value.get_sign_extended_constant().unwrap() as u64,
                                    false,
                                );

                                let allocation =
                                    builder.build_alloca(cast_res.get_type(), "cast_result")?;

                                builder.build_store(allocation, cast_res)?;

                                Some((
                                    allocation,
                                    BasicMetadataTypeEnum::IntType(cast_res.get_type()),
                                    desired_type,
                                ))
                            },
                            Type::I16 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res = ctx.i16_type().const_int(
                                    value.get_sign_extended_constant().unwrap() as u64,
                                    true,
                                );

                                let allocation =
                                    builder.build_alloca(cast_res.get_type(), "cast_result")?;

                                builder.build_store(allocation, cast_res)?;

                                Some((
                                    allocation,
                                    BasicMetadataTypeEnum::IntType(cast_res.get_type()),
                                    desired_type,
                                ))
                            },
                            Type::F16 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res = builder.build_unsigned_int_to_float(
                                    value,
                                    ctx.f16_type(),
                                    "casted_value",
                                )?;

                                let allocation =
                                    builder.build_alloca(cast_res.get_type(), "cast_result")?;

                                builder.build_store(allocation, cast_res)?;

                                Some((
                                    allocation,
                                    BasicMetadataTypeEnum::FloatType(cast_res.get_type()),
                                    desired_type,
                                ))
                            },
                            Type::U16 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res = ctx.i16_type().const_int(
                                    value.get_sign_extended_constant().unwrap() as u64,
                                    false,
                                );

                                let allocation =
                                    builder.build_alloca(cast_res.get_type(), "cast_result")?;

                                builder.build_store(allocation, cast_res)?;

                                Some((
                                    allocation,
                                    BasicMetadataTypeEnum::IntType(cast_res.get_type()),
                                    desired_type,
                                ))
                            },
                            Type::U8 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res = ctx.i8_type().const_int(
                                    value.get_sign_extended_constant().unwrap() as u64,
                                    false,
                                );

                                let allocation =
                                    builder.build_alloca(cast_res.get_type(), "cast_result")?;

                                builder.build_store(allocation, cast_res)?;

                                Some((
                                    allocation,
                                    BasicMetadataTypeEnum::IntType(cast_res.get_type()),
                                    desired_type,
                                ))
                            },
                            Type::String => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let raw_val = value.get_sign_extended_constant().unwrap();

                                let int_string = raw_val.to_string();

                                let (buf_ptr, buf_ty) =
                                    allocate_string(builder, ctx.i8_type(), int_string)?;

                                Some((
                                    buf_ptr,
                                    BasicMetadataTypeEnum::ArrayType(buf_ty),
                                    desired_type,
                                ))
                            },
                            Type::Boolean => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let bool_ty = ctx.bool_type();

                                let bool_value = if value.get_sign_extended_constant().unwrap() == 0
                                {
                                    bool_ty.const_int(0, false)
                                }
                                else {
                                    bool_ty.const_int(1, false)
                                };

                                let allocation =
                                    builder.build_alloca(bool_value.get_type(), "cast_result")?;

                                builder.build_store(allocation, bool_value)?;

                                Some((
                                    allocation,
                                    BasicMetadataTypeEnum::IntType(bool_value.get_type()),
                                    desired_type,
                                ))
                            },
                            Type::Void => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            },
                            Type::Struct(_) => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            },
                            Type::Array(_) => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            },
                            Type::Pointer(_) => todo!(),
                            Type::Enum(_) => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            },
                        }
                    },
                    Type::String => {
                        match desired_type {
                            Type::I64 => todo!(),
                            Type::F64 => todo!(),
                            Type::U64 => todo!(),
                            Type::I32 => todo!(),
                            Type::F32 => todo!(),
                            Type::U32 => todo!(),
                            Type::I16 => todo!(),
                            Type::F16 => todo!(),
                            Type::U16 => todo!(),
                            Type::U8 => todo!(),
                            Type::String => todo!(),
                            Type::Boolean => todo!(),
                            Type::Void => todo!(),
                            Type::Struct(_) => todo!(),
                            Type::Array(_) => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            },
                            Type::Pointer(_) => todo!(),
                            Type::Enum(_) => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            },
                        }
                    },
                    Type::Boolean => {
                        match desired_type {
                            Type::I64 => todo!(),
                            Type::F64 => todo!(),
                            Type::U64 => todo!(),
                            Type::I32 => todo!(),
                            Type::F32 => todo!(),
                            Type::U32 => todo!(),
                            Type::I16 => todo!(),
                            Type::F16 => todo!(),
                            Type::U16 => todo!(),
                            Type::U8 => todo!(),
                            Type::String => todo!(),
                            Type::Boolean => todo!(),
                            Type::Void => todo!(),
                            Type::Struct(_) => todo!(),
                            Type::Array(_) => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            },
                            Type::Pointer(_) => todo!(),
                            Type::Enum(_) => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            },
                        }
                    },
                    Type::Void => {
                        match desired_type {
                            Type::I64 => todo!(),
                            Type::F64 => todo!(),
                            Type::U64 => todo!(),
                            Type::I32 => todo!(),
                            Type::F32 => todo!(),
                            Type::U32 => todo!(),
                            Type::I16 => todo!(),
                            Type::F16 => todo!(),
                            Type::U16 => todo!(),
                            Type::U8 => todo!(),
                            Type::String => todo!(),
                            Type::Boolean => todo!(),
                            Type::Void => todo!(),
                            Type::Struct(_) => todo!(),
                            Type::Array(_) => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            },
                            Type::Pointer(_) => todo!(),
                            Type::Enum(_) => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            },
                        }
                    },
                    Type::Struct(_) => {
                        match desired_type {
                            Type::I64 => todo!(),
                            Type::F64 => todo!(),
                            Type::U64 => todo!(),
                            Type::I32 => todo!(),
                            Type::F32 => todo!(),
                            Type::U32 => todo!(),
                            Type::I16 => todo!(),
                            Type::F16 => todo!(),
                            Type::U16 => todo!(),
                            Type::U8 => todo!(),
                            Type::String => todo!(),
                            Type::Boolean => todo!(),
                            Type::Void => todo!(),
                            Type::Struct(_) => todo!(),
                            Type::Array(_) => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            },
                            Type::Pointer(_) => todo!(),
                            Type::Enum(_) => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            },
                        }
                    },
                    Type::Array(type_discriminant) => todo!(),
                    Type::Pointer(_) => todo!(),
                    Type::Enum(_) => {
                        return Err(CodeGenError::InvalidTypeCast(ty_disc, desired_type).into());
                    },
                };

                if let Some((ptr, ptr_ty, var_type)) = returned_alloca {
                    pre_allocation_list.push((
                        parsed_token_instance.clone(),
                        ptr,
                        ptr_ty,
                        var_type,
                    ));
                }
            }
            else {
                return Err(CodeGenError::InternalParsingError.into());
            }
        },
        ParsedToken::FunctionCall((fn_sig, fn_name), arguments) => {
            for (arg_idx, (arg_name, (arg, arg_ty))) in arguments.iter().enumerate() {
                // We create a pre allocated temp variable for the function's arguments, we use the function arg's name to indicate which temp variable is for which argument.
                // If the argument name is None, it means that the function we are calling has an indefinite amount of arguments, in this case having llvm automaticly name the variable is accepted
                let (ptr, ty) = create_new_variable(
                    ctx,
                    builder,
                    &match arg_name.clone() {
                        common::codegen::FunctionArgumentIdentifier::Identifier(ident) => {
                            ident.to_string()
                        },
                        common::codegen::FunctionArgumentIdentifier::Index(idx) => {
                            format!("{fn_name}_idx_{idx}_arg")
                        },
                    },
                    arg_ty,
                    custom_types.clone(),
                )?;

                pre_allocation_list.push((arg.clone(), ptr, ty, arg_ty.clone()));
            }

            // Check if the returned value of the function is Void
            // If it is, then we dont need to allocate anything
            if fn_sig.return_type != Type::Void {
                let (ptr, ty) = create_new_variable(
                    ctx,
                    builder,
                    "",
                    &fn_sig.return_type,
                    custom_types.clone(),
                )?;

                pre_allocation_list.push((
                    parsed_token_instance.clone(),
                    ptr,
                    ty,
                    fn_sig.return_type,
                ));
            }
        },
        ParsedToken::SetValue(_var_ref, value) => {
            let allocation_list = fetch_alloca_ptr(
                ctx,
                module,
                builder,
                *value,
                variable_map,
                fn_ret_ty,
                this_fn_block,
                this_fn,
                parsed_functions.clone(),
                custom_types.clone(),
            )?;

            pre_allocation_list.extend(allocation_list);
        },
        ParsedToken::MathematicalExpression(lhs_token, _expr, rhs_token) => {
            let lhs_alloca = create_ir_from_parsed_token(
                ctx,
                module,
                builder,
                *(lhs_token.clone()),
                variable_map,
                None,
                fn_ret_ty.clone(),
                this_fn_block,
                this_fn,
                &mut VecDeque::new(),
                None,
                parsed_functions.clone(),
                custom_types.clone(),
            )?;

            let rhs_alloca = create_ir_from_parsed_token(
                ctx,
                module,
                builder,
                *(rhs_token.clone()),
                variable_map,
                None,
                fn_ret_ty.clone(),
                this_fn_block,
                this_fn,
                &mut VecDeque::new(),
                None,
                parsed_functions.clone(),
                custom_types.clone(),
            )?;

            // Store the pointer of either one of the allocable values
            if let (Some((l_ptr, l_ty, l_ty_disc)), Some((r_ptr, r_ty, r_ty_disc))) =
                (lhs_alloca, rhs_alloca)
            {
                pre_allocation_list.push((*(lhs_token.clone()), l_ptr, l_ty, l_ty_disc));
                pre_allocation_list.push((*(rhs_token.clone()), r_ptr, r_ty, r_ty_disc));
            }
            else {
                return Err(CodeGenError::InvalidMathematicalValue.into());
            }
        },
        ParsedToken::If(inner) => {
            let condition_allocations = fetch_alloca_ptr(
                ctx,
                module,
                builder,
                *inner.condition,
                variable_map,
                fn_ret_ty.clone(),
                this_fn_block,
                this_fn,
                parsed_functions.clone(),
                custom_types.clone(),
            )?;

            pre_allocation_list.extend(condition_allocations);

            for parsed_token in inner.complete_body {
                let body_pre_allocs = fetch_alloca_ptr(
                    ctx,
                    module,
                    builder,
                    parsed_token,
                    variable_map,
                    fn_ret_ty.clone(),
                    this_fn_block,
                    this_fn,
                    parsed_functions.clone(),
                    custom_types.clone(),
                )?;

                pre_allocation_list.extend(body_pre_allocs);
            }

            for parsed_token in inner.incomplete_body {
                let body_pre_allocs = fetch_alloca_ptr(
                    ctx,
                    module,
                    builder,
                    parsed_token,
                    variable_map,
                    fn_ret_ty.clone(),
                    this_fn_block,
                    this_fn,
                    parsed_functions.clone(),
                    custom_types.clone(),
                )?;

                pre_allocation_list.extend(body_pre_allocs);
            }
        },
        ParsedToken::Comparison(lhs, _, rhs, _) => {
            let lhs_allocations = fetch_alloca_ptr(
                ctx,
                module,
                builder,
                *lhs,
                variable_map,
                fn_ret_ty.clone(),
                this_fn_block,
                this_fn,
                parsed_functions.clone(),
                custom_types.clone(),
            )?;
            let rhs_allocations = fetch_alloca_ptr(
                ctx,
                module,
                builder,
                *rhs,
                variable_map,
                fn_ret_ty.clone(),
                this_fn_block,
                this_fn,
                parsed_functions.clone(),
                custom_types.clone(),
            )?;

            pre_allocation_list.extend(lhs_allocations);
            pre_allocation_list.extend(rhs_allocations);

            // Create a variable which stores the cmp result of the two
            let ptr = builder.build_alloca(ctx.bool_type(), "cmp_result")?;

            pre_allocation_list.push((
                parsed_token_instance.clone(),
                ptr,
                ctx.bool_type().into(),
                Type::Boolean,
            ));
        },
        // We can safely ignore this variant as it doesn't allocate anything
        ParsedToken::ControlFlow(_) => (),
        _ => {
            unimplemented!()
        },
    };

    Ok(pre_allocation_list)
}

pub fn allocate_string<'a>(
    builder: &'a Builder<'_>,
    i8_type: common::inkwell::types::IntType<'a>,
    string_buffer: String,
) -> Result<(PointerValue<'a>, ArrayType<'a>)>
{
    // Create a buffer from the String
    let mut string_bytes = string_buffer.as_bytes().to_vec();

    // If the last byte is not a null byte we automaticly add the null byte
    if let Some(last_byte) = string_bytes.last() {
        // Check if the last byte is not a null byte
        if *last_byte != 0 {
            // Push the \0 byte
            string_bytes.push(0);
        }
    }
    // Create a Sized array type with every element being an i8
    let sized_array_ty = i8_type.array_type(string_bytes.len() as u32);

    // Allocate the Array based on its type
    let buffer_ptr = builder.build_alloca(sized_array_ty, "string_buffer")?;

    // Create a String array from the byte values of the string and the array
    let str_array = i8_type.const_array(
        string_bytes
            .iter()
            .map(|byte| i8_type.const_int(*byte as u64, false))
            .collect::<Vec<IntValue>>()
            .as_slice(),
    );

    // Store the array in the buffer ptr
    builder.build_store(buffer_ptr, str_array)?;

    // Return the buffer's ptr
    Ok((buffer_ptr, sized_array_ty))
}

/// Creates a new variable from a `TypeDiscriminant`.
/// It is UB to access the value of the variable created here before initilazing it with actual data.
pub fn create_new_variable<'a, 'b>(
    ctx: &'a Context,
    builder: &'a Builder<'_>,
    var_name: &str,
    var_type: &Type,
    custom_types: Arc<IndexMap<String, CustomType>>,
) -> Result<(PointerValue<'a>, BasicMetadataTypeEnum<'a>)>
{
    // Turn a `TypeDiscriminant` into an LLVM type
    let var_type = ty_to_llvm_ty(ctx, var_type, custom_types.clone())?;

    // Allocate an instance of the converted type
    let v_ptr = builder.build_alloca(var_type, var_name)?;

    // Return the pointer of the allocation and the type
    Ok((v_ptr, var_type.into()))
}
