use common::{
    anyhow::{self, Result},
    codegen::{
        CustomItem, FunctionArgumentIdentifier, LoopBodyBlocks, create_fn_type_from_ty_disc,
        fn_arg_to_string, ty_enum_to_metadata_ty_enum, ty_to_llvm_ty,
    },
    error::codegen::CodeGenError,
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
        common::{StatementVariant, ParsedTokenInstance},
        function::{CompilerInstruction, FunctionDefinition},
        value::MathematicalSymbol,
        variable::{ControlFlowType, UniqueId},
    },
    tokenizer::Token,
    ty::{OrdMap, OrdSet, Type},
};
use std::{collections::HashMap, rc::Rc};

use crate::{
    allocate::{allocate_string, create_allocation_table, create_new_variable},
    debug::create_subprogram_debug_information,
    pointer::set_value_of_ptr,
};

pub fn create_ir<'main, 'ctx>(
    module: &Module<'ctx>,
    // Inkwell IR builder
    builder: &'ctx Builder<'ctx>,
    // Inkwell Context
    ctx: &'main Context,
    // The list of ParsedToken-s
    parsed_tokens: Vec<ParsedTokenInstance>,
    // This argument is initialized with the HashMap of the arguments
    available_arguments: HashMap<String, (BasicValueEnum<'ctx>, (Type, UniqueId))>,
    // Type returned type of the Function
    fn_ret_ty: Type,
    this_fn_block: BasicBlock<'ctx>,
    this_fn: FunctionValue<'ctx>,
    parsed_functions: Rc<IndexMap<String, FunctionDefinition>>,
    custom_items: Rc<IndexMap<String, CustomItem>>,
) -> Result<()>
where
    'main: 'ctx,
{
    let mut variable_map: HashMap<
        String,
        ((PointerValue, BasicMetadataTypeEnum), (Type, UniqueId)),
    > = HashMap::new();

    for (arg_name, (arg_val, arg_ty)) in available_arguments {
        let (v_ptr, ty) = match arg_val {
            BasicValueEnum::ArrayValue(value) => {
                let v_ptr = builder.build_alloca(value.get_type(), &arg_name)?;
                builder.build_store(v_ptr, value)?;

                (v_ptr, BasicMetadataTypeEnum::ArrayType(value.get_type()))
            },
            BasicValueEnum::IntValue(value) => {
                let v_ptr = builder.build_alloca(value.get_type(), &arg_name)?;
                builder.build_store(v_ptr, value)?;
                (v_ptr, BasicMetadataTypeEnum::IntType(value.get_type()))
            },
            BasicValueEnum::FloatValue(value) => {
                let v_ptr = builder.build_alloca(value.get_type(), &arg_name)?;
                builder.build_store(v_ptr, value)?;
                (v_ptr, BasicMetadataTypeEnum::FloatType(value.get_type()))
            },
            BasicValueEnum::PointerValue(value) => {
                let v_ptr = builder.build_alloca(value.get_type(), &arg_name)?;
                builder.build_store(v_ptr, value)?;
                (v_ptr, BasicMetadataTypeEnum::PointerType(value.get_type()))
            },
            BasicValueEnum::StructValue(value) => {
                let v_ptr = builder.build_alloca(value.get_type(), &arg_name)?;
                builder.build_store(v_ptr, value)?;
                (v_ptr, BasicMetadataTypeEnum::StructType(value.get_type()))
            },
            BasicValueEnum::VectorValue(value) => {
                let v_ptr = builder.build_alloca(value.get_type(), &arg_name)?;
                builder.build_store(v_ptr, value)?;
                (v_ptr, BasicMetadataTypeEnum::VectorType(value.get_type()))
            },
            BasicValueEnum::ScalableVectorValue(_scalable_vector_value) => todo!(),
        };

        variable_map.insert(arg_name, ((v_ptr, ty), arg_ty));
    }

    // create_ir_from_parsed_token_list(
    //     module,
    //     builder,
    //     ctx,
    //     parsed_tokens,
    //     fn_ret_ty,
    //     this_fn_block,
    //     &mut variable_map,
    //     this_fn,
    //     &HashMap::new(),
    //     None,
    //     parsed_functions.clone(),
    //     custom_items.clone(),
    // )?;

    Ok(())
}

pub fn create_function_call_args<'ctx>(
    ctx: &'ctx Context,
    module: &Module<'ctx>,
    builder: &'ctx Builder<'ctx>,
    variable_map: &mut HashMap<
        String,
        (
            (PointerValue<'ctx>, BasicMetadataTypeEnum<'ctx>),
            (Type, usize),
        ),
    >,
    fn_ret_ty: &Type,
    this_fn_block: BasicBlock<'ctx>,
    this_fn: FunctionValue<'ctx>,
    allocation_list: &HashMap<usize, PointerValue<'ctx>>,
    is_loop_body: &Option<LoopBodyBlocks<'_>>,
    parsed_functions: &Rc<IndexMap<String, FunctionDefinition>>,
    custom_types: &Rc<IndexMap<String, CustomItem>>,
    fn_name: String,
    fn_argument_list: &OrdMap<
        FunctionArgumentIdentifier<String, usize>,
        (ParsedTokenInstance, (Type, usize)),
    >,
) -> Result<Vec<BasicMetadataValueEnum<'ctx>>, anyhow::Error>
{
    let mut arguments_passed_in: Vec<BasicMetadataValueEnum> = Vec::new();
    for (arg_ident, (arg_token, (arg_type, arg_id))) in fn_argument_list.iter() {
        let fn_name_clone = fn_name.clone();

        let (ptr, ptr_ty) = create_new_variable(
            ctx,
            builder,
            &fn_arg_to_string(&fn_name_clone, arg_ident),
            arg_type,
            Some(*arg_id),
            allocation_list,
            custom_types.clone(),
        )?;

        // Set the value of the temp variable to the value the argument has
        // create_ir_from_parsed_token(
        //     ctx,
        //     module,
        //     builder,
        //     arg_token.clone(),
        //     variable_map,
        //     Some((
        //         fn_arg_to_string(&fn_name, arg_ident),
        //         (ptr, ptr_ty),
        //         arg_type.clone(),
        //     )),
        //     fn_ret_ty.clone(),
        //     this_fn_block,
        //     this_fn,
        //     allocation_list,
        //     is_loop_body.clone(),
        //     parsed_functions.clone(),
        //     custom_types.clone(),
        // )?;

        // Push the argument to the list of arguments
        match ptr_ty {
            BasicMetadataTypeEnum::ArrayType(array_type) => {
                let loaded_val =
                    builder.build_load(array_type, ptr, &fn_arg_to_string(&fn_name, arg_ident))?;

                arguments_passed_in.push(loaded_val.into());
            },
            BasicMetadataTypeEnum::FloatType(float_type) => {
                let loaded_val =
                    builder.build_load(float_type, ptr, &fn_arg_to_string(&fn_name, arg_ident))?;

                arguments_passed_in.push(loaded_val.into());
            },
            BasicMetadataTypeEnum::IntType(int_type) => {
                let loaded_val =
                    builder.build_load(int_type, ptr, &fn_arg_to_string(&fn_name, arg_ident))?;

                arguments_passed_in.push(loaded_val.into());
            },
            BasicMetadataTypeEnum::PointerType(pointer_type) => {
                let loaded_val = builder.build_load(
                    pointer_type,
                    ptr,
                    &fn_arg_to_string(&fn_name, arg_ident),
                )?;

                arguments_passed_in.push(loaded_val.into());
            },
            BasicMetadataTypeEnum::StructType(struct_type) => {
                let loaded_val =
                    builder.build_load(struct_type, ptr, &fn_arg_to_string(&fn_name, arg_ident))?;

                arguments_passed_in.push(loaded_val.into());
            },
            BasicMetadataTypeEnum::VectorType(vector_type) => {
                let loaded_val =
                    builder.build_load(vector_type, ptr, &fn_arg_to_string(&fn_name, arg_ident))?;

                arguments_passed_in.push(loaded_val.into());
            },

            _ => unimplemented!(),
        }
    }
    Ok(arguments_passed_in)
}

/// This function is solely for generating the LLVM-IR from the main sourec file.
pub fn generate_ir<'ctx>(
    parsed_functions: Rc<IndexMap<String, FunctionDefinition>>,
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &'ctx Builder<'ctx>,
    custom_types: Rc<IndexMap<String, CustomItem>>,
    is_optimized: bool,
    flags_passed_in: &str,
    path_to_src_file: &str,
) -> Result<()>
{
    let (debug_info_builder, debug_info_compile_uint) = module.create_debug_info_builder(
        false,
        DWARFSourceLanguage::C,
        module.get_name().to_str()?,
        path_to_src_file,
        &format!(
            "Fog (ver.: {}) with LLVM {}",
            env!("CARGO_PKG_VERSION"),
            env!("LLVM_VERSION")
        ),
        is_optimized,
        flags_passed_in,
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

    let mut unique_id_source = 0;

    for (_item_name, item) in custom_types.iter() {
        if let CustomItem::Struct((_name, _fields, attr)) = item {
            for (_, impl_fn) in attr.impl_fn_list.iter() {
                // It is safe to unwrap this here since all the functions have been parsed.
                let impl_fn = impl_fn.try_as_parsed_ref().unwrap();

                // If there are any generics present in the function arguments, the function should not be statically parsed and is generated after call during compile
                if !impl_fn.signature.args.generics.is_empty() {
                    continue;
                }

                // Generate IR of the function
                create_function_with_ir(
                    &parsed_functions,
                    context,
                    module,
                    builder,
                    &custom_types,
                    is_optimized,
                    &debug_info_builder,
                    debug_info_file,
                    debug_scope,
                    &mut unique_id_source,
                    &format!("__internal_fn_{_name}_{}", impl_fn.signature.name),
                    impl_fn,
                )?;
            }
        }
    }

    for (function_name, function_definition) in parsed_functions.iter() {
        // If there are any generics present in the function arguments, the function cannot be statically parsed and is generated after call during compile
        if !function_definition.signature.args.generics.is_empty() {
            continue;
        }

        create_function_with_ir(
            &parsed_functions,
            context,
            module,
            builder,
            &custom_types,
            is_optimized,
            &debug_info_builder,
            debug_info_file,
            debug_scope,
            &mut unique_id_source,
            function_name,
            function_definition,
        )?;
    }

    Ok(())
}

fn create_function_with_ir<'ctx>(
    parsed_functions: &Rc<IndexMap<String, FunctionDefinition>>,
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &'ctx Builder<'ctx>,
    custom_types: &Rc<IndexMap<String, CustomItem>>,
    is_optimized: bool,
    debug_info_builder: &common::inkwell::debug_info::DebugInfoBuilder<'ctx>,
    debug_info_file: common::inkwell::debug_info::DIFile<'ctx>,
    debug_scope: common::inkwell::debug_info::DIScope<'ctx>,
    unique_id_source: &mut usize,
    function_name: &String,
    function_definition: &FunctionDefinition,
) -> Result<(), anyhow::Error>
{
    let function_type = create_fn_type_from_ty_disc(
        context,
        function_definition.signature.clone(),
        custom_types.clone(),
    )?;

    let function = module.add_function(function_name, function_type, None);

    add_compiler_hints_to_fn(
        context,
        &function_definition.signature.compiler_instructions,
        function,
    )?;

    let return_type = function_definition.signature.return_type.clone();

    if !is_optimized {
        let debug_subprogram = create_subprogram_debug_information(
            context,
            module,
            custom_types.clone(),
            is_optimized,
            debug_info_builder,
            debug_info_file,
            debug_scope,
            unique_id_source,
            function_name,
            function_definition,
            return_type,
        )
        .map_err(|err| CodeGenError::LibraryLLVMError(err.to_string()))?;

        function.set_subprogram(debug_subprogram);
    }

    let basic_block = context.append_basic_block(function, "main");

    builder.position_at_end(basic_block);

    if function_definition.signature.return_type == Type::Void {
        // Insert the return void instruction
        let instruction = builder.build_return(None)?;

        // Put the builder before that instruction so that it can resume generating IR
        builder.position_before(&instruction);
    }

    let mut arguments: HashMap<String, (BasicValueEnum, (Type, UniqueId))> = HashMap::new();

    for (idx, argument) in function.get_param_iter().enumerate() {
        // Get the name of the argument from the function signature's argument list
        let argument_entry = function_definition
            .signature
            .args
            .arguments
            .get_index(idx)
            .unwrap();

        // Set the name of the arguments so that it is easier to debug later
        argument.set_name(argument_entry.0);

        // Insert the entry
        arguments.insert(
            argument_entry.0.clone(),
            (argument, argument_entry.1.clone()),
        );
    }

    create_ir(
        module,
        builder,
        context,
        // function_definition.body.clone(),
        todo!(),
        arguments,
        function_definition.signature.return_type.clone(),
        basic_block,
        function,
        parsed_functions.clone(),
        custom_types.clone(),
    )?;

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