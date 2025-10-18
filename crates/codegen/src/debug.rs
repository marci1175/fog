use fog_common::{
    anyhow::Result,
    codegen::CustomType,
    indexmap::IndexMap,
    inkwell::{
        context::Context,
        debug_info::{
            DIFile, DIFlagsConstants, DIScope, DIType,
            DWARFSourceLanguage, DebugInfoBuilder,
        },
        llvm_sys::{
            core::LLVMDisposeMessage,
            error::LLVMDisposeErrorMessage,
            target::{LLVMABIAlignmentOfType, LLVMDisposeTargetData, LLVMStoreSizeOfType},
            target_machine::{
                LLVMCodeGenOptLevel, LLVMCodeModel, LLVMCreateTargetDataLayout,
                LLVMCreateTargetMachine, LLVMDisposeTargetMachine, LLVMGetDefaultTargetTriple,
                LLVMGetTargetFromTriple, LLVMRelocMode,
            },
        },
        module::Module,
        types::AsTypeRef,
    },
    parser::FunctionDefinition,
    ty::{TypeDiscriminant, token_to_ty},
};
use std::{
    ffi::{CStr, CString},
    ptr,
    sync::Arc,
};

use crate::get_unique_id;


/// Stores the DebugInformation type equivalents of the passed in [`TypeDiscriminant`]s.
pub fn generate_debug_inforamtion_types<'ctx>(
    ctx: &'ctx Context,
    module: &Module<'ctx>,
    types_buffer: &mut Vec<fog_common::inkwell::debug_info::DIType<'ctx>>,
    debug_info_builder: &DebugInfoBuilder<'ctx>,
    type_discriminants: Vec<TypeDiscriminant>,
    custom_types: Arc<IndexMap<String, CustomType>>,
    scope: DIScope<'ctx>,
    file: DIFile<'ctx>,
    unique_id_source: &mut u32,
) -> Result<(), String>
{
    for type_disc in type_discriminants {
        let di_ty = generate_debug_type_from_type_disc(
            ctx,
            module,
            debug_info_builder,
            &custom_types,
            type_disc,
            scope,
            file,
            unique_id_source,
        )?;

        types_buffer.push(di_ty);
    }

    Ok(())
}

/// Generates a debug type from a [`TypeDiscriminant`].
/// This can be used to generate debug information, which can be added into the llvm-ir.
pub fn generate_debug_type_from_type_disc<'ctx>(
    ctx: &'ctx Context,
    module: &Module<'ctx>,
    debug_info_builder: &DebugInfoBuilder<'ctx>,
    custom_types: &Arc<IndexMap<String, CustomType>>,
    type_disc: TypeDiscriminant,
    scope: DIScope<'ctx>,
    file: DIFile<'ctx>,
    unique_id_source: &mut u32,
) -> Result<DIType<'ctx>, String>
{
    let debug_type = match type_disc.clone() {
        TypeDiscriminant::Array((array_ty, len)) => {
            let inner_ty_disc = token_to_ty(*array_ty, custom_types.clone()).unwrap();

            let inner_type = get_basic_debug_type_from_ty(
                debug_info_builder,
                custom_types.clone(),
                inner_ty_disc.clone(),
            )?;

            debug_info_builder
                .create_array_type(
                    inner_type.as_type(),
                    (inner_ty_disc.sizeof(custom_types.clone()) * len) as u64,
                    inner_type.as_type().get_align_in_bits(),
                    &[0..len as i64],
                )
                .as_type()
        },
        TypeDiscriminant::Struct((struct_name, struct_def)) => {
            let mut struct_field_types: Vec<DIType> = Vec::new();

            let type_discs = struct_def
                .iter()
                .map(|(_, val)| val.clone())
                .collect::<Vec<TypeDiscriminant>>();

            // Call the function recursively
            generate_debug_inforamtion_types(
                ctx,
                module,
                &mut struct_field_types,
                debug_info_builder,
                type_discs,
                custom_types.clone(),
                scope,
                file,
                unique_id_source,
            )?;

            let struct_type = type_disc
                .to_basic_type_enum(ctx, custom_types.clone())
                .unwrap()
                .into_struct_type();

            let (size_bits, align_bits) = unsafe {
                let target_triple = LLVMGetDefaultTargetTriple();

                let mut target = ptr::null_mut();
                let mut error_message = ptr::null_mut();

                // Get target from triple
                let target_triple_result =
                    LLVMGetTargetFromTriple(target_triple, &mut target, &mut error_message);

                if target_triple_result != 0 {
                    // Failed to get target
                    let c_str = CStr::from_ptr(error_message);

                    LLVMDisposeMessage(error_message);
                    LLVMDisposeMessage(target_triple);

                    return Err(format!(
                        "An error occured while getting the target: {}",
                        c_str.to_string_lossy()
                    ));
                }

                let features = CString::new("").unwrap();
                let generic = CString::new("generic").unwrap();

                let target_machine = LLVMCreateTargetMachine(
                    target,
                    target_triple,
                    generic.as_ptr(),
                    features.as_ptr(),
                    LLVMCodeGenOptLevel::LLVMCodeGenLevelDefault,
                    LLVMRelocMode::LLVMRelocDefault,
                    LLVMCodeModel::LLVMCodeModelDefault,
                );

                let layout = LLVMCreateTargetDataLayout(target_machine);

                let struct_type = struct_type.as_type_ref();

                let ty_size = LLVMStoreSizeOfType(layout, struct_type);
                let alignment = LLVMABIAlignmentOfType(layout, struct_type);

                // Free memory allocated by unsafe calls
                LLVMDisposeMessage(target_triple);
                LLVMDisposeErrorMessage(error_message);
                LLVMDisposeTargetMachine(target_machine);
                LLVMDisposeTargetData(layout);

                (ty_size, alignment)
            };

            debug_info_builder
                .create_struct_type(
                    scope,
                    &struct_name,
                    file,
                    69,
                    size_bits,
                    align_bits,
                    DIFlagsConstants::ZERO,
                    None,
                    &struct_field_types,
                    DWARFSourceLanguage::C as u32,
                    None,
                    &get_unique_id(unique_id_source).to_string(),
                )
                .as_type()
        },
        _ => {
            get_basic_debug_type_from_ty(debug_info_builder, custom_types.clone(), type_disc)?
                .as_type()
        },
    };

    Ok(debug_type)
}

/// Creates a basic debug type from a simple type.
/// A simple type is basically any primitive which encoding is int or uint.
fn get_basic_debug_type_from_ty<'ctx>(
    debug_info_builder: &DebugInfoBuilder<'ctx>,
    custom_types: Arc<IndexMap<String, CustomType>>,
    type_disc: TypeDiscriminant,
) -> Result<fog_common::inkwell::debug_info::DIBasicType<'ctx>, &'static str>
{
    let debug_type = debug_info_builder.create_basic_type(
        &type_disc.to_string(),
        type_disc.sizeof(custom_types.clone()) as u64,
        type_disc.get_dwarf_encoding(),
        DIFlagsConstants::ZERO,
    )?;

    Ok(debug_type)
}


/// Creates a subprogram from a [`FunctionDefinition`] which can be used later to create a debug signatures and information.
/// Please note that this function should only really be used when compiling a debug build by the user.
pub fn create_subprogram_debug_information<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    custom_types: &Arc<IndexMap<String, CustomType>>,
    is_optimized: bool,
    debug_info_builder: &DebugInfoBuilder<'ctx>,
    debug_info_file: DIFile<'ctx>,
    debug_scope: DIScope<'ctx>,
    unique_id_source: &mut u32,
    function_name: &String,
    function_definition: &FunctionDefinition,
    return_type: TypeDiscriminant,
) -> Result<fog_common::inkwell::debug_info::DISubprogram<'ctx>, String>
{
    let debug_return_type = if return_type == TypeDiscriminant::Void {
        None
    }
    else {
        Some(generate_debug_type_from_type_disc(
            context,
            module,
            debug_info_builder,
            custom_types,
            return_type,
            debug_scope,
            debug_info_file,
            unique_id_source,
        )?)
    };

    let mut param_types: Vec<fog_common::inkwell::debug_info::DIType<'ctx>> = Vec::new();

    generate_debug_inforamtion_types(
        context,
        module,
        &mut param_types,
        debug_info_builder,
        function_definition
            .function_sig
            .args
            .arguments_list
            .iter()
            .map(|(_key, value)| value.clone())
            .collect::<Vec<TypeDiscriminant>>(),
        custom_types.clone(),
        debug_scope,
        debug_info_file,
        unique_id_source,
    )?;

    let debug_subroutine_type: fog_common::inkwell::debug_info::DISubroutineType<'_> =
        debug_info_builder.create_subroutine_type(
            debug_info_file,
            debug_return_type,
            &param_types,
            DIFlagsConstants::ZERO,
        );

    Ok(debug_info_builder.create_function(
        debug_scope,
        function_name,
        None,
        debug_info_file,
        69,
        debug_subroutine_type,
        true,
        true,
        69,
        DIFlagsConstants::ZERO,
        is_optimized,
    ))
}