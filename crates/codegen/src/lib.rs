pub mod allocate;
pub mod pointer;
pub mod irgen;
pub mod debug;
pub mod import;

use fog_common::{
    anyhow::Result,
    codegen::{
        CustomType, LoopBodyBlocks, ty_to_llvm_ty,
    },
    error::{application::ApplicationError, codegen::CodeGenError},
    indexmap::IndexMap,
    inkwell::{
        basic_block::BasicBlock,
        builder::Builder,
        context::Context,
        module::Module,
        passes::PassBuilderOptions,
        targets::{InitializationConfig, RelocMode, Target, TargetMachine},
        types::BasicMetadataTypeEnum,
        values::{FunctionValue, PointerValue},
    },
    parser::{FunctionDefinition, FunctionSignature, ParsedToken},
    ty::{TypeDiscriminant, token_to_ty},
};
use std::{
    collections::{HashMap, VecDeque},
    fs,
    io::ErrorKind,
    path::PathBuf,
    rc::Rc,
    sync::Arc,
};

use crate::{
    import::import_user_lib_functions, irgen::{create_ir_from_parsed_token, generate_ir}
};

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

pub fn get_unique_id(source: &mut u32) -> u32
{
    *source += 1;

    *source
}

pub fn access_array_index<'main, 'ctx>(
    ctx: &'main Context,
    module: &Module<'ctx>,
    builder: &'ctx Builder<'ctx>,
    variable_map: &mut HashMap<
        String,
        (
            (PointerValue<'ctx>, BasicMetadataTypeEnum<'ctx>),
            TypeDiscriminant,
        ),
    >,
    fn_ret_ty: &TypeDiscriminant,
    this_fn_block: BasicBlock<'ctx>,
    this_fn: FunctionValue<'ctx>,
    allocation_list: &mut VecDeque<(
        ParsedToken,
        PointerValue<'ctx>,
        BasicMetadataTypeEnum<'ctx>,
        TypeDiscriminant,
    )>,
    is_loop_body: &Option<LoopBodyBlocks<'_>>,
    parsed_functions: &Rc<IndexMap<String, FunctionDefinition>>,
    custom_types: &Arc<IndexMap<String, CustomType>>,
    ((array_ptr, _ptr_ty), ty_disc): (
        (PointerValue<'ctx>, BasicMetadataTypeEnum<'ctx>),
        TypeDiscriminant,
    ),
    index: Box<ParsedToken>,
) -> Result<(
    PointerValue<'ctx>,
    BasicMetadataTypeEnum<'ctx>,
    TypeDiscriminant,
)>
where
    'main: 'ctx,
{
    let index_val = create_ir_from_parsed_token(
        ctx,
        module,
        builder,
        *index.clone(),
        variable_map,
        None,
        fn_ret_ty.clone(),
        this_fn_block,
        this_fn,
        allocation_list,
        is_loop_body.clone(),
        parsed_functions.clone(),
        custom_types.clone(),
    )?;

    if let Some((idx_ptr, ptr_ty, idx_ty_disc)) = index_val {
        let idx = builder.build_load(
            ty_to_llvm_ty(ctx, &idx_ty_disc, custom_types.clone())?,
            idx_ptr,
            "array_idx_val",
        )?;

        let pointee_ty = ty_disc
            .clone()
            .to_basic_type_enum(ctx, custom_types.clone())?;

        let gep_ptr = unsafe {
            builder.build_gep(
                pointee_ty,
                array_ptr,
                &[ctx.i32_type().const_int(0, false), idx.into_int_value()],
                "array_idx_elem_ptr",
            )?
        };

        let (inner_ty_token, _len) = ty_disc.try_as_array().unwrap();
        let inner_ty = token_to_ty(*inner_ty_token, custom_types.clone())?;

        Ok((
            gep_ptr,
            inner_ty
                .clone()
                .to_basic_type_enum(ctx, custom_types.clone())?
                .into(),
            inner_ty.clone(),
        ))
    }
    else {
        Err(CodeGenError::InvalidIndexValue(*index.clone()).into())
    }
}

pub fn create_ir_from_parsed_token_list<'main, 'ctx>(
    module: &Module<'ctx>,
    // Inkwell IR builder
    builder: &'ctx Builder<'ctx>,
    // Inkwell Context
    ctx: &'main Context,
    // The list of ParsedToken-s
    parsed_tokens: Vec<ParsedToken>,
    // Type returned type of the Function
    fn_ret_ty: TypeDiscriminant,
    this_fn_block: BasicBlock<'ctx>,
    variable_map: &mut HashMap<
        String,
        (
            (PointerValue<'ctx>, BasicMetadataTypeEnum<'ctx>),
            TypeDiscriminant,
        ),
    >,
    this_fn: FunctionValue<'ctx>,
    // Allocation tables are used when the ParsedTokens run in a loop
    // We store the addresses and names of the variables which have been allocated previously to entering the loop, to avoid a stack overflow
    // Loops should not create new variables on the stack instead they should be using `alloca_table` to look up pointers.
    // If the code we are running is not in a loop we can pass in `None`.
    alloca_table: &mut VecDeque<(
        ParsedToken,
        PointerValue<'ctx>,
        BasicMetadataTypeEnum<'ctx>,
        TypeDiscriminant,
    )>,
    is_loop_body: Option<LoopBodyBlocks>,
    parsed_functions: Rc<IndexMap<String, FunctionDefinition>>,
    custom_items: Arc<IndexMap<String, CustomType>>,
) -> Result<()>
where
    'main: 'ctx,
{
    #[cfg(debug_assertions)]
    {
        use std::fs;

        fs::write(
            format!("{}/input_ir.dbg", env!("CARGO_MANIFEST_DIR")),
            format!("[COMPILER IR]\n{:#?}", parsed_tokens.clone()),
        )?;
    }

    for token in parsed_tokens {
        create_ir_from_parsed_token(
            ctx,
            module,
            builder,
            token.clone(),
            variable_map,
            None,
            fn_ret_ty.clone(),
            this_fn_block,
            this_fn,
            alloca_table,
            is_loop_body.clone(),
            parsed_functions.clone(),
            custom_items.clone(),
        )?;
    }

    Ok(())
}
