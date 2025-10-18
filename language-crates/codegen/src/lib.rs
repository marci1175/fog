/// Handles everything allocation related. (Strings, Variables, etc.)
pub mod allocate;
/// Handles pointers in the programming language
pub mod pointer;
/// Generates the llvm-ir from language code.
pub mod irgen;
/// Handles the llvm-ir generation od debug symbols and information.
pub mod debug;
/// Handles the llvm-ir generation of external libaries / functions
pub mod import;

use fog_common::{
    anyhow::Result,
    codegen::CustomType,
    error::{application::ApplicationError, codegen::CodeGenError},
    indexmap::IndexMap,
    inkwell::{
        builder::Builder,
        context::Context,
        module::Module,
        passes::PassBuilderOptions,
        targets::{InitializationConfig, RelocMode, Target, TargetMachine},
    },
    parser::{FunctionDefinition, FunctionSignature},
};
use std::{
    collections::HashMap,
    fs,
    io::ErrorKind,
    path::PathBuf,
    rc::Rc,
    sync::Arc,
};

use crate::{
    import::import_user_lib_functions, irgen::{create_ir_from_parsed_token, generate_ir}
};

/// Main function to the codegen module.
/// This function handles everything IR generation related.
pub fn codegen_main<'ctx>(
    context: &'ctx Context,
    builder: &'ctx Builder<'ctx>,
    module: &Module<'ctx>,

    parsed_functions: Rc<IndexMap<String, FunctionDefinition>>,
    path_to_ir_output: PathBuf,
    path_to_o_output: PathBuf,
    is_optimized: bool,
    imported_functions: &'ctx HashMap<String, FunctionSignature>,
    custom_types: Arc<IndexMap<String, CustomType>>,
    flags_passed_in: &str,
) -> Result<TargetMachine>
{
    // Import functions defined by the user via llvm
    import_user_lib_functions(
        context,
        module,
        imported_functions,
        parsed_functions.clone(),
        custom_types.clone(),
    )?;

    generate_ir(
        parsed_functions,
        context,
        module,
        builder,
        custom_types,
        is_optimized,
        flags_passed_in,
    )?;

    // Init target
    Target::initialize_x86(&InitializationConfig::default());

    // create target triple
    let traget_triple = TargetMachine::get_default_triple();

    // Create target
    let target = Target::from_triple(&traget_triple)
        .map_err(|_| fog_common::anyhow::Error::from(CodeGenError::FaliedToAcquireTargetTriple))?;

    // Create target machine
    let target_machine = target
        .create_target_machine(
            &traget_triple,
            "generic",
            "",
            fog_common::inkwell::OptimizationLevel::Aggressive,
            RelocMode::PIC,
            fog_common::inkwell::targets::CodeModel::Default,
        )
        .unwrap();

    // Create opt passes list
    let passes = ["globaldce", "sink", "mem2reg"].join(",");

    // Run optimization passes if the user prompted to
    if is_optimized {
        let passes = passes.as_str();

        println!("Running optimization passes: {passes}...");
        module
            .run_passes(passes, &target_machine, PassBuilderOptions::create())
            .map_err(|_| CodeGenError::InternalOptimisationPassFailed)?;
    }

    println!("Writing LLVM-IR to output...");

    // Write LLVM IR to a file.
    module.print_to_file(&path_to_ir_output).map_err(|err| {
        ApplicationError::FileError(std::io::Error::new(
            ErrorKind::ExecutableFileBusy,
            err.to_string(),
        ))
    })?;

    println!(
        "Compilation finished, llvm-ir output is located at: {:?}",
        fs::canonicalize(path_to_ir_output).unwrap_or_default()
    );

    println!("Writing LLVM object code to output...");

    target_machine
        .write_to_file(
            module,
            fog_common::inkwell::targets::FileType::Object,
            &path_to_o_output,
        )
        .map_err(|err| {
            ApplicationError::FileError(std::io::Error::new(
                ErrorKind::ExecutableFileBusy,
                err.to_string(),
            ))
        })?;

    println!(
        "Compilation finished, object code output is located at: {:?}",
        fs::canonicalize(path_to_o_output).unwrap_or_default()
    );

    Ok(target_machine)
}

/// This function takes in a mutable reference to a number and increments it while returning the current number.
/// This can be used to create incrementing identification numbers.
pub fn get_unique_id(source: &mut u32) -> u32
{
    *source += 1;

    *source
}
