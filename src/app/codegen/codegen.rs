use std::{collections::{HashMap, HashSet}, io::ErrorKind, path::PathBuf};

use anyhow::{ensure, Result};
use inkwell::{
    builder::Builder, context::Context, module::Module, types::{BasicMetadataTypeEnum, FunctionType}, values::BasicValueEnum, AddressSpace
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
    expose_lib_functions(&context, module.clone());

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

    for token in parsed_tokens {
        match token {
            ParsedToken::NewVariable((name, init_val)) => {
                match *init_val {
                    ParsedToken::NewVariable(_) => unreachable!("Setting the Value of the Variable as creating a Variable, is a syntax error."),
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
                    },
                    ParsedToken::TypeCast(parsed_token, type_discriminants) => todo!(),
                    ParsedToken::MathematicalExpression(
                        parsed_token,
                        mathematical_symbol,
                        parsed_token1,
                    ) => todo!(),
                    ParsedToken::Brackets(parsed_tokens, type_discriminants) => todo!(),
                    ParsedToken::FunctionCall((fn_sig, fn_name), parsed_tokens) => {
                        // Try accessing the function in the current module
                        let function_value = module.get_function(&fn_name).ok_or(CodeGenError::InternalFunctionNotFound(fn_name))?;
                        
                        // Create function call
                        let call = builder.build_call(function_value, &vec![], "")?;
                        
                        // Handle returned value
                        let returned_value = call.try_as_basic_value().left();

                        if let Some(returned) = returned_value {
                            match fn_sig.return_type {
                                TypeDiscriminants::I32 => {
                                    // Get returned float value
                                    let returned_float = i32_type.const_int(returned.into_int_value().get_sign_extended_constant().unwrap() as u64, true);

                                    // Allocate a new variable
                                    let v_ptr = builder.build_alloca(i32_type, &name)?;

                                    // Store the const in the pointer
                                    builder.build_store(v_ptr, returned_float)?;
                                },
                                TypeDiscriminants::F32 => {
                                    // Get returned float value
                                    let returned_float = f32_type.const_float(returned.into_float_value().get_constant().unwrap().0);

                                    // Allocate a new variable
                                    let v_ptr = builder.build_alloca(f32_type, &name)?;

                                    // Store the const in the pointer
                                    builder.build_store(v_ptr, returned_float)?;
                                },
                                TypeDiscriminants::U32 => {
                                    // Get returned float value
                                    let returned_float = i32_type.const_int(returned.into_int_value().get_zero_extended_constant().unwrap(), false);

                                    // Allocate a new variable
                                    let v_ptr = builder.build_alloca(i32_type, &name)?;

                                    // Store the const in the pointer
                                    builder.build_store(v_ptr, returned_float)?;
                                },
                                TypeDiscriminants::U8 => {
                                    // Get returned float value
                                    let returned_float = i8_type.const_int(returned.into_int_value().get_zero_extended_constant().unwrap(), false);

                                    // Allocate a new variable
                                    let v_ptr = builder.build_alloca(i8_type, &name)?;

                                    // Store the const in the pointer
                                    builder.build_store(v_ptr, returned_float)?;
                                },
                                TypeDiscriminants::String => {
                                    // Get returned pointer value
                                    let returned_ptr = returned.into_pointer_value();

                                    // Allocate a new variable
                                    let v_ptr = builder.build_alloca(pointer_type, &name)?;

                                    // Store the const in the pointer
                                    builder.build_store(v_ptr, returned_ptr)?;
                                },
                                TypeDiscriminants::Boolean => {
                                    // Get returned boolean value
                                    let returned_bool = returned.into_int_value();

                                    let v_ptr = builder.build_alloca(bool_type, &name)?;

                                    builder.build_store(v_ptr, returned_bool)?;
                                },
                                TypeDiscriminants::Void => {
                                    unreachable!("A void can not be parsed, as a void functuion returns a `None`.");
                                },
                            };
                        }
                        else {
                            // Ensure the return type was `Void` else raise an erro
                            if fn_sig.return_type != TypeDiscriminants::Void {
                                return Err(CodeGenError::InternalFunctionReturnedVoid(fn_sig.return_type).into());
                            }
                        }
                    },
                    ParsedToken::SetValue(_, parsed_token) => todo!(),
                    ParsedToken::MathematicalBlock(parsed_token) => todo!(),
                    ParsedToken::ReturnValue(parsed_token) => todo!(),
                    ParsedToken::If(_) => todo!(),
                };
            }
            ParsedToken::ReturnValue(parsed_token) => match *parsed_token {
                ParsedToken::Literal(inner_val) => match inner_val {
                    Type::I32(inner) => {
                        let returned_val = i32_type.const_int(inner as u64, true);

                        builder.build_return(Some(&returned_val))?;
                    }

                    _ => unimplemented!(),
                },

                _ => unimplemented!(),
            },
            ParsedToken::FunctionCall((fn_sig, fn_name), args) => {
                // Try accessing the function in the current module
                let function_value = module.get_function(&fn_name).ok_or(CodeGenError::InternalFunctionNotFound(fn_name))?;
                
                // Create function call
                // We don't have to handle the returned type 
                builder.build_call(function_value, &vec![], "")?;
            },
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

/// Write this function
pub fn get_basic_value_from_parsed_token(ctx: &Context, token: ParsedToken) -> Result<BasicValueEnum> {
    panic!();
    Ok(BasicValueEnum::IntValue(ctx.i32_type().const_int(12, false)))
}

// use fog_lib::{getchar, putchar};

crate::expose_lib_functions! {
    ((putchar -> i32), i32),
    ((getchar -> i32), ),
    ((return_1 -> i32), )
}

#[macro_export]
macro_rules! expose_lib_functions {
    {$((($fn_name:ident -> $fn_ret:ty), $($fn_arg:ty), *)), +} => {
        pub fn expose_lib_functions<'a>(context: &'a Context, module: inkwell::module::Module<'a>) {
            $(
                let type_discriminant = crate::match_type!($fn_ret);

                if let Some(type_disc) = type_discriminant {
                    let function_type = match type_disc {
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
                }
            )+;
        }
        
        pub fn create_function_table() -> std::collections::HashSet<String> {
            let mut function_table = std::collections::HashSet::new();

            $(
                function_table.insert(stringify!($fn_name).to_string());
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
                if let Some(arg_ty) = crate::match_type!($fn_arg) {
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
                    }
                }

            )*;

            args
        }
    };
}

#[macro_export]
macro_rules! match_type {
    (f32) => {
        Some(TypeDiscriminants::F32)
    };
    (i32) => {
        Some(TypeDiscriminants::I32)
    };
    (u32) => {
        Some(TypeDiscriminants::U32)
    };
    (u8) => {
        Some(TypeDiscriminants::U8)
    };
    (bool) => {
        Some(TypeDiscriminants::Boolean)
    };
    (String) => {
        Some(TypeDiscriminants::String)
    };
    ($other:ty) => {
        None
    };
}
