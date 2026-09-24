use common::{
    anyhow,
    error::codegen::CodeGenError,
    inkwell::{
        context::Context,
        module::{Linkage, Module},
        types::{AnyType, BasicMetadataTypeEnum, BasicType, FunctionType},
    },
    parser::function::FunctionSignature,
};

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
