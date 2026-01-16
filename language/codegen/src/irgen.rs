use common::{
    anyhow::Result,
    codegen::{
        CustomType, FunctionArgumentIdentifier, LoopBodyBlocks, create_fn_type_from_ty_disc,
        fn_arg_to_string, ty_enum_to_metadata_ty_enum, ty_to_llvm_ty,
    },
    error::codegen::CodeGenError,
    indexmap::IndexMap,
    inkwell::{
        AddressSpace,
        attributes::Attribute,
        basic_block::BasicBlock,
        builder::Builder,
        context::Context,
        debug_info::{AsDIScope, DWARFEmissionKind, DWARFSourceLanguage},
        module::Module,
        types::BasicMetadataTypeEnum,
        values::{BasicMetadataValueEnum, BasicValue, BasicValueEnum, FunctionValue, PointerValue},
    },
    parser::{
        common::{ParsedToken, ParsedTokenInstance},
        function::{CompilerHint, FunctionDefinition},
        value::MathematicalSymbol,
        variable::{ControlFlowType, UniqueId, VARIABLE_ID_SOURCE, VariableReference},
    },
    tokenizer::Token,
    ty::{OrdMap, Type, ty_from_token},
};
use std::{
    collections::{HashMap, VecDeque},
    rc::Rc,
};

use crate::{
    allocate::{allocate_string, create_allocation_table, create_new_variable},
    debug::create_subprogram_debug_information,
    pointer::{access_array_index, access_variable_ptr, set_value_of_ptr},
};

pub fn create_ir<'main, 'ctx>(
    module: &Module<'ctx>,
    // Inkwell IR builder
    builder: &'ctx Builder<'ctx>,
    // Inkwell Context
    ctx: &'main Context,
    // The list of ParsedToken-s
    parsed_tokens: Vec<ParsedTokenInstance>,
    // This argument is initialized with the HashMap of the arguments
    available_arguments: HashMap<String, (BasicValueEnum<'ctx>, (Type, UniqueId))>,
    // Type returned type of the Function
    fn_ret_ty: Type,
    this_fn_block: BasicBlock<'ctx>,
    this_fn: FunctionValue<'ctx>,
    parsed_functions: Rc<IndexMap<String, FunctionDefinition>>,
    custom_items: Rc<IndexMap<String, CustomType>>,
) -> Result<()>
where
    'main: 'ctx,
{
    let mut variable_map: HashMap<
        String,
        ((PointerValue, BasicMetadataTypeEnum), (Type, UniqueId)),
    > = HashMap::new();

    for (arg_name, (arg_val, arg_ty)) in available_arguments {
        let (v_ptr, ty) = match arg_val {
            BasicValueEnum::ArrayValue(value) => {
                let v_ptr = builder.build_alloca(value.get_type(), &arg_name)?;
                builder.build_store(v_ptr, value)?;

                (v_ptr, BasicMetadataTypeEnum::ArrayType(value.get_type()))
            },
            BasicValueEnum::IntValue(value) => {
                let v_ptr = builder.build_alloca(value.get_type(), &arg_name)?;
                builder.build_store(v_ptr, value)?;
                (v_ptr, BasicMetadataTypeEnum::IntType(value.get_type()))
            },
            BasicValueEnum::FloatValue(value) => {
                let v_ptr = builder.build_alloca(value.get_type(), &arg_name)?;
                builder.build_store(v_ptr, value)?;
                (v_ptr, BasicMetadataTypeEnum::FloatType(value.get_type()))
            },
            BasicValueEnum::PointerValue(value) => {
                let v_ptr = builder.build_alloca(value.get_type(), &arg_name)?;
                builder.build_store(v_ptr, value)?;
                (v_ptr, BasicMetadataTypeEnum::PointerType(value.get_type()))
            },
            BasicValueEnum::StructValue(value) => {
                let v_ptr = builder.build_alloca(value.get_type(), &arg_name)?;
                builder.build_store(v_ptr, value)?;
                (v_ptr, BasicMetadataTypeEnum::StructType(value.get_type()))
            },
            BasicValueEnum::VectorValue(value) => {
                let v_ptr = builder.build_alloca(value.get_type(), &arg_name)?;
                builder.build_store(v_ptr, value)?;
                (v_ptr, BasicMetadataTypeEnum::VectorType(value.get_type()))
            },
            BasicValueEnum::ScalableVectorValue(scalable_vector_value) => todo!(),
        };

        variable_map.insert(arg_name, ((v_ptr, ty), arg_ty));
    }

    create_ir_from_parsed_token_list(
        module,
        builder,
        ctx,
        parsed_tokens,
        fn_ret_ty,
        this_fn_block,
        &mut variable_map,
        this_fn,
        &HashMap::new(),
        None,
        parsed_functions.clone(),
        custom_items.clone(),
    )?;

    Ok(())
}

pub fn create_ir_from_parsed_token<'main, 'ctx>(
    ctx: &'main Context,
    module: &Module<'ctx>,
    builder: &'ctx Builder<'ctx>,
    parsed_token_instance: ParsedTokenInstance,
    variable_map: &mut HashMap<
        String,
        (
            (PointerValue<'ctx>, BasicMetadataTypeEnum<'ctx>),
            (Type, UniqueId),
        ),
    >,
    variable_reference: Option<(
        String,
        (PointerValue<'ctx>, BasicMetadataTypeEnum<'ctx>),
        Type,
    )>,
    // Type returned type of the Function
    fn_ret_ty: Type,
    this_fn_block: BasicBlock<'ctx>,
    this_fn: FunctionValue<'ctx>,
    allocation_table: &HashMap<UniqueId, PointerValue<'ctx>>,
    is_loop_body: Option<LoopBodyBlocks>,
    parsed_functions: Rc<IndexMap<String, FunctionDefinition>>,
    custom_types: Rc<IndexMap<String, CustomType>>,
) -> Result<
    // This optional return value is the reference to the value of a ParsedToken's result. ie: Comparsions return a Some(ptr) to the bool value of the comparison
    // The return value is None if the `variable_reference` of the function is `Some`, as the variable will have its value set to the value of the returned value.
    Option<(
        // Pointer to the referenced variable
        PointerValue<'ctx>,
        // Type of the variable
        BasicMetadataTypeEnum<'ctx>,
        // TypeDiscriminant of the variable
        Type,
    )>,
>
where
    'main: 'ctx,
{
    let parsed_token = parsed_token_instance.inner;
    // Debug info for returning error spans
    let parsed_token_debug_info = parsed_token_instance.debug_information;

    let created_var = match parsed_token.clone() {
        ParsedToken::NewVariable(var_name, var_type, var_set_val, var_id) => {
            let (ptr, ty) = create_new_variable(
                ctx,
                builder,
                &var_name,
                &var_type,
                Some(var_id),
                allocation_table,
                custom_types.clone(),
            )?;

            variable_map.insert(var_name.clone(), ((ptr, ty), (var_type.clone(), var_id)));

            // Set the value of the newly created variable
            create_ir_from_parsed_token(
                ctx,
                module,
                builder,
                *var_set_val.clone(),
                variable_map,
                Some((var_name.clone(), (ptr, ty), var_type.clone())),
                fn_ret_ty,
                this_fn_block,
                this_fn,
                allocation_table,
                is_loop_body,
                parsed_functions.clone(),
                custom_types.clone(),
            )?;

            // We do not have to return anything here since a variable handle cannot really be casted to anything, its also top level
            None
        },
        ParsedToken::VariableReference(var_ref_variant) => {
            if let Some((var_ref_name, (var_ref_ptr, var_ref_ty), var_ref_ty_disc)) =
                variable_reference
            {
                let (ptr, ptr_ty, ty) = access_variable_ptr(
                    ctx,
                    module,
                    &fn_ret_ty,
                    this_fn_block,
                    this_fn,
                    allocation_table,
                    &is_loop_body,
                    parsed_functions,
                    builder,
                    Box::new(var_ref_variant),
                    variable_map,
                    custom_types,
                )?;

                let value = builder.build_load(ptr_ty, ptr, &format!("{var_ref_name}_deref"))?;

                builder.build_store(var_ref_ptr, value)?;

                None
            }
            else {
                let (ptr, ptr_ty, ty) = access_variable_ptr(
                    ctx,
                    module,
                    &fn_ret_ty,
                    this_fn_block,
                    this_fn,
                    allocation_table,
                    &is_loop_body,
                    parsed_functions,
                    builder,
                    Box::new(var_ref_variant),
                    variable_map,
                    custom_types,
                )?;

                Some((ptr, ty_enum_to_metadata_ty_enum(ptr_ty), ty))
            }
        },
        ParsedToken::Literal(literal) => {
            // There this is None there is nothing we can do with this so just return
            if let Some(var_ref) = variable_reference {
                let (ptr, _var_type) = var_ref.1;

                let var_ref_ty = var_ref.2;

                // Check the type of the value, check for a type mismatch
                if literal.discriminant() != var_ref_ty {
                    return Err(CodeGenError::VariableTypeMismatch(
                        literal.discriminant(),
                        var_ref_ty,
                    )
                    .into());
                }

                set_value_of_ptr(
                    ctx,
                    builder,
                    module,
                    literal,
                    ptr,
                    custom_types.clone(),
                    variable_map,
                    &fn_ret_ty,
                    this_fn_block,
                    this_fn,
                    allocation_table,
                    &is_loop_body,
                    parsed_functions,
                )?;

                None
            }
            else {
                let ty_disc = literal.discriminant();

                let (v_ptr, v_ty) = create_new_variable(
                    ctx,
                    builder,
                    "",
                    &ty_disc,
                    None,
                    allocation_table,
                    custom_types.clone(),
                )?;

                set_value_of_ptr(
                    ctx,
                    builder,
                    module,
                    literal,
                    v_ptr,
                    custom_types.clone(),
                    variable_map,
                    &fn_ret_ty,
                    this_fn_block,
                    this_fn,
                    allocation_table,
                    &is_loop_body,
                    parsed_functions,
                )?;

                Some((v_ptr, v_ty, ty_disc))
            }
        },
        ParsedToken::TypeCast(parsed_token, desired_type) => {
            let created_var = create_ir_from_parsed_token(
                ctx,
                module,
                builder,
                *parsed_token.clone(),
                variable_map,
                None,
                fn_ret_ty,
                this_fn_block,
                this_fn,
                allocation_table,
                is_loop_body.clone(),
                parsed_functions.clone(),
                custom_types.clone(),
            )?;

            if let Some((var_ptr, var_ty, ty_disc)) = created_var {
                let ref_ptr = if let Some((_, (ref_ptr, _), _)) = variable_reference.clone() {
                    ref_ptr
                }
                else {
                    let (ptr, _) = create_new_variable(
                        ctx,
                        builder,
                        "ty_cast_temp_val",
                        &ty_disc,
                        None,
                        allocation_table,
                        custom_types.clone(),
                    )?;

                    ptr
                };

                // Try to get the literal of the original value which hasnt been converted yet.
                if let Some(literal) = parsed_token.inner.try_as_literal_ref() {
                    // Check if the type is an enum
                    if let Type::Enum((ty, _body)) = literal.discriminant() {
                        // Check if the enum's inner type matches with the desired type. If not raise an error
                        if &*ty == &desired_type {
                            builder.build_store(
                                ref_ptr,
                                builder.build_load(
                                    desired_type.to_basic_type_enum(&ctx, custom_types.clone())?,
                                    var_ptr,
                                    "get_enum_inner",
                                )?,
                            )?;

                            return Ok(None);
                        }
                        else {
                            return Err(CodeGenError::EnumInnerTypeMismatch(
                                (*ty).clone(),
                                desired_type.clone(),
                            )
                            .into());
                        }
                    }
                }

                // If the shorter path fails try manually converting the values
                match ty_disc {
                    Type::I64 | Type::I32 | Type::I16 => {
                        match desired_type {
                            Type::I64 => {
                                builder.build_store(
                                    ref_ptr,
                                    builder.build_load(var_ty.into_int_type(), var_ptr, "")?,
                                )?;
                            },
                            Type::F64 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res = builder.build_signed_int_to_float(
                                    value,
                                    ctx.f64_type(),
                                    "casted_value",
                                )?;

                                builder.build_store(ref_ptr, cast_res)?;
                            },
                            Type::U64 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res =
                                    builder.build_int_cast(value, ctx.i64_type(), "i64_to_u64")?;

                                builder.build_store(ref_ptr, cast_res)?;
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

                                builder.build_store(ref_ptr, cast_res)?;
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

                                builder.build_store(ref_ptr, cast_res)?;
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

                                builder.build_store(ref_ptr, cast_res)?;
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

                                builder.build_store(ref_ptr, cast_res)?;
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

                                builder.build_store(ref_ptr, cast_res)?;
                            },
                            Type::String => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let raw_val = value.get_sign_extended_constant().unwrap();

                                let int_string = raw_val.to_string();

                                let (buf_ptr, buf_ty) =
                                    allocate_string(builder, ctx.i8_type(), int_string)?;

                                builder.build_store(ref_ptr, buf_ptr)?;
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

                                builder.build_store(ref_ptr, bool_value)?;
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
                            Type::Enum(_) => unreachable!(),
                        }
                    },
                    Type::F64 | Type::F32 | Type::F16 => {
                        match desired_type {
                            Type::I64 => {
                                let value = builder.build_float_to_signed_int(
                                    builder
                                        .build_load(var_ty.into_float_type(), var_ptr, "")?
                                        .into_float_value(),
                                    ctx.i64_type(),
                                    "",
                                )?;
                                builder.build_store(ref_ptr, value)?;
                            },
                            Type::F64 => {
                                builder.build_store(
                                    ref_ptr,
                                    builder.build_load(var_ty.into_float_type(), var_ptr, "")?,
                                )?;
                            },
                            Type::U64 => {
                                let value = builder.build_float_to_unsigned_int(
                                    builder
                                        .build_load(var_ty.into_float_type(), var_ptr, "")?
                                        .into_float_value(),
                                    ctx.i64_type(),
                                    "",
                                )?;
                                builder.build_store(ref_ptr, value)?;
                            },
                            Type::I32 => {
                                let value = builder.build_float_to_signed_int(
                                    builder
                                        .build_load(var_ty.into_float_type(), var_ptr, "")?
                                        .into_float_value(),
                                    ctx.i32_type(),
                                    "",
                                )?;
                                builder.build_store(ref_ptr, value)?;
                            },
                            Type::F32 => {
                                let value = builder
                                    .build_load(var_ty.into_float_type(), var_ptr, "")?
                                    .into_float_value();

                                let cast_res =
                                    ctx.f32_type().const_float(value.get_constant().unwrap().0);

                                builder.build_store(ref_ptr, cast_res)?;
                            },
                            Type::U32 => {
                                let value = builder.build_float_to_unsigned_int(
                                    builder
                                        .build_load(var_ty.into_float_type(), var_ptr, "")?
                                        .into_float_value(),
                                    ctx.i32_type(),
                                    "",
                                )?;
                                builder.build_store(ref_ptr, value)?;
                            },
                            Type::I16 => {
                                let value = builder.build_float_to_signed_int(
                                    builder
                                        .build_load(var_ty.into_float_type(), var_ptr, "")?
                                        .into_float_value(),
                                    ctx.i16_type(),
                                    "",
                                )?;
                                builder.build_store(ref_ptr, value)?;
                            },
                            Type::F16 => {
                                let value = builder
                                    .build_load(var_ty.into_float_type(), var_ptr, "")?
                                    .into_float_value();

                                let cast_res =
                                    ctx.f16_type().const_float(value.get_constant().unwrap().0);

                                builder.build_store(ref_ptr, cast_res)?;
                            },
                            Type::U16 => {
                                let value = builder.build_float_to_unsigned_int(
                                    builder
                                        .build_load(var_ty.into_float_type(), var_ptr, "")?
                                        .into_float_value(),
                                    ctx.i16_type(),
                                    "",
                                )?;
                                builder.build_store(ref_ptr, value)?;
                            },
                            Type::U8 => {
                                let value = builder.build_float_to_unsigned_int(
                                    builder
                                        .build_load(var_ty.into_float_type(), var_ptr, "")?
                                        .into_float_value(),
                                    ctx.i8_type(),
                                    "",
                                )?;
                                builder.build_store(ref_ptr, value)?;
                            },
                            Type::String => {
                                let value = builder
                                    .build_load(var_ty.into_float_type(), var_ptr, "")?
                                    .into_float_value();

                                let raw_val = value.get_constant().unwrap().0;

                                let int_string = raw_val.to_string();

                                let (buf_ptr, buf_ty) =
                                    allocate_string(builder, ctx.i8_type(), int_string)?;

                                builder.build_store(ref_ptr, buf_ptr)?;
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

                                builder.build_store(ref_ptr, bool_value)?;
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
                            Type::Enum(_) => unreachable!(),
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

                                builder.build_store(ref_ptr, cast_res)?;
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

                                builder.build_store(ref_ptr, cast_res)?;
                            },
                            Type::U64 => {
                                builder.build_store(
                                    ref_ptr,
                                    builder.build_load(var_ty.into_int_type(), var_ptr, "")?,
                                )?;
                            },
                            Type::I32 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res = ctx.i32_type().const_int(
                                    value.get_sign_extended_constant().unwrap() as u64,
                                    true,
                                );

                                builder.build_store(ref_ptr, cast_res)?;
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

                                builder.build_store(ref_ptr, cast_res)?;
                            },
                            Type::U32 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res = ctx.i32_type().const_int(
                                    value.get_sign_extended_constant().unwrap() as u64,
                                    false,
                                );

                                builder.build_store(ref_ptr, cast_res)?;
                            },
                            Type::I16 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res = ctx.i16_type().const_int(
                                    value.get_sign_extended_constant().unwrap() as u64,
                                    true,
                                );

                                builder.build_store(ref_ptr, cast_res)?;
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

                                builder.build_store(ref_ptr, cast_res)?;
                            },
                            Type::U16 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res = ctx.i16_type().const_int(
                                    value.get_sign_extended_constant().unwrap() as u64,
                                    false,
                                );

                                builder.build_store(ref_ptr, cast_res)?;
                            },
                            Type::U8 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res = ctx.i8_type().const_int(
                                    value.get_sign_extended_constant().unwrap() as u64,
                                    false,
                                );

                                builder.build_store(ref_ptr, cast_res)?;
                            },
                            Type::String => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let raw_val = value.get_sign_extended_constant().unwrap();

                                let int_string = raw_val.to_string();

                                let (buf_ptr, buf_ty) =
                                    allocate_string(builder, ctx.i8_type(), int_string)?;

                                builder.build_store(ref_ptr, buf_ptr)?;
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

                                builder.build_store(ref_ptr, bool_value)?;
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
                            Type::Enum(_) => unreachable!(),
                        }
                    },
                    Type::String => {
                        return Err(CodeGenError::InvalidTypeCast(ty_disc, desired_type).into());
                    },
                    Type::Boolean => {
                        match desired_type {
                            Type::I64 => {
                                let val = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let val = if val.get_zero_extended_constant().unwrap() == 0 {
                                    ctx.i64_type().const_int(0, true)
                                }
                                else {
                                    ctx.i64_type().const_int(1, true)
                                };

                                builder.build_store(var_ptr, val)?;
                            },
                            Type::F64 => {
                                let val = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let val = if val.get_zero_extended_constant().unwrap() == 0 {
                                    ctx.f64_type().const_float(0.0)
                                }
                                else {
                                    ctx.f64_type().const_float(1.0)
                                };

                                builder.build_store(var_ptr, val)?;
                            },
                            Type::U64 => {
                                let val = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let val = if val.get_zero_extended_constant().unwrap() == 0 {
                                    ctx.i64_type().const_int(0, false)
                                }
                                else {
                                    ctx.i64_type().const_int(1, false)
                                };

                                builder.build_store(var_ptr, val)?;
                            },
                            Type::I32 => {
                                let val = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let i32_val = builder.build_int_z_extend(
                                    val,
                                    ctx.i32_type(),
                                    "bool_to_i32",
                                )?;

                                builder.build_store(ref_ptr, i32_val)?;
                            },
                            Type::F32 => {
                                let val = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let val = if val.get_zero_extended_constant().unwrap() == 0 {
                                    ctx.f32_type().const_float(0.0)
                                }
                                else {
                                    ctx.f32_type().const_float(1.0)
                                };

                                builder.build_store(var_ptr, val)?;
                            },
                            Type::U32 => {
                                let val = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let val = if val.get_zero_extended_constant().unwrap() == 0 {
                                    ctx.i32_type().const_int(0, false)
                                }
                                else {
                                    ctx.i32_type().const_int(1, false)
                                };

                                builder.build_store(var_ptr, val)?;
                            },
                            Type::I16 => {
                                let val = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let val = if val.get_zero_extended_constant().unwrap() == 0 {
                                    ctx.i16_type().const_int(0, true)
                                }
                                else {
                                    ctx.i16_type().const_int(1, true)
                                };

                                builder.build_store(var_ptr, val)?;
                            },
                            Type::F16 => {
                                let val = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let val = if val.get_zero_extended_constant().unwrap() == 0 {
                                    ctx.f32_type().const_float(0.0)
                                }
                                else {
                                    ctx.f32_type().const_float(1.0)
                                };

                                builder.build_store(var_ptr, val)?;
                            },
                            Type::U16 => {
                                let val = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let val = if val.get_zero_extended_constant().unwrap() == 0 {
                                    ctx.i16_type().const_int(0, false)
                                }
                                else {
                                    ctx.i16_type().const_int(1, false)
                                };

                                builder.build_store(var_ptr, val)?;
                            },
                            Type::U8 => {
                                let val = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let val = if val.get_zero_extended_constant().unwrap() == 0 {
                                    ctx.i8_type().const_int(0, false)
                                }
                                else {
                                    ctx.i8_type().const_int(1, false)
                                };

                                builder.build_store(var_ptr, val)?;
                            },
                            Type::String => {
                                let val = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let (buf, buf_ty) = if val.get_zero_extended_constant().unwrap()
                                    == 0
                                {
                                    allocate_string(builder, ctx.i8_type(), "false".to_string())?
                                }
                                else {
                                    allocate_string(builder, ctx.i8_type(), "true".to_string())?
                                };

                                builder.build_store(var_ptr, buf)?;
                            },
                            Type::Boolean => {
                                builder.build_store(
                                    ref_ptr,
                                    builder.build_load(var_ty.into_int_type(), var_ptr, "")?,
                                )?;
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
                            Type::Enum(_) => unreachable!(),
                        }
                    },
                    Type::Void => {
                        return Err(CodeGenError::InvalidTypeCast(ty_disc, desired_type).into());
                    },
                    Type::Struct(_) => {
                        return Err(CodeGenError::InvalidTypeCast(ty_disc, desired_type).into());
                    },
                    Type::Array(_) => {
                        return Err(CodeGenError::InvalidTypeCast(ty_disc, desired_type).into());
                    },
                    Type::Pointer(_) => {
                        return Err(CodeGenError::InvalidTypeCast(ty_disc, desired_type).into());
                    },
                    Type::Enum(_) => unreachable!(),
                }

                if variable_reference.is_none() {
                    return Ok(Some((
                        ref_ptr,
                        ty_to_llvm_ty(ctx, &ty_disc, custom_types.clone())?.into(),
                        ty_disc.clone(),
                    )));
                }
            }

            None
        },
        ParsedToken::MathematicalExpression(lhs, mathematical_symbol, rhs) => {
            // Allocate memory on the stack for the value stored in the lhs
            let parsed_lhs = create_ir_from_parsed_token(
                ctx,
                module,
                builder,
                *lhs.clone(),
                variable_map,
                None,
                fn_ret_ty.clone(),
                this_fn_block,
                this_fn,
                allocation_table,
                is_loop_body.clone(),
                parsed_functions.clone(),
                custom_types.clone(),
            )?;

            // Allocate memory on the stack for the value stored in the rhs
            let parsed_rhs = create_ir_from_parsed_token(
                ctx,
                module,
                builder,
                *rhs.clone(),
                variable_map,
                None,
                fn_ret_ty.clone(),
                this_fn_block,
                this_fn,
                allocation_table,
                is_loop_body.clone(),
                parsed_functions.clone(),
                custom_types.clone(),
            )?;

            // Check if both sides return a valid variable reference
            if let (Some((lhs_ptr, lhs_ty, l_ty_disc)), Some((rhs_ptr, rhs_ty, r_ty_disc))) =
                (parsed_lhs, parsed_rhs)
            {
                if l_ty_disc.is_float() && r_ty_disc.is_float() {
                    let math_res = match mathematical_symbol {
                        MathematicalSymbol::Addition => {
                            builder.build_float_add(
                                builder
                                    .build_load(lhs_ty.into_float_type(), lhs_ptr, "lhs")?
                                    .into_float_value(),
                                builder
                                    .build_load(rhs_ty.into_float_type(), rhs_ptr, "rhs")?
                                    .into_float_value(),
                                "float_add_float",
                            )?
                        },
                        MathematicalSymbol::Subtraction => {
                            builder.build_float_sub(
                                builder
                                    .build_load(lhs_ty.into_float_type(), lhs_ptr, "lhs")?
                                    .into_float_value(),
                                builder
                                    .build_load(rhs_ty.into_float_type(), rhs_ptr, "rhs")?
                                    .into_float_value(),
                                "float_sub_float",
                            )?
                        },
                        MathematicalSymbol::Division => {
                            builder.build_float_div(
                                builder
                                    .build_load(lhs_ty.into_float_type(), lhs_ptr, "lhs")?
                                    .into_float_value(),
                                builder
                                    .build_load(rhs_ty.into_float_type(), rhs_ptr, "rhs")?
                                    .into_float_value(),
                                "float_add_float",
                            )?
                        },
                        MathematicalSymbol::Multiplication => {
                            builder.build_float_mul(
                                builder
                                    .build_load(lhs_ty.into_float_type(), lhs_ptr, "lhs")?
                                    .into_float_value(),
                                builder
                                    .build_load(rhs_ty.into_float_type(), rhs_ptr, "rhs")?
                                    .into_float_value(),
                                "float_add_float",
                            )?
                        },
                        MathematicalSymbol::Modulo => {
                            builder.build_float_rem(
                                builder
                                    .build_load(lhs_ty.into_float_type(), lhs_ptr, "lhs")?
                                    .into_float_value(),
                                builder
                                    .build_load(rhs_ty.into_float_type(), rhs_ptr, "rhs")?
                                    .into_float_value(),
                                "float_add_float",
                            )?
                        },
                    };

                    if let Some((var_ref_name, (var_ptr, var_ty), disc)) = variable_reference {
                        builder.build_store(var_ptr, math_res)?;
                    }
                    else {
                        let (ptr, ty) = create_new_variable(
                            ctx,
                            builder,
                            "math_expr_res",
                            &r_ty_disc,
                            None,
                            allocation_table,
                            custom_types.clone(),
                        )?;

                        builder.build_store(ptr, math_res)?;

                        return Ok(Some((ptr, ty, r_ty_disc)));
                    }
                }
                else if l_ty_disc.is_int() && r_ty_disc.is_int() {
                    let math_res = match mathematical_symbol {
                        MathematicalSymbol::Addition => {
                            builder.build_int_add(
                                builder
                                    .build_load(lhs_ty.into_int_type(), lhs_ptr, "lhs")?
                                    .into_int_value(),
                                builder
                                    .build_load(rhs_ty.into_int_type(), rhs_ptr, "rhs")?
                                    .into_int_value(),
                                "int_add_int",
                            )?
                        },
                        MathematicalSymbol::Subtraction => {
                            builder.build_int_sub(
                                builder
                                    .build_load(lhs_ty.into_int_type(), lhs_ptr, "lhs")?
                                    .into_int_value(),
                                builder
                                    .build_load(rhs_ty.into_int_type(), rhs_ptr, "rhs")?
                                    .into_int_value(),
                                "int_sub_int",
                            )?
                        },
                        MathematicalSymbol::Division => {
                            builder.build_int_signed_div(
                                builder
                                    .build_load(lhs_ty.into_int_type(), lhs_ptr, "lhs")?
                                    .into_int_value(),
                                builder
                                    .build_load(rhs_ty.into_int_type(), rhs_ptr, "rhs")?
                                    .into_int_value(),
                                "int_div_int",
                            )?
                        },
                        MathematicalSymbol::Multiplication => {
                            builder.build_int_mul(
                                builder
                                    .build_load(lhs_ty.into_int_type(), lhs_ptr, "lhs")?
                                    .into_int_value(),
                                builder
                                    .build_load(rhs_ty.into_int_type(), rhs_ptr, "rhs")?
                                    .into_int_value(),
                                "int_mul_int",
                            )?
                        },
                        MathematicalSymbol::Modulo => {
                            builder.build_int_signed_rem(
                                builder
                                    .build_load(lhs_ty.into_int_type(), lhs_ptr, "lhs")?
                                    .into_int_value(),
                                builder
                                    .build_load(rhs_ty.into_int_type(), rhs_ptr, "rhs")?
                                    .into_int_value(),
                                "int_rem_int",
                            )?
                        },
                    };
                    if let Some((var_ref_name, (var_ptr, var_ty), disc)) = variable_reference {
                        builder.build_store(var_ptr, math_res)?;
                    }
                    else {
                        let (ptr, ty) = create_new_variable(
                            ctx,
                            builder,
                            "math_expr_res",
                            &r_ty_disc,
                            None,
                            allocation_table,
                            custom_types.clone(),
                        )?;

                        builder.build_store(ptr, math_res)?;

                        return Ok(Some((ptr, ty, r_ty_disc)));
                    }
                }
                else {
                    return Err(CodeGenError::VariableTypeMismatch(l_ty_disc, r_ty_disc).into());
                }
            }
            // If either didn't that means that either side contained a parsed token which couldnt be referenced as a variable. Ie it is not a value in any way.
            else {
                return Err(CodeGenError::InvalidMathematicalValue.into());
            }

            None
        },
        ParsedToken::Brackets(parsed_tokens, type_discriminants) => todo!(),
        ParsedToken::FunctionCall((fn_sig, fn_name), passed_arguments) => {
            // Try accessing the function in the current module
            let function_value = module
                .get_function(&fn_name)
                .ok_or(CodeGenError::InternalFunctionNotFound(fn_name.clone()))?;

            let arg_iter =
                passed_arguments
                    .iter()
                    .enumerate()
                    .map(|(argument_idx, (arg_name, value))| {
                        (
                            match fn_sig
                                .args
                                .arguments
                                .get_index(argument_idx)
                                .map(|inner| inner.0.clone())
                            {
                                Some(arg_name) => FunctionArgumentIdentifier::Identifier(arg_name),
                                None => FunctionArgumentIdentifier::Index(argument_idx),
                            },
                            (value.clone()),
                        )
                    });

            // The arguments are in order, if theyre parsed in this order they can be passed to a function as an argument
            let fn_argument_list: OrdMap<
                FunctionArgumentIdentifier<String, usize>,
                (ParsedTokenInstance, (Type, UniqueId)),
            > = IndexMap::from_iter(arg_iter).into();

            // Keep the list of the arguments passed in
            let mut arguments_passed_in: Vec<BasicMetadataValueEnum> = Vec::new();

            for (arg_ident, (arg_token, (arg_type, arg_id))) in fn_argument_list.iter() {
                let fn_name_clone = fn_name.clone();

                let (ptr, ptr_ty) = create_new_variable(
                    ctx,
                    builder,
                    &fn_arg_to_string(&fn_name_clone, arg_ident),
                    arg_type,
                    Some(*arg_id),
                    allocation_table,
                    custom_types.clone(),
                )?;

                // Set the value of the temp variable to the value the argument has
                create_ir_from_parsed_token(
                    ctx,
                    module,
                    builder,
                    arg_token.clone(),
                    variable_map,
                    Some((
                        fn_arg_to_string(&fn_name, arg_ident),
                        (ptr, ptr_ty),
                        arg_type.clone(),
                    )),
                    fn_ret_ty.clone(),
                    this_fn_block,
                    this_fn,
                    allocation_table,
                    is_loop_body.clone(),
                    parsed_functions.clone(),
                    custom_types.clone(),
                )?;

                // Push the argument to the list of arguments
                match ptr_ty {
                    BasicMetadataTypeEnum::ArrayType(array_type) => {
                        let loaded_val = builder.build_load(
                            array_type,
                            ptr,
                            &fn_arg_to_string(&fn_name, arg_ident),
                        )?;

                        arguments_passed_in.push(loaded_val.into());
                    },
                    BasicMetadataTypeEnum::FloatType(float_type) => {
                        let loaded_val = builder.build_load(
                            float_type,
                            ptr,
                            &fn_arg_to_string(&fn_name, arg_ident),
                        )?;

                        arguments_passed_in.push(loaded_val.into());
                    },
                    BasicMetadataTypeEnum::IntType(int_type) => {
                        let loaded_val = builder.build_load(
                            int_type,
                            ptr,
                            &fn_arg_to_string(&fn_name, arg_ident),
                        )?;

                        arguments_passed_in.push(loaded_val.into());
                    },
                    BasicMetadataTypeEnum::PointerType(pointer_type) => {
                        let loaded_val = builder.build_load(
                            pointer_type,
                            ptr,
                            &fn_arg_to_string(&fn_name, arg_ident),
                        )?;

                        arguments_passed_in.push(loaded_val.into());
                    },
                    BasicMetadataTypeEnum::StructType(struct_type) => {
                        let loaded_val = builder.build_load(
                            struct_type,
                            ptr,
                            &fn_arg_to_string(&fn_name, arg_ident),
                        )?;

                        arguments_passed_in.push(loaded_val.into());
                    },
                    BasicMetadataTypeEnum::VectorType(vector_type) => {
                        let loaded_val = builder.build_load(
                            vector_type,
                            ptr,
                            &fn_arg_to_string(&fn_name, arg_ident),
                        )?;

                        arguments_passed_in.push(loaded_val.into());
                    },

                    _ => unimplemented!(),
                }
            }

            // Create function call
            let call = builder.build_call(function_value, &arguments_passed_in, "function_call")?;

            // Handle returned value
            let returned_value = call.try_as_basic_value().basic();

            if let Some(returned) = returned_value {
                let (v_ptr, v_ty) = if let Some(ref variable_name) = variable_reference {
                    let (v_ptr, var_ty) = variable_name.1;

                    (v_ptr, var_ty)
                }
                else {
                    create_new_variable(
                        ctx,
                        builder,
                        "",
                        &fn_sig.return_type,
                        None,
                        allocation_table,
                        custom_types.clone(),
                    )?
                };

                // Store the returned value
                builder.build_store(v_ptr, returned)?;

                if let Some((variable_name, (var_ptr, _), ty_disc)) = variable_reference {
                    // Check for type mismatch
                    if ty_disc != fn_sig.return_type {
                        return Err(CodeGenError::VariableTypeMismatch(
                            ty_disc,
                            fn_sig.return_type,
                        )
                        .into());
                    }

                    // Get what the function returned
                    let function_result = builder.build_load(
                        ty_to_llvm_ty(ctx, &fn_sig.return_type, custom_types.clone())?,
                        v_ptr,
                        &variable_name,
                    )?;

                    // Set the value of the pointer to whatever the function has returned
                    builder.build_store(var_ptr, function_result)?;

                    // We dont have to return a newly created variable reference here
                    None
                }
                else {
                    Some((v_ptr, v_ty, fn_sig.return_type))
                }
            }
            else {
                // Ensure the return type was `Void` else raise an error
                if fn_sig.return_type != Type::Void {
                    return Err(
                        CodeGenError::InternalFunctionReturnedVoid(fn_sig.return_type).into(),
                    );
                }

                // We dont return anything, as nothing is allocated
                None
            }
        },
        ParsedToken::SetValue(var_ref_ty, value) => {
            let (ptr, ty, ty_disc) = access_variable_ptr(
                ctx,
                module,
                &fn_ret_ty,
                this_fn_block,
                this_fn,
                allocation_table,
                &is_loop_body,
                parsed_functions.clone(),
                builder,
                Box::new(
                    var_ref_ty
                        .inner
                        .clone()
                        .try_as_variable_reference()
                        .ok_or(CodeGenError::InvalidVariableReference(var_ref_ty.inner))?,
                ),
                variable_map,
                custom_types.clone(),
            )?;

            create_ir_from_parsed_token(
                ctx,
                module,
                builder,
                *value,
                variable_map,
                Some((
                    String::from("set_value_var_ref"),
                    (ptr, ty.into()),
                    ty_disc.clone(),
                )),
                fn_ret_ty,
                this_fn_block,
                this_fn,
                allocation_table,
                is_loop_body.clone(),
                parsed_functions.clone(),
                custom_types.clone(),
            )?;

            None
        },
        ParsedToken::MathematicalBlock(parsed_token) => todo!(),
        ParsedToken::ReturnValue(parsed_token) => {
            // Create a temporary variable to store the literal in
            // This temporary variable is used to return the value
            let var_name = String::from("ret_tmp_var");

            let (ptr, ptr_ty) = create_new_variable(
                ctx,
                builder,
                &var_name,
                &fn_ret_ty,
                None,
                allocation_table,
                custom_types.clone(),
            )?;

            // Set the value of the newly created variable
            create_ir_from_parsed_token(
                ctx,
                module,
                builder,
                *parsed_token.clone(),
                variable_map,
                Some((var_name.clone(), (ptr, ptr_ty), fn_ret_ty.clone())),
                fn_ret_ty.clone(),
                this_fn_block,
                this_fn,
                allocation_table,
                is_loop_body.clone(),
                parsed_functions.clone(),
                custom_types.clone(),
            )?;

            match ptr_ty {
                BasicMetadataTypeEnum::ArrayType(array_type) => {
                    builder.build_return(Some(&builder.build_load(array_type, ptr, &var_name)?))?;
                },
                BasicMetadataTypeEnum::FloatType(float_type) => {
                    builder.build_return(Some(&builder.build_load(float_type, ptr, &var_name)?))?;
                },
                BasicMetadataTypeEnum::IntType(int_type) => {
                    builder.build_return(Some(&builder.build_load(int_type, ptr, &var_name)?))?;
                },
                BasicMetadataTypeEnum::PointerType(pointer_type) => {
                    builder.build_return(Some(&builder.build_load(
                        pointer_type,
                        ptr,
                        &var_name,
                    )?))?;
                },
                BasicMetadataTypeEnum::StructType(struct_type) => {
                    let loaded_struct = builder.build_load(struct_type, ptr, &var_name)?;

                    builder.build_return(Some(&loaded_struct))?;
                },
                BasicMetadataTypeEnum::VectorType(vector_type) => {
                    builder.build_return(Some(&builder.build_load(
                        vector_type,
                        ptr,
                        &var_name,
                    )?))?;
                },

                _ => unimplemented!(),
            };

            None
        },
        ParsedToken::If(if_definition) => {
            // Solve condition, this will contain whether the condition completes or not.
            let created_var = create_ir_from_parsed_token(
                ctx,
                module,
                builder,
                *if_definition.condition,
                variable_map,
                variable_reference,
                fn_ret_ty,
                this_fn_block,
                this_fn,
                allocation_table,
                is_loop_body.clone(),
                parsed_functions.clone(),
                custom_types.clone(),
            )?;

            if let Some((cond_ptr, cond_ty, ty_disc)) = created_var {
                let branch_compl = ctx.append_basic_block(this_fn, "cond_branch_true");
                let branch_incompl = ctx.append_basic_block(this_fn, "cond_branch_false");
                let branch_uncond = ctx.append_basic_block(this_fn, "cond_branch_uncond");

                builder.build_conditional_branch(
                    builder
                        .build_load(cond_ty.into_int_type(), cond_ptr, "condition")?
                        .into_int_value(),
                    branch_compl,
                    branch_incompl,
                )?;

                builder.position_at_end(branch_compl);

                create_ir_from_parsed_token_list(
                    module,
                    builder,
                    ctx,
                    if_definition.true_branch,
                    Type::Void,
                    branch_compl,
                    variable_map,
                    this_fn,
                    allocation_table,
                    is_loop_body.clone(),
                    parsed_functions.clone(),
                    custom_types.clone(),
                )?;

                builder.build_unconditional_branch(branch_uncond)?;

                builder.position_at_end(branch_incompl);

                // Parse the tokens from the incomplete branch
                create_ir_from_parsed_token_list(
                    module,
                    builder,
                    ctx,
                    if_definition.false_branch,
                    Type::Void,
                    branch_incompl,
                    variable_map,
                    this_fn,
                    allocation_table,
                    is_loop_body.clone(),
                    parsed_functions.clone(),
                    custom_types.clone(),
                )?;

                // Position the builder at the original position
                builder.build_unconditional_branch(branch_uncond)?;

                builder.position_at_end(branch_uncond);
            }
            else {
                return Err(CodeGenError::InvalidIfCondition.into());
            }

            None
        },
        // ParsedToken::InitializeStruct(struct_tys, struct_fields) => {
        //     if let Some((var_name, (var_ptr, var_ty), var_ty_disc)) = variable_reference {
        //         // Get the struct pointer's ty
        //         let pointee_struct_ty = var_ty.into_struct_type();

        //         // Pre-Allocate a struct so that it can be accessed later
        //         let allocate_struct = builder.build_alloca(pointee_struct_ty, "strct_init")?;

        //         // Iterate over the struct's fields
        //         for (field_idx, (field_name, field_ty)) in struct_tys.iter().enumerate() {
        //             // Convert to llvm type
        //             let llvm_ty = ty_to_llvm_ty(ctx, field_ty, custom_types.clone())?;

        //             // Create a new temp variable according to the struct's field type
        //             let (ptr, ty) = create_new_variable(
        //                 ctx,
        //                 builder,
        //                 field_name,
        //                 field_ty,
        //                 custom_types.clone(),
        //             )?;

        //             // Parse the value for the temp var
        //             create_ir_from_parsed_token(
        //                 ctx,
        //                 module,
        //                 builder,
        //                 *(struct_fields.get_index(field_idx).unwrap().1.clone()),
        //                 variable_map,
        //                 Some((field_name.to_string(), (ptr, ty), field_ty.clone())),
        //                 fn_ret_ty.clone(),
        //                 this_fn_block,
        //                 this_fn,
        //                 allocation_list,
        //                 is_loop_body.clone(),
        //                 parsed_functions.clone(),
        //                 custom_types.clone(),
        //             )?;

        //             // Load the temp value to memory and store it
        //             let temp_val = builder.build_load(llvm_ty, ptr, field_name)?;

        //             // Get the struct's field gep
        //             let struct_field_ptr = builder.build_struct_gep(
        //                 pointee_struct_ty,
        //                 allocate_struct,
        //                 field_idx as u32,
        //                 "field_gep",
        //             )?;

        //             // Store the temp value in the struct through the struct's field gep
        //             builder.build_store(struct_field_ptr, temp_val)?;
        //         }

        //         // Load the allocated struct into memory
        //         let constructed_struct = builder
        //             .build_load(pointee_struct_ty, allocate_struct, "constructed_struct")?
        //             .into_struct_value();

        //         // Store the struct in the main variable
        //         builder.build_store(var_ptr, constructed_struct)?;
        //     }

        //     // A struct will not be allocated without a variable storing it.
        //     None
        // },
        ParsedToken::Comparison(lhs, order, rhs, comparison_hand_side_ty) => {
            let pointee_ty = ty_to_llvm_ty(ctx, &comparison_hand_side_ty, custom_types.clone())?;

            let (lhs_ptr, lhs_ptr_ty) = create_new_variable(
                ctx,
                builder,
                "lhs_tmp",
                &comparison_hand_side_ty,
                None,
                allocation_table,
                custom_types.clone(),
            )?;
            let (rhs_ptr, rhs_ptr_ty) = create_new_variable(
                ctx,
                builder,
                "rhs_tmp",
                &comparison_hand_side_ty,
                None,
                allocation_table,
                custom_types.clone(),
            )?;

            create_ir_from_parsed_token(
                ctx,
                module,
                builder,
                *rhs,
                variable_map,
                Some((
                    "rhs_tmp".to_string(),
                    (rhs_ptr, rhs_ptr_ty),
                    comparison_hand_side_ty.clone(),
                )),
                fn_ret_ty.clone(),
                this_fn_block,
                this_fn,
                allocation_table,
                is_loop_body.clone(),
                parsed_functions.clone(),
                custom_types.clone(),
            )?;

            create_ir_from_parsed_token(
                ctx,
                module,
                builder,
                *lhs,
                variable_map,
                Some((
                    "lhs_tmp".to_string(),
                    (lhs_ptr, lhs_ptr_ty),
                    comparison_hand_side_ty.clone(),
                )),
                fn_ret_ty.clone(),
                this_fn_block,
                this_fn,
                allocation_table,
                is_loop_body.clone(),
                parsed_functions.clone(),
                custom_types.clone(),
            )?;

            let lhs_val = builder.build_load(pointee_ty, lhs_ptr, "lhs_tmp_val")?;
            let rhs_val = builder.build_load(pointee_ty, rhs_ptr, "rhs_tmp_val")?;

            let cmp_result = match comparison_hand_side_ty {
                Type::I16 | Type::I32 | Type::I64 => {
                    builder.build_int_compare(
                        order.into_int_predicate(true),
                        lhs_val.into_int_value(),
                        rhs_val.into_int_value(),
                        "cmp",
                    )?
                },
                Type::F16 | Type::F32 | Type::F64 => {
                    builder.build_float_compare(
                        order.into_float_predicate(),
                        lhs_val.into_float_value(),
                        rhs_val.into_float_value(),
                        "cmp",
                    )?
                },
                Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::Boolean => {
                    builder.build_int_compare(
                        order.into_int_predicate(false),
                        lhs_val.into_int_value(),
                        rhs_val.into_int_value(),
                        "cmp",
                    )?
                },
                Type::String => {
                    unimplemented!()
                },
                Type::Void => ctx.bool_type().const_int(1, false),
                Type::Struct(_) => {
                    unimplemented!()
                },
                Type::Array(type_discriminant) => unimplemented!(),
                Type::Pointer(_) => todo!(),
                Type::Enum(_) => todo!(),
            };

            if let Some((_, (var_ptr, _), ref_var_ty_disc)) = variable_reference {
                // Make sure that the variable we are setting is of type `Boolean` as a comparison always returns a `Bool`.
                if ref_var_ty_disc != Type::Boolean {
                    return Err(
                        CodeGenError::VariableTypeMismatch(ref_var_ty_disc, Type::Boolean).into(),
                    );
                }

                builder.build_store(var_ptr, cmp_result)?;

                None
            }
            else {
                let (v_ptr, v_ty) = create_new_variable(
                    ctx,
                    builder,
                    "cmp_result",
                    &Type::Boolean,
                    None,
                    allocation_table,
                    custom_types.clone(),
                )?;

                builder.build_store(v_ptr, cmp_result)?;

                Some((v_ptr, v_ty, Type::Boolean))
            }
        },
        ParsedToken::CodeBlock(parsed_tokens) => todo!(),
        ParsedToken::Loop(parsed_tokens) => {
            // Create the loop body
            let loop_body = ctx.append_basic_block(this_fn, "loop_body");

            // Create a the body of the code which would get executed after the loop body
            let loop_body_exit = ctx.append_basic_block(this_fn, "loop_body_exit");

            // Create a table of pre allocated variables so that they can be reused on every iteration
            // This contains all the pre allocated variables for the loop body. This makes it so that we dont allocate anything inside the loop body, thus avoiding stack overflows.
            let allocation_table =
                create_allocation_table(ctx, builder, &parsed_tokens, custom_types.clone())?;

            // Make the jump to the loop body
            builder.build_unconditional_branch(loop_body)?;

            // Position the builder at the loop body's beginning
            builder.position_at_end(loop_body);

            // Parse the tokens in the loop's body
            create_ir_from_parsed_token_list(
                module,
                builder,
                ctx,
                parsed_tokens,
                Type::Void,
                loop_body,
                variable_map,
                this_fn,
                &allocation_table,
                Some(LoopBodyBlocks::new(loop_body, loop_body_exit)),
                parsed_functions.clone(),
                custom_types.clone(),
            )?;

            // Create a jump to the beginning to the loop for an infinite loop
            builder.build_unconditional_branch(loop_body)?;

            // Reset the position of the builder
            builder.position_at_end(loop_body_exit);

            None
        },
        ParsedToken::ControlFlow(control_flow_variant) => {
            if let Some(loop_body_blocks) = is_loop_body {
                match control_flow_variant {
                    ControlFlowType::Break => {
                        builder.build_unconditional_branch(loop_body_blocks.loop_body_exit)?;
                    },
                    ControlFlowType::Continue => {
                        builder.build_unconditional_branch(loop_body_blocks.loop_body)?;
                    },
                }
            }
            else {
                return Err(CodeGenError::InvalidControlFlowUsage.into());
            }

            None
        },
        ParsedToken::ArrayInitialization(values, inner_ty) => {
            if let Some((_, (ptr, _ptr_ty), ty_disc)) = variable_reference
                && let Type::Array((_, len)) = ty_disc
            {
                let mut array_values: Vec<BasicValueEnum> = Vec::new();

                for val in values {
                    let (temp_var_ptr, temp_var_ty) = create_new_variable(
                        ctx,
                        builder,
                        "array_temp_val_var",
                        &inner_ty,
                        None,
                        allocation_table,
                        custom_types.clone(),
                    )?;

                    create_ir_from_parsed_token(
                        ctx,
                        module,
                        builder,
                        val,
                        variable_map,
                        Some((
                            "array_temp_val_var".to_string(),
                            (temp_var_ptr, temp_var_ty),
                            (inner_ty).clone(),
                        )),
                        fn_ret_ty.clone(),
                        this_fn_block,
                        this_fn,
                        allocation_table,
                        is_loop_body.clone(),
                        parsed_functions.clone(),
                        custom_types.clone(),
                    )?;

                    let value = builder.build_load(
                        ty_to_llvm_ty(ctx, &inner_ty, custom_types.clone())?,
                        temp_var_ptr,
                        "array_temp_val_deref",
                    )?;

                    array_values.push(value);
                }

                if array_values.len() != len {
                    return Err(CodeGenError::ArrayLengthMismatch(len, array_values.len()).into());
                }

                let array_base = ctx.i32_type().const_int(0, false);

                for (idx, val) in array_values.iter().enumerate() {
                    let array_idx = ctx.i32_type().const_int(idx as u64, false);

                    let elem_ptr = unsafe {
                        builder.build_gep(
                            ty_to_llvm_ty(ctx, &ty_disc, custom_types.clone())?,
                            ptr,
                            &[array_base, array_idx],
                            "array_idx_val",
                        )?
                    };

                    builder.build_store(elem_ptr, *val)?;
                }
            }

            None
        },
        ParsedToken::GetPointerTo(value) => {
            // Try to get a value from this enum
            let val_reference = create_ir_from_parsed_token(
                ctx,
                module,
                builder,
                *value.clone(),
                variable_map,
                None,
                fn_ret_ty.clone(),
                this_fn_block,
                this_fn,
                allocation_table,
                is_loop_body.clone(),
                parsed_functions.clone(),
                custom_types.clone(),
            )?;

            // If we did not get any value, we cannot return a pointer to it, thus we return an error
            match val_reference {
                Some((ptr, ty, ty_disc)) => {
                    match variable_reference {
                        Some((_, var_ty, var_ty_disc)) => {
                            let (var_ptr, _var_type) = var_ty;

                            // Check the type of the passed in variable to see if it is a pointer
                            if !matches!(var_ty_disc, Type::Pointer(_)) {
                                return Err(CodeGenError::VariableTypeMismatch(
                                    var_ty_disc,
                                    Type::Pointer(None),
                                )
                                .into());
                            }

                            // If the pointer's inner type has been defined run the type check
                            // If not ignore the type check
                            if let Type::Pointer(Some(inner_token)) = var_ty_disc {
                                let reference_inner_ty =
                                    ty_from_token(&*inner_token, &custom_types)?;

                                // Check the type of the value, check for a type mismatch inside the pointer
                                if reference_inner_ty != ty_disc {
                                    return Err(CodeGenError::VariableTypeMismatch(
                                        reference_inner_ty,
                                        ty_disc,
                                    )
                                    .into());
                                }
                            }

                            builder.build_store(var_ptr, ptr)?;

                            return Ok(None);
                        },
                        None => {
                            return Ok(Some((
                                ptr,
                                ty,
                                Type::Pointer(Some(Box::new(Token::TypeDefinition(ty_disc)))),
                            )));
                        },
                    }
                },
                None => {
                    return Err(CodeGenError::GetPointerToFailed(value.inner.clone()).into());
                },
            }
        },
        ParsedToken::DerefPointer(inner_value) => {
            // Try to get a pointer from this inner_value
            // This is a shallow pointer (if not nested by user)
            // Pointer -> Value
            let val_reference = create_ir_from_parsed_token(
                ctx,
                module,
                builder,
                *inner_value.clone(),
                variable_map,
                None,
                fn_ret_ty.clone(),
                this_fn_block,
                this_fn,
                allocation_table,
                is_loop_body.clone(),
                parsed_functions.clone(),
                custom_types.clone(),
            )?;

            match val_reference {
                Some((ptr, ty, ty_disc)) => {
                    // Check the type of the passed in variable to see if it is a pointer
                    if !matches!(ty_disc, Type::Pointer(_)) {
                        return Err(CodeGenError::VariableTypeMismatch(
                            ty_disc,
                            Type::Pointer(None),
                        )
                        .into());
                    }

                    match variable_reference {
                        Some((_, var_ty, var_ref_ty_disc)) => {
                            let (var_ptr, _var_type) = var_ty;

                            let ptr_variant = ty_disc.try_as_pointer().unwrap();

                            // If the inner value does not have a pre-determined inner type of the value the pointer is pointing to, assume the type we want to dereference to is the variable's type
                            let deref_ty: Type = if let Some(pointer_inner) = ptr_variant {
                                ty_from_token(&*pointer_inner, &custom_types)?
                            }
                            else {
                                var_ref_ty_disc.clone()
                            };

                            // Check for a type mismatch inside the pointer
                            if deref_ty != var_ref_ty_disc {
                                return Err(CodeGenError::VariableTypeMismatch(
                                    deref_ty,
                                    var_ref_ty_disc,
                                )
                                .into());
                            }

                            // Load the pointer
                            let inner_ptr = builder.build_load(
                                ctx.ptr_type(AddressSpace::default()),
                                ptr,
                                "ptr_load",
                            )?;

                            // Dereference the pointer
                            let dereferenced_value = builder.build_load(
                                deref_ty.to_basic_type_enum(ctx, custom_types)?,
                                inner_ptr.into_pointer_value(),
                                "ptr_deref",
                            )?;

                            // Store the deref-ed value inside the variable.
                            builder.build_store(var_ptr, dereferenced_value)?;

                            None
                        },
                        None => {
                            Some((ptr, ty, {
                                match ty_disc.try_as_pointer().unwrap() {
                                    Some(pointer_inner) => {
                                        ty_from_token(&*pointer_inner, &custom_types)?
                                    },
                                    None => {
                                        return Err(CodeGenError::VagueDereference.into());
                                    },
                                }
                            }))
                        },
                    }
                },
                None => {
                    return Err(
                        CodeGenError::InvalidValueDereference(inner_value.inner.clone()).into(),
                    );
                },
            }
        },
    };

    Ok(created_var)
}

/// This function is solely for generating the LLVM-IR from the main sourec file.
pub fn generate_ir<'ctx>(
    parsed_functions: Rc<IndexMap<String, FunctionDefinition>>,
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &'ctx Builder<'ctx>,
    custom_types: Rc<IndexMap<String, CustomType>>,
    is_optimized: bool,
    flags_passed_in: &str,
    path_to_src_file: &str,
) -> Result<()>
{
    let (debug_info_builder, debug_info_compile_uint) = module.create_debug_info_builder(
        false,
        DWARFSourceLanguage::C,
        module.get_name().to_str()?,
        path_to_src_file,
        &format!(
            "Fog (ver.: {}) with LLVM {}",
            env!("CARGO_PKG_VERSION"),
            env!("LLVM_VERSION")
        ),
        is_optimized,
        flags_passed_in,
        1,
        "",
        {
            if is_optimized {
                DWARFEmissionKind::LineTablesOnly
            }
            else {
                DWARFEmissionKind::Full
            }
        },
        0,
        false,
        !is_optimized,
        "",
        "",
    );

    let dbg_version = context.i32_type().const_int(1, false);
    let dbg_version_md = context.metadata_node(&[dbg_version.as_basic_value_enum().into()]);
    module
        .add_global_metadata("llvm.debug.version", &dbg_version_md)
        .unwrap();

    let debug_info_file = debug_info_compile_uint.get_file();

    let debug_scope = debug_info_file.as_debug_info_scope();

    let mut unique_id_source = 0;

    for (function_name, function_definition) in parsed_functions.iter() {
        let function_type = create_fn_type_from_ty_disc(
            context,
            function_definition.signature.clone(),
            custom_types.clone(),
        )?;

        // Create function signature
        let function = module.add_function(function_name, function_type, None);

        for hint in function_definition.signature.compiler_hints.iter() {
            match hint {
                CompilerHint::Cold => {
                    let attr =
                        context.create_enum_attribute(Attribute::get_named_enum_kind_id("cold"), 0);

                    function
                        .add_attribute(common::inkwell::attributes::AttributeLoc::Function, attr);
                },
                CompilerHint::NoFree => {
                    let attr = context
                        .create_enum_attribute(Attribute::get_named_enum_kind_id("nofree"), 0);

                    function
                        .add_attribute(common::inkwell::attributes::AttributeLoc::Function, attr);
                },
                CompilerHint::Inline => {
                    let attr = context
                        .create_enum_attribute(Attribute::get_named_enum_kind_id("inlinehint"), 0);

                    function
                        .add_attribute(common::inkwell::attributes::AttributeLoc::Function, attr);
                },
                CompilerHint::NoUnWind => {
                    let attr = context
                        .create_enum_attribute(Attribute::get_named_enum_kind_id("nounwind"), 0);

                    function
                        .add_attribute(common::inkwell::attributes::AttributeLoc::Function, attr);
                },
                CompilerHint::Feature => {
                    return Err(CodeGenError::InternalFunctionCompilerHintParsingError(
                        hint.clone(),
                    )
                    .into());
                },
            }
        }

        let return_type = function_definition.signature.return_type.clone();

        if !is_optimized {
            let debug_subprogram = create_subprogram_debug_information(
                context,
                module,
                custom_types.clone(),
                is_optimized,
                &debug_info_builder,
                debug_info_file,
                debug_scope,
                &mut unique_id_source,
                function_name,
                function_definition,
                return_type,
            )
            .map_err(|err| CodeGenError::LibraryLLVMError(err.to_string()))?;

            function.set_subprogram(debug_subprogram);
        }

        // Create a BasicBlock to store the IR in
        let basic_block = context.append_basic_block(function, "main_fn_entry");

        // Insert the BasicBlock at the end
        builder.position_at_end(basic_block);

        // Check if the return type is Void.
        // We dont require the user to insert a return void instruction, instead we do it automaticly.
        if function_definition.signature.return_type == Type::Void {
            // Insert the return void instruction
            let instruction = builder.build_return(None)?;

            // Put the builder before that instruction so that it can resume generating IR
            builder.position_before(&instruction);
        }

        // Create a HashMap of the arguments the function takes
        let mut arguments: HashMap<String, (BasicValueEnum, (Type, UniqueId))> = HashMap::new();

        // Get the arguments and store them in the HashMap
        for (idx, argument) in function.get_param_iter().enumerate() {
            // Get the name of the argument from the function signature's argument list
            let argument_entry = function_definition
                .signature
                .args
                .arguments
                .get_index(idx)
                .unwrap();

            // Set the name of the arguments so that it is easier to debug later
            argument.set_name(argument_entry.0);

            // Insert the entry
            arguments.insert(
                argument_entry.0.clone(),
                (argument, argument_entry.1.clone()),
            );
        }

        // Iterate through all the `ParsedToken`-s and create the LLVM-IR from the tokens
        create_ir(
            module,
            builder,
            context,
            function_definition.inner.clone(),
            arguments,
            function_definition.signature.return_type.clone(),
            basic_block,
            function,
            parsed_functions.clone(),
            custom_types.clone(),
        )?;
    }

    Ok(())
}

pub fn create_ir_from_parsed_token_list<'main, 'ctx>(
    module: &Module<'ctx>,
    // Inkwell IR builder
    builder: &'ctx Builder<'ctx>,
    // Inkwell Context
    ctx: &'main Context,
    // The list of `ParsedToken`s
    parsed_tokens: Vec<ParsedTokenInstance>,
    // Type returned type of the Function
    fn_ret_ty: Type,
    this_fn_block: BasicBlock<'ctx>,
    variable_map: &mut HashMap<
        String,
        (
            (PointerValue<'ctx>, BasicMetadataTypeEnum<'ctx>),
            (Type, UniqueId),
        ),
    >,
    this_fn: FunctionValue<'ctx>,
    // Allocation tables are used when the ParsedTokens run in a loop
    // We store the addresses and ids of the variables which have been allocated previously to entering the loop, to avoid a stack overflow
    // Loops should not create new variables on the stack instead they should be using `alloca_table` to look up pointers.
    alloca_table: &HashMap<UniqueId, PointerValue<'ctx>>,
    is_loop_body: Option<LoopBodyBlocks>,
    parsed_functions: Rc<IndexMap<String, FunctionDefinition>>,
    custom_items: Rc<IndexMap<String, CustomType>>,
) -> Result<()>
where
    'main: 'ctx,
{
    for token in parsed_tokens {
        create_ir_from_parsed_token(
            ctx,
            module,
            builder,
            token.clone(),
            variable_map,
            None,
            fn_ret_ty.clone(),
            this_fn_block,
            this_fn,
            alloca_table,
            is_loop_body.clone(),
            parsed_functions.clone(),
            custom_items.clone(),
        )?;
    }

    Ok(())
}
