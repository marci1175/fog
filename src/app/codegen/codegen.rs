use std::{collections::HashMap, io::ErrorKind, path::PathBuf, slice::Iter};

use anyhow::Result;
use indexmap::IndexMap;
use inkwell::{
    AddressSpace,
    builder::Builder,
    context::Context,
    module::Module,
    passes::PassBuilderOptions,
    targets::{InitializationConfig, RelocMode, Target, TargetMachine},
    types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FunctionType},
    values::{BasicMetadataValueEnum, BasicValueEnum, IntValue, PointerValue},
};

use crate::{
    ApplicationError,
    app::{
        parser::types::{FunctionDefinition, FunctionSignature, ParsedToken},
        type_system::type_system::{Type, TypeDiscriminant},
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
    let module = context.create_module("main");
    let builder = context.create_builder();

    // Import functions defined by the user via llvm
    import_user_lib_functions(&context, &module, imported_functions, parsed_functions);

    // We strictly only want to iterate trough the function definitions' order, because then we will avoid functions not being generated before usage.
    for (function_name, function_definition) in parsed_functions.iter() {
        // Create function signature
        let function = module.add_function(
            function_name,
            create_fn_type_from_ty_disc(&context, function_definition.function_sig.clone()),
            None,
        );

        // Create a BasicBlock
        let basic_block = context.append_basic_block(function, "main_fn_entry");

        // Insert the BasicBlock at the end
        builder.position_at_end(basic_block);

        // Create a HashMap of the arguments the function takes
        let mut arguments: HashMap<String, BasicValueEnum> = HashMap::new();

        // Get the arguments and store them in the HashMap
        for (idx, argument) in function.get_param_iter().enumerate() {
            // Get the name of the argument from the function signature's argument list
            let argument_name = function_definition
                .function_sig
                .args
                .get_index(idx)
                .unwrap()
                .0
                .clone();

            // Set the name of the arguments so that it is easier to debug later
            argument.set_name(&argument_name);

            // Insert the entry
            arguments.insert(argument_name, argument);
        }

        // Iterate through all the `ParsedToken`-s and create the LLVM-IR from the tokens
        create_ir(
            &module,
            &builder,
            &context,
            function_definition.inner.clone(),
            arguments,
            function_definition.function_sig.return_type.clone(),
        )?;
    }

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
    let passes = ["sink", "mem2reg"].join(",");

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

pub fn import_user_lib_functions<'a>(
    ctx: &'a Context,
    module: &Module<'a>,
    imported_functions: &'a HashMap<String, FunctionSignature>,
    parsed_functions: &IndexMap<String, FunctionDefinition>,
) {
    for (import_name, import_sig) in imported_functions.iter() {
        // If a function with the same name as the imports exists, do not expose the function signature instead define the whole function
        // This means that the function has been imported, and we do not need to expose it in the LLVM-IR
        if parsed_functions.contains_key(import_name) {
            continue;
        }

        let mut args = Vec::new();

        for (_, arg_ty) in &import_sig.args {
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
                    let field_ty = struct_field_to_ty_list(ctx, struct_inner);

                    BasicMetadataTypeEnum::StructType(ctx.struct_type(&field_ty, false))
                }
            };

            args.push(argument_sig);
        }

        let function_type = match &import_sig.return_type {
            TypeDiscriminant::I32 => {
                let return_type = ctx.i32_type();

                return_type.fn_type(&args, false)
            }
            TypeDiscriminant::F32 => {
                let return_type = ctx.f32_type();

                return_type.fn_type(&args, false)
            }
            TypeDiscriminant::U32 => {
                let return_type = ctx.i32_type();

                return_type.fn_type(&args, false)
            }
            TypeDiscriminant::U8 => {
                let return_type = ctx.i32_type();

                return_type.fn_type(&args, false)
            }
            TypeDiscriminant::String => {
                let return_type = ctx.ptr_type(AddressSpace::default());

                return_type.fn_type(&args, false)
            }
            TypeDiscriminant::Boolean => {
                let return_type = ctx.bool_type();

                return_type.fn_type(&args, false)
            }
            TypeDiscriminant::Void => {
                let return_type = ctx.void_type();

                return_type.fn_type(&args, false)
            }
            TypeDiscriminant::Struct((_struct_name, struct_inner)) => {
                let return_type =
                    ctx.struct_type(&struct_field_to_ty_list(ctx, struct_inner), false);

                return_type.fn_type(&args, false)
            }
        };

        module.add_function(import_name, function_type, None);
    }
}

pub fn create_ir(
    module: &Module,
    // Inkwell IR builder
    builder: &Builder,
    // Inkwell Context
    ctx: &Context,
    // The list of ParsedToken-s
    parsed_tokens: Vec<ParsedToken>,
    // This argument is initialized with the HashMap of the arguments
    available_arguments: HashMap<String, BasicValueEnum>,
    // Type returned type of the Function
    fn_ret_ty: TypeDiscriminant,
) -> Result<()> {
    let mut variable_map: HashMap<String, (PointerValue, BasicMetadataTypeEnum)> = HashMap::new();

    for (arg_name, arg_val) in available_arguments {
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

        variable_map.insert(arg_name, (v_ptr, ty));
    }

    for token in parsed_tokens {
        create_ir_from_parsed_token(
            ctx,
            module,
            builder,
            token,
            &mut variable_map,
            None,
            fn_ret_ty.clone(),
        )?;
    }

    Ok(())
}

pub fn create_ir_from_parsed_token<'a>(
    ctx: &'a Context,
    module: &'a Module,
    builder: &'a Builder,
    parsed_token: ParsedToken,
    variable_map: &mut HashMap<String, (PointerValue<'a>, BasicMetadataTypeEnum<'a>)>,
    variable_reference: Option<(String, (PointerValue<'a>, BasicMetadataTypeEnum<'a>))>,
    // Type returned type of the Function
    fn_ret_ty: TypeDiscriminant,
) -> anyhow::Result<()> {
    match parsed_token {
        ParsedToken::NewVariable(var_name, var_type, var_set_val) => {
            let (ptr, ptr_ty) = create_new_variable(ctx, builder, var_name.clone(), var_type)?;

            variable_map.insert(var_name.clone(), (ptr, ptr_ty));

            // Set the value of the newly created variable
            create_ir_from_parsed_token(
                ctx,
                module,
                builder,
                *var_set_val,
                variable_map,
                Some((var_name, (ptr, ptr_ty))),
                fn_ret_ty,
            )?;
        }
        ParsedToken::VariableReference(var_ref_variant) => {
            if let Some((var_ref_name, (var_ref_ptr, var_ref_ty))) = variable_reference {
                match var_ref_variant {
                    crate::app::parser::types::VariableReference::StructFieldReference(
                        struct_field_stack,
                        (struct_name, struct_fields),
                    ) => {
                        let mut field_stack_iter = struct_field_stack.field_stack.iter();

                        if let Some(main_struct_var_name) = field_stack_iter.next() {
                            if let Some((ptr, ty)) = variable_map.get(main_struct_var_name) {
                                let (f_ptr, f_ty) = access_nested_field(
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

                        if let ((orig_ptr, orig_ty), Some((ref_ptr, ref_ty))) = (
                            // The original variable we are going to modify
                            (var_ref_ptr, var_ref_ty),
                            // The referenced variable we are going to set the value of the orginal variable with
                            ref_variable_query,
                        ) {
                            if dbg!(orig_ty) != dbg!(*ref_ty) {
                                return Err(CodeGenError::InternalVariableTypeMismatch(
                                    var_ref_name.clone(),
                                    var_name.clone(),
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
            }
        }
        ParsedToken::Literal(literal) => {
            // There this is None there is nothing we can do with this so just return
            if let Some(var_ref) = variable_reference {
                let (ptr, _var_type) = var_ref.1;

                set_value_of_ptr(ctx, builder, literal, ptr)?;
            }
        }
        ParsedToken::TypeCast(parsed_token, type_discriminants) => {
            todo!()
        }
        ParsedToken::MathematicalExpression(parsed_token, mathematical_symbol, parsed_token1) => {
            todo!()
        }
        ParsedToken::Brackets(parsed_tokens, type_discriminants) => todo!(),
        ParsedToken::FunctionCall((fn_sig, fn_name), parsed_tokens) => {
            // Try accessing the function in the current module
            let function_value = module
                .get_function(&fn_name)
                .ok_or(CodeGenError::InternalFunctionNotFound(fn_name))?;

            let sig_iter = fn_sig.args.iter().map(|(key, value)| {
                (
                    key.clone(),
                    (value.clone(), parsed_tokens.get(key).unwrap().clone()),
                )
            });

            // The arguments are in order, if theyre parsed in this order they can be passed to a function as an argument
            let fn_argument_list: IndexMap<String, (TypeDiscriminant, ParsedToken)> =
                IndexMap::from_iter(sig_iter);

            // Keep the list of the arguments passed in
            let mut arguments_passed_in: Vec<BasicMetadataValueEnum> = Vec::new();

            for (arg_name, (arg_type, parsed_token)) in fn_argument_list.iter() {
                // Create a temporary variable for the argument thats passed in
                let (ptr, ptr_ty) =
                    create_new_variable(ctx, builder, arg_name.clone(), arg_type.clone())?;

                // Set the value of the temp variable to the value the argument has
                create_ir_from_parsed_token(
                    ctx,
                    module,
                    builder,
                    parsed_token.clone(),
                    variable_map,
                    Some((arg_name.clone(), (ptr, ptr_ty))),
                    fn_ret_ty.clone(),
                )?;

                // Push the argument to the list of arguments
                match ptr_ty {
                    BasicMetadataTypeEnum::ArrayType(array_type) => {
                        let loaded_val = builder.build_load(array_type, ptr, arg_name)?;

                        arguments_passed_in.push(loaded_val.into());
                    }
                    BasicMetadataTypeEnum::FloatType(float_type) => {
                        let loaded_val = builder.build_load(float_type, ptr, arg_name)?;

                        arguments_passed_in.push(loaded_val.into());
                    }
                    BasicMetadataTypeEnum::IntType(int_type) => {
                        let loaded_val = builder.build_load(int_type, ptr, arg_name)?;

                        arguments_passed_in.push(loaded_val.into());
                    }
                    BasicMetadataTypeEnum::PointerType(pointer_type) => {
                        let loaded_val = builder.build_load(pointer_type, ptr, arg_name)?;

                        arguments_passed_in.push(loaded_val.into());
                    }
                    BasicMetadataTypeEnum::StructType(struct_type) => {
                        let loaded_val = builder.build_load(struct_type, ptr, arg_name)?;

                        arguments_passed_in.push(loaded_val.into());
                    }
                    BasicMetadataTypeEnum::VectorType(vector_type) => {
                        let loaded_val = builder.build_load(vector_type, ptr, arg_name)?;

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
                if let Some(variable_name) = variable_reference {
                    let (v_ptr, _var_ty) = variable_name.1;
                    match fn_sig.return_type {
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
                    };
                }
            } else {
                // Ensure the return type was `Void` else raise an error
                if fn_sig.return_type != TypeDiscriminant::Void {
                    return Err(
                        CodeGenError::InternalFunctionReturnedVoid(fn_sig.return_type).into(),
                    );
                }
            }
        }
        ParsedToken::SetValue(var_ref_ty, value) => match var_ref_ty {
            crate::app::parser::types::VariableReference::StructFieldReference(
                struct_field_reference,
                (struct_name, struct_def),
            ) => {
                let mut field_stack_iter = struct_field_reference.field_stack.iter();

                if let Some(main_struct_var_name) = field_stack_iter.next() {
                    if let Some((ptr, ty)) = variable_map.get(main_struct_var_name) {
                        let (f_ptr, f_ty) = access_nested_field(
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
                            Some((String::new(), (f_ptr, f_ty.into()))),
                            fn_ret_ty,
                        )?;
                    }
                }
            }
            crate::app::parser::types::VariableReference::BasicReference(variable_name) => {
                let variable_query = variable_map.get(&variable_name);

                if let Some((ptr, ty)) = variable_query {
                    // Set the value of the variable which was referenced
                    create_ir_from_parsed_token(
                        ctx,
                        module,
                        builder,
                        *value,
                        variable_map,
                        Some((variable_name, (*ptr, *ty))),
                        fn_ret_ty,
                    )?;
                }
            }
        },
        ParsedToken::MathematicalBlock(parsed_token) => todo!(),
        ParsedToken::ReturnValue(parsed_token) => {
            // Create a temporary variable to store the literal in
            // This temporary variable is used to return the value
            let var_name = String::from("ret_tmp_var");

            let (ptr, ptr_ty) =
                create_new_variable(ctx, builder, var_name.clone(), fn_ret_ty.clone())?;

            // Set the value of the newly created variable
            create_ir_from_parsed_token(
                ctx,
                module,
                builder,
                *parsed_token,
                variable_map,
                Some((var_name, (ptr, ptr_ty))),
                fn_ret_ty.clone(),
            )?;

            match ptr_ty {
                BasicMetadataTypeEnum::ArrayType(array_type) => {
                    builder.build_return(Some(&builder.build_load(
                        array_type,
                        ptr,
                        "ret_tmp_var",
                    )?))?;
                }
                BasicMetadataTypeEnum::FloatType(float_type) => {
                    builder.build_return(Some(&builder.build_load(
                        float_type,
                        ptr,
                        "ret_tmp_var",
                    )?))?;
                }
                BasicMetadataTypeEnum::IntType(int_type) => {
                    builder.build_return(Some(&builder.build_load(
                        int_type,
                        ptr,
                        "ret_tmp_var",
                    )?))?;
                }
                BasicMetadataTypeEnum::PointerType(pointer_type) => {
                    builder.build_return(Some(&builder.build_load(
                        pointer_type,
                        ptr,
                        "ret_tmp_var",
                    )?))?;
                }
                BasicMetadataTypeEnum::StructType(struct_type) => {
                    let loaded_struct = builder.build_load(struct_type, ptr, "ret_tmp_var")?;

                    builder.build_return(Some(&loaded_struct))?;
                }
                BasicMetadataTypeEnum::VectorType(vector_type) => {
                    builder.build_return(Some(&builder.build_load(
                        vector_type,
                        ptr,
                        "ret_tmp_var",
                    )?))?;
                }

                _ => unimplemented!(),
            };
        }
        ParsedToken::If(_) => todo!(),
        ParsedToken::InitializeStruct(struct_tys, struct_fields) => {
            if let Some((var_name, (var_ptr, var_ty))) = variable_reference {
                // Get the struct pointer's ty
                let pointee_struct_ty = var_ty.into_struct_type();

                // Pre-Allocate a struct so that it can be accessed later
                let allocate_struct = builder.build_alloca(pointee_struct_ty, "strct_init")?;

                // Iterate over the struct's fields
                for (field_idx, (field_name, field_ty)) in struct_tys.iter().enumerate() {
                    // Convert to llvm type
                    let llvm_ty = ty_to_llvm_ty(ctx, field_ty);

                    // Create a new temp variable according to the struct's field type
                    let (ptr, ty) = create_new_variable(
                        ctx,
                        builder,
                        field_name.to_string(),
                        field_ty.clone(),
                    )?;

                    // Parse the value for the temp var
                    create_ir_from_parsed_token(
                        ctx,
                        module,
                        builder,
                        *(struct_fields.get_index(field_idx).unwrap().1.clone()),
                        variable_map,
                        Some((field_name.to_string(), (ptr, ty))),
                        fn_ret_ty.clone(),
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
        }
    }

    Ok(())
}

fn access_nested_field<'a>(
    ctx: &'a Context,
    builder: &'a Builder,
    field_stack_iter: &mut Iter<String>,
    struct_definition: &IndexMap<String, TypeDiscriminant>,
    last_field_ptr: (PointerValue<'a>, BasicMetadataTypeEnum<'a>),
) -> Result<(PointerValue<'a>, BasicTypeEnum<'a>)> {
    if let Some(field_stack_entry) = field_stack_iter.next() {
        if let Some((field_idx, _, field_ty)) = struct_definition.get_full(field_stack_entry)
        {
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
                let pointee_ty = ty_to_llvm_ty(ctx, field_ty);

                Ok((last_field_ptr.0, pointee_ty))
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
    var_name: String,
    var_type: TypeDiscriminant,
) -> Result<(PointerValue<'a>, BasicMetadataTypeEnum<'a>), anyhow::Error> {
    // Turn a `TypeDiscriminant` into an LLVM type
    let var_type = ty_to_llvm_ty(ctx, &var_type);

    // Allocate an instance of the converted type
    let v_ptr = builder.build_alloca(var_type, &var_name)?;

    // Return the pointer of the allocation and the type
    Ok((v_ptr, var_type.into()))
}

/// Creates a function type from a FunctionSignature.
/// It uses the Function's return type and arguments to create a `FunctionType` which can be used later in llvm context.
pub fn create_fn_type_from_ty_disc(ctx: &Context, fn_sig: FunctionSignature) -> FunctionType<'_> {
    // Create an LLVM type
    let llvm_ty = ty_to_llvm_ty(ctx, &fn_sig.return_type);

    // Create the actual function type and parse the function's arguments
    llvm_ty.fn_type(&get_args_from_sig(ctx, fn_sig), false)
}

/// Fetches the arguments (and converts it into an LLVM type) from the function's signature
pub fn get_args_from_sig(ctx: &Context, fn_sig: FunctionSignature) -> Vec<BasicMetadataTypeEnum> {
    // Create an iterator over the function's arguments
    let fn_args = fn_sig.args.iter();

    // Create a list for all the arguments
    let mut arg_list: Vec<BasicMetadataTypeEnum> = vec![];

    // Iter over all the arguments and store the converted variants of the argument types
    for (_arg_name, arg_ty) in fn_args {
        // Create an llvm ty
        let argument_sig = ty_to_llvm_ty(ctx, arg_ty);

        // Convert the type and store it
        arg_list.push(argument_sig.into());
    }

    // Return the list
    arg_list
}

/// This function takes in the variable pointer which is dereferenced to set the variable's value.
/// Ensure that we are setting variable type `T` with value `T`
pub fn set_value_of_ptr(
    ctx: &Context,
    builder: &Builder,
    value: Type,
    v_ptr: PointerValue<'_>,
) -> anyhow::Result<()> {
    let bool_type = ctx.bool_type();
    let i32_type = ctx.i32_type();
    let i8_type = ctx.i8_type();
    let f32_type = ctx.f32_type();

    match value {
        Type::I32(inner) => {
            // Initialize const value
            let init_val = i32_type.const_int(inner as u64, true);

            // Store const
            builder.build_store(v_ptr, init_val)?;
        }
        Type::F32(inner) => {
            // Initialize const value
            let init_val = f32_type.const_float(inner as f64);

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
            let buffer_ptr = allocate_string(builder, i8_type, inner)?;

            // Store const
            builder.build_store(v_ptr, buffer_ptr)?;
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
) -> Result<PointerValue<'a>, anyhow::Error> {
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
    Ok(buffer_ptr)
}

/// Converts a `TypeDiscriminant` into a `BasicTypeEnum` which can be used by inkwell.
pub fn ty_to_llvm_ty<'a>(ctx: &'a Context, ty: &TypeDiscriminant) -> BasicTypeEnum<'a> {
    let i32_type = ctx.i32_type();
    let f32_type = ctx.f32_type();
    let ptr_type = ctx.ptr_type(AddressSpace::default());

    // Pattern match the type
    let field_ty = match ty {
        TypeDiscriminant::I32 => BasicTypeEnum::IntType(i32_type),
        TypeDiscriminant::F32 => BasicTypeEnum::FloatType(f32_type),
        TypeDiscriminant::U32 => BasicTypeEnum::IntType(i32_type),
        TypeDiscriminant::U8 => BasicTypeEnum::IntType(i32_type),
        TypeDiscriminant::String => BasicTypeEnum::PointerType(ptr_type),
        TypeDiscriminant::Boolean => BasicTypeEnum::IntType(i32_type),
        TypeDiscriminant::Void => {
            unreachable!();
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
                op_struct_type.set_body(&struct_field_to_ty_list(ctx, struct_inner), false);

                // Return the type of the struct
                op_struct_type
            };

            BasicTypeEnum::StructType(struct_type)
        }
    };

    field_ty
}

/// This function takes the field of a struct, and returns the fields' [`BasicTypeEnum`] variant.
/// The returned types are in order with the struct's fields
pub fn struct_field_to_ty_list<'a>(
    ctx: &'a Context,
    struct_inner: &IndexMap<String, TypeDiscriminant>,
) -> Vec<BasicTypeEnum<'a>> {
    // Allocate a new list for storing the types
    let mut type_list = Vec::new();

    // Iterate over the struct's fields and convert the types into BasicTypeEnums
    for (_, ty) in struct_inner.iter() {
        // Convert the ty
        let basic_ty = ty_to_llvm_ty(ctx, ty);

        // Store the ty
        type_list.push(basic_ty);
    }

    type_list
}
