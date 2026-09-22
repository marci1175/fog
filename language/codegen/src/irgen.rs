use common::{
    anyhow::{self, Result},
    codegen::{
        CustomItem, FunctionArgumentIdentifier, LoopBodyBlocks, create_fn_type_from_ty_disc,
        fn_arg_to_string, ty_enum_to_metadata_ty_enum, ty_to_llvm_ty,
    },
    error::{Spanned, codegen::CodeGenError},
    indexmap::IndexMap,
    inkwell::{
        AddressSpace,
        attributes::Attribute,
        basic_block::BasicBlock,
        builder::Builder,
        context::Context,
        debug_info::{AsDIScope, DWARFEmissionKind, DWARFSourceLanguage},
        module::Module,
        types::BasicMetadataTypeEnum,
        values::{BasicMetadataValueEnum, BasicValue, BasicValueEnum, FunctionValue, PointerValue},
    },
    parser::{
        common::StatementVariant,
        function::{CompilerInstruction, FunctionDefinition},
        numeric_value::MathematicalSymbol,
        variable::{ControlFlowType, UniqueId},
    },
    tokenizer::Token,
    ty::{OrdMap, OrdSet, Type},
};
use std::{collections::HashMap, rc::Rc};

use crate::{
    debug::create_subprogram_debug_information,
    // pointer::set_value_of_ptr,
};

/// This function is solely for generating the LLVM-IR from the main sourec file.
pub fn generate_ir<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &'ctx Builder<'ctx>,
    is_optimized: bool,
) -> Result<()>
{
    let (debug_info_builder, debug_info_compile_uint) = module.create_debug_info_builder(
        false,
        DWARFSourceLanguage::C,
        module.get_name().to_str()?,
        "<UNUSED>",
        &format!(
            "Fog (ver.: {}) with LLVM {}",
            env!("CARGO_PKG_VERSION"),
            env!("LLVM_VERSION")
        ),
        is_optimized,
        "",
        1,
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

    let dbg_version = context.i32_type().const_int(1, false);
    let dbg_version_md = context.metadata_node(&[dbg_version.as_basic_value_enum().into()]);

    module
        .add_global_metadata("llvm.debug.version", &dbg_version_md)
        .unwrap();

    let debug_info_file = debug_info_compile_uint.get_file();
    let debug_scope = debug_info_file.as_debug_info_scope();

    // for (function_name, function_definition) in parsed_functions.iter() {

    // }

    Ok(())
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
