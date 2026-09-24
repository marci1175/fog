use std::collections::HashMap;

use common::{
    anyhow::{self, Result},
    codegen::ty::OrdSet,
    error::codegen::CodeGenError,
    inkwell::{
        attributes::Attribute, builder::Builder, context::Context, module::Module,
        targets::TargetMachine, values::FunctionValue,
    },
    parser::{common::GlobalContext, function::CompilerInstruction},
};

use crate::{
    debug::{DebugInformation, create_debug_information},
    items::function::store_fn_in_module,
};

/// This function is solely for generating the LLVM-IR from the main sourec file.
pub fn start_codegen<'ctx>(
    ctx: &'ctx Context,
    builder: &'ctx Builder<'ctx>,
    global_context: &GlobalContext,
    is_optimized: bool,
    target_machine: &TargetMachine,
) -> Result<HashMap<String, (Module<'ctx>, DebugInformation<'ctx>)>>
{
    // Create a map of the available modules.
    // A module is created if its not found in the map. A module contains every function which has the module name as its first item in its path. (ie. module: `foo` contains foo::bar, foo::bar::baz, etc.)
    let mut modules: HashMap<String, (Module, DebugInformation<'ctx>)> = HashMap::new();

    // The function has its appropriate module found, then the function is parsed as a whole.
    // Every function's name must follow a common rule as following.
    // All functions must have their full paths in their name. (ie. foo::bar::baz => define i32 @"foo::bar::baz"...) This helps the linking process later.
    for (path, name, definition) in global_context.functions.iter() {
        // Lookup module in module map
        let module_name = path
            .first()
            .ok_or(CodeGenError::InternalItemPathEmpty(name.clone()))?;

        // Try to find the module
        if let Some((module, _dbg)) = modules.get(module_name) {
            store_fn_in_module(ctx, builder, path, name, definition, module)?;
        }
        // If the module was not found
        else {
            // Create new module based on the module's name
            let module = ctx.create_module(module_name);

            // Create debug information
            let debug_information = create_debug_information(&module, ctx, is_optimized)?;

            // Set module data
            module.set_data_layout(&target_machine.get_target_data().get_data_layout());
            module.set_triple(&target_machine.get_triple());

            // Store function in module when the module is first created
            store_fn_in_module(ctx, builder, path, name, definition, &module)?;

            // Store module with function and its information
            modules.insert(module_name.clone(), (module, debug_information));
        }
    }

    Ok(modules)
}

pub fn add_compiler_hints_to_fn(
    context: &Context,
    compiler_hints: &OrdSet<CompilerInstruction>,
    function: FunctionValue<'_>,
) -> anyhow::Result<()>
{
    for hint in compiler_hints.iter() {
        match hint {
            CompilerInstruction::Cold => {
                let attr =
                    context.create_enum_attribute(Attribute::get_named_enum_kind_id("cold"), 0);

                function.add_attribute(common::inkwell::attributes::AttributeLoc::Function, attr);
            },
            CompilerInstruction::NoFree => {
                let attr =
                    context.create_enum_attribute(Attribute::get_named_enum_kind_id("nofree"), 0);

                function.add_attribute(common::inkwell::attributes::AttributeLoc::Function, attr);
            },
            CompilerInstruction::Inline => {
                let attr = context
                    .create_enum_attribute(Attribute::get_named_enum_kind_id("inlinehint"), 0);

                function.add_attribute(common::inkwell::attributes::AttributeLoc::Function, attr);
            },
            CompilerInstruction::NoUnWind => {
                let attr =
                    context.create_enum_attribute(Attribute::get_named_enum_kind_id("nounwind"), 0);

                function.add_attribute(common::inkwell::attributes::AttributeLoc::Function, attr);
            },
            CompilerInstruction::Feature(_) => {
                return Err(
                    CodeGenError::InternalFunctionCompilerHintParsingError(hint.clone()).into(),
                );
            },
        }
    }

    Ok(())
}
