use std::{collections::HashMap, path::PathBuf};

use inkwell::{builder::Builder, context::Context, execution_engine::ExecutionEngine, module::Module, values::BasicValueEnum, AddressSpace, OptimizationLevel};
use anyhow::Result;

use crate::app::{parser::types::FunctionDefinition, standard_lib::lib::print};

pub struct CodeGen<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub execution_engine: ExecutionEngine<'ctx>,
}

pub fn codegen_main(parsed_functions: HashMap<String, FunctionDefinition>) -> Result<()> {
    let ctx = Context::create();

    let module = ctx.create_module("main");

    let builder = ctx.create_builder();
    
    let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None).map_err(|err| anyhow::Error::msg(err.to_string()))?;

    let i8_ptr_type = ctx.ptr_type(AddressSpace::default());

    let return_type = ctx.i32_type().const_int(0, false);

    let print_type = ctx.void_type().fn_type(&[i8_ptr_type.into()], false);
    let print_fn = module.add_function("print", print_type, None);

    let main_fn_type = ctx.i32_type().fn_type(&[], false);
    let main_fn = module.add_function("main", main_fn_type, None);
    let entry = ctx.append_basic_block(main_fn, "entry");
    builder.position_at_end(entry);

    let hello_str = builder.build_global_string_ptr("Csá geci!", "msg")?;

    builder.build_call(print_fn, &[hello_str.as_pointer_value().into()], "call_print")?;

    builder.build_return(Some(&BasicValueEnum::IntValue(return_type)))?;

        let print_ptr = print as usize;
        execution_engine.add_global_mapping(&print_fn, print_ptr);

    module.print_to_file(PathBuf::from("codegen.ll")).unwrap();

    unsafe {
        let main_fn_ptr = execution_engine.get_function::<unsafe extern "C" fn()>("main")
            .expect("Failed to get main function");
        main_fn_ptr.call();
    }

    Ok(())
}