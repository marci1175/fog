use core::panic;
use std::{
    collections::{HashMap, VecDeque},
    io::ErrorKind,
    path::PathBuf,
    slice::Iter,
    sync::Arc,
};

use anyhow::Result;
use indexmap::IndexMap;
use inkwell::{
    AddressSpace,
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    module::Module,
    passes::PassBuilderOptions,
    targets::{InitializationConfig, RelocMode, Target, TargetMachine},
    types::{ArrayType, BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FunctionType},
    values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, IntValue, PointerValue},
};

use crate::{
    ApplicationError,
    app::{
        codegen::LoopBodyBlocks,
        parser::types::{FunctionDefinition, FunctionSignature, ParsedToken, PreAllocationEntry},
        type_system::type_system::{OrdMap, Type, TypeDiscriminant},
    },
};

use super::error::CodeGenError;

pub fn codegen_main(
    parsed_functions: &IndexMap<String, FunctionDefinition>,
    path_to_output: PathBuf,
    optimization: bool,
    imported_functions: &HashMap<String, FunctionSignature>,
) -> Result<()> {
    let context = Context::create();
    let builder = context.create_builder();
    let module = context.create_module("main");

    // Import functions defined by the user via llvm
    import_user_lib_functions(&context, &module, imported_functions, parsed_functions)?;

    generate_ir(parsed_functions, &context, &module, &builder)?;

    // Init target
    Target::initialize_x86(&InitializationConfig::default());

    // create target triple
    let traget_triple = TargetMachine::get_default_triple();

    // Create target
    let target = Target::from_triple(&traget_triple)
        .map_err(|_| anyhow::Error::from(CodeGenError::FaliedToAcquireTargetTriple))?;

    // Create target machine
    let target_machine = target
        .create_target_machine(
            &traget_triple,
            "generic",
            "",
            inkwell::OptimizationLevel::Aggressive,
            RelocMode::PIC,
            inkwell::targets::CodeModel::Default,
        )
        .unwrap();

    // Create opt passes list
    let passes = ["globaldce", "sink", "mem2reg"].join(",");

    // Run optimization passes if the user prompted to
    if optimization {
        let passes = passes.as_str();

        println!("Running optimization passes: {passes}...");
        module
            .run_passes(passes, &target_machine, PassBuilderOptions::create())
            .map_err(|_| CodeGenError::InternalOptimisationPassFailed)?;
    }

    println!("Writing LLVM-IR to output file...");

    // Write LLVM IR to a file.
    module.print_to_file(path_to_output).map_err(|err| {
        ApplicationError::FileError(std::io::Error::new(
            ErrorKind::ExecutableFileBusy,
            err.to_string(),
        ))
    })?;

    Ok(())
}

fn generate_ir<'ctx>(
    parsed_functions: &IndexMap<String, FunctionDefinition>,
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &'ctx Builder<'ctx>,
) -> Result<(), anyhow::Error> {
    for (function_name, function_definition) in parsed_functions.iter() {
        // Create function signature
        let function = module.add_function(
            function_name,
            create_fn_type_from_ty_disc(context, function_definition.function_sig.clone())?,
            None,
        );

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
        )?;
    }

    Ok(())
}

pub fn import_user_lib_functions<'a>(
    ctx: &'a Context,
    module: &Module<'a>,
    imported_functions: &'a HashMap<String, FunctionSignature>,
    parsed_functions: &IndexMap<String, FunctionDefinition>,
) -> Result<()> {
    for (import_name, import_sig) in imported_functions.iter() {
        // If a function with the same name as the imports exists, do not expose the function signature instead define the whole function
        // This means that the function has been imported, and we do not need to expose it in the LLVM-IR
        if parsed_functions.contains_key(import_name) {
            continue;
        }

        let mut args = Vec::new();

        for (_, arg_ty) in import_sig.args.arguments_list.iter() {
            let argument_sig = match arg_ty {
                TypeDiscriminant::I32 => BasicMetadataTypeEnum::IntType(ctx.i32_type()),
                TypeDiscriminant::F32 => BasicMetadataTypeEnum::FloatType(ctx.f32_type()),
                TypeDiscriminant::U32 => BasicMetadataTypeEnum::IntType(ctx.i32_type()),
                TypeDiscriminant::U8 => BasicMetadataTypeEnum::IntType(ctx.i32_type()),
                TypeDiscriminant::String => {
                    BasicMetadataTypeEnum::PointerType(ctx.ptr_type(AddressSpace::default()))
                }
                TypeDiscriminant::Boolean => BasicMetadataTypeEnum::IntType(ctx.bool_type()),
                TypeDiscriminant::Void => {
                    panic!("Can't take a `Void` as an argument")
                }
                TypeDiscriminant::Struct((_struct_name, struct_inner)) => {
                    let field_ty = struct_field_to_ty_list(ctx, struct_inner)?;

                    BasicMetadataTypeEnum::StructType(ctx.struct_type(&field_ty, false))
                }
                TypeDiscriminant::I64 => BasicMetadataTypeEnum::IntType(ctx.i64_type()),
                TypeDiscriminant::F64 => BasicMetadataTypeEnum::FloatType(ctx.f64_type()),
                TypeDiscriminant::U64 => BasicMetadataTypeEnum::IntType(ctx.i64_type()),
                TypeDiscriminant::I16 => BasicMetadataTypeEnum::IntType(ctx.i16_type()),
                TypeDiscriminant::F16 => BasicMetadataTypeEnum::FloatType(ctx.f16_type()),
                TypeDiscriminant::U16 => BasicMetadataTypeEnum::IntType(ctx.i16_type()),
            };

            args.push(argument_sig);
        }

        let function_type = match &import_sig.return_type {
            TypeDiscriminant::I32 => {
                let return_type = ctx.i32_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            }
            TypeDiscriminant::F32 => {
                let return_type = ctx.f32_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            }
            TypeDiscriminant::U32 => {
                let return_type = ctx.i32_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            }
            TypeDiscriminant::U8 => {
                let return_type = ctx.i32_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            }
            TypeDiscriminant::String => {
                let return_type = ctx.ptr_type(AddressSpace::default());

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            }
            TypeDiscriminant::Boolean => {
                let return_type = ctx.bool_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            }
            TypeDiscriminant::Void => {
                let return_type = ctx.void_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            }
            TypeDiscriminant::Struct((_struct_name, struct_inner)) => {
                let return_type = ctx.struct_type(
                    &struct_field_to_ty_list(ctx, struct_inner)?,
                    import_sig.args.ellipsis_present,
                );

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            }
            TypeDiscriminant::I64 => {
                let return_type = ctx.i64_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            }
            TypeDiscriminant::F64 => {
                let return_type = ctx.f32_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            }
            TypeDiscriminant::U64 => {
                let return_type = ctx.i64_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            }
            TypeDiscriminant::I16 => {
                let return_type = ctx.i16_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            }
            TypeDiscriminant::F16 => {
                let return_type = ctx.f16_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            }
            TypeDiscriminant::U16 => {
                let return_type = ctx.i16_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            }
        };

        module.add_function(import_name, function_type, None);
    }

    Ok(())
}

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
            }
            BasicValueEnum::IntValue(value) => {
                let v_ptr = builder.build_alloca(value.get_type(), &arg_name)?;
                builder.build_store(v_ptr, value)?;
                (v_ptr, BasicMetadataTypeEnum::IntType(value.get_type()))
            }
            BasicValueEnum::FloatValue(value) => {
                let v_ptr = builder.build_alloca(value.get_type(), &arg_name)?;
                builder.build_store(v_ptr, value)?;
                (v_ptr, BasicMetadataTypeEnum::FloatType(value.get_type()))
            }
            BasicValueEnum::PointerValue(value) => {
                let v_ptr = builder.build_alloca(value.get_type(), &arg_name)?;
                builder.build_store(v_ptr, value)?;
                (v_ptr, BasicMetadataTypeEnum::PointerType(value.get_type()))
            }
            BasicValueEnum::StructValue(value) => {
                let v_ptr = builder.build_alloca(value.get_type(), &arg_name)?;
                builder.build_store(v_ptr, value)?;
                (v_ptr, BasicMetadataTypeEnum::StructType(value.get_type()))
            }
            BasicValueEnum::VectorValue(value) => {
                let v_ptr = builder.build_alloca(value.get_type(), &arg_name)?;
                builder.build_store(v_ptr, value)?;
                (v_ptr, BasicMetadataTypeEnum::VectorType(value.get_type()))
            }
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
    )?;

    Ok(())
}

fn create_ir_from_parsed_token_list<'main, 'ctx>(
    module: &Module<'ctx>,
    // Inkwell IR builder
    builder: &'ctx Builder<'ctx>,
    // Inkwell Context
    ctx: &'main Context,
    // The list of ParsedToken-s
    parsed_tokens: Vec<ParsedToken>,
    // Type returned type of the Function
    fn_ret_ty: TypeDiscriminant,
    this_fn_block: BasicBlock<'ctx>,
    variable_map: &mut HashMap<
        String,
        (
            (PointerValue<'ctx>, BasicMetadataTypeEnum<'ctx>),
            TypeDiscriminant,
        ),
    >,
    this_fn: FunctionValue<'ctx>,
    // Allocation tables are used when the ParsedTokens run in a loop
    // We store the addresses and names of the variables which have been allocated previously to entering the loop, to avoid a stack overflow
    // Loops should not create new variables on the stack instead they should be using `alloca_table` to look up pointers.
    // If the code we are running is not in a loop we can pass in `None`.
    alloca_table: &mut VecDeque<(
        ParsedToken,
        PointerValue<'ctx>,
        BasicMetadataTypeEnum<'ctx>,
        TypeDiscriminant,
    )>,
    is_loop_body: Option<LoopBodyBlocks>,
) -> Result<(), anyhow::Error>
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
        )?;
    }

    Ok(())
}

pub fn create_alloca_table<'main, 'ctx>(
    module: &Module<'ctx>,
    // Inkwell IR builder
    builder: &'ctx Builder<'ctx>,
    // Inkwell Context
    ctx: &'main Context,
    // The list of ParsedToken-s
    parsed_tokens: Vec<ParsedToken>,
    // Type returned type of the Function
    fn_ret_ty: TypeDiscriminant,
    this_fn_block: BasicBlock<'ctx>,
    variable_map: &mut HashMap<
        String,
        (
            (PointerValue<'ctx>, BasicMetadataTypeEnum<'ctx>),
            TypeDiscriminant,
        ),
    >,
    this_fn: FunctionValue<'ctx>,
) -> Result<
    VecDeque<(
        ParsedToken,
        PointerValue<'ctx>,
        BasicMetadataTypeEnum<'ctx>,
        TypeDiscriminant,
    )>,
    anyhow::Error,
>
where
    'main: 'ctx,
{
    let mut alloc_list: VecDeque<(
        ParsedToken,
        PointerValue<'ctx>,
        BasicMetadataTypeEnum<'ctx>,
        TypeDiscriminant,
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
        )?;

        alloc_list.extend(allocations);

        // If the token we have parsed was a loop token that means any code from now becomes inaccessible, therefor we can stop parsing it.
        // if let ParsedToken::Loop(_) = token {
        //     break;
        // }
    }

    Ok(alloc_list)
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
) -> anyhow::Result<
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
                if let Some((current_token, ptr, ptr_ty, ty)) = allocation_list.front().cloned() {
                    if current_token == *var_set_val {
                        if ty == var_type {
                            was_preallocated = true;

                            allocation_list.pop_front();

                            return Ok((ptr, ptr_ty));
                        } else {
                            return Err(CodeGenError::InvalidPreAllocation.into());
                        }
                    }
                }

                let (ptr, ptr_ty) = create_new_variable(ctx, builder, &var_name, &var_type)?;

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
                )?;
            }

            allocation_list.pop_front();

            // We do not have to return anything here since a variable handle cannot really be casted to anything, its also top level
            None
        }
        ParsedToken::VariableReference(var_ref_variant) => {
            if let Some((var_ref_name, (var_ref_ptr, var_ref_ty), var_ref_ty_disc)) =
                variable_reference
            {
                match var_ref_variant {
                    crate::app::parser::types::VariableReference::StructFieldReference(
                        struct_field_stack,
                        (struct_name, struct_fields),
                    ) => {
                        let mut field_stack_iter = struct_field_stack.field_stack.iter();

                        if let Some(main_struct_var_name) = field_stack_iter.next() {
                            if let Some(((ptr, ty), ty_disc)) =
                                variable_map.get(main_struct_var_name)
                            {
                                let (f_ptr, f_ty, ty_disc) = access_nested_field(
                                    ctx,
                                    builder,
                                    &mut field_stack_iter,
                                    &struct_fields,
                                    (*ptr, *ty),
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
                                } else {
                                    return Err(CodeGenError::InternalTypeMismatch.into());
                                }
                            }
                        } else {
                            return Err(CodeGenError::InternalStructReference.into());
                        }
                    }
                    crate::app::parser::types::VariableReference::BasicReference(var_name) => {
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
                                }
                                BasicMetadataTypeEnum::FloatType(float_type) => {
                                    // Get the referenced variable's value
                                    let ref_var_val =
                                        builder.build_load(*float_type, *ref_ptr, "var_deref")?;

                                    // Store the referenced variable's value in the original
                                    builder.build_store(orig_ptr, ref_var_val)?;
                                }
                                BasicMetadataTypeEnum::IntType(int_type) => {
                                    // Get the referenced variable's value
                                    let ref_var_val =
                                        builder.build_load(*int_type, *ref_ptr, "var_deref")?;

                                    // Store the referenced variable's value in the original
                                    builder.build_store(orig_ptr, ref_var_val)?;
                                }
                                BasicMetadataTypeEnum::PointerType(pointer_type) => {
                                    // Get the referenced variable's value
                                    let ref_var_val =
                                        builder.build_load(*pointer_type, *ref_ptr, "var_deref")?;

                                    // Store the referenced variable's value in the original
                                    builder.build_store(orig_ptr, ref_var_val)?;
                                }
                                BasicMetadataTypeEnum::StructType(struct_type) => {
                                    // Get the referenced variable's value
                                    let ref_var_val =
                                        builder.build_load(*struct_type, *ref_ptr, "var_deref")?;

                                    // Store the referenced variable's value in the original
                                    builder.build_store(orig_ptr, ref_var_val)?;
                                }
                                BasicMetadataTypeEnum::VectorType(vector_type) => {
                                    // Get the referenced variable's value
                                    let ref_var_val =
                                        builder.build_load(*vector_type, *ref_ptr, "var_deref")?;

                                    // Store the referenced variable's value in the original
                                    builder.build_store(orig_ptr, ref_var_val)?;
                                }

                                _ => unimplemented!(),
                            };
                        }
                    }
                }

                None
            } else {
                match var_ref_variant {
                    crate::app::parser::types::VariableReference::StructFieldReference(
                        struct_field_stack,
                        (struct_name, struct_def),
                    ) => {
                        let mut field_stack_iter = struct_field_stack.field_stack.iter();

                        if let Some(main_struct_var_name) = field_stack_iter.next() {
                            if let Some(((ptr, ty), ty_disc)) =
                                variable_map.get(main_struct_var_name)
                            {
                                let (f_ptr, f_ty, ty_disc) = access_nested_field(
                                    ctx,
                                    builder,
                                    &mut field_stack_iter,
                                    &struct_def,
                                    (*ptr, *ty),
                                )?;

                                Some((f_ptr, ty_enum_to_metadata_ty_enum(f_ty), ty_disc))
                            } else {
                                return Err(CodeGenError::InternalVariableNotFound(
                                    main_struct_var_name.clone(),
                                )
                                .into());
                            }
                        } else {
                            return Err(CodeGenError::InternalStructReference.into());
                        }
                    }
                    crate::app::parser::types::VariableReference::BasicReference(basic_ref) => {
                        let ((ptr, ty), ty_disc) = variable_map
                            .get(&basic_ref)
                            .ok_or(CodeGenError::InternalVariableNotFound(basic_ref.clone()))?;

                        Some((*ptr, *ty, ty_disc.clone()))
                    }
                }
            }
        }
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
            } else {
                let ty_disc = literal.discriminant();

                let (v_ptr, v_ty) = create_new_variable(ctx, builder, "", &ty_disc)?;

                set_value_of_ptr(ctx, builder, module, literal, v_ptr)?;

                Some((v_ptr, v_ty, ty_disc))
            }
        }
        ParsedToken::TypeCast(parsed_token, desired_type) => {
            if let Some((var_name, (ref_ptr, ref_ty), ty_disc)) = variable_reference {
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
                )?;

                if let Some((var_ptr, var_ty, ty_disc)) = created_var {
                    match ty_disc {
                        // This match implements turning an I64 into other types
                        TypeDiscriminant::I64 | TypeDiscriminant::I32 | TypeDiscriminant::I16 => {
                            match desired_type {
                                TypeDiscriminant::I64 => {
                                    builder.build_store(
                                        ref_ptr,
                                        builder.build_load(var_ty.into_int_type(), var_ptr, "")?,
                                    )?;
                                }
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
                                }
                                TypeDiscriminant::U64 => {
                                    let value = builder
                                        .build_load(var_ty.into_int_type(), var_ptr, "")?
                                        .into_int_value();

                                    let cast_res = builder.build_int_cast(
                                        value,
                                        ctx.i64_type(),
                                        "i64_to_u64",
                                    )?;

                                    builder.build_store(ref_ptr, cast_res)?;
                                }
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
                                }
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
                                }
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
                                }
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
                                }
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
                                }
                                TypeDiscriminant::String => {
                                    let value = builder
                                        .build_load(var_ty.into_int_type(), var_ptr, "")?
                                        .into_int_value();

                                    let raw_val = value.get_sign_extended_constant().unwrap();

                                    let int_string = raw_val.to_string();

                                    let (buf_ptr, buf_ty) =
                                        allocate_string(builder, ctx.i8_type(), int_string)?;

                                    builder.build_store(ref_ptr, buf_ptr)?;
                                }
                                TypeDiscriminant::Boolean => {
                                    let value = builder
                                        .build_load(var_ty.into_int_type(), var_ptr, "")?
                                        .into_int_value();

                                    let bool_ty = ctx.bool_type();

                                    let bool_value =
                                        if value.get_sign_extended_constant().unwrap() == 0 {
                                            bool_ty.const_int(0, false)
                                        } else {
                                            bool_ty.const_int(1, false)
                                        };

                                    builder.build_store(ref_ptr, bool_value)?;
                                }
                                TypeDiscriminant::Void => {
                                    return Err(CodeGenError::InvalidTypeCast(
                                        ty_disc,
                                        desired_type,
                                    )
                                    .into());
                                }
                                TypeDiscriminant::Struct(_) => {
                                    return Err(CodeGenError::InvalidTypeCast(
                                        ty_disc,
                                        desired_type,
                                    )
                                    .into());
                                }
                            }
                        }
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
                                }
                                TypeDiscriminant::F64 => {
                                    builder.build_store(
                                        ref_ptr,
                                        builder.build_load(
                                            var_ty.into_float_type(),
                                            var_ptr,
                                            "",
                                        )?,
                                    )?;
                                }
                                TypeDiscriminant::U64 => {
                                    let value = builder.build_float_to_unsigned_int(
                                        builder
                                            .build_load(var_ty.into_float_type(), var_ptr, "")?
                                            .into_float_value(),
                                        ctx.i64_type(),
                                        "",
                                    )?;
                                    builder.build_store(ref_ptr, value)?;
                                }
                                TypeDiscriminant::I32 => {
                                    let value = builder.build_float_to_signed_int(
                                        builder
                                            .build_load(var_ty.into_float_type(), var_ptr, "")?
                                            .into_float_value(),
                                        ctx.i32_type(),
                                        "",
                                    )?;
                                    builder.build_store(ref_ptr, value)?;
                                }
                                TypeDiscriminant::F32 => {
                                    let value = builder
                                        .build_load(var_ty.into_float_type(), var_ptr, "")?
                                        .into_float_value();

                                    let cast_res =
                                        ctx.f32_type().const_float(value.get_constant().unwrap().0);

                                    builder.build_store(ref_ptr, cast_res)?;
                                }
                                TypeDiscriminant::U32 => {
                                    let value = builder.build_float_to_unsigned_int(
                                        builder
                                            .build_load(var_ty.into_float_type(), var_ptr, "")?
                                            .into_float_value(),
                                        ctx.i32_type(),
                                        "",
                                    )?;
                                    builder.build_store(ref_ptr, value)?;
                                }
                                TypeDiscriminant::I16 => {
                                    let value = builder.build_float_to_signed_int(
                                        builder
                                            .build_load(var_ty.into_float_type(), var_ptr, "")?
                                            .into_float_value(),
                                        ctx.i16_type(),
                                        "",
                                    )?;
                                    builder.build_store(ref_ptr, value)?;
                                }
                                TypeDiscriminant::F16 => {
                                    let value = builder
                                        .build_load(var_ty.into_float_type(), var_ptr, "")?
                                        .into_float_value();

                                    let cast_res =
                                        ctx.f16_type().const_float(value.get_constant().unwrap().0);

                                    builder.build_store(ref_ptr, cast_res)?;
                                }
                                TypeDiscriminant::U16 => {
                                    let value = builder.build_float_to_unsigned_int(
                                        builder
                                            .build_load(var_ty.into_float_type(), var_ptr, "")?
                                            .into_float_value(),
                                        ctx.i16_type(),
                                        "",
                                    )?;
                                    builder.build_store(ref_ptr, value)?;
                                }
                                TypeDiscriminant::U8 => {
                                    let value = builder.build_float_to_unsigned_int(
                                        builder
                                            .build_load(var_ty.into_float_type(), var_ptr, "")?
                                            .into_float_value(),
                                        ctx.i8_type(),
                                        "",
                                    )?;
                                    builder.build_store(ref_ptr, value)?;
                                }
                                TypeDiscriminant::String => {
                                    let value = builder
                                        .build_load(var_ty.into_float_type(), var_ptr, "")?
                                        .into_float_value();

                                    let raw_val = value.get_constant().unwrap().0;

                                    let int_string = raw_val.to_string();

                                    let (buf_ptr, buf_ty) =
                                        allocate_string(builder, ctx.i8_type(), int_string)?;

                                    builder.build_store(ref_ptr, buf_ptr)?;
                                }
                                TypeDiscriminant::Boolean => {
                                    let value = builder
                                        .build_load(var_ty.into_float_type(), var_ptr, "")?
                                        .into_float_value();

                                    let bool_ty = ctx.bool_type();

                                    let bool_value = if value.get_constant().unwrap().0 == 0.0 {
                                        bool_ty.const_int(0, false)
                                    } else {
                                        bool_ty.const_int(1, false)
                                    };

                                    builder.build_store(ref_ptr, bool_value)?;
                                }
                                TypeDiscriminant::Void => {
                                    return Err(CodeGenError::InvalidTypeCast(
                                        ty_disc,
                                        desired_type,
                                    )
                                    .into());
                                }
                                TypeDiscriminant::Struct(_) => {
                                    return Err(CodeGenError::InvalidTypeCast(
                                        ty_disc,
                                        desired_type,
                                    )
                                    .into());
                                }
                            }
                        }
                        TypeDiscriminant::U64
                        | TypeDiscriminant::U32
                        | TypeDiscriminant::U16
                        | TypeDiscriminant::U8 => match desired_type {
                            TypeDiscriminant::I64 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res = ctx.i64_type().const_int(
                                    value.get_sign_extended_constant().unwrap() as u64,
                                    true,
                                );

                                builder.build_store(ref_ptr, cast_res)?;
                            }
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
                            }
                            TypeDiscriminant::U64 => {
                                builder.build_store(
                                    ref_ptr,
                                    builder.build_load(var_ty.into_int_type(), var_ptr, "")?,
                                )?;
                            }
                            TypeDiscriminant::I32 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res = ctx.i32_type().const_int(
                                    value.get_sign_extended_constant().unwrap() as u64,
                                    true,
                                );

                                builder.build_store(ref_ptr, cast_res)?;
                            }
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
                            }
                            TypeDiscriminant::U32 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res = ctx.i32_type().const_int(
                                    value.get_sign_extended_constant().unwrap() as u64,
                                    false,
                                );

                                builder.build_store(ref_ptr, cast_res)?;
                            }
                            TypeDiscriminant::I16 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res = ctx.i16_type().const_int(
                                    value.get_sign_extended_constant().unwrap() as u64,
                                    true,
                                );

                                builder.build_store(ref_ptr, cast_res)?;
                            }
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
                            }
                            TypeDiscriminant::U16 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res = ctx.i16_type().const_int(
                                    value.get_sign_extended_constant().unwrap() as u64,
                                    false,
                                );

                                builder.build_store(ref_ptr, cast_res)?;
                            }
                            TypeDiscriminant::U8 => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let cast_res = ctx.i8_type().const_int(
                                    value.get_sign_extended_constant().unwrap() as u64,
                                    false,
                                );

                                builder.build_store(ref_ptr, cast_res)?;
                            }
                            TypeDiscriminant::String => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let raw_val = value.get_sign_extended_constant().unwrap();

                                let int_string = raw_val.to_string();

                                let (buf_ptr, buf_ty) =
                                    allocate_string(builder, ctx.i8_type(), int_string)?;

                                builder.build_store(ref_ptr, buf_ptr)?;
                            }
                            TypeDiscriminant::Boolean => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let bool_ty = ctx.bool_type();

                                let bool_value = if value.get_sign_extended_constant().unwrap() == 0
                                {
                                    bool_ty.const_int(0, false)
                                } else {
                                    bool_ty.const_int(1, false)
                                };

                                builder.build_store(ref_ptr, bool_value)?;
                            }
                            TypeDiscriminant::Void => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            }
                            TypeDiscriminant::Struct(_) => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            }
                        },
                        TypeDiscriminant::String => match desired_type {
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
                        },
                        TypeDiscriminant::Boolean => match desired_type {
                            TypeDiscriminant::I64 => {
                                let val = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let val = if val.get_zero_extended_constant().unwrap() == 0 {
                                    ctx.i64_type().const_int(0, true)
                                } else {
                                    ctx.i64_type().const_int(1, true)
                                };

                                builder.build_store(var_ptr, val)?;
                            }
                            TypeDiscriminant::F64 => {
                                let val = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let val = if val.get_zero_extended_constant().unwrap() == 0 {
                                    ctx.f64_type().const_float(0.0)
                                } else {
                                    ctx.f64_type().const_float(1.0)
                                };

                                builder.build_store(var_ptr, val)?;
                            }
                            TypeDiscriminant::U64 => {
                                let val = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let val = if val.get_zero_extended_constant().unwrap() == 0 {
                                    ctx.i64_type().const_int(0, false)
                                } else {
                                    ctx.i64_type().const_int(1, false)
                                };

                                builder.build_store(var_ptr, val)?;
                            }
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
                            }
                            TypeDiscriminant::F32 => {
                                let val = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let val = if val.get_zero_extended_constant().unwrap() == 0 {
                                    ctx.f32_type().const_float(0.0)
                                } else {
                                    ctx.f32_type().const_float(1.0)
                                };

                                builder.build_store(var_ptr, val)?;
                            }
                            TypeDiscriminant::U32 => {
                                let val = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let val = if val.get_zero_extended_constant().unwrap() == 0 {
                                    ctx.i32_type().const_int(0, false)
                                } else {
                                    ctx.i32_type().const_int(1, false)
                                };

                                builder.build_store(var_ptr, val)?;
                            }
                            TypeDiscriminant::I16 => {
                                let val = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let val = if val.get_zero_extended_constant().unwrap() == 0 {
                                    ctx.i16_type().const_int(0, true)
                                } else {
                                    ctx.i16_type().const_int(1, true)
                                };

                                builder.build_store(var_ptr, val)?;
                            }
                            TypeDiscriminant::F16 => {
                                let val = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let val = if val.get_zero_extended_constant().unwrap() == 0 {
                                    ctx.f32_type().const_float(0.0)
                                } else {
                                    ctx.f32_type().const_float(1.0)
                                };

                                builder.build_store(var_ptr, val)?;
                            }
                            TypeDiscriminant::U16 => {
                                let val = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let val = if val.get_zero_extended_constant().unwrap() == 0 {
                                    ctx.i16_type().const_int(0, false)
                                } else {
                                    ctx.i16_type().const_int(1, false)
                                };

                                builder.build_store(var_ptr, val)?;
                            }
                            TypeDiscriminant::U8 => {
                                let val = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let val = if val.get_zero_extended_constant().unwrap() == 0 {
                                    ctx.i8_type().const_int(0, false)
                                } else {
                                    ctx.i8_type().const_int(1, false)
                                };

                                builder.build_store(var_ptr, val)?;
                            }
                            TypeDiscriminant::String => {
                                let val = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let (buf, buf_ty) = if val.get_zero_extended_constant().unwrap()
                                    == 0
                                {
                                    allocate_string(builder, ctx.i8_type(), "false".to_string())?
                                } else {
                                    allocate_string(builder, ctx.i8_type(), "true".to_string())?
                                };

                                builder.build_store(var_ptr, buf)?;
                            }
                            TypeDiscriminant::Boolean => {
                                builder.build_store(
                                    ref_ptr,
                                    builder.build_load(var_ty.into_int_type(), var_ptr, "")?,
                                )?;
                            }
                            TypeDiscriminant::Void => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            }
                            TypeDiscriminant::Struct(_) => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            }
                        },
                        TypeDiscriminant::Void => match desired_type {
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
                        },
                        TypeDiscriminant::Struct(_) => match desired_type {
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
                        },
                    }
                } else {
                    return Err(CodeGenError::InvalidTypeCast(ty_disc, desired_type).into());
                }
            }

            None
        }
        ParsedToken::MathematicalExpression(lhs, mathematical_symbol, rhs) => {
            // Allocate memory on the stack for the value stored in the lhs
            let parsed_lhs =
                (|| -> Result<Option<(PointerValue, BasicMetadataTypeEnum, TypeDiscriminant)>> {
                    if let Some((current_token, ptr, ptr_ty, disc)) =
                        allocation_list.front().cloned()
                    {
                        if *lhs == current_token {
                            allocation_list.pop_front();
                            return Ok(Some((ptr, ptr_ty, disc)));
                        }
                    }

                    create_ir_from_parsed_token(
                        ctx,
                        module,
                        builder,
                        *lhs.clone(),
                        variable_map,
                        None,
                        fn_ret_ty.clone(),
                        this_fn_block,
                        this_fn,
                        allocation_list,
                        is_loop_body.clone(),
                    )
                })()?;

            // Allocate memory on the stack for the value stored in the rhs
            let parsed_rhs =
                (|| -> Result<Option<(PointerValue, BasicMetadataTypeEnum, TypeDiscriminant)>> {
                    if let Some((current_token, ptr, ptr_ty, disc)) =
                        allocation_list.front().cloned()
                    {
                        if *lhs == current_token {
                            allocation_list.pop_front();
                            return Ok(Some((ptr, ptr_ty, disc)));
                        }
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
                    )
                })()?;

            // Check if both sides return a valid variable reference
            if let (Some((lhs_ptr, lhs_ty, l_ty_disc)), Some((rhs_ptr, rhs_ty, r_ty_disc))) =
                (parsed_lhs, parsed_rhs)
            {
                if l_ty_disc.is_float() && r_ty_disc.is_float() {
                    let math_res = match mathematical_symbol {
                        crate::app::parser::types::MathematicalSymbol::Addition => builder
                            .build_float_add(
                                builder
                                    .build_load(lhs_ty.into_float_type(), lhs_ptr, "lhs")?
                                    .into_float_value(),
                                builder
                                    .build_load(rhs_ty.into_float_type(), rhs_ptr, "rhs")?
                                    .into_float_value(),
                                "float_add_float",
                            )?,
                        crate::app::parser::types::MathematicalSymbol::Subtraction => builder
                            .build_float_sub(
                                builder
                                    .build_load(lhs_ty.into_float_type(), lhs_ptr, "lhs")?
                                    .into_float_value(),
                                builder
                                    .build_load(rhs_ty.into_float_type(), rhs_ptr, "rhs")?
                                    .into_float_value(),
                                "float_sub_float",
                            )?,
                        crate::app::parser::types::MathematicalSymbol::Division => builder
                            .build_float_div(
                                builder
                                    .build_load(lhs_ty.into_float_type(), lhs_ptr, "lhs")?
                                    .into_float_value(),
                                builder
                                    .build_load(rhs_ty.into_float_type(), rhs_ptr, "rhs")?
                                    .into_float_value(),
                                "float_add_float",
                            )?,
                        crate::app::parser::types::MathematicalSymbol::Multiplication => builder
                            .build_float_mul(
                                builder
                                    .build_load(lhs_ty.into_float_type(), lhs_ptr, "lhs")?
                                    .into_float_value(),
                                builder
                                    .build_load(rhs_ty.into_float_type(), rhs_ptr, "rhs")?
                                    .into_float_value(),
                                "float_add_float",
                            )?,
                        crate::app::parser::types::MathematicalSymbol::Modulo => builder
                            .build_float_rem(
                                builder
                                    .build_load(lhs_ty.into_float_type(), lhs_ptr, "lhs")?
                                    .into_float_value(),
                                builder
                                    .build_load(rhs_ty.into_float_type(), rhs_ptr, "rhs")?
                                    .into_float_value(),
                                "float_add_float",
                            )?,
                    };

                    if let Some((var_ref_name, (var_ptr, var_ty), disc)) = variable_reference {
                        builder.build_store(var_ptr, math_res)?;
                    } else {
                        let (ptr, ty) =
                            create_new_variable(ctx, builder, "math_expr_res", &r_ty_disc)?;

                        builder.build_store(ptr, math_res)?;

                        return Ok(Some((ptr, ty, r_ty_disc)));
                    }
                } else if l_ty_disc.is_int() && r_ty_disc.is_int() {
                    let math_res = match mathematical_symbol {
                        crate::app::parser::types::MathematicalSymbol::Addition => builder
                            .build_int_add(
                                builder
                                    .build_load(lhs_ty.into_int_type(), lhs_ptr, "lhs")?
                                    .into_int_value(),
                                builder
                                    .build_load(rhs_ty.into_int_type(), rhs_ptr, "rhs")?
                                    .into_int_value(),
                                "int_add_int",
                            )?,
                        crate::app::parser::types::MathematicalSymbol::Subtraction => builder
                            .build_int_sub(
                                builder
                                    .build_load(lhs_ty.into_int_type(), lhs_ptr, "lhs")?
                                    .into_int_value(),
                                builder
                                    .build_load(rhs_ty.into_int_type(), rhs_ptr, "rhs")?
                                    .into_int_value(),
                                "int_sub_int",
                            )?,
                        crate::app::parser::types::MathematicalSymbol::Division => builder
                            .build_int_signed_div(
                                builder
                                    .build_load(lhs_ty.into_int_type(), lhs_ptr, "lhs")?
                                    .into_int_value(),
                                builder
                                    .build_load(rhs_ty.into_int_type(), rhs_ptr, "rhs")?
                                    .into_int_value(),
                                "int_div_int",
                            )?,
                        crate::app::parser::types::MathematicalSymbol::Multiplication => builder
                            .build_int_mul(
                                builder
                                    .build_load(lhs_ty.into_int_type(), lhs_ptr, "lhs")?
                                    .into_int_value(),
                                builder
                                    .build_load(rhs_ty.into_int_type(), rhs_ptr, "rhs")?
                                    .into_int_value(),
                                "int_mul_int",
                            )?,
                        crate::app::parser::types::MathematicalSymbol::Modulo => builder
                            .build_int_signed_rem(
                                builder
                                    .build_load(lhs_ty.into_int_type(), lhs_ptr, "lhs")?
                                    .into_int_value(),
                                builder
                                    .build_load(rhs_ty.into_int_type(), rhs_ptr, "rhs")?
                                    .into_int_value(),
                                "int_rem_int",
                            )?,
                    };
                    if let Some((var_ref_name, (var_ptr, var_ty), disc)) = variable_reference {
                        builder.build_store(var_ptr, math_res)?;
                    } else {
                        let (ptr, ty) =
                            create_new_variable(ctx, builder, "math_expr_res", &r_ty_disc)?;

                        builder.build_store(ptr, math_res)?;

                        return Ok(Some((ptr, ty, r_ty_disc)));
                    }
                } else {
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
        }
        ParsedToken::Brackets(parsed_tokens, type_discriminants) => todo!(),
        ParsedToken::FunctionCall((fn_sig, fn_name), passed_arguments) => {
            // Try accessing the function in the current module
            let function_value = module
                .get_function(&fn_name)
                .ok_or(CodeGenError::InternalFunctionNotFound(fn_name))?;

            let arg_iter =
                passed_arguments
                    .iter()
                    .enumerate()
                    .map(|(argument_idx, (arg_name, value))| {
                        (
                            fn_sig
                                .args
                                .arguments_list
                                .get_index(argument_idx)
                                .map(|inner| inner.0.clone()),
                            (value.clone()),
                        )
                    });

            // The arguments are in order, if theyre parsed in this order they can be passed to a function as an argument
            let fn_argument_list: OrdMap<Option<String>, (ParsedToken, TypeDiscriminant)> =
                IndexMap::from_iter(arg_iter).into();

            // Keep the list of the arguments passed in
            let mut arguments_passed_in: Vec<BasicMetadataValueEnum> = Vec::new();

            for (arg_name, (arg_token, arg_type)) in dbg!(fn_argument_list.iter()) {
                let (ptr, ptr_ty) = (|| -> Result<(PointerValue, BasicMetadataTypeEnum)> {
                    if let Some((current_token, ptr, ptr_ty, disc)) =
                        allocation_list.front().cloned()
                    {
                        dbg!(allocation_list.pop_front());
                        return Ok((ptr, ptr_ty));
                    }

                    // Create a temporary variable for the argument thats passed in, if the argument name is None that means that the argument has been passed to a function which has an indenfinite amount of arguments.
                    let (ptr, ptr_ty) = create_new_variable(
                        ctx,
                        builder,
                        &arg_name.clone().unwrap_or_default(),
                        arg_type,
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
                        arg_name.clone().unwrap_or_default(),
                        (ptr, ptr_ty),
                        arg_type.clone(),
                    )),
                    fn_ret_ty.clone(),
                    this_fn_block,
                    this_fn,
                    allocation_list,
                    is_loop_body.clone(),
                )?;

                // Push the argument to the list of arguments
                match ptr_ty {
                    BasicMetadataTypeEnum::ArrayType(array_type) => {
                        let loaded_val = builder.build_load(
                            array_type,
                            ptr,
                            &arg_name.clone().unwrap_or_default(),
                        )?;

                        arguments_passed_in.push(loaded_val.into());
                    }
                    BasicMetadataTypeEnum::FloatType(float_type) => {
                        let loaded_val = builder.build_load(
                            float_type,
                            ptr,
                            &arg_name.clone().unwrap_or_default(),
                        )?;

                        arguments_passed_in.push(loaded_val.into());
                    }
                    BasicMetadataTypeEnum::IntType(int_type) => {
                        let loaded_val = builder.build_load(
                            int_type,
                            ptr,
                            &arg_name.clone().unwrap_or_default(),
                        )?;

                        arguments_passed_in.push(loaded_val.into());
                    }
                    BasicMetadataTypeEnum::PointerType(pointer_type) => {
                        let loaded_val = builder.build_load(
                            pointer_type,
                            ptr,
                            &arg_name.clone().unwrap_or_default(),
                        )?;

                        arguments_passed_in.push(loaded_val.into());
                    }
                    BasicMetadataTypeEnum::StructType(struct_type) => {
                        let loaded_val = builder.build_load(
                            struct_type,
                            ptr,
                            &arg_name.clone().unwrap_or_default(),
                        )?;

                        arguments_passed_in.push(loaded_val.into());
                    }
                    BasicMetadataTypeEnum::VectorType(vector_type) => {
                        let loaded_val = builder.build_load(
                            vector_type,
                            ptr,
                            &arg_name.clone().unwrap_or_default(),
                        )?;

                        arguments_passed_in.push(loaded_val.into());
                    }

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
                } else {
                    let (v_ptr, v_ty) = if let Some((current_token, ptr, ty, _disc)) =
                        allocation_list.front().cloned()
                    {
                        allocation_list.pop_front();
                        (ptr, ty)
                    } else {
                        create_new_variable(ctx, builder, "", &fn_sig.return_type)?
                    };

                    (v_ptr, v_ty)
                };

                match fn_sig.return_type.clone() {
                    TypeDiscriminant::I32 => {
                        // Get returned float value
                        let returned_int = returned.into_int_value();

                        // Store the const in the pointer
                        builder.build_store(v_ptr, returned_int)?;
                    }
                    TypeDiscriminant::F32 => {
                        // Get returned float value
                        let returned_float = returned.into_float_value();

                        // Store the const in the pointer
                        builder.build_store(v_ptr, returned_float)?;
                    }
                    TypeDiscriminant::U32 => {
                        // Get returned float value
                        let returned_float = returned.into_int_value();

                        // Store the const in the pointer
                        builder.build_store(v_ptr, returned_float)?;
                    }
                    TypeDiscriminant::U8 => {
                        // Get returned float value
                        let returned_smalint = returned.into_int_value();

                        // Store the const in the pointer
                        builder.build_store(v_ptr, returned_smalint)?;
                    }
                    TypeDiscriminant::String => {
                        // Get returned pointer value
                        let returned_ptr = returned.into_pointer_value();

                        // Store the const in the pointer
                        builder.build_store(v_ptr, returned_ptr)?;
                    }
                    TypeDiscriminant::Boolean => {
                        // Get returned boolean value
                        let returned_bool = returned.into_int_value();

                        builder.build_store(v_ptr, returned_bool)?;
                    }
                    TypeDiscriminant::Void => {
                        unreachable!(
                            "A void can not be parsed, as a void functuion returns a `None`."
                        );
                    }
                    TypeDiscriminant::Struct((struct_name, struct_inner)) => {
                        // Get returned pointer value
                        let returned_struct = returned.into_struct_value();

                        // Store the const in the pointer
                        builder.build_store(v_ptr, returned_struct)?;
                    }
                    TypeDiscriminant::I64 => {
                        let returned_int = returned.into_int_value();

                        builder.build_store(v_ptr, returned_int)?;
                    }
                    TypeDiscriminant::F64 => {
                        let returned_float = returned.into_float_value();

                        builder.build_store(v_ptr, returned_float)?;
                    }
                    TypeDiscriminant::U64 => {
                        let returned_int = returned.into_int_value();

                        builder.build_store(v_ptr, returned_int)?;
                    }
                    TypeDiscriminant::I16 => {
                        let returned_int = returned.into_int_value();

                        builder.build_store(v_ptr, returned_int)?;
                    }
                    TypeDiscriminant::F16 => {
                        let returned_float = returned.into_float_value();

                        builder.build_store(v_ptr, returned_float)?;
                    }
                    TypeDiscriminant::U16 => {
                        let returned_int = returned.into_int_value();

                        builder.build_store(v_ptr, returned_int)?;
                    }
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
                        ty_to_llvm_ty(ctx, &fn_sig.return_type)?,
                        v_ptr,
                        &variable_name,
                    )?;

                    // Set the value of the pointer to whatever the function has returned
                    builder.build_store(var_ptr, function_result)?;

                    // We dont have to return a newly created variable reference here
                    None
                } else {
                    Some((v_ptr, v_ty, fn_sig.return_type))
                }
            } else {
                // Ensure the return type was `Void` else raise an error
                if fn_sig.return_type != TypeDiscriminant::Void {
                    return Err(
                        CodeGenError::InternalFunctionReturnedVoid(fn_sig.return_type).into(),
                    );
                }

                // We dont return anything, as nothing is allocated
                None
            }
        }
        ParsedToken::SetValue(var_ref_ty, value) => {
            match var_ref_ty {
                crate::app::parser::types::VariableReference::StructFieldReference(
                    struct_field_reference,
                    (_struct_name, struct_def),
                ) => {
                    let mut field_stack_iter = struct_field_reference.field_stack.iter();

                    if let Some(main_struct_var_name) = field_stack_iter.next() {
                        if let Some(((ptr, ty), ty_disc)) = variable_map.get(main_struct_var_name) {
                            let (f_ptr, f_ty, ty_disc) = access_nested_field(
                                ctx,
                                builder,
                                &mut field_stack_iter,
                                &struct_def,
                                (*ptr, *ty),
                            )?;

                            create_ir_from_parsed_token(
                                ctx,
                                module,
                                builder,
                                *value,
                                variable_map,
                                Some((String::new(), (f_ptr, f_ty.into()), ty_disc.clone())),
                                fn_ret_ty,
                                this_fn_block,
                                this_fn,
                                allocation_list,
                                is_loop_body.clone(),
                            )?;
                        }
                    }
                }
                crate::app::parser::types::VariableReference::BasicReference(variable_name) => {
                    let variable_query = variable_map.get(&variable_name);

                    if let Some(((ptr, ty), ty_disc)) = variable_query {
                        // Set the value of the variable which was referenced
                        create_ir_from_parsed_token(
                            ctx,
                            module,
                            builder,
                            *value,
                            variable_map,
                            Some((variable_name, (*ptr, *ty), ty_disc.clone())),
                            fn_ret_ty,
                            this_fn_block,
                            this_fn,
                            allocation_list,
                            is_loop_body.clone(),
                        )?;
                    }
                }
            }

            None
        }
        ParsedToken::MathematicalBlock(parsed_token) => todo!(),
        ParsedToken::ReturnValue(parsed_token) => {
            // Create a temporary variable to store the literal in
            // This temporary variable is used to return the value
            let var_name = String::from("ret_tmp_var");

            let (ptr, ptr_ty) = create_new_variable(ctx, builder, &var_name, &fn_ret_ty)?;

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
            )?;

            match ptr_ty {
                BasicMetadataTypeEnum::ArrayType(array_type) => {
                    builder.build_return(Some(&builder.build_load(array_type, ptr, &var_name)?))?;
                }
                BasicMetadataTypeEnum::FloatType(float_type) => {
                    builder.build_return(Some(&builder.build_load(float_type, ptr, &var_name)?))?;
                }
                BasicMetadataTypeEnum::IntType(int_type) => {
                    builder.build_return(Some(&builder.build_load(int_type, ptr, &var_name)?))?;
                }
                BasicMetadataTypeEnum::PointerType(pointer_type) => {
                    builder.build_return(Some(&builder.build_load(
                        pointer_type,
                        ptr,
                        &var_name,
                    )?))?;
                }
                BasicMetadataTypeEnum::StructType(struct_type) => {
                    let loaded_struct = builder.build_load(struct_type, ptr, &var_name)?;

                    builder.build_return(Some(&loaded_struct))?;
                }
                BasicMetadataTypeEnum::VectorType(vector_type) => {
                    builder.build_return(Some(&builder.build_load(
                        vector_type,
                        ptr,
                        &var_name,
                    )?))?;
                }

                _ => unimplemented!(),
            };

            None
        }
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
                )?;

                // Position the builder at the original position
                builder.build_unconditional_branch(branch_uncond)?;

                builder.position_at_end(branch_uncond);
            } else {
                return Err(CodeGenError::InvalidIfCondition.into());
            }

            None
        }
        ParsedToken::InitializeStruct(struct_tys, struct_fields) => {
            if let Some((var_name, (var_ptr, var_ty), var_ty_disc)) = variable_reference {
                // Get the struct pointer's ty
                let pointee_struct_ty = var_ty.into_struct_type();

                // Pre-Allocate a struct so that it can be accessed later
                let allocate_struct = builder.build_alloca(pointee_struct_ty, "strct_init")?;

                // Iterate over the struct's fields
                for (field_idx, (field_name, field_ty)) in struct_tys.iter().enumerate() {
                    // Convert to llvm type
                    let llvm_ty = ty_to_llvm_ty(ctx, field_ty)?;

                    // Create a new temp variable according to the struct's field type
                    let (ptr, ty) = create_new_variable(ctx, builder, field_name, field_ty)?;

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
        }
        ParsedToken::Comparison(lhs, order, rhs, comparison_hand_side_ty) => {
            let pointee_ty = ty_to_llvm_ty(ctx, &comparison_hand_side_ty)?;

            let ((lhs_ptr, lhs_ty), (rhs_ptr, rhs_ty)) = if let Some((lhs_token, lhs_ptr, lhs_ty, lhs_disc)) = allocation_list.front().cloned() {
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
                    )?;

                    allocation_list.pop_front();

                    (lhs_ptr, lhs_ty)
                } else {
                    panic!()
                };

                let rhs_ptrs = if let Some((rhs_token, rhs_ptr, rhs_ty, rhs_disc)) =
                    allocation_list.front().cloned()
                {
                    let ptr = if dbg!(rhs_token) == (*rhs).clone() {
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
                        )?;
                        
                        allocation_list.pop_front();

                        (rhs_ptr, rhs_ty)
                    } else {
                        panic!()
                    };

                    ptr
                } else {
                    create_new_variable(ctx, builder, "rhs_tmp", &comparison_hand_side_ty)?
                };

                ((lhs_ptr, lhs_ty), rhs_ptrs)
            } else {
                (
                    create_new_variable(ctx, builder, "lhs_tmp", &comparison_hand_side_ty)?,
                    create_new_variable(ctx, builder, "rhs_tmp", &comparison_hand_side_ty)?,
                )
            };

            let lhs_val = builder.build_load(pointee_ty, lhs_ptr, "lhs_tmp_val")?;
            let rhs_val = builder.build_load(pointee_ty, rhs_ptr, "rhs_tmp_val")?;

            let cmp_result = match comparison_hand_side_ty {
                TypeDiscriminant::I16 | TypeDiscriminant::I32 | TypeDiscriminant::I64 => builder
                    .build_int_compare(
                        order.into_int_predicate(true),
                        lhs_val.into_int_value(),
                        rhs_val.into_int_value(),
                        "cmp",
                    )?,
                TypeDiscriminant::F16 | TypeDiscriminant::F32 | TypeDiscriminant::F64 => builder
                    .build_float_compare(
                        order.into_float_predicate(),
                        lhs_val.into_float_value(),
                        rhs_val.into_float_value(),
                        "cmp",
                    )?,
                TypeDiscriminant::U8
                | TypeDiscriminant::U16
                | TypeDiscriminant::U32
                | TypeDiscriminant::U64
                | TypeDiscriminant::Boolean => builder.build_int_compare(
                    order.into_int_predicate(false),
                    lhs_val.into_int_value(),
                    rhs_val.into_int_value(),
                    "cmp",
                )?,
                TypeDiscriminant::String => {
                    unimplemented!()
                }
                TypeDiscriminant::Void => ctx.bool_type().const_int(1, false),
                TypeDiscriminant::Struct(_) => {
                    unimplemented!()
                }
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
            } else if let Some((cmp_token, cmp_ptr, cmp_ty, ty_disc)) = allocation_list.front().cloned() {
                if cmp_token != parsed_token.clone() {
                    return Err(CodeGenError::InvalidPreAllocation.into());
                }

                builder.build_store(cmp_ptr, cmp_result)?;

                allocation_list.pop_front();

                Some((cmp_ptr, cmp_ty, TypeDiscriminant::Boolean))
            } 
            else {
                let (v_ptr, v_ty) =
                    create_new_variable(ctx, builder, "cmp_result", &TypeDiscriminant::Boolean)?;

                builder.build_store(v_ptr, cmp_result)?;

                Some((v_ptr, v_ty, TypeDiscriminant::Boolean))
            }
        }
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
            )?;

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
            )?;

            // Create a jump to the beginning to the loop for an infinite loop
            builder.build_unconditional_branch(loop_body)?;

            // Reset the position of the builder
            builder.position_at_end(loop_body_exit);

            None
        }
        ParsedToken::ControlFlow(control_flow_variant) => {
            if let Some(loop_body_blocks) = is_loop_body {
                match control_flow_variant {
                    crate::app::parser::types::ControlFlowType::Break => {
                        builder.build_unconditional_branch(loop_body_blocks.loop_body_exit)?;
                    }
                    crate::app::parser::types::ControlFlowType::Continue => {
                        builder.build_unconditional_branch(loop_body_blocks.loop_body)?;
                    }
                }
            } else {
                return Err(CodeGenError::InvalidControlFlowUsage.into());
            }

            None
        }
    };

    Ok(created_var)
}

/// This function returns a pointer to the allocation made by according to the specific [`ParsedToken`] which had been passed in.
/// It serves as a way to make allocations before entering a loop, to avoid stack overflows.
/// If no allocation had been made the function will return [`None`].
pub fn fetch_alloca_ptr<'main, 'ctx>(
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
    // Type returned type of the Function
    fn_ret_ty: TypeDiscriminant,
    this_fn_block: BasicBlock<'ctx>,
    this_fn: FunctionValue<'ctx>,
) -> anyhow::Result<
    Vec<(
        ParsedToken,
        PointerValue<'ctx>,
        BasicMetadataTypeEnum<'ctx>,
        TypeDiscriminant,
    )>,
>
where
    'main: 'ctx,
{
    let mut pre_allocation_list = Vec::new();

    match parsed_token.clone() {
        ParsedToken::NewVariable(var_name, var_type, var_set_val) => {
            let (ptr, ty) = if let Some(((ptr, ty), _)) = variable_map.get(&var_name) {
                (*ptr, *ty)
            } else {
                let (ptr, ty) = create_new_variable(ctx, builder, &var_name, &var_type)?;

                variable_map.insert(var_name.clone(), ((ptr, ty), var_type.clone()));

                (ptr, ty)
            };

            // We dont check for the actual ParsedToken::NewVariable whether it is allocated already, we only check for its value
            // pre_allocation_list.push((parsed_token.clone(), ptr, ty, var_type.clone()));

            // We only set the value of the pre-allocated variable if its a constant, like if its a literal
            // This skips a step of setting the value in the loop, however this pre evaluation cannot be applied safely to all of the types
            // Check if the value is a literal
            // We also check if its a literal when we are checking for pre-allocated variables so that we dont set the value twice.
            if matches!(&*var_set_val, ParsedToken::Literal(_)) {
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
                )?;

                pre_allocation_list.push(((*var_set_val).clone(), ptr, ty, var_type.clone()));
            } else {
                let allocas = fetch_alloca_ptr(
                    ctx,
                    module,
                    builder,
                    dbg!(*var_set_val.clone()),
                    variable_map,
                    fn_ret_ty,
                    this_fn_block,
                    this_fn,
                )?;

                pre_allocation_list.extend(allocas);
            }
        }
        ParsedToken::VariableReference(var_ref) => match var_ref {
            crate::app::parser::types::VariableReference::StructFieldReference(
                struct_field_stack,
                (struct_name, struct_def),
            ) => {
                let mut field_stack_iter = struct_field_stack.field_stack.iter();

                if let Some(main_struct_var_name) = field_stack_iter.next() {
                    if let Some(((ptr, ty), _)) = variable_map.get(main_struct_var_name) {
                        let (f_ptr, f_ty, ty_disc) = access_nested_field(
                            ctx,
                            builder,
                            &mut field_stack_iter,
                            &struct_def,
                            (*ptr, *ty),
                        )?;

                        pre_allocation_list.push((
                            parsed_token.clone(),
                            f_ptr,
                            ty_enum_to_metadata_ty_enum(f_ty),
                            ty_disc,
                        ));
                    } else {
                        return Err(CodeGenError::InternalVariableNotFound(
                            main_struct_var_name.clone(),
                        )
                        .into());
                    }
                } else {
                    return Err(CodeGenError::InternalStructReference.into());
                }
            }
            crate::app::parser::types::VariableReference::BasicReference(name) => {
                if let Some(((ptr, ty), disc)) = variable_map.get(&name) {
                    pre_allocation_list.push((parsed_token.clone(), *ptr, *ty, disc.clone()));
                }
            }
        },
        ParsedToken::Literal(literal) => {
            let var_type = literal.discriminant();

            let (ptr, ty) = create_new_variable(ctx, builder, "", &var_type)?;

            pre_allocation_list.push((parsed_token.clone(), ptr, ty, var_type));
        }
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
                &mut VecDeque::new(),
                None,
            )?;

            if let Some((var_ptr, var_ty, ty_disc)) = created_var {
                let returned_alloca = match ty_disc {
                    // This match implements turning an I64 into other types
                    TypeDiscriminant::I64 | TypeDiscriminant::I32 | TypeDiscriminant::I16 => {
                        match desired_type {
                            TypeDiscriminant::I64 => Some((var_ptr, var_ty, TypeDiscriminant::I64)),
                            TypeDiscriminant::F64 => {
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
                            }
                            TypeDiscriminant::U64 => {
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
                            }
                            TypeDiscriminant::I32 | TypeDiscriminant::U32 => {
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
                            }
                            TypeDiscriminant::F32 => {
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
                            }
                            TypeDiscriminant::I16 | TypeDiscriminant::U16 => {
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
                            }
                            TypeDiscriminant::F16 => {
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
                            }
                            TypeDiscriminant::U8 => {
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
                            }
                            TypeDiscriminant::String => {
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
                            }
                            TypeDiscriminant::Boolean => {
                                let value = builder
                                    .build_load(var_ty.into_int_type(), var_ptr, "")?
                                    .into_int_value();

                                let bool_ty = ctx.bool_type();

                                let bool_value = if value.get_sign_extended_constant().unwrap() == 0
                                {
                                    bool_ty.const_int(0, false)
                                } else {
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
                            }
                            TypeDiscriminant::Void => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            }
                            TypeDiscriminant::Struct(_) => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            }
                        }
                    }
                    TypeDiscriminant::F64 | TypeDiscriminant::F32 | TypeDiscriminant::F16 => {
                        match desired_type {
                            TypeDiscriminant::I64 => {
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
                            }
                            TypeDiscriminant::F64 => Some((var_ptr, var_ty, TypeDiscriminant::F64)),
                            TypeDiscriminant::U64 => {
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
                            }
                            TypeDiscriminant::I32 => {
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
                            }
                            TypeDiscriminant::F32 => {
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
                            }
                            TypeDiscriminant::U32 => {
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
                            }
                            TypeDiscriminant::I16 => {
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
                            }
                            TypeDiscriminant::F16 => {
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
                            }
                            TypeDiscriminant::U16 => {
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
                            }
                            TypeDiscriminant::U8 => {
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
                            }
                            TypeDiscriminant::String => {
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
                            }
                            TypeDiscriminant::Boolean => {
                                let value = builder
                                    .build_load(var_ty.into_float_type(), var_ptr, "")?
                                    .into_float_value();

                                let bool_ty = ctx.bool_type();

                                let bool_value = if value.get_constant().unwrap().0 == 0.0 {
                                    bool_ty.const_int(0, false)
                                } else {
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
                            }
                            TypeDiscriminant::Void => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            }
                            TypeDiscriminant::Struct(_) => {
                                return Err(
                                    CodeGenError::InvalidTypeCast(ty_disc, desired_type).into()
                                );
                            }
                        }
                    }
                    TypeDiscriminant::U64
                    | TypeDiscriminant::U32
                    | TypeDiscriminant::U16
                    | TypeDiscriminant::U8 => match desired_type {
                        TypeDiscriminant::I64 => {
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
                        }
                        TypeDiscriminant::F64 => {
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
                        }
                        TypeDiscriminant::U64 => Some((var_ptr, var_ty, TypeDiscriminant::U64)),
                        TypeDiscriminant::I32 => {
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
                        }
                        TypeDiscriminant::F32 => {
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
                        }
                        TypeDiscriminant::U32 => {
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
                        }
                        TypeDiscriminant::I16 => {
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
                        }
                        TypeDiscriminant::F16 => {
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
                        }
                        TypeDiscriminant::U16 => {
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
                        }
                        TypeDiscriminant::U8 => {
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
                        }
                        TypeDiscriminant::String => {
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
                        }
                        TypeDiscriminant::Boolean => {
                            let value = builder
                                .build_load(var_ty.into_int_type(), var_ptr, "")?
                                .into_int_value();

                            let bool_ty = ctx.bool_type();

                            let bool_value = if value.get_sign_extended_constant().unwrap() == 0 {
                                bool_ty.const_int(0, false)
                            } else {
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
                        }
                        TypeDiscriminant::Void => {
                            return Err(CodeGenError::InvalidTypeCast(ty_disc, desired_type).into());
                        }
                        TypeDiscriminant::Struct(_) => {
                            return Err(CodeGenError::InvalidTypeCast(ty_disc, desired_type).into());
                        }
                    },
                    TypeDiscriminant::String => match desired_type {
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
                    },
                    TypeDiscriminant::Boolean => match desired_type {
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
                    },
                    TypeDiscriminant::Void => match desired_type {
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
                    },
                    TypeDiscriminant::Struct(_) => match desired_type {
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
                    },
                };

                if let Some((ptr, ptr_ty, var_type)) = returned_alloca {
                    pre_allocation_list.push((*parsed_token.clone(), ptr, ptr_ty, var_type));
                }
            } else {
                return Err(CodeGenError::InternalParsingError.into());
            }
        }
        ParsedToken::FunctionCall((fn_sig, fn_name), arguments) => {
            for (arg_idx, (arg_name, (arg, arg_ty))) in arguments.iter().enumerate() {
                // We create a pre allocated temp variable for the function's arguments, we use the function arg's name to indicate which temp variable is for which argument.
                // If the argument name is None, it means that the function we are calling has an indefinite amount of arguments, in this case having llvm automaticly name the variable is accepted
                let (ptr, ty) = create_new_variable(
                    ctx,
                    builder,
                    &arg_name.clone().unwrap_or_default(),
                    arg_ty,
                )?;

                pre_allocation_list.push((arg.clone(), ptr, ty, arg_ty.clone()));
            }

            // Check if the returned value of the function is Void
            // If it is, then we dont need to allocate anything
            if fn_sig.return_type != TypeDiscriminant::Void {
                let (ptr, ty) = create_new_variable(ctx, builder, "", &fn_sig.return_type)?;

                pre_allocation_list.push((parsed_token.clone(), ptr, ty, fn_sig.return_type));
            }

            dbg!(&pre_allocation_list);
        }
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
            )?;

            pre_allocation_list.extend(allocation_list);
        }
        ParsedToken::MathematicalExpression(lhs_token, _expr, rhs_token) => {
            let math_expr = create_ir_from_parsed_token(
                ctx,
                module,
                builder,
                parsed_token.clone(),
                variable_map,
                None,
                fn_ret_ty.clone(),
                this_fn_block,
                this_fn,
                &mut VecDeque::new(),
                None,
            )?;

            // Store the pointer of either one of the allocable values
            if let Some((ptr, ty, ty_disc)) = math_expr {
                pre_allocation_list.push((parsed_token.clone(), ptr, ty, ty_disc));
            }
        }
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
                )?;

                pre_allocation_list.extend(body_pre_allocs);
            }
        }
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
            )?;

            pre_allocation_list.extend(lhs_allocations);
            pre_allocation_list.extend(rhs_allocations);

            // Create a variable which stores the cmp result of the two
            let ptr = builder.build_alloca(ctx.bool_type(), "cmp_result")?;

            pre_allocation_list.push((parsed_token.clone(), ptr, ctx.bool_type().into(), TypeDiscriminant::Boolean));
        }
        // We can safely ignore this variant as it doesn't allocate anything
        ParsedToken::ControlFlow(_) => (),
        _ => {
            unimplemented!()
        }
    };

    Ok(pre_allocation_list)
}

fn access_nested_field<'a>(
    ctx: &'a Context,
    builder: &'a Builder,
    field_stack_iter: &mut Iter<String>,
    struct_definition: &IndexMap<String, TypeDiscriminant>,
    last_field_ptr: (PointerValue<'a>, BasicMetadataTypeEnum<'a>),
) -> Result<(PointerValue<'a>, BasicTypeEnum<'a>, TypeDiscriminant)> {
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

                access_nested_field(
                    ctx,
                    builder,
                    field_stack_iter,
                    struct_def,
                    (struct_field_ptr, pointee_ty.into()),
                )
            } else {
                let pointee_ty = ty_to_llvm_ty(ctx, field_ty)?;

                Ok((last_field_ptr.0, pointee_ty, field_ty.clone()))
            }
        } else {
            Err(CodeGenError::InternalStructFieldNotFound.into())
        }
    } else {
        panic!()
    }
}

/// Creates a new variable from a `TypeDiscriminant`.
/// It is UB to access the value of the variable created here before initilazing it with actual data.
fn create_new_variable<'a, 'b>(
    ctx: &'a Context,
    builder: &'a Builder<'_>,
    var_name: &str,
    var_type: &TypeDiscriminant,
) -> Result<(PointerValue<'a>, BasicMetadataTypeEnum<'a>), anyhow::Error> {
    // Turn a `TypeDiscriminant` into an LLVM type
    let var_type = ty_to_llvm_ty(ctx, var_type)?;

    // Allocate an instance of the converted type
    let v_ptr = builder.build_alloca(var_type, var_name)?;

    // Return the pointer of the allocation and the type
    Ok((v_ptr, var_type.into()))
}

/// Creates a function type from a FunctionSignature.
/// It uses the Function's return type and arguments to create a `FunctionType` which can be used later in llvm context.
pub fn create_fn_type_from_ty_disc(
    ctx: &Context,
    fn_sig: FunctionSignature,
) -> Result<FunctionType<'_>> {
    // Make an exception if the return type is Void
    if fn_sig.return_type == TypeDiscriminant::Void {
        return Ok(ctx
            .void_type()
            .fn_type(&get_args_from_sig(ctx, fn_sig.clone())?, false));
    }

    // Create an LLVM type
    let llvm_ty = ty_to_llvm_ty(ctx, &fn_sig.return_type)?;

    // Create the actual function type and parse the function's arguments
    Ok(llvm_ty.fn_type(
        &get_args_from_sig(ctx, fn_sig.clone())?,
        false, /* Variable arguments can not be used on source code defined functions */
    ))
}

/// Fetches the arguments (and converts it into an LLVM type) from the function's signature
pub fn get_args_from_sig(
    ctx: &Context,
    fn_sig: FunctionSignature,
) -> Result<Vec<BasicMetadataTypeEnum>> {
    // Create an iterator over the function's arguments
    let fn_args = fn_sig.args.arguments_list.iter();

    // Create a list for all the arguments
    let mut arg_list: Vec<BasicMetadataTypeEnum> = vec![];

    // Iter over all the arguments and store the converted variants of the argument types
    for (_arg_name, arg_ty) in fn_args {
        // Create an llvm ty
        let argument_sig = ty_to_llvm_ty(ctx, arg_ty)?;

        // Convert the type and store it
        arg_list.push(argument_sig.into());
    }

    // Return the list
    Ok(arg_list)
}

/// This function takes in the variable pointer which is dereferenced to set the variable's value.
/// Ensure that we are setting variable type `T` with value `T`
pub fn set_value_of_ptr<'ctx>(
    ctx: &'ctx Context,
    builder: &Builder,
    module: &Module<'ctx>,
    value: Type,
    v_ptr: PointerValue<'_>,
) -> anyhow::Result<()> {
    let bool_type = ctx.bool_type();
    let i8_type = ctx.i8_type();
    let i32_type = ctx.i32_type();
    let f32_type = ctx.f32_type();
    let f64_type = ctx.f64_type();
    let i64_type = ctx.i64_type();
    let i16_type = ctx.i16_type();
    let f16_type = ctx.f16_type();

    match value {
        Type::I64(inner) => {
            // Initialize const value
            let init_val = i64_type.const_int(inner as u64, true);

            // Store const
            builder.build_store(v_ptr, init_val)?;
        }
        Type::F64(inner) => {
            // Initialize const value
            let init_val = f64_type.const_float(*inner);

            // Store const
            builder.build_store(v_ptr, init_val)?;
        }
        Type::U64(inner) => {
            // Initialize const value
            let init_val = i64_type.const_int(inner, false);

            // Store const
            builder.build_store(v_ptr, init_val)?;
        }
        Type::I16(inner) => {
            // Initialize const value
            let init_val = i16_type.const_int(inner as u64, true);

            // Store const
            builder.build_store(v_ptr, init_val)?;
        }
        Type::F16(inner) => {
            // Initialize const value
            let init_val = f16_type.const_float(*inner as f64);

            // Store const
            builder.build_store(v_ptr, init_val)?;
        }
        Type::U16(inner) => {
            // Initialize const value
            let init_val = i16_type.const_int(inner as u64, false);

            // Store const
            builder.build_store(v_ptr, init_val)?;
        }
        Type::I32(inner) => {
            // Initialize const value
            let init_val = i32_type.const_int(inner as u64, true);

            // Store const
            builder.build_store(v_ptr, init_val)?;
        }
        Type::F32(inner) => {
            // Initialize const value
            let init_val = f32_type.const_float(*inner as f64);

            // Store const
            builder.build_store(v_ptr, init_val)?;
        }
        Type::U32(inner) => {
            // Initialize const value
            let init_val = i32_type.const_int(inner as u64, false);

            // Store const
            builder.build_store(v_ptr, init_val)?;
        }
        Type::U8(inner) => {
            // Initialize const value
            let init_val = i8_type.const_int(inner as u64, false);

            // Store const
            builder.build_store(v_ptr, init_val)?;
        }
        Type::String(inner) => {
            let string_bytes = inner.as_bytes();

            let char_array =
                ctx.const_string(string_bytes, Some(0) != string_bytes.last().copied());

            let global_string_handle = if let Some(global_string) = module.get_global(&inner) {
                global_string
            } else {
                let handle =
                    module.add_global(char_array.get_type(), Some(AddressSpace::default()), &inner);

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
        }
        Type::Boolean(inner) => {
            // Initialize const value
            let init_val = bool_type.const_int(inner as u64, false);

            // Store const
            builder.build_store(v_ptr, init_val)?;
        }
        Type::Void => {
            unreachable!()
        }
        Type::Struct((struct_name, struct_inner)) => {
            unreachable!()
        }
    }

    Ok(())
}

pub fn allocate_string<'a>(
    builder: &'a Builder<'_>,
    i8_type: inkwell::types::IntType<'a>,
    string_buffer: String,
) -> Result<(PointerValue<'a>, ArrayType<'a>), anyhow::Error> {
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

/// Converts a `TypeDiscriminant` into a `BasicTypeEnum` which can be used by inkwell.
pub fn ty_to_llvm_ty<'a>(ctx: &'a Context, ty: &TypeDiscriminant) -> Result<BasicTypeEnum<'a>> {
    let bool_type = ctx.bool_type();
    let i8_type = ctx.i8_type();
    let i16_type = ctx.i16_type();
    let i32_type = ctx.i32_type();
    let f16_type = ctx.f16_type();
    let f32_type = ctx.f32_type();
    let i64_type = ctx.i64_type();
    let f64_type = ctx.f64_type();
    let ptr_type = ctx.ptr_type(AddressSpace::default());

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
        }
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
                op_struct_type.set_body(&struct_field_to_ty_list(ctx, struct_inner)?, false);

                // Return the type of the struct
                op_struct_type
            };

            BasicTypeEnum::StructType(struct_type)
        }
        TypeDiscriminant::I64 => BasicTypeEnum::IntType(i64_type),
        TypeDiscriminant::F64 => BasicTypeEnum::FloatType(f64_type),
        TypeDiscriminant::U64 => BasicTypeEnum::IntType(i64_type),
        TypeDiscriminant::I16 => BasicTypeEnum::IntType(i16_type),
        TypeDiscriminant::F16 => BasicTypeEnum::FloatType(f16_type),
        TypeDiscriminant::U16 => BasicTypeEnum::IntType(i16_type),
    };

    Ok(field_ty)
}

pub fn ty_enum_to_metadata_ty_enum(ty_enum: BasicTypeEnum<'_>) -> BasicMetadataTypeEnum<'_> {
    match ty_enum {
        BasicTypeEnum::ArrayType(array_type) => BasicMetadataTypeEnum::ArrayType(array_type),
        BasicTypeEnum::FloatType(float_type) => BasicMetadataTypeEnum::FloatType(float_type),
        BasicTypeEnum::IntType(int_type) => BasicMetadataTypeEnum::IntType(int_type),
        BasicTypeEnum::PointerType(pointer_type) => {
            BasicMetadataTypeEnum::PointerType(pointer_type)
        }
        BasicTypeEnum::StructType(struct_type) => BasicMetadataTypeEnum::StructType(struct_type),
        BasicTypeEnum::VectorType(vector_type) => BasicMetadataTypeEnum::VectorType(vector_type),
    }
}

/// This function takes the field of a struct, and returns the fields' [`BasicTypeEnum`] variant.
/// The returned types are in order with the struct's fields
pub fn struct_field_to_ty_list<'a>(
    ctx: &'a Context,
    struct_inner: &IndexMap<String, TypeDiscriminant>,
) -> Result<Vec<BasicTypeEnum<'a>>> {
    // Allocate a new list for storing the types
    let mut type_list = Vec::new();

    // Iterate over the struct's fields and convert the types into BasicTypeEnums
    for (_, ty) in struct_inner.iter() {
        // Convert the ty
        let basic_ty = ty_to_llvm_ty(ctx, ty)?;

        // Store the ty
        type_list.push(basic_ty);
    }

    Ok(type_list)
}
