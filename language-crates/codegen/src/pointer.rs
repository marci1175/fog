use fog_common::{
    DEFAULT_COMPILER_ADDRESS_SPACE_SIZE,
    anyhow::Result,
    codegen::{CustomType, LoopBodyBlocks, ty_to_llvm_ty},
    error::codegen::CodeGenError,
    indexmap::IndexMap,
    inkwell::{
        AddressSpace,
        basic_block::BasicBlock,
        builder::Builder,
        context::Context,
        module::Module,
        types::{BasicMetadataTypeEnum, BasicTypeEnum},
        values::{FunctionValue, PointerValue},
    },
    parser::{FunctionDefinition, ParsedToken},
    ty::{Type, TypeDiscriminant, token_to_ty},
};
use std::{
    collections::{HashMap, VecDeque},
    rc::Rc,
    slice::Iter,
    sync::Arc,
};

use crate::create_ir_from_parsed_token;

/// This function accesses any kind of variable.
/// This is a recursive function so that nested variables i.e nested variables inside arrays and structs can be fetched.
/// The parsed token is the token containing the entire reference to the variable.
/// The variable_ptr passed in is supposed to be the ptr equal to the `ParsedToken`'s "nestedness".
pub fn access_variable_ptr<'main, 'ctx>(
    ctx: &'ctx Context,
    module: &Module<'ctx>,
    builder: &'ctx Builder<'ctx>,
    variable_map: &mut HashMap<
        String,
        (
            (PointerValue<'ctx>, BasicMetadataTypeEnum<'ctx>),
            TypeDiscriminant,
        ),
    >,
    fn_ret_ty: &TypeDiscriminant,
    this_fn_block: BasicBlock<'ctx>,
    this_fn: FunctionValue<'ctx>,
    allocation_list: &mut VecDeque<(
        ParsedToken,
        PointerValue<'ctx>,
        BasicMetadataTypeEnum<'ctx>,
        TypeDiscriminant,
    )>,
    is_loop_body: &Option<LoopBodyBlocks<'_>>,
    parsed_functions: &Rc<IndexMap<String, FunctionDefinition>>,
    custom_types: &Arc<IndexMap<String, CustomType>>,
    parsed_token: ParsedToken,
) -> Result<(
    (PointerValue<'ctx>, BasicMetadataTypeEnum<'ctx>),
    TypeDiscriminant,
)>
{
    match parsed_token {
        ParsedToken::ArrayIndexing(var_ref, index) => {
            // This variable is supposed to fetch the inner value of this array indexing, this is how this function is recursive.
            // When you are trying to understand the code, just imagine as if this were inside the function as an argument.
            let inner_variable = access_variable_ptr(
                ctx,
                module,
                builder,
                variable_map,
                fn_ret_ty,
                this_fn_block,
                this_fn,
                allocation_list,
                is_loop_body,
                parsed_functions,
                custom_types,
                *var_ref,
            )?;

            // Access the value available the index value provided
            let array_val_ptr = access_array_index(
                ctx,
                module,
                builder,
                variable_map,
                fn_ret_ty,
                this_fn_block,
                this_fn,
                allocation_list,
                is_loop_body,
                parsed_functions,
                custom_types,
                inner_variable,
                index,
            )?;

            Ok(((array_val_ptr.0, array_val_ptr.1), array_val_ptr.2))
        },
        ParsedToken::VariableReference(variable_reference) => {
            match variable_reference {
                fog_common::parser::VariableReference::StructFieldReference(
                    struct_field_reference,
                    (_struct_name, struct_definition),
                ) => {
                    let mut field_stack_iter = struct_field_reference.field_stack.iter();

                    if let Some(main_struct_var_name) = field_stack_iter.next()
                        && let Some(((ptr, ty), _ty_disc)) = variable_map.get(main_struct_var_name)
                    {
                        let (f_ptr, f_ty, ty_disc) = access_nested_struct_field_ptr(
                            ctx,
                            builder,
                            &mut field_stack_iter,
                            &struct_definition,
                            (*ptr, *ty),
                            custom_types.clone(),
                        )?;

                        Ok(((f_ptr, f_ty.into()), ty_disc))
                    }
                    else {
                        Err(CodeGenError::InternalInvalidStructReference.into())
                    }
                },
                fog_common::parser::VariableReference::BasicReference(basic_reference) => {
                    let variable_ref = variable_map.get(&basic_reference).ok_or_else(|| {
                        fog_common::anyhow::Error::from(CodeGenError::InternalVariableNotFound(
                            basic_reference.clone(),
                        ))
                    })?;

                    Ok(variable_ref.clone())
                },
                fog_common::parser::VariableReference::ArrayReference(array_name, indexing) => {
                    let ((ptr, ptr_ty), ty_disc) = variable_map.get(&array_name).unwrap().clone();

                    let index_val = create_ir_from_parsed_token(
                        ctx,
                        module,
                        builder,
                        (*indexing).clone(),
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

                    if let Some((idx_ptr, _idx_ptr_val, idx_ty_disc)) = index_val {
                        let idx = builder.build_load(
                            ty_to_llvm_ty(ctx, &idx_ty_disc, custom_types.clone())?,
                            idx_ptr,
                            "array_idx_val",
                        )?;

                        let gep_ptr = unsafe {
                            builder.build_gep(
                                ty_disc
                                    .clone()
                                    .to_basic_type_enum(ctx, custom_types.clone())?,
                                ptr,
                                &[ctx.i32_type().const_int(0, false), idx.into_int_value()],
                                "array_idx_elem_ptr",
                            )?
                        };

                        if let TypeDiscriminant::Array((inner_ty, _len)) = &ty_disc {
                            let array_inner_type =
                                token_to_ty((**inner_ty).clone(), custom_types.clone())?;

                            return Ok((
                                (
                                    gep_ptr,
                                    array_inner_type
                                        .clone()
                                        .to_basic_type_enum(ctx, custom_types.clone())?
                                        .into(),
                                ),
                                array_inner_type.clone(),
                            ));
                        }
                        else {
                            unreachable!("This must be an `Array`.");
                        }
                    }
                    else {
                        Err(CodeGenError::InvalidIndexValue((*indexing).clone()).into())
                    }
                },
            }
        },
        _ => Err(CodeGenError::InvalidVariableReference(parsed_token.clone()).into()),
    }
}

/// This function is used to get a ptr to a field that a nested struct contains.
pub fn access_nested_struct_field_ptr<'a>(
    ctx: &'a Context,
    builder: &'a Builder,
    field_stack_iter: &mut Iter<String>,
    struct_definition: &IndexMap<String, TypeDiscriminant>,
    last_field_ptr: (PointerValue<'a>, BasicMetadataTypeEnum<'a>),
    custom_types: Arc<IndexMap<String, CustomType>>,
) -> Result<(PointerValue<'a>, BasicTypeEnum<'a>, TypeDiscriminant)>
{
    if let Some(field_stack_entry) = field_stack_iter.next() {
        if let Some((field_idx, _, field_ty)) = struct_definition.get_full(field_stack_entry) {
            if let TypeDiscriminant::Struct((_, struct_def)) = field_ty {
                let pointee_ty = last_field_ptr.1.into_struct_type();
                let struct_field_ptr = builder.build_struct_gep(
                    pointee_ty,
                    last_field_ptr.0,
                    field_idx as u32,
                    "deref_nested_strct",
                )?;

                access_nested_struct_field_ptr(
                    ctx,
                    builder,
                    field_stack_iter,
                    struct_def,
                    (struct_field_ptr, pointee_ty.into()),
                    custom_types.clone(),
                )
            }
            else {
                let pointee_ty = ty_to_llvm_ty(ctx, field_ty, custom_types.clone())?;

                Ok((last_field_ptr.0, pointee_ty, field_ty.clone()))
            }
        }
        else {
            Err(CodeGenError::InternalStructFieldNotFound.into())
        }
    }
    else {
        panic!()
    }
}

/// This function takes in the variable pointer which is dereferenced to set the variable's value.
/// Ensure that we are setting variable type `T` with value `T`
pub fn set_value_of_ptr<'ctx>(
    ctx: &'ctx Context,
    builder: &Builder,
    module: &Module<'ctx>,
    value: Type,
    v_ptr: PointerValue<'_>,
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
        Type::I64(inner) => {
            // Initialize const value
            let init_val = i64_type.const_int(inner as u64, true);

            // Store const
            builder.build_store(v_ptr, init_val)?;
        },
        Type::F64(inner) => {
            // Initialize const value
            let init_val = f64_type.const_float(*inner);

            // Store const
            builder.build_store(v_ptr, init_val)?;
        },
        Type::U64(inner) => {
            // Initialize const value
            let init_val = i64_type.const_int(inner, false);

            // Store const
            builder.build_store(v_ptr, init_val)?;
        },
        Type::I16(inner) => {
            // Initialize const value
            let init_val = i16_type.const_int(inner as u64, true);

            // Store const
            builder.build_store(v_ptr, init_val)?;
        },
        Type::F16(inner) => {
            // Initialize const value
            let init_val = f16_type.const_float(*inner as f64);

            // Store const
            builder.build_store(v_ptr, init_val)?;
        },
        Type::U16(inner) => {
            // Initialize const value
            let init_val = i16_type.const_int(inner as u64, false);

            // Store const
            builder.build_store(v_ptr, init_val)?;
        },
        Type::I32(inner) => {
            // Initialize const value
            let init_val = i32_type.const_int(inner as u64, true);

            // Store const
            builder.build_store(v_ptr, init_val)?;
        },
        Type::F32(inner) => {
            // Initialize const value
            let init_val = f32_type.const_float(*inner as f64);

            // Store const
            builder.build_store(v_ptr, init_val)?;
        },
        Type::U32(inner) => {
            // Initialize const value
            let init_val = i32_type.const_int(inner as u64, false);

            // Store const
            builder.build_store(v_ptr, init_val)?;
        },
        Type::U8(inner) => {
            // Initialize const value
            let init_val = i8_type.const_int(inner as u64, false);

            // Store const
            builder.build_store(v_ptr, init_val)?;
        },
        Type::String(inner) => {
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
        Type::Boolean(inner) => {
            // Initialize const value
            let init_val = bool_type.const_int(inner as u64, false);

            // Store const
            builder.build_store(v_ptr, init_val)?;
        },
        Type::Void => {
            unreachable!()
        },
        Type::Struct((struct_name, struct_inner)) => {
            unreachable!()
        },
        Type::Array(inner_ty) => unimplemented!(),
        Type::Pointer(inner) => {
            let init_ptr = {
                #[cfg(target_pointer_width = "64")]
                {
                    i64_type.const_int(inner as u64, false)
                }

                #[cfg(target_pointer_width = "32")]
                {
                    i32_type.const_int(inner as u32, false)
                }
            };
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
            TypeDiscriminant,
        ),
    >,
    fn_ret_ty: &TypeDiscriminant,
    this_fn_block: BasicBlock<'ctx>,
    this_fn: FunctionValue<'ctx>,
    allocation_list: &mut VecDeque<(
        ParsedToken,
        PointerValue<'ctx>,
        BasicMetadataTypeEnum<'ctx>,
        TypeDiscriminant,
    )>,
    is_loop_body: &Option<LoopBodyBlocks<'_>>,
    parsed_functions: &Rc<IndexMap<String, FunctionDefinition>>,
    custom_types: &Arc<IndexMap<String, CustomType>>,
    ((array_ptr, _ptr_ty), ty_disc): (
        (PointerValue<'ctx>, BasicMetadataTypeEnum<'ctx>),
        TypeDiscriminant,
    ),
    index: Box<ParsedToken>,
) -> Result<(
    PointerValue<'ctx>,
    BasicMetadataTypeEnum<'ctx>,
    TypeDiscriminant,
)>
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
        let inner_ty = token_to_ty(*inner_ty_token, custom_types.clone())?;

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
        Err(CodeGenError::InvalidIndexValue(*index.clone()).into())
    }
}
