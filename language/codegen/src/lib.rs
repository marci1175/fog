/// Handles everything allocation related. (Strings, Variables, etc.)
pub mod allocate;
/// Handles the llvm-ir generation od debug symbols and information.
pub mod debug;
/// Handles the llvm-ir generation of external libaries / functions
pub mod import;
/// Generates the llvm-ir from language code.
pub mod irgen;
/// Handles pointers in the programming language
pub mod pointer;

use common::{
    anyhow::Result,
    codegen::CustomItem,
    error::{application::ApplicationError, codegen::CodeGenError},
    indexmap::IndexMap,
    inkwell::{
        builder::Builder,
        context::Context,
        module::Module,
        passes::PassBuilderOptions,
        targets::{InitializationConfig, RelocMode, Target, TargetMachine, TargetTriple},
    },
    parser::{
        common::GlobalContext,
        function::{FunctionDefinition, FunctionSignature},
    },
    tracing::info,
};
use parser::parser::Settings;
use std::{collections::HashMap, io::ErrorKind, path::PathBuf, rc::Rc};

use crate::{import::import_user_lib_functions, irgen::generate_ir};

/// Main function to the codegen module.
/// This function handles everything IR generation related.
pub fn start_codegen<'ctx>(
    context: &'ctx Context,
    builder: &'ctx Builder<'ctx>,
    module: &Module<'ctx>,
    project: GlobalContext,
    path_to_ir_output: PathBuf,
    is_optimized: bool,
    target_triple: TargetTriple,
) -> Result<TargetMachine>
{
    generate_ir(context, module, builder, is_optimized)?;

    // Init target
    Target::initialize_x86(&InitializationConfig::default());

    // Create target
    let target = Target::from_triple(&target_triple)
        .map_err(|_| common::anyhow::Error::from(CodeGenError::FaliedToAcquireTargetTriple))?;

    // Create target machine
    let target_machine = target
        .create_target_machine(
            &target_triple,
            &TargetMachine::get_host_cpu_name().to_string(),
            &TargetMachine::get_host_cpu_features().to_string(),
            common::inkwell::OptimizationLevel::Aggressive,
            RelocMode::Default,
            common::inkwell::targets::CodeModel::Default,
        )
        .unwrap();

    // Create opt passes list
    let passes = ["globaldce", "sink", "mem2reg"].join(",");

    // Run optimization passes if the user prompted to
    if is_optimized {
        let passes = passes.as_str();

        info!("Running optimization passes: {passes}...");
        module
            .run_passes(passes, &target_machine, PassBuilderOptions::create())
            .map_err(|_| CodeGenError::InternalOptimisationPassFailed)?;
    }

    // Set target triple
    module.set_triple(&target_machine.get_triple());

    // Set target data layout
    module.set_data_layout(&target_machine.get_target_data().get_data_layout());

    // Write LLVM IR to a file.
    module.print_to_file(&path_to_ir_output).map_err(|err| {
        ApplicationError::FileError(std::io::Error::new(
            ErrorKind::ExecutableFileBusy,
            err.to_string(),
        ))
    })?;

    // This returns a panic when we want to display a `break` statement
    // target_machine
    //     .write_to_file(
    //         module,
    //         common::inkwell::targets::FileType::Object,
    //         &path_to_o_output,
    //     )
    //     .map_err(|err| {
    //         ApplicationError::FileError(std::io::Error::new(
    //             ErrorKind::ExecutableFileBusy,
    //             err.to_string(),
    //         ))
    //     })?;

    Ok(target_machine)
}
