use std::{collections::HashMap, io::ErrorKind, path::PathBuf};

use anyhow::Result;
use inkwell::{
    AddressSpace,
    context::Context,
    types::{BasicMetadataTypeEnum, FunctionType},
    values::{FunctionValue, GlobalValue},
};

use crate::{
    CompilerError,
    app::{
        parser::tokens::{FunctionDefinition, FunctionSignature},
        type_system::type_system::TypeDiscriminants,
    },
};

pub fn codegen_main(
    parsed_functions: &HashMap<String, FunctionDefinition>,
    path_to_output: PathBuf,
) -> Result<()> {
    let context = Context::create();
    let module = context.create_module("main");

    for (function_name, function_definition) in parsed_functions.iter() {
        let function_val = module.add_function(
            function_name,
            create_fn_type_from_ty_disc(&context, function_definition.function_sig.clone()),
            None,
        );
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
                panic!("Cant take a void as an argument")
            }
        };

        arg_list.push(argument_sig);
    }

    arg_list
}

pub fn generate_code_from_parsed_tokens() {}

#[derive(Debug, Clone)]
pub struct GlobalCodeGenState {
    pub global_functions: HashMap<String, FunctionValue<'static>>,
    pub global_values: HashMap<String, GlobalValue<'static>>,
}

impl Default for GlobalCodeGenState {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalCodeGenState {
    pub fn new() -> Self {
        Self {
            global_functions: HashMap::new(),
            global_values: HashMap::new(),
        }
    }

    pub fn from_hashmap(
        global_functions: HashMap<String, FunctionValue<'static>>,
        global_values: HashMap<String, GlobalValue<'static>>,
    ) -> Self {
        Self {
            global_functions,
            global_values,
        }
    }
}
