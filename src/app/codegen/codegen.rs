use std::{
    collections::{HashMap, HashSet},
    io::ErrorKind,
    path::PathBuf,
};

use anyhow::{Result, ensure};
use indexmap::IndexMap;
use inkwell::{
    builder::Builder, context::Context, module::Module, types::{BasicMetadataTypeEnum, FunctionType}, values::{BasicMetadataValueEnum, BasicValueEnum, PointerValue}, AddressSpace
};

use crate::{
    CompilerError,
    app::{
        parser::tokens::{FunctionDefinition, FunctionSignature, ParsedToken},
        type_system::type_system::{Type, TypeDiscriminants},
    },
};

use super::error::CodeGenError;

pub fn codegen_main(
    parsed_functions: &HashMap<String, FunctionDefinition>,
    path_to_output: PathBuf,
) -> Result<()> {
    let context = Context::create();
    let module = context.create_module("main");
    let builder = context.create_builder();

    // Expose library functions
    expose_lib_functions(&context, &module);

    for (function_name, function_definition) in parsed_functions.iter() {
        // Create function signature
        let function = module.add_function(
            function_name,
            create_fn_type_from_ty_disc(&context, function_definition.function_sig.clone()),
            None,
        );

        // Create a BasicBlock
        let basic_block = context.append_basic_block(function, "fn_main_entry");

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
        )?;
    }

    // Write LLVM IR to a file.
    module.print_to_file(path_to_output).map_err(|err| {
        CompilerError::FileError(std::io::Error::new(
            ErrorKind::ExecutableFileBusy,
            err.to_string(),
        ))
    })?;

    Ok(())
}

pub fn create_ir(
    module: &Module,
    // Inkwell IR builder
    builder: &Builder,
    // Inkwell Context
    ctx: &Context,
    // The list of ParsedToken-s
    parsed_tokens: Vec<ParsedToken>,
    // This argument is initalized with the HashMap of the arguments
    available_variables: HashMap<String, BasicValueEnum>,
) -> Result<()> {
    let bool_type = ctx.bool_type();
    let i32_type = ctx.i32_type();
    let i8_type = ctx.i8_type();
    let f32_type = ctx.f32_type();
    let pointer_type = ctx.ptr_type(AddressSpace::default());

    let mut variable_map: HashMap<String, (PointerValue, BasicMetadataTypeEnum)> = HashMap::new();

    for token in parsed_tokens {
        match token {
            ParsedToken::NewVariable((name, init_val)) => {
                match *init_val {
                    ParsedToken::NewVariable(_) => unreachable!(
                        "Setting the Value of the Variable as creating a Variable, is a syntax error."
                    ),
                    ParsedToken::VariableReference(_) => unimplemented!(),
                    ParsedToken::Literal(literal) => {
                        match literal {
                            crate::app::type_system::type_system::Type::I32(inner) => {
                                // Allocate a new variable
                                let v_ptr = builder.build_alloca(i32_type, &name)?;

                                // Initalize const value
                                let init_val = i32_type.const_int(inner as u64, true);

                                // Store const
                                builder.build_store(v_ptr, init_val)?;

                                // Load const into pointer
                                builder.build_load(i32_type, v_ptr, &name)?;
                            }

                            _ => unimplemented!(),
                        }
                    }
                    ParsedToken::TypeCast(parsed_token, type_discriminants) => todo!(),
                    ParsedToken::MathematicalExpression(
                        parsed_token,
                        mathematical_symbol,
                        parsed_token1,
                    ) => todo!(),
                    ParsedToken::Brackets(parsed_tokens, type_discriminants) => todo!(),
                    ParsedToken::FunctionCall((fn_sig, fn_name), parsed_tokens) => {
                        // Try accessing the function in the current module
                        let function_value = module
                            .get_function(&fn_name)
                            .ok_or(CodeGenError::InternalFunctionNotFound(fn_name))?;

                        
                        let sig_iter = fn_sig.args.iter().map(|(key, value)| {
                            ((key.clone(), *value), parsed_tokens.get(key).unwrap().clone())
                        });

                        // The arguments are in order, if theyre parsed in this order they can be passed to a function as an argument
                        let fn_argument_list: IndexMap<(String, TypeDiscriminants), ParsedToken> = IndexMap::from_iter(sig_iter);

                        // Keep the list of the arguments passed in
                        let mut arguments_passed_in: Vec<BasicMetadataValueEnum> = Vec::new();

                        for ((arg_name, arg_type), parsed_token) in fn_argument_list.iter() {
                            match parsed_token {
                                ParsedToken::NewVariable(_) => todo!(),
                                ParsedToken::VariableReference(variable_ref) => {
                                    let (var_ptr, data_type) = variable_map.get(variable_ref).ok_or(CodeGenError::InternalVariableNotFound(variable_ref.clone()))?;
                                    
                                    let value = match arg_type {
                                        TypeDiscriminants::I32 => {
                                            builder.build_load(i32_type, var_ptr.clone(), "dereferenced_variable_reference")?
                                        }
                                        TypeDiscriminants::F32 => {
                                            builder.build_load(f32_type, var_ptr.clone(), "dereferenced_variable_reference")?
                                        }
                                        TypeDiscriminants::U32 => {
                                            builder.build_load(i32_type, var_ptr.clone(), "dereferenced_variable_reference")?
                                        }
                                        TypeDiscriminants::U8 => {
                                            builder.build_load(i32_type, var_ptr.clone(), "dereferenced_variable_reference")?
                                        }
                                        TypeDiscriminants::String => {
                                            unimplemented!()
                                        }
                                        TypeDiscriminants::Boolean => {
                                            builder.build_load(i32_type, var_ptr.clone(), "dereferenced_variable_reference")?
                                        }
                                        TypeDiscriminants::Void => unreachable!(),
                                    };

                                    match arg_type {
                                        TypeDiscriminants::I32 => arguments_passed_in.push(BasicMetadataValueEnum::IntValue(value.into_int_value())),
                                        TypeDiscriminants::F32 => arguments_passed_in.push(BasicMetadataValueEnum::FloatValue(value.into_float_value())),
                                        TypeDiscriminants::U32 => arguments_passed_in.push(BasicMetadataValueEnum::IntValue(value.into_int_value())),
                                        TypeDiscriminants::U8 => arguments_passed_in.push(BasicMetadataValueEnum::IntValue(value.into_int_value())),
                                        TypeDiscriminants::String => arguments_passed_in.push(BasicMetadataValueEnum::PointerValue(value.into_pointer_value())),
                                        TypeDiscriminants::Boolean => arguments_passed_in.push(BasicMetadataValueEnum::IntValue(value.into_int_value())),
                                        TypeDiscriminants::Void => unreachable!(),
                                    }
                                },
                                ParsedToken::Literal(literal) => {
                                    match literal {
                                        Type::I32(inner) => {
                                            arguments_passed_in.push(BasicMetadataValueEnum::IntValue(i32_type.const_int(*inner as u64, true)));
                                        },
                                        Type::F32(inner) => {
                                            arguments_passed_in.push(BasicMetadataValueEnum::FloatValue(f32_type.const_float(*inner as f64)));
                                        },
                                        Type::U32(inner) => {
                                            arguments_passed_in.push(BasicMetadataValueEnum::IntValue(i32_type.const_int(*inner as u64, false)));
                                        },
                                        Type::U8(inner) => {
                                            arguments_passed_in.push(BasicMetadataValueEnum::IntValue(i32_type.const_int(*inner as u64, false)));
                                        },
                                        Type::String(inner) => {
                                            unimplemented!();
                                            // arguments_passed_in.push(BasicMetadataValueEnum::PointerValue());
                                        },
                                        Type::Boolean(inner) => {
                                            arguments_passed_in.push(BasicMetadataValueEnum::IntValue(bool_type.const_int(*inner as u64, true)));
                                        },
                                        Type::Void => {
                                            unreachable!();
                                            // arguments_passed_in.push(BasicMetadataValueEnum::IntValue(i32_type.const_int(*inner as u64, true)));
                                        },
                                    }
                                },
                                ParsedToken::TypeCast(parsed_token, type_discriminants) => todo!(),
                                ParsedToken::MathematicalExpression(parsed_token, mathematical_symbol, parsed_token1) => todo!(),
                                ParsedToken::Brackets(parsed_tokens, type_discriminants) => todo!(),
                                ParsedToken::FunctionCall(_, index_map) => todo!(),
                                ParsedToken::SetValue(_, parsed_token) => todo!(),
                                ParsedToken::MathematicalBlock(parsed_token) => todo!(),
                                ParsedToken::ReturnValue(parsed_token) => todo!(),
                                ParsedToken::If(_) => todo!(),
                            }
                        }

                        // Create function call
                        let call = builder.build_call(function_value, &arguments_passed_in, "function_call")?;

                        // Handle returned value
                        let returned_value = call.try_as_basic_value().left();

                        if let Some(returned) = returned_value {
                            match fn_sig.return_type {
                                TypeDiscriminants::I32 => {
                                    // Get returned float value
                                    let returned_int = returned.into_int_value();

                                    // Allocate a new variable
                                    let v_ptr = builder.build_alloca(i32_type, &name)?;

                                    variable_map.insert(
                                        name.clone(),
                                        (v_ptr.clone(), BasicMetadataTypeEnum::IntType(i32_type)),
                                    );

                                    // Store the const in the pointer
                                    builder.build_store(v_ptr, returned_int)?;
                                }
                                TypeDiscriminants::F32 => {
                                    // Get returned float value
                                    let returned_float = returned.into_float_value();

                                    // Allocate a new variable
                                    let v_ptr = builder.build_alloca(f32_type, &name)?;

                                    variable_map.insert(
                                        name.clone(),
                                        (v_ptr.clone(), BasicMetadataTypeEnum::FloatType(f32_type)),
                                    );

                                    // Store the const in the pointer
                                    builder.build_store(v_ptr, returned_float)?;
                                }
                                TypeDiscriminants::U32 => {
                                    // Get returned float value
                                    let returned_float = returned.into_int_value();

                                    // Allocate a new variable
                                    let v_ptr = builder.build_alloca(i32_type, &name)?;

                                    variable_map.insert(
                                        name.clone(),
                                        (v_ptr.clone(), BasicMetadataTypeEnum::IntType(i32_type)),
                                    );

                                    // Store the const in the pointer
                                    builder.build_store(v_ptr, returned_float)?;
                                }
                                TypeDiscriminants::U8 => {
                                    // Get returned float value
                                    let returned_smalint = returned.into_int_value();

                                    // Allocate a new variable
                                    let v_ptr = builder.build_alloca(i8_type, &name)?;

                                    variable_map.insert(
                                        name.clone(),
                                        (v_ptr.clone(), BasicMetadataTypeEnum::IntType(i8_type)),
                                    );

                                    // Store the const in the pointer
                                    builder.build_store(v_ptr, returned_smalint)?;
                                }
                                TypeDiscriminants::String => {
                                    // Get returned pointer value
                                    let returned_ptr = returned.into_pointer_value();

                                    // Allocate a new variable
                                    let v_ptr = builder.build_alloca(pointer_type, &name)?;

                                    variable_map.insert(
                                        name.clone(),
                                        (
                                            v_ptr.clone(),
                                            BasicMetadataTypeEnum::PointerType(pointer_type),
                                        ),
                                    );

                                    // Store the const in the pointer
                                    builder.build_store(v_ptr, returned_ptr)?;
                                }
                                TypeDiscriminants::Boolean => {
                                    // Get returned boolean value
                                    let returned_bool = returned.into_int_value();

                                    let v_ptr = builder.build_alloca(bool_type, &name)?;

                                    variable_map.insert(
                                        name.clone(),
                                        (v_ptr.clone(), BasicMetadataTypeEnum::IntType(bool_type)),
                                    );

                                    builder.build_store(v_ptr, returned_bool)?;
                                }
                                TypeDiscriminants::Void => {
                                    unreachable!(
                                        "A void can not be parsed, as a void functuion returns a `None`."
                                    );
                                }
                            };
                        } else {
                            // Ensure the return type was `Void` else raise an erro
                            if fn_sig.return_type != TypeDiscriminants::Void {
                                return Err(CodeGenError::InternalFunctionReturnedVoid(
                                    fn_sig.return_type,
                                )
                                .into());
                            }
                        }
                    }
                    ParsedToken::SetValue(_, parsed_token) => todo!(),
                    ParsedToken::MathematicalBlock(parsed_token) => todo!(),
                    ParsedToken::ReturnValue(parsed_token) => todo!(),
                    ParsedToken::If(_) => todo!(),
                };
            }
            ParsedToken::ReturnValue(parsed_token) => {
                match *parsed_token {
                    ParsedToken::Literal(inner_val) => match inner_val {
                        Type::I32(inner) => {
                            let returned_val = i32_type.const_int(inner as u64, true);

                            builder.build_return(Some(&returned_val))?;
                        }

                        _ => unimplemented!(),
                    },
                    ParsedToken::VariableReference(variable_name) => {
                        let get_variable = variable_map.get(&variable_name);

                        if let Some((pointer, var_type)) = get_variable {
                            let variable_value =
                                match var_type {
                                    BasicMetadataTypeEnum::ArrayType(array_type) => {
                                        builder.build_load(*array_type, *pointer, "variable_ref")?
                                    }
                                    BasicMetadataTypeEnum::FloatType(float_type) => {
                                        builder.build_load(*float_type, *pointer, "variable_ref")?
                                    }
                                    BasicMetadataTypeEnum::IntType(int_type) => {
                                        builder.build_load(*int_type, *pointer, "variable_ref")?
                                    }
                                    BasicMetadataTypeEnum::PointerType(pointer_type) => builder
                                        .build_load(*pointer_type, *pointer, "variable_ref")?,
                                    BasicMetadataTypeEnum::StructType(struct_type) => builder
                                        .build_load(*struct_type, *pointer, "variable_ref")?,
                                    BasicMetadataTypeEnum::VectorType(vector_type) => builder
                                        .build_load(*vector_type, *pointer, "variable_ref")?,
                                    BasicMetadataTypeEnum::MetadataType(metadata_type) => {
                                        unimplemented!()
                                    }
                                };

                            builder.build_return(Some(&variable_value))?;
                        }
                    }
                    ParsedToken::FunctionCall((fn_sig, fn_name), arguments) => {}

                    _ => unimplemented!(),
                }
            }
            ParsedToken::FunctionCall((fn_sig, fn_name), args) => {
                // Try accessing the function in the current module
                let function_value = module
                    .get_function(&fn_name)
                    .ok_or(CodeGenError::InternalFunctionNotFound(fn_name))?;

                // Create function call
                // We don't have to handle the returned type
                builder.build_call(function_value, &vec![], "")?;
            }
            _ => unimplemented!(),
        }
    }

    Ok(())
}

pub fn create_fn_type_from_ty_disc(ctx: &Context, fn_sig: FunctionSignature) -> FunctionType<'_> {
    match fn_sig.return_type {
        TypeDiscriminants::I32 => ctx
            .i32_type()
            .fn_type(&get_args_from_sig(ctx, fn_sig), false),
        TypeDiscriminants::F32 => ctx
            .f32_type()
            .fn_type(&get_args_from_sig(ctx, fn_sig), false),
        TypeDiscriminants::U32 => ctx
            .i32_type()
            .fn_type(&get_args_from_sig(ctx, fn_sig), false),
        TypeDiscriminants::U8 => ctx
            .i8_type()
            .fn_type(&get_args_from_sig(ctx, fn_sig), false),
        TypeDiscriminants::String => ctx
            .ptr_type(AddressSpace::default())
            .fn_type(&get_args_from_sig(ctx, fn_sig), false),
        TypeDiscriminants::Boolean => ctx
            .bool_type()
            .fn_type(&get_args_from_sig(ctx, fn_sig), false),
        TypeDiscriminants::Void => ctx
            .void_type()
            .fn_type(&get_args_from_sig(ctx, fn_sig), false),
    }
}

pub fn get_args_from_sig(ctx: &Context, fn_sig: FunctionSignature) -> Vec<BasicMetadataTypeEnum> {
    let fn_args = fn_sig.args.iter();

    let mut arg_list: Vec<BasicMetadataTypeEnum> = vec![];

    for (_arg_name, arg_ty) in fn_args {
        let argument_sig = match arg_ty {
            TypeDiscriminants::I32 => BasicMetadataTypeEnum::IntType(ctx.i32_type()),
            TypeDiscriminants::F32 => BasicMetadataTypeEnum::FloatType(ctx.f32_type()),
            TypeDiscriminants::U32 => BasicMetadataTypeEnum::IntType(ctx.i32_type()),
            TypeDiscriminants::U8 => BasicMetadataTypeEnum::IntType(ctx.i32_type()),
            TypeDiscriminants::String => {
                BasicMetadataTypeEnum::PointerType(ctx.ptr_type(AddressSpace::default()))
            }
            TypeDiscriminants::Boolean => BasicMetadataTypeEnum::IntType(ctx.bool_type()),
            TypeDiscriminants::Void => {
                panic!("Can't take a `Void` as an argument")
            }
        };

        arg_list.push(argument_sig);
    }

    arg_list
}

crate::expose_lib_functions! {
    ((putchar -> i32), "char" = i32),
    ((printchar -> i32), "char" = i32),
    ((getchar -> i32), ),
    ((return_1 -> i32), )
}

#[macro_export]
macro_rules! expose_lib_functions {
    {$((($fn_name:ident -> $fn_ret:ty), $($fn_arg_name: literal = $fn_arg:ty), *)), +} => {
        pub fn expose_lib_functions<'a>(context: &'a Context, module: &inkwell::module::Module<'a>) {
            $(
                let type_discriminant = crate::match_type!($fn_ret);

                let function_type = match type_discriminant {
                    TypeDiscriminants::I32 => {
                        let return_type = context.i32_type();

                        let args = crate::parse_function_args!(context; $($fn_arg),*);

                        let function_type = return_type.fn_type(&args, false);

                        function_type
                    },
                    TypeDiscriminants::F32 => {
                        let return_type = context.f32_type();

                        let args = crate::parse_function_args!(context; $($fn_arg),*);

                        let function_type = return_type.fn_type(&args, false);

                        function_type
                    },
                    TypeDiscriminants::U32 => {
                        let return_type = context.i32_type();

                        let args = crate::parse_function_args!(context; $($fn_arg),*);

                        let function_type = return_type.fn_type(&args, false);

                        function_type
                    },
                    TypeDiscriminants::U8 => {
                        let return_type = context.i32_type();

                        let args = crate::parse_function_args!(context; $($fn_arg),*);

                        let function_type = return_type.fn_type(&args, false);

                        function_type
                    },
                    TypeDiscriminants::String => {
                        let return_type = context.ptr_type(AddressSpace::default());

                        let args = crate::parse_function_args!(context; $($fn_arg),*);

                        let function_type = return_type.fn_type(&args, false);

                        function_type
                    },
                    TypeDiscriminants::Boolean => {
                        let return_type = context.bool_type();

                        let args = crate::parse_function_args!(context; $($fn_arg),*);

                        let function_type = return_type.fn_type(&args, false);

                        function_type
                    },
                    TypeDiscriminants::Void => {
                        let return_type = context.void_type();

                        let args = crate::parse_function_args!(context; $($fn_arg),*);

                        let function_type = return_type.fn_type(&args, false);

                        function_type
                    },

                };

                module.add_function(stringify!($fn_name), function_type, None);
            )+;
        }

        pub fn create_function_table() -> std::collections::HashMap<String, FunctionSignature> {
            let mut function_table = std::collections::HashMap::new();

            $(
                let mut args = indexmap::IndexMap::new();

                $(
                    let arg_type = crate::match_type!($fn_arg);

                    let arg_name = $fn_arg_name.to_string();

                    args.insert(arg_name.to_string(), arg_type);
                )*;

                function_table.insert(stringify!($fn_name).to_string(), FunctionSignature {
                    return_type: crate::match_type!($fn_ret),
                    args: args,
                });
            )+;

            return function_table;
        }
    };
}

#[macro_export]
macro_rules! parse_function_args {
    ( $context:expr; $($fn_arg:ty), *) => {
        {
            let mut args: Vec<BasicMetadataTypeEnum> = Vec::new();

            $(
                let arg_ty = crate::match_type!($fn_arg);

                match arg_ty {
                    TypeDiscriminants::I32 => {
                        args.push(BasicMetadataTypeEnum::IntType($context.i32_type()));
                    },
                    TypeDiscriminants::F32 => {
                        args.push(BasicMetadataTypeEnum::FloatType($context.f32_type()));
                    },
                    TypeDiscriminants::U32 => {
                        args.push(BasicMetadataTypeEnum::IntType($context.i32_type()));
                    },
                    TypeDiscriminants::U8 => {
                        args.push(BasicMetadataTypeEnum::IntType($context.i32_type()));
                    },
                    TypeDiscriminants::String => {
                        args.push(BasicMetadataTypeEnum::PointerType($context.ptr_type(AddressSpace::default())));
                    },
                    TypeDiscriminants::Boolean => {
                        args.push(BasicMetadataTypeEnum::IntType($context.bool_type()));
                    },
                    TypeDiscriminants::Void => {
                        panic!("Can't take `Void` as an argument.");
                    },
                };
            )*;

            args
        }
    };
}

#[macro_export]
macro_rules! match_type {
    ($t:ty) => {{
        match stringify!($t) {
            "i32" => TypeDiscriminants::I32,
            "f32" => TypeDiscriminants::F32,
            "u32" => TypeDiscriminants::U32,
            "u8" => TypeDiscriminants::U8,
            "bool" => TypeDiscriminants::Boolean,
            "String" => TypeDiscriminants::String,
            _ => TypeDiscriminants::Void,
        }
    }};
}
