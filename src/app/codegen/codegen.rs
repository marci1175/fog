use std::{collections::HashMap, io::ErrorKind, path::PathBuf};

use anyhow::Result;
use inkwell::{
    AddressSpace,
    builder::Builder,
    context::Context,
    types::{BasicMetadataTypeEnum, FunctionType},
    values::BasicValueEnum,
};

use crate::{
    CompilerError,
    app::{
        parser::tokens::{FunctionDefinition, FunctionSignature, ParsedToken},
        type_system::type_system::{Type, TypeDiscriminants},
    },
};

pub fn codegen_main(
    parsed_functions: &HashMap<String, FunctionDefinition>,
    path_to_output: PathBuf,
) -> Result<()> {
    let context = Context::create();
    let module = context.create_module("main");
    let builder = context.create_builder();

    // Expose library functions
    expose_lib_functions(context, module);

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
    // Inkwell IR builder
    builder: &Builder,
    // Inkwell Context
    ctx: &Context,
    // The list of ParsedToken-s
    parsed_tokens: Vec<ParsedToken>,
    // This argument is initalized with the HashMap of the arguments
    available_variables: HashMap<String, BasicValueEnum>,
) -> Result<()> {
    let i32_type = ctx.i32_type();

    for token in parsed_tokens {
        match token {
            ParsedToken::NewVariable((name, init_val)) => {
                let lit = match *init_val {
                    ParsedToken::NewVariable(_) => todo!(),
                    ParsedToken::VariableReference(_) => todo!(),
                    ParsedToken::Literal(literal) => literal,
                    ParsedToken::TypeCast(parsed_token, type_discriminants) => todo!(),
                    ParsedToken::MathematicalExpression(
                        parsed_token,
                        mathematical_symbol,
                        parsed_token1,
                    ) => todo!(),
                    ParsedToken::Brackets(parsed_tokens, type_discriminants) => todo!(),
                    ParsedToken::FunctionCall(_, parsed_tokens) => todo!(),
                    ParsedToken::SetValue(_, parsed_token) => todo!(),
                    ParsedToken::MathematicalBlock(parsed_token) => todo!(),
                    ParsedToken::ReturnValue(parsed_token) => todo!(),
                    ParsedToken::If(_) => todo!(),
                };

                match lit {
                    crate::app::type_system::type_system::Type::I32(inner) => {
                        // Allocate a new variable
                        let v_ptr = builder.build_alloca(i32_type, &name)?;

                        let init_val = i32_type.const_int(inner as u64, true);

                        builder.build_store(v_ptr, init_val)?;

                        let loaded_val = builder.build_load(i32_type, v_ptr, &name)?;
                    }

                    _ => unimplemented!(),
                }
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

    for (arg_name, arg_ty) in fn_args {
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

use fog_lib::{putchar, getchar};

crate::expose_lib_functions! {
    ((putchar -> i32), i32),
    ((getchar -> i32), )
}

#[macro_export]
macro_rules! expose_lib_functions {
    {$((($fn_name:ident -> $fn_ret:ty), $($fn_arg:ty), *)), +} => {
        pub fn print_function_signatures() {
            $(
                println!("Function: {}", stringify!($fn_name));
                println!("Return Type: {}", stringify!($fn_ret));
                $(
                    println!("Arg Type: {}", stringify!($fn_arg));
                )*
                println!("---");
            )+
        }

        pub fn expose_lib_functions(context: Context, module: inkwell::module::Module) {
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

                    module.add_function($fn_name, function_type, None);
                }
            )+;
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
