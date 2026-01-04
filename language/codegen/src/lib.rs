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
    codegen::CustomType,
    error::{application::ApplicationError, codegen::CodeGenError},
    indexmap::IndexMap,
    inkwell::{
        builder::Builder,
        context::Context,
        module::Module,
        passes::PassBuilderOptions,
        targets::{InitializationConfig, RelocMode, Target, TargetMachine, TargetTriple},
    },
    parser::function::{FunctionDefinition, FunctionSignature},
    tracing::info,
};
use parser::parser_instance::Parser;
use std::{collections::HashMap, fs, io::ErrorKind, path::PathBuf, rc::Rc, sync::Arc};

use crate::{
    import::import_user_lib_functions,
    irgen::{create_ir_from_parsed_token, generate_ir},
};

/// Main function to the codegen module.
/// This function handles everything IR generation related.
pub fn llvm_codegen_main<'ctx>(
    context: &'ctx Context,
    builder: &'ctx Builder<'ctx>,
    module: &Module<'ctx>,
    parsed_functions: Rc<IndexMap<String, FunctionDefinition>>,
    path_to_ir_output: PathBuf,
    path_to_o_output: PathBuf,
    is_optimized: bool,
    imported_functions: Rc<HashMap<String, FunctionSignature>>,
    custom_types: Rc<IndexMap<String, CustomType>>,
    flags_passed_in: &str,
    path_to_src: &str,
    target_triple: Rc<TargetTriple>,
    cpu_name: Option<String>,
    cpu_features: Option<String>,
) -> Result<TargetMachine>
{
    #[cfg(debug_assertions)]
    {
        use std::{fs::OpenOptions, io::Write};

        if let Ok(mut o_opt) = OpenOptions::new()
            .create(true)
            .write(true)
            .open(format!("{}/input_ir.dbg", env!("CARGO_MANIFEST_DIR")))
        {
            for (_, def) in parsed_functions.iter() {
                o_opt.write_all(format!("------------------- FUNCTION DEFINITION START-------------------\n{:#?}\n------------------- FUNCTION DEFINITION END-------------------\n------------------- FUNCTION BODY START-------------------{:#?}------------------- FUNCTION BODY END-------------------\n", def.signature, def.inner.clone()).as_bytes())?;
            }
        }
    }

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
        path_to_src,
    )?;

    // Init target
    Target::initialize_x86(&InitializationConfig::default());

    // Create target
    let target = Target::from_triple(&target_triple)
        .map_err(|_| common::anyhow::Error::from(CodeGenError::FaliedToAcquireTargetTriple))?;

    // Create target machine
    let target_machine = target
        .create_target_machine(
            &target_triple,
            &cpu_name.unwrap_or_else(|| TargetMachine::get_host_cpu_name().to_string()),
            &cpu_features.unwrap_or_else(|| TargetMachine::get_host_cpu_features().to_string()),
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

/// This function takes in a mutable reference to a number and increments it while returning the current number.
/// This can be used to create incrementing identification numbers.
pub fn get_unique_id(source: &mut u32) -> u32
{
    *source += 1;

    *source
}

/// Wrapper function for the LLVM codegen init function.
pub fn llvm_codegen<'ctx>(
    target_ir_path: PathBuf,
    target_o_path: PathBuf,
    optimization: bool,
    parser_state: Parser,
    function_table: &common::indexmap::IndexMap<String, FunctionDefinition>,
    imported_functions: Rc<std::collections::HashMap<String, FunctionSignature>>,
    context: &'ctx Context,
    builder: &'ctx common::inkwell::builder::Builder<'ctx>,
    module: common::inkwell::module::Module<'ctx>,
    path_to_src: &str,
    flags_passed_in: &str,
    target_triple: Rc<TargetTriple>,
    cpu_name: Option<String>,
    cpu_features: Option<String>,
) -> Result<(), common::anyhow::Error>
{
    panic!();
    
    let _target = llvm_codegen_main(
        context,
        builder,
        &module,
        Rc::new(function_table.clone()),
        target_ir_path,
        target_o_path.clone(),
        optimization,
        imported_functions,
        parser_state.custom_types(),
        flags_passed_in,
        path_to_src,
        target_triple,
        cpu_name,
        cpu_features,
    )?;

    Ok(())
}
