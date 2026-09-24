use std::ops::Add;

use common::{
    anyhow,
    error::{Spanned, codegen::CodeGenError},
    inkwell::{
        builder::Builder,
        context::Context,
        module::Module,
        types::{BasicMetadataTypeEnum, BasicType, FunctionType},
    },
    parser::{
        common::{ItemVisibility, StatementVariant},
        function::{FunctionDefinition, FunctionSignature},
    },
};

use crate::irgen::add_compiler_hints_to_fn;

pub fn create_function_type<'ctx>(
    ctx: &'ctx Context,
    sig: &FunctionSignature,
) -> anyhow::Result<FunctionType<'ctx>>
{
    // Check if this function can be stored in advance (ie. doesnt have any generics which makes its arguments need to be generated later)
    if !sig.args.generics.is_empty() {
        return Err(CodeGenError::InternalFunctionGeneratable(sig.name.clone()).into());
    }

    // Store the parsed argument types here in order
    let mut arguments: Vec<BasicMetadataTypeEnum<'_>> = Vec::new();

    // Parse the function's arguments
    for (_, (ty, _)) in sig.args.arguments.iter() {
        arguments.push(ty.to_basic_type_enum(ctx)?.into());
    }

    Ok(sig
        .return_type
        .to_basic_type_enum(ctx)?
        .fn_type(&arguments, sig.args.ellipsis_present))
}

pub fn parse_function_body<'ctx>(
    _ctx: &'ctx Context,
    _builder: &Builder<'ctx>,
    _function_signature: &FunctionSignature,
    body: &[Spanned<StatementVariant>],
) -> anyhow::Result<()>
{
    for _stmt in body {}

    Ok(())
}

pub fn store_fn_in_module<'ctx>(
    ctx: &'ctx Context,
    builder: &Builder<'ctx>,
    path: &[String],
    name: &str,
    definition: &FunctionDefinition,
    module: &Module<'ctx>,
) -> Result<(), anyhow::Error>
{
    let function = module.add_function(
        &path.join("::").add(&format!("::{name}")),
        create_function_type(ctx, &definition.signature)?,
        Some({
            match definition.visibility {
                ItemVisibility::Private => common::inkwell::module::Linkage::Internal,
                ItemVisibility::Public => common::inkwell::module::Linkage::External,
                ItemVisibility::Branch => {
                    return Err(CodeGenError::InternalInvalidStructReference.into());
                },
            }
        }),
    );

    // Add hints to the compiler
    add_compiler_hints_to_fn(ctx, &definition.compiler_instructions, function)?;

    // Create a basic body for the function
    let entry = ctx.append_basic_block(function, "main");

    // Set the position of the builder
    builder.position_at_end(entry);

    // Parse the function's body
    parse_function_body(ctx, builder, &definition.signature, &definition.body)?;

    Ok(())
}
