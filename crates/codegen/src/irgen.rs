use fog_common::{
    anyhow::Result,
    codegen::{
        CustomType, FunctionArgumentIdentifier, LoopBodyBlocks, create_fn_type_from_ty_disc,
        fn_arg_to_string, ty_enum_to_metadata_ty_enum, ty_to_llvm_ty,
    },
    error::codegen::CodeGenError,
    indexmap::IndexMap,
    inkwell::{
        basic_block::BasicBlock,
        builder::Builder,
        context::Context,
        debug_info::{
            AsDIScope, DWARFEmissionKind,
            DWARFSourceLanguage,
        },
        module::Module,
        types::BasicMetadataTypeEnum,
        values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, PointerValue},
    },
    parser::{FunctionDefinition, ParsedToken},
    ty::{OrdMap, TypeDiscriminant, token_to_ty},
};
use std::{
    collections::{HashMap, VecDeque},
    rc::Rc,
    sync::Arc,
};

use crate::{access_array_index, allocate::{allocate_string, create_alloca_table, create_new_variable}, create_ir_from_parsed_token_list, debug::create_subprogram_debug_information, pointer::{access_nested_struct_field_ptr, access_variable_ptr, set_value_of_ptr}};

pub fn create_ir<'main, 'ctx>(
    module: &Module<'ctx>,
    // Inkwell IR builder
    builder: &'ctx Builder<'ctx>,
    // Inkwell Context
    ctx: &'main Context,
    // The list of ParsedToken-s
    parsed_tokens: Vec<ParsedToken>,
    // This argument is initialized with the HashMap of the arguments
    available_arguments: HashMap<String, (BasicValueEnum<'ctx>, TypeDiscriminant)>,
    // Type returned type of the Function
    fn_ret_ty: TypeDiscriminant,
    this_fn_block: BasicBlock<'ctx>,
    this_fn: FunctionValue<'ctx>,
    parsed_functions: Rc<IndexMap<String, FunctionDefinition>>,
    custom_items: Arc<IndexMap<String, CustomType>>,
) -> Result<()>
where
    'main: 'ctx,
{
    let mut variable_map: HashMap<
        String,
        ((PointerValue, BasicMetadataTypeEnum), TypeDiscriminant),
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
        &mut VecDeque::new(),
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
    parsed_token: ParsedToken,
    variable_map: &mut HashMap<
        String,
        (
            (PointerValue<'ctx>, BasicMetadataTypeEnum<'ctx>),
            TypeDiscriminant,
        ),
    >,
    variable_reference: Option<(
        String,
        (PointerValue<'ctx>, BasicMetadataTypeEnum<'ctx>),
        TypeDiscriminant,
    )>,

    // Type returned type of the Function
    fn_ret_ty: TypeDiscriminant,
    this_fn_block: BasicBlock<'ctx>,
    this_fn: FunctionValue<'ctx>,
    allocation_list: &mut VecDeque<(
        ParsedToken,
        PointerValue<'ctx>,
        BasicMetadataTypeEnum<'ctx>,
        TypeDiscriminant,
    )>,
    is_loop_body: Option<LoopBodyBlocks>,
    parsed_functions: Rc<IndexMap<String, FunctionDefinition>>,
    custom_types: Arc<IndexMap<String, CustomType>>,
) -> Result<
    // This optional return value is the reference to the value of a ParsedToken's result. ie: Comparsions return a Some(ptr) to the bool value of the comparison
    // The return value is None if the `variable_reference` of the function is `Some`, as the variable will have its value set to the value of the returned value.
    Option<(
        // Pointer to the referenced variable
        PointerValue<'ctx>,
        // Type of the variable
        BasicMetadataTypeEnum<'ctx>,
        // TypeDiscriminant of the variable
        TypeDiscriminant,
    )>,
>
where
    'main: 'ctx,
{
    let created_var = match parsed_token.clone() {
        ParsedToken::NewVariable(var_name, var_type, var_set_val) => {
            let mut was_preallocated = false;

            // Check if the function has been called with an allocation table
            let (ptr, ty) = (|| -> Result<(PointerValue, BasicMetadataTypeEnum)> {
                if let Some((current_token, ptr, ptr_ty, ty)) = allocation_list.front().cloned()
                    && current_token == parsed_token.clone()
                {
                    if ty == var_type {
                        was_preallocated = true;

                        allocation_list.pop_front();

                        return Ok((ptr, ptr_ty));
                    }
                    else {
                        return Err(CodeGenError::InvalidPreAllocation.into());
                    }
                }

                let (ptr, ptr_ty) =
                    create_new_variable(ctx, builder, &var_name, &var_type, custom_types.clone())?;

                variable_map.insert(var_name.clone(), ((ptr, ptr_ty), var_type.clone()));

                Ok((ptr, ptr_ty))
            })()?;

            // Check if the value was preallocated and is a literal, if yes we dont need to set the value of the variable as it was done beforehand.
            if !(matches!(&*var_set_val, ParsedToken::Literal(_)) && was_preallocated) {
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
                    allocation_list,
                    is_loop_body,
                    parsed_functions.clone(),
                    custom_types.clone(),
                )?;
            }

            // We do not have to return anything here since a variable handle cannot really be casted to anything, its also top level
            None
        },
        // In one of the cases we are returning a value in the other we are returning a pointer to the value please check it
        ParsedToken::VariableReference(var_ref_variant) => {
            if let Some((var_ref_name, (var_ref_ptr, var_ref_ty), var_ref_ty_disc)) =
                variable_reference
            {
                match var_ref_variant {
                    fog_common::parser::VariableReference::StructFieldReference(
                        struct_field_stack,
                        (struct_name, struct_fields),
                    ) => {
                        let mut field_stack_iter = struct_field_stack.field_stack.iter();

                        if let Some(main_struct_var_name) = field_stack_iter.next() {
                            if let Some(((ptr, ty), ty_disc)) =
                                variable_map.get(main_struct_var_name)
                            {
                                let (f_ptr, f_ty, ty_disc) = access_nested_struct_field_ptr(
                                    ctx,
                                    builder,
                                    &mut field_stack_iter,
                                    &struct_fields,
                                    (*ptr, *ty),
                                    custom_types.clone(),
                                )?;

                                let basic_value =
                                    builder.build_load(f_ty, f_ptr, "deref_strct_val")?;

                                if var_ref_ty.is_struct_type()
                                    && basic_value.is_struct_value()
                                    && var_ref_ty.into_struct_type().get_name()
                                        != Some(basic_value.into_struct_value().get_name())
                                {
                                    return Err(CodeGenError::InternalTypeMismatch.into());
                                }

                                if var_ref_ty == basic_value.get_type().into() {
                                    builder.build_store(var_ref_ptr, basic_value)?;
                                }
                                else {
                                    return Err(CodeGenError::InternalTypeMismatch.into());
                                }
                            }
                        }
                        else {
                            return Err(CodeGenError::InternalInvalidStructReference.into());
                        }
                    },
                    fog_common::parser::VariableReference::BasicReference(var_name) => {
                        // The referenced variable
                        let ref_variable_query = variable_map.get(&var_name);

                        if let ((orig_ptr, orig_ty), Some(((ref_ptr, ref_ty), ref_ty_disc))) = (
                            // The original variable we are going to modify
                            (var_ref_ptr, var_ref_ty),
                            // The referenced variable we are going to set the value of the orginal variable with
                            ref_variable_query,
                        ) {
                            if *ref_ty_disc != var_ref_ty_disc {
                                return Err(CodeGenError::InternalVariableTypeMismatch(
                                    ref_ty_disc.clone(),
                                    var_ref_ty_disc.clone(),
                                )
                                .into());
                            }

                            match ref_ty {
                                BasicMetadataTypeEnum::ArrayType(array_type) => {
                                    // Get the referenced variable's value
                                    let ref_var_val =
                                        builder.build_load(*array_type, *ref_ptr, "var_deref")?;

                                    // Store the referenced variable's value in the original
                                    builder.build_store(orig_ptr, ref_var_val)?;
                                },
                                BasicMetadataTypeEnum::FloatType(float_type) => {
                                    // Get the referenced variable's value
                                    let ref_var_val =
                                        builder.build_load(*float_type, *ref_ptr, "var_deref")?;

                                    // Store the referenced variable's value in the original
                                    builder.build_store(orig_ptr, ref_var_val)?;
                                },
                                BasicMetadataTypeEnum::IntType(int_type) => {
                                    // Get the referenced variable's value
                                    let ref_var_val =
                                        builder.build_load(*int_type, *ref_ptr, "var_deref")?;

                                    // Store the referenced variable's value in the original
                                    builder.build_store(orig_ptr, ref_var_val)?;
                                },
                                BasicMetadataTypeEnum::PointerType(pointer_type) => {
                                    // Get the referenced variable's value
                                    let ref_var_val =
                                        builder.build_load(*pointer_type, *ref_ptr, "var_deref")?;

                                    // Store the referenced variable's value in the original
                                    builder.build_store(orig_ptr, ref_var_val)?;
                                },
                                BasicMetadataTypeEnum::StructType(struct_type) => {
                                    // Get the referenced variable's value
                                    let ref_var_val =
                                        builder.build_load(*struct_type, *ref_ptr, "var_deref")?;

                                    // Store the referenced variable's value in the original
                                    builder.build_store(orig_ptr, ref_var_val)?;
                                },
                                BasicMetadataTypeEnum::VectorType(vector_type) => {
                                    // Get the referenced variable's value
                                    let ref_var_val =
                                        builder.build_load(*vector_type, *ref_ptr, "var_deref")?;

                                    // Store the referenced variable's value in the original
                                    builder.build_store(orig_ptr, ref_var_val)?;
                                },

                                _ => unimplemented!(),
                            };
                        }
                    },
                    fog_common::parser::VariableReference::ArrayReference(
                        variable_reference,
                        index,
                    ) => {
                        let variable_ptr = variable_map
                            .get(&variable_reference)
                            .ok_or(CodeGenError::InternalVariableNotFound(
                                variable_reference.clone(),
                            ))?
                            .clone();

                        let (ptr, ptr_ty, ty_disc) = access_array_index(
                            ctx,
                            module,
                            builder,
                            variable_map,
                            &fn_ret_ty,
                            this_fn_block,
                            this_fn,
                            allocation_list,
                            &is_loop_body,
                            &parsed_functions,
                            &custom_types,
                            variable_ptr,
                            index,
                        )?;

                        if var_ref_ty_disc != ty_disc {
                            return Err(CodeGenError::InternalVariableTypeMismatch(
                                var_ref_ty_disc.clone(),
                                ty_disc.clone(),
                            )
                            .into());
                        }

                        builder.build_store(var_ref_ptr, ptr)?;
                    },
                }

                None
            }
            else {
                match var_ref_variant {
                    fog_common::parser::VariableReference::StructFieldReference(
                        struct_field_stack,
                        (struct_name, struct_def),
                    ) => {
                        let mut field_stack_iter = struct_field_stack.field_stack.iter();

                        if let Some(main_struct_var_name) = field_stack_iter.next() {
                            if let Some(((ptr, ty), ty_disc)) =
                                variable_map.get(main_struct_var_name)
                            {
                                let (f_ptr, f_ty, ty_disc) = access_nested_struct_field_ptr(
                                    ctx,
                                    builder,
                                    &mut field_stack_iter,
                                    &struct_def,
                                    (*ptr, *ty),
                                    custom_types.clone(),
                                )?;

                                Some((f_ptr, ty_enum_to_metadata_ty_enum(f_ty), ty_disc))
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
                    fog_common::parser::VariableReference::BasicReference(basic_ref) => {
                        let ((ptr, ty), ty_disc) = variable_map
                            .get(&basic_ref)
                            .ok_or(CodeGenError::InternalVariableNotFound(basic_ref.clone()))?;

                        Some((*ptr, *ty, ty_disc.clone()))
                    },
                    fog_common::parser::VariableReference::ArrayReference(
                        variable_reference,
                        index,
                    ) => {
                        let variable_ptr = variable_map
                            .get(&variable_reference)
                            .ok_or(CodeGenError::InternalVariableNotFound(
                                variable_reference.clone(),
                            ))?
                            .clone();

                        let array_ptr = access_array_index(
                            ctx,
                            module,
                            builder,
                            variable_map,
                            &fn_ret_ty,
                            this_fn_block,
                            this_fn,
                            allocation_list,
                            &is_loop_body,
                            &parsed_functions,
                            &custom_types,
                            variable_ptr,
                            index,
                        )?;

                        Some(array_ptr)
                    },
                }
            }
        },
        ParsedToken::Literal(literal) => {
            // There this is None there is nothing we can do with this so just return
            if let Some(var_ref) = variable_reference {
                let (ptr, _var_type) = var_ref.1;

                // Check the type of the value, check for a type mismatch
                if literal.discriminant() != var_ref.2 {
                    return Err(CodeGenError::InternalVariableTypeMismatch(
                        literal.discriminant(),
                        var_ref.2,
                    )
                    .into());
                }

                set_value_of_ptr(ctx, builder, module, literal, ptr)?;

                None
            }
            else {
                let ty_disc = literal.discriminant();

                let (v_ptr, v_ty) =
                    create_new_variable(ctx, builder, "", &ty_disc, custom_types.clone())?;

                set_value_of_ptr(ctx, builder, module, literal, v_ptr)?;

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
                allocation_list,
                is_loop_body.clone(),
                parsed_functions.clone(),
                custom_types.clone(),
            )?;

            if let Some((var_ptr, var_ty, ty_disc)) = created_var {
                let (ref_ptr, ty_disc) = if let Some((var_name, (ref_ptr, ref_ty), ty_disc)) =
                    variable_reference.clone()
                {
                    (ref_ptr, ty_disc)
                }
                else {
                    let (ptr, ptr_ty) = create_new_variable(
                        ctx,
                        builder,
                        "ty_cast_temp_val",
                        &ty_disc,
                        custom_types.clone(),
                    )?;

                    (ptr, ty_disc)
                };

                match ty_disc {
                    TypeDiscriminant::I64 | TypeDiscriminant::I32 | TypeDiscriminant::I16 => {
                        match desired_type {
                            TypeDiscriminant::I64 => {
                                builder.build_store(
                                    ref_ptr,
                                    builder.build_load(var_ty.into_int_type(), var_ptr, "")?,
                                )?;
                            },
                            TypeDiscriminant::F64 => {
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
                            TypeDiscriminant::U64 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res =
                                    builder.build_int_cast(value, ctx.i64_type(), "i64_to_u64")?;

                                builder.build_store(ref_ptr, cast_res)?;
                            },
                            TypeDiscriminant::I32 | TypeDiscriminant::U32 => {
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
                            TypeDiscriminant::F32 => {
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
                            TypeDiscriminant::I16 | TypeDiscriminant::U16 => {
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
                            TypeDiscriminant::F16 => {
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
                            TypeDiscriminant::U8 => {
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
                            TypeDiscriminant::String => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let raw_val = value.get_sign_extended_constant().unwrap();

                                let int_string = raw_val.to_string();

                                let (buf_ptr, buf_ty) =
                                    allocate_string(builder, ctx.i8_type(), int_string)?;

                                builder.build_store(ref_ptr, buf_ptr)?;
                            },
                            TypeDiscriminant::Boolean => {
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
                            TypeDiscriminant::Void => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            },
                            TypeDiscriminant::Struct(_) => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            },
                            TypeDiscriminant::Array(_) => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            },
                        }
                    },
                    TypeDiscriminant::F64 | TypeDiscriminant::F32 | TypeDiscriminant::F16 => {
                        match desired_type {
                            TypeDiscriminant::I64 => {
                                let value = builder.build_float_to_signed_int(
                                    builder
                                        .build_load(var_ty.into_float_type(), var_ptr, "")?
                                        .into_float_value(),
                                    ctx.i64_type(),
                                    "",
                                )?;
                                builder.build_store(ref_ptr, value)?;
                            },
                            TypeDiscriminant::F64 => {
                                builder.build_store(
                                    ref_ptr,
                                    builder.build_load(var_ty.into_float_type(), var_ptr, "")?,
                                )?;
                            },
                            TypeDiscriminant::U64 => {
                                let value = builder.build_float_to_unsigned_int(
                                    builder
                                        .build_load(var_ty.into_float_type(), var_ptr, "")?
                                        .into_float_value(),
                                    ctx.i64_type(),
                                    "",
                                )?;
                                builder.build_store(ref_ptr, value)?;
                            },
                            TypeDiscriminant::I32 => {
                                let value = builder.build_float_to_signed_int(
                                    builder
                                        .build_load(var_ty.into_float_type(), var_ptr, "")?
                                        .into_float_value(),
                                    ctx.i32_type(),
                                    "",
                                )?;
                                builder.build_store(ref_ptr, value)?;
                            },
                            TypeDiscriminant::F32 => {
                                let value = builder
                                    .build_load(var_ty.into_float_type(), var_ptr, "")?
                                    .into_float_value();

                                let cast_res =
                                    ctx.f32_type().const_float(value.get_constant().unwrap().0);

                                builder.build_store(ref_ptr, cast_res)?;
                            },
                            TypeDiscriminant::U32 => {
                                let value = builder.build_float_to_unsigned_int(
                                    builder
                                        .build_load(var_ty.into_float_type(), var_ptr, "")?
                                        .into_float_value(),
                                    ctx.i32_type(),
                                    "",
                                )?;
                                builder.build_store(ref_ptr, value)?;
                            },
                            TypeDiscriminant::I16 => {
                                let value = builder.build_float_to_signed_int(
                                    builder
                                        .build_load(var_ty.into_float_type(), var_ptr, "")?
                                        .into_float_value(),
                                    ctx.i16_type(),
                                    "",
                                )?;
                                builder.build_store(ref_ptr, value)?;
                            },
                            TypeDiscriminant::F16 => {
                                let value = builder
                                    .build_load(var_ty.into_float_type(), var_ptr, "")?
                                    .into_float_value();

                                let cast_res =
                                    ctx.f16_type().const_float(value.get_constant().unwrap().0);

                                builder.build_store(ref_ptr, cast_res)?;
                            },
                            TypeDiscriminant::U16 => {
                                let value = builder.build_float_to_unsigned_int(
                                    builder
                                        .build_load(var_ty.into_float_type(), var_ptr, "")?
                                        .into_float_value(),
                                    ctx.i16_type(),
                                    "",
                                )?;
                                builder.build_store(ref_ptr, value)?;
                            },
                            TypeDiscriminant::U8 => {
                                let value = builder.build_float_to_unsigned_int(
                                    builder
                                        .build_load(var_ty.into_float_type(), var_ptr, "")?
                                        .into_float_value(),
                                    ctx.i8_type(),
                                    "",
                                )?;
                                builder.build_store(ref_ptr, value)?;
                            },
                            TypeDiscriminant::String => {
                                let value = builder
                                    .build_load(var_ty.into_float_type(), var_ptr, "")?
                                    .into_float_value();

                                let raw_val = value.get_constant().unwrap().0;

                                let int_string = raw_val.to_string();

                                let (buf_ptr, buf_ty) =
                                    allocate_string(builder, ctx.i8_type(), int_string)?;

                                builder.build_store(ref_ptr, buf_ptr)?;
                            },
                            TypeDiscriminant::Boolean => {
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
                            TypeDiscriminant::Void => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            },
                            TypeDiscriminant::Struct(_) => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            },
                            TypeDiscriminant::Array(_) => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            },
                        }
                    },
                    TypeDiscriminant::U64
                    | TypeDiscriminant::U32
                    | TypeDiscriminant::U16
                    | TypeDiscriminant::U8 => {
                        match desired_type {
                            TypeDiscriminant::I64 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res = ctx.i64_type().const_int(
                                    value.get_sign_extended_constant().unwrap() as u64,
                                    true,
                                );

                                builder.build_store(ref_ptr, cast_res)?;
                            },
                            TypeDiscriminant::F64 => {
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
                            TypeDiscriminant::U64 => {
                                builder.build_store(
                                    ref_ptr,
                                    builder.build_load(var_ty.into_int_type(), var_ptr, "")?,
                                )?;
                            },
                            TypeDiscriminant::I32 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res = ctx.i32_type().const_int(
                                    value.get_sign_extended_constant().unwrap() as u64,
                                    true,
                                );

                                builder.build_store(ref_ptr, cast_res)?;
                            },
                            TypeDiscriminant::F32 => {
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
                            TypeDiscriminant::U32 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res = ctx.i32_type().const_int(
                                    value.get_sign_extended_constant().unwrap() as u64,
                                    false,
                                );

                                builder.build_store(ref_ptr, cast_res)?;
                            },
                            TypeDiscriminant::I16 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res = ctx.i16_type().const_int(
                                    value.get_sign_extended_constant().unwrap() as u64,
                                    true,
                                );

                                builder.build_store(ref_ptr, cast_res)?;
                            },
                            TypeDiscriminant::F16 => {
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
                            TypeDiscriminant::U16 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res = ctx.i16_type().const_int(
                                    value.get_sign_extended_constant().unwrap() as u64,
                                    false,
                                );

                                builder.build_store(ref_ptr, cast_res)?;
                            },
                            TypeDiscriminant::U8 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res = ctx.i8_type().const_int(
                                    value.get_sign_extended_constant().unwrap() as u64,
                                    false,
                                );

                                builder.build_store(ref_ptr, cast_res)?;
                            },
                            TypeDiscriminant::String => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let raw_val = value.get_sign_extended_constant().unwrap();

                                let int_string = raw_val.to_string();

                                let (buf_ptr, buf_ty) =
                                    allocate_string(builder, ctx.i8_type(), int_string)?;

                                builder.build_store(ref_ptr, buf_ptr)?;
                            },
                            TypeDiscriminant::Boolean => {
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
                            TypeDiscriminant::Void => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            },
                            TypeDiscriminant::Struct(_) => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            },
                            TypeDiscriminant::Array(_) => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            },
                        }
                    },
                    TypeDiscriminant::String => {
                        match desired_type {
                            TypeDiscriminant::I64 => todo!(),
                            TypeDiscriminant::F64 => todo!(),
                            TypeDiscriminant::U64 => todo!(),
                            TypeDiscriminant::I32 => todo!(),
                            TypeDiscriminant::F32 => todo!(),
                            TypeDiscriminant::U32 => todo!(),
                            TypeDiscriminant::I16 => todo!(),
                            TypeDiscriminant::F16 => todo!(),
                            TypeDiscriminant::U16 => todo!(),
                            TypeDiscriminant::U8 => todo!(),
                            TypeDiscriminant::String => todo!(),
                            TypeDiscriminant::Boolean => todo!(),
                            TypeDiscriminant::Void => todo!(),
                            TypeDiscriminant::Struct(_) => todo!(),
                            TypeDiscriminant::Array(_) => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            },
                        }
                    },
                    TypeDiscriminant::Boolean => {
                        match desired_type {
                            TypeDiscriminant::I64 => {
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
                            TypeDiscriminant::F64 => {
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
                            TypeDiscriminant::U64 => {
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
                            TypeDiscriminant::I32 => {
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
                            TypeDiscriminant::F32 => {
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
                            TypeDiscriminant::U32 => {
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
                            TypeDiscriminant::I16 => {
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
                            TypeDiscriminant::F16 => {
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
                            TypeDiscriminant::U16 => {
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
                            TypeDiscriminant::U8 => {
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
                            TypeDiscriminant::String => {
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
                            TypeDiscriminant::Boolean => {
                                builder.build_store(
                                    ref_ptr,
                                    builder.build_load(var_ty.into_int_type(), var_ptr, "")?,
                                )?;
                            },
                            TypeDiscriminant::Void => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            },
                            TypeDiscriminant::Struct(_) => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            },
                            TypeDiscriminant::Array(_) => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            },
                        }
                    },
                    TypeDiscriminant::Void => {
                        match desired_type {
                            TypeDiscriminant::I64 => todo!(),
                            TypeDiscriminant::F64 => todo!(),
                            TypeDiscriminant::U64 => todo!(),
                            TypeDiscriminant::I32 => todo!(),
                            TypeDiscriminant::F32 => todo!(),
                            TypeDiscriminant::U32 => todo!(),
                            TypeDiscriminant::I16 => todo!(),
                            TypeDiscriminant::F16 => todo!(),
                            TypeDiscriminant::U16 => todo!(),
                            TypeDiscriminant::U8 => todo!(),
                            TypeDiscriminant::String => todo!(),
                            TypeDiscriminant::Boolean => todo!(),
                            TypeDiscriminant::Void => todo!(),
                            TypeDiscriminant::Struct(_) => todo!(),
                            TypeDiscriminant::Array(_) => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            },
                        }
                    },
                    TypeDiscriminant::Struct(_) => {
                        match desired_type {
                            TypeDiscriminant::I64 => todo!(),
                            TypeDiscriminant::F64 => todo!(),
                            TypeDiscriminant::U64 => todo!(),
                            TypeDiscriminant::I32 => todo!(),
                            TypeDiscriminant::F32 => todo!(),
                            TypeDiscriminant::U32 => todo!(),
                            TypeDiscriminant::I16 => todo!(),
                            TypeDiscriminant::F16 => todo!(),
                            TypeDiscriminant::U16 => todo!(),
                            TypeDiscriminant::U8 => todo!(),
                            TypeDiscriminant::String => todo!(),
                            TypeDiscriminant::Boolean => todo!(),
                            TypeDiscriminant::Void => todo!(),
                            TypeDiscriminant::Struct(_) => todo!(),
                            TypeDiscriminant::Array(_) => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            },
                        }
                    },
                    TypeDiscriminant::Array(ref type_discriminant) => {
                        match desired_type {
                            TypeDiscriminant::I64 => todo!(),
                            TypeDiscriminant::F64 => todo!(),
                            TypeDiscriminant::U64 => todo!(),
                            TypeDiscriminant::I32 => todo!(),
                            TypeDiscriminant::F32 => todo!(),
                            TypeDiscriminant::U32 => todo!(),
                            TypeDiscriminant::I16 => todo!(),
                            TypeDiscriminant::F16 => todo!(),
                            TypeDiscriminant::U16 => todo!(),
                            TypeDiscriminant::U8 => todo!(),
                            TypeDiscriminant::String => todo!(),
                            TypeDiscriminant::Boolean => todo!(),
                            TypeDiscriminant::Void => todo!(),
                            TypeDiscriminant::Struct(_) => todo!(),
                            TypeDiscriminant::Array(_) => {
                                return Err(CodeGenError::InvalidTypeCast(
                                    ty_disc.clone(),
                                    desired_type,
                                )
                                .into());
                            },
                        }
                    },
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
            let parsed_lhs =
                (|| -> Result<Option<(PointerValue, BasicMetadataTypeEnum, TypeDiscriminant)>> {
                    if let Some((current_token, ptr, ptr_ty, disc)) =
                        dbg!(allocation_list.front().cloned())
                        && *lhs == current_token
                    {
                        allocation_list.pop_front();
                        return Ok(Some((ptr, ptr_ty, disc)));
                    }

                    create_ir_from_parsed_token(
                        ctx,
                        module,
                        builder,
                        dbg!(*lhs.clone()),
                        variable_map,
                        None,
                        fn_ret_ty.clone(),
                        this_fn_block,
                        this_fn,
                        allocation_list,
                        is_loop_body.clone(),
                        parsed_functions.clone(),
                        custom_types.clone(),
                    )
                })()?;

            // Allocate memory on the stack for the value stored in the rhs
            let parsed_rhs =
                (|| -> Result<Option<(PointerValue, BasicMetadataTypeEnum, TypeDiscriminant)>> {
                    if let Some((current_token, ptr, ptr_ty, disc)) =
                        allocation_list.front().cloned()
                        && *rhs == current_token
                    {
                        allocation_list.pop_front();
                        return Ok(Some((ptr, ptr_ty, disc)));
                    }

                    create_ir_from_parsed_token(
                        ctx,
                        module,
                        builder,
                        *rhs.clone(),
                        variable_map,
                        None,
                        fn_ret_ty.clone(),
                        this_fn_block,
                        this_fn,
                        allocation_list,
                        is_loop_body.clone(),
                        parsed_functions.clone(),
                        custom_types.clone(),
                    )
                })()?;

            // Check if both sides return a valid variable reference
            if let (Some((lhs_ptr, lhs_ty, l_ty_disc)), Some((rhs_ptr, rhs_ty, r_ty_disc))) =
                (parsed_lhs, parsed_rhs)
            {
                if l_ty_disc.is_float() && r_ty_disc.is_float() {
                    let math_res = match mathematical_symbol {
                        fog_common::parser::MathematicalSymbol::Addition => {
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
                        fog_common::parser::MathematicalSymbol::Subtraction => {
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
                        fog_common::parser::MathematicalSymbol::Division => {
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
                        fog_common::parser::MathematicalSymbol::Multiplication => {
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
                        fog_common::parser::MathematicalSymbol::Modulo => {
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
                            custom_types.clone(),
                        )?;

                        builder.build_store(ptr, math_res)?;

                        return Ok(Some((ptr, ty, r_ty_disc)));
                    }
                }
                else if l_ty_disc.is_int() && r_ty_disc.is_int() {
                    let math_res = match mathematical_symbol {
                        fog_common::parser::MathematicalSymbol::Addition => {
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
                        fog_common::parser::MathematicalSymbol::Subtraction => {
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
                        fog_common::parser::MathematicalSymbol::Division => {
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
                        fog_common::parser::MathematicalSymbol::Multiplication => {
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
                        fog_common::parser::MathematicalSymbol::Modulo => {
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
                            custom_types.clone(),
                        )?;

                        builder.build_store(ptr, math_res)?;

                        return Ok(Some((ptr, ty, r_ty_disc)));
                    }
                }
                else {
                    return Err(
                        CodeGenError::InternalVariableTypeMismatch(l_ty_disc, r_ty_disc).into(),
                    );
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
                                .arguments_list
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
                (ParsedToken, TypeDiscriminant),
            > = IndexMap::from_iter(arg_iter).into();

            // Keep the list of the arguments passed in
            let mut arguments_passed_in: Vec<BasicMetadataValueEnum> = Vec::new();

            for (arg_ident, (arg_token, arg_type)) in fn_argument_list.iter() {
                let fn_name_clone = fn_name.clone();
                let (ptr, ptr_ty) = (|| -> Result<(PointerValue, BasicMetadataTypeEnum)> {
                    if let Some((current_token, ptr, ptr_ty, disc)) =
                        allocation_list.front().cloned()
                    {
                        allocation_list.pop_front();
                        return Ok((ptr, ptr_ty));
                    }

                    // Create a temporary variable for the argument thats passed in, if the argument name is None that means that the argument has been passed to a function which has an indenfinite amount of arguments.
                    let (ptr, ptr_ty) = create_new_variable(
                        ctx,
                        builder,
                        &fn_arg_to_string(&fn_name_clone, arg_ident),
                        arg_type,
                        custom_types.clone(),
                    )?;

                    Ok((ptr, ptr_ty))
                })()?;

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
                    allocation_list,
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
            let returned_value = call.try_as_basic_value().left();

            if let Some(returned) = returned_value {
                let (v_ptr, v_ty) = if let Some(ref variable_name) = variable_reference {
                    let (v_ptr, var_ty) = variable_name.1;

                    (v_ptr, var_ty)
                }
                else {
                    let (v_ptr, v_ty) = if let Some((current_token, ptr, ty, _disc)) =
                        allocation_list.front().cloned()
                    {
                        allocation_list.pop_front();
                        (ptr, ty)
                    }
                    else {
                        create_new_variable(
                            ctx,
                            builder,
                            "",
                            &fn_sig.return_type,
                            custom_types.clone(),
                        )?
                    };

                    (v_ptr, v_ty)
                };

                match fn_sig.return_type.clone() {
                    TypeDiscriminant::I32 => {
                        // Get returned float value
                        let returned_int = returned.into_int_value();

                        // Store the const in the pointer
                        builder.build_store(v_ptr, returned_int)?;
                    },
                    TypeDiscriminant::F32 => {
                        // Get returned float value
                        let returned_float = returned.into_float_value();

                        // Store the const in the pointer
                        builder.build_store(v_ptr, returned_float)?;
                    },
                    TypeDiscriminant::U32 => {
                        // Get returned float value
                        let returned_float = returned.into_int_value();

                        // Store the const in the pointer
                        builder.build_store(v_ptr, returned_float)?;
                    },
                    TypeDiscriminant::U8 => {
                        // Get returned float value
                        let returned_smalint = returned.into_int_value();

                        // Store the const in the pointer
                        builder.build_store(v_ptr, returned_smalint)?;
                    },
                    TypeDiscriminant::String => {
                        // Get returned pointer value
                        let returned_ptr = returned.into_pointer_value();

                        // Store the const in the pointer
                        builder.build_store(v_ptr, returned_ptr)?;
                    },
                    TypeDiscriminant::Boolean => {
                        // Get returned boolean value
                        let returned_bool = returned.into_int_value();

                        builder.build_store(v_ptr, returned_bool)?;
                    },
                    TypeDiscriminant::Void => {
                        unreachable!(
                            "A void can not be parsed, as a void functuion returns a `None`."
                        );
                    },
                    TypeDiscriminant::Struct((struct_name, struct_inner)) => {
                        // Get returned pointer value
                        let returned_struct = returned.into_struct_value();

                        // Store the const in the pointer
                        builder.build_store(v_ptr, returned_struct)?;
                    },
                    TypeDiscriminant::I64 => {
                        let returned_int = returned.into_int_value();

                        builder.build_store(v_ptr, returned_int)?;
                    },
                    TypeDiscriminant::F64 => {
                        let returned_float = returned.into_float_value();

                        builder.build_store(v_ptr, returned_float)?;
                    },
                    TypeDiscriminant::U64 => {
                        let returned_int = returned.into_int_value();

                        builder.build_store(v_ptr, returned_int)?;
                    },
                    TypeDiscriminant::I16 => {
                        let returned_int = returned.into_int_value();

                        builder.build_store(v_ptr, returned_int)?;
                    },
                    TypeDiscriminant::F16 => {
                        let returned_float = returned.into_float_value();

                        builder.build_store(v_ptr, returned_float)?;
                    },
                    TypeDiscriminant::U16 => {
                        let returned_int = returned.into_int_value();

                        builder.build_store(v_ptr, returned_int)?;
                    },
                    TypeDiscriminant::Array(_) => {
                        let returned_struct = returned.into_struct_value();

                        builder.build_store(v_ptr, returned_struct)?;
                    },
                };

                if let Some((variable_name, (var_ptr, _), ty_disc)) = variable_reference {
                    // Check for type mismatch
                    if ty_disc != fn_sig.return_type {
                        return Err(CodeGenError::InternalVariableTypeMismatch(
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
                if fn_sig.return_type != TypeDiscriminant::Void {
                    return Err(
                        CodeGenError::InternalFunctionReturnedVoid(fn_sig.return_type).into(),
                    );
                }

                // We dont return anything, as nothing is allocated
                None
            }
        },
        ParsedToken::SetValue(var_ref_ty, value) => {
            let ((ptr, ty), ty_disc) = access_variable_ptr(
                ctx,
                module,
                builder,
                variable_map,
                &fn_ret_ty,
                this_fn_block,
                this_fn,
                allocation_list,
                &is_loop_body,
                &parsed_functions,
                &custom_types,
                *var_ref_ty,
            )?;

            create_ir_from_parsed_token(
                ctx,
                module,
                builder,
                *value,
                variable_map,
                Some((
                    String::from("set_value_var_ref"),
                    (ptr, ty),
                    ty_disc.clone(),
                )),
                fn_ret_ty,
                this_fn_block,
                this_fn,
                allocation_list,
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

            let (ptr, ptr_ty) =
                create_new_variable(ctx, builder, &var_name, &fn_ret_ty, custom_types.clone())?;

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
                allocation_list,
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
                allocation_list,
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
                    if_definition.complete_body,
                    TypeDiscriminant::Void,
                    branch_compl,
                    variable_map,
                    this_fn,
                    allocation_list,
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
                    if_definition.incomplete_body,
                    TypeDiscriminant::Void,
                    branch_incompl,
                    variable_map,
                    this_fn,
                    allocation_list,
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
        ParsedToken::InitializeStruct(struct_tys, struct_fields) => {
            if let Some((var_name, (var_ptr, var_ty), var_ty_disc)) = variable_reference {
                // Get the struct pointer's ty
                let pointee_struct_ty = var_ty.into_struct_type();

                // Pre-Allocate a struct so that it can be accessed later
                let allocate_struct = builder.build_alloca(pointee_struct_ty, "strct_init")?;

                // Iterate over the struct's fields
                for (field_idx, (field_name, field_ty)) in struct_tys.iter().enumerate() {
                    // Convert to llvm type
                    let llvm_ty = ty_to_llvm_ty(ctx, field_ty, custom_types.clone())?;

                    // Create a new temp variable according to the struct's field type
                    let (ptr, ty) = create_new_variable(
                        ctx,
                        builder,
                        field_name,
                        field_ty,
                        custom_types.clone(),
                    )?;

                    // Parse the value for the temp var
                    create_ir_from_parsed_token(
                        ctx,
                        module,
                        builder,
                        *(struct_fields.get_index(field_idx).unwrap().1.clone()),
                        variable_map,
                        Some((field_name.to_string(), (ptr, ty), field_ty.clone())),
                        fn_ret_ty.clone(),
                        this_fn_block,
                        this_fn,
                        allocation_list,
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
                builder.build_store(var_ptr, constructed_struct)?;
            }

            // A struct will not be allocated without a variable storing it.
            None
        },
        ParsedToken::Comparison(lhs, order, rhs, comparison_hand_side_ty) => {
            let pointee_ty = ty_to_llvm_ty(ctx, &comparison_hand_side_ty, custom_types.clone())?;

            let ((lhs_ptr, lhs_ty), (rhs_ptr, rhs_ty)) =
                if let Some((lhs_token, lhs_ptr, lhs_ty, lhs_disc)) =
                    allocation_list.front().cloned()
                {
                    if dbg!(lhs_token) == dbg!((*lhs).clone()) {
                        create_ir_from_parsed_token(
                            ctx,
                            module,
                            builder,
                            *lhs,
                            variable_map,
                            Some((
                                "lhs_tmp".to_string(),
                                ((lhs_ptr, lhs_ty)),
                                comparison_hand_side_ty.clone(),
                            )),
                            fn_ret_ty.clone(),
                            this_fn_block,
                            this_fn,
                            allocation_list,
                            is_loop_body.clone(),
                            parsed_functions.clone(),
                            custom_types.clone(),
                        )?;

                        allocation_list.pop_front();

                        (lhs_ptr, lhs_ty)
                    }
                    else {
                        panic!()
                    };

                    let rhs_ptrs = if let Some((rhs_token, rhs_ptr, rhs_ty, rhs_disc)) =
                        allocation_list.front().cloned()
                    {
                        if dbg!(rhs_token) == dbg!((*rhs).clone()) {
                            create_ir_from_parsed_token(
                                ctx,
                                module,
                                builder,
                                *rhs,
                                variable_map,
                                Some((
                                    "rhs_tmp".to_string(),
                                    ((rhs_ptr, rhs_ty)),
                                    comparison_hand_side_ty.clone(),
                                )),
                                fn_ret_ty.clone(),
                                this_fn_block,
                                this_fn,
                                allocation_list,
                                is_loop_body.clone(),
                                parsed_functions.clone(),
                                custom_types.clone(),
                            )?;

                            allocation_list.pop_front();

                            (rhs_ptr, rhs_ty)
                        }
                        else {
                            panic!()
                        }
                    }
                    else {
                        create_new_variable(
                            ctx,
                            builder,
                            "rhs_tmp",
                            &comparison_hand_side_ty,
                            custom_types.clone(),
                        )?
                    };

                    ((lhs_ptr, lhs_ty), rhs_ptrs)
                }
                else {
                    let temp_lhs = create_new_variable(
                        ctx,
                        builder,
                        "lhs_tmp",
                        &comparison_hand_side_ty,
                        custom_types.clone(),
                    )?;
                    let temp_rhs = create_new_variable(
                        ctx,
                        builder,
                        "rhs_tmp",
                        &comparison_hand_side_ty,
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
                            (temp_rhs),
                            comparison_hand_side_ty.clone(),
                        )),
                        fn_ret_ty.clone(),
                        this_fn_block,
                        this_fn,
                        allocation_list,
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
                            (temp_lhs),
                            comparison_hand_side_ty.clone(),
                        )),
                        fn_ret_ty.clone(),
                        this_fn_block,
                        this_fn,
                        allocation_list,
                        is_loop_body.clone(),
                        parsed_functions.clone(),
                        custom_types.clone(),
                    )?;

                    (temp_lhs, temp_rhs)
                };

            let lhs_val = builder.build_load(pointee_ty, lhs_ptr, "lhs_tmp_val")?;
            let rhs_val = builder.build_load(pointee_ty, rhs_ptr, "rhs_tmp_val")?;

            let cmp_result = match comparison_hand_side_ty {
                TypeDiscriminant::I16 | TypeDiscriminant::I32 | TypeDiscriminant::I64 => {
                    builder.build_int_compare(
                        order.into_int_predicate(true),
                        lhs_val.into_int_value(),
                        rhs_val.into_int_value(),
                        "cmp",
                    )?
                },
                TypeDiscriminant::F16 | TypeDiscriminant::F32 | TypeDiscriminant::F64 => {
                    builder.build_float_compare(
                        order.into_float_predicate(),
                        lhs_val.into_float_value(),
                        rhs_val.into_float_value(),
                        "cmp",
                    )?
                },
                TypeDiscriminant::U8
                | TypeDiscriminant::U16
                | TypeDiscriminant::U32
                | TypeDiscriminant::U64
                | TypeDiscriminant::Boolean => {
                    builder.build_int_compare(
                        order.into_int_predicate(false),
                        lhs_val.into_int_value(),
                        rhs_val.into_int_value(),
                        "cmp",
                    )?
                },
                TypeDiscriminant::String => {
                    unimplemented!()
                },
                TypeDiscriminant::Void => ctx.bool_type().const_int(1, false),
                TypeDiscriminant::Struct(_) => {
                    unimplemented!()
                },
                TypeDiscriminant::Array(type_discriminant) => unimplemented!(),
            };

            if let Some((_, (var_ptr, _), ref_var_ty_disc)) = variable_reference {
                // Make sure that the variable we are setting is of type `Boolean` as a comparison always returns a `Bool`.
                if ref_var_ty_disc != TypeDiscriminant::Boolean {
                    return Err(CodeGenError::InternalVariableTypeMismatch(
                        ref_var_ty_disc,
                        TypeDiscriminant::Boolean,
                    )
                    .into());
                }

                builder.build_store(var_ptr, cmp_result)?;

                None
            }
            else if let Some((cmp_token, cmp_ptr, cmp_ty, ty_disc)) =
                allocation_list.front().cloned()
            {
                if cmp_token != parsed_token.clone() {
                    return Err(CodeGenError::InvalidPreAllocation.into());
                }

                builder.build_store(cmp_ptr, cmp_result)?;

                allocation_list.pop_front();

                Some((cmp_ptr, cmp_ty, TypeDiscriminant::Boolean))
            }
            else {
                let (v_ptr, v_ty) = create_new_variable(
                    ctx,
                    builder,
                    "cmp_result",
                    &TypeDiscriminant::Boolean,
                    custom_types.clone(),
                )?;

                builder.build_store(v_ptr, cmp_result)?;

                Some((v_ptr, v_ty, TypeDiscriminant::Boolean))
            }
        },
        ParsedToken::CodeBlock(parsed_tokens) => todo!(),
        ParsedToken::Loop(parsed_tokens) => {
            // Create the loop body
            let loop_body = ctx.append_basic_block(this_fn, "loop_body");

            // Create a the body of the code which would get executed after the loop body
            let loop_body_exit = ctx.append_basic_block(this_fn, "loop_body_exit");

            // Create an alloca_table
            // This contains all the pre allocated variables for the loop body. This makes it so that we dont allocate anything inside the loop body, thus avoiding stack overflows.
            let mut alloca_table = create_alloca_table(
                module,
                builder,
                ctx,
                parsed_tokens.clone(),
                fn_ret_ty,
                this_fn_block,
                variable_map,
                this_fn,
                parsed_functions.clone(),
                custom_types.clone(),
            )?;

            dbg!(&alloca_table);

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
                TypeDiscriminant::Void,
                loop_body,
                variable_map,
                this_fn,
                &mut alloca_table,
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
                    fog_common::parser::ControlFlowType::Break => {
                        builder.build_unconditional_branch(loop_body_blocks.loop_body_exit)?;
                    },
                    fog_common::parser::ControlFlowType::Continue => {
                        builder.build_unconditional_branch(loop_body_blocks.loop_body)?;
                    },
                }
            }
            else {
                return Err(CodeGenError::InvalidControlFlowUsage.into());
            }

            None
        },
        ParsedToken::ArrayIndexing(variable_ref, indexing) => {
            let var_ref = create_ir_from_parsed_token(
                ctx,
                module,
                builder,
                *variable_ref.clone(),
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

            let index_val = create_ir_from_parsed_token(
                ctx,
                module,
                builder,
                *indexing,
                variable_map,
                None,
                fn_ret_ty,
                this_fn_block,
                this_fn,
                allocation_list,
                is_loop_body,
                parsed_functions,
                custom_types.clone(),
            )?;

            if let Some((ptr, ptr_ty, type_disc)) = var_ref {
                if let TypeDiscriminant::Array((inner_ty, len)) = type_disc.clone() {
                    let inner_ty = token_to_ty(*inner_ty, custom_types.clone())?;

                    let pointee_ty = ty_to_llvm_ty(ctx, &type_disc, custom_types.clone())?;

                    if let Some((idx_ptr, idx_ptr_val, idx_ty_disc)) = index_val {
                        let idx = builder.build_load(
                            ty_to_llvm_ty(ctx, &idx_ty_disc, custom_types.clone())?,
                            idx_ptr,
                            "array_idx_val",
                        )?;

                        if !idx_ty_disc.is_int() {
                            return Err(CodeGenError::NonIndexType(idx_ty_disc).into());
                        }

                        let gep_ptr = unsafe {
                            builder.build_gep(
                                pointee_ty,
                                ptr,
                                &[ctx.i32_type().const_int(0, false), idx.into_int_value()],
                                "array_idx_elem_ptr",
                            )?
                        };

                        // Decide what we want to do with the extracted value
                        match variable_reference {
                            // If there is a variable ref passed in
                            Some((_, (ptr, _ptr_ty), var_ty_disc)) => {
                                if inner_ty != var_ty_disc {
                                    return Err(CodeGenError::CodegenTypeMismatch(
                                        dbg!(inner_ty),
                                        dbg!(var_ty_disc),
                                    )
                                    .into());
                                }

                                builder.build_store(
                                    ptr,
                                    builder.build_load(
                                        ty_to_llvm_ty(ctx, &inner_ty, custom_types.clone())?,
                                        gep_ptr,
                                        "idx_array_val_deref",
                                    )?,
                                )?;

                                return Ok(None);
                            },
                            // If there isnt one, we should return the ptr to this value
                            None => {
                                let array_val = builder.build_load(
                                    ty_to_llvm_ty(ctx, &inner_ty, custom_types.clone())?,
                                    gep_ptr,
                                    "idx_array_val_deref",
                                )?;

                                let array_val_ty =
                                    ty_to_llvm_ty(ctx, &inner_ty, custom_types.clone())?;

                                let ptr = builder
                                    .build_alloca(array_val_ty, "temp_deref_var")
                                    .unwrap();

                                builder.build_store(ptr, array_val)?;

                                return Ok(Some((ptr, array_val_ty.into(), inner_ty)));
                            },
                        }
                    }
                    else {
                        return Err(CodeGenError::InternalVariableNotFound(
                            "(INTERNAL_TEMPORARY_VARIABLE) array_idx_val".to_string(),
                        )
                        .into());
                    }
                }
                else {
                    return Err(CodeGenError::NonIndexType(type_disc.clone()).into());
                }
            }
            else {
                return Err(
                    CodeGenError::InternalVariableNotFound(variable_ref.to_string()).into(),
                );
            }
        },
        ParsedToken::ArrayInitialization(values, inner_ty) => {
            if let Some((_, (ptr, _ptr_ty), ty_disc)) = variable_reference
                && let TypeDiscriminant::Array((_, len)) = ty_disc
            {
                let mut array_values: Vec<BasicValueEnum> = Vec::new();

                for val in values {
                    let (temp_var_ptr, temp_var_ty) = create_new_variable(
                        ctx,
                        builder,
                        "array_temp_val_var",
                        &inner_ty,
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
                        allocation_list,
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
    };

    Ok(created_var)
}

/// This function is solely for generating the LLVM-IR from the main sourec file.
pub fn generate_ir<'ctx>(
    parsed_functions: Rc<IndexMap<String, FunctionDefinition>>,
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &'ctx Builder<'ctx>,
    custom_types: Arc<IndexMap<String, CustomType>>,
    is_optimized: bool,
    flags_passed_in: &str,
) -> Result<()>
{
    let (debug_info_builder, debug_info_compile_uint) = module.create_debug_info_builder(
        false,
        DWARFSourceLanguage::C,
        module.get_name().to_str()?,
        "src/",
        &format!("Fog (ver.: {}) with LLVM 18-1-8", env!("CARGO_PKG_VERSION")),
        is_optimized,
        flags_passed_in,
        0,
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

    let debug_info_file = debug_info_compile_uint.get_file();

    let debug_scope = debug_info_file.as_debug_info_scope();

    let mut unique_id_source = 0;

    for (function_name, function_definition) in parsed_functions.iter() {
        let function_type = create_fn_type_from_ty_disc(
            context,
            function_definition.function_sig.clone(),
            custom_types.clone(),
        )?;

        // Create function signature
        let function = module.add_function(function_name, function_type, None);

        let return_type = function_definition.function_sig.return_type.clone();

        if !is_optimized {
            let debug_subprogram = create_subprogram_debug_information(
                context,
                module,
                &custom_types,
                is_optimized,
                &debug_info_builder,
                debug_info_file,
                debug_scope,
                &mut unique_id_source,
                function_name,
                function_definition,
                return_type,
            )
            .map_err(CodeGenError::LibraryLLVMMessage)?;

            function.set_subprogram(debug_subprogram);
        }

        // Create a BasicBlock to store the IR in
        let basic_block = context.append_basic_block(function, "main_fn_entry");

        // Insert the BasicBlock at the end
        builder.position_at_end(basic_block);

        // Check if the return type is Void.
        // We dont require the user to insert a return void instruction, instead we do it automaticly.
        if function_definition.function_sig.return_type == TypeDiscriminant::Void {
            // Insert the return void instruction
            let instruction = builder.build_return(None)?;

            // Put the builder before that instruction so that it can resume generating IR
            builder.position_before(&instruction);
        }

        // Create a HashMap of the arguments the function takes
        let mut arguments: HashMap<String, (BasicValueEnum, TypeDiscriminant)> = HashMap::new();

        // Get the arguments and store them in the HashMap
        for (idx, argument) in function.get_param_iter().enumerate() {
            // Get the name of the argument from the function signature's argument list
            let argument_entry = function_definition
                .function_sig
                .args
                .arguments_list
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
            function_definition.function_sig.return_type.clone(),
            basic_block,
            function,
            parsed_functions.clone(),
            custom_types.clone(),
        )?;
    }

    Ok(())
}
