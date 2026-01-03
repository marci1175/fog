use common::{
    DEFAULT_COMPILER_ADDRESS_SPACE_SIZE,
    anyhow::Result,
    codegen::{CustomType, struct_field_to_ty_list, ty_enum_to_metadata_ty_enum},
    indexmap::IndexMap,
    inkwell::{AddressSpace, context::Context, module::Module, types::BasicType},
    parser::function::{FunctionDefinition, FunctionSignature},
    ty::Type,
};
use std::{collections::HashMap, rc::Rc};

pub fn import_user_lib_functions<'a>(
    ctx: &'a Context,
    module: &Module<'a>,
    imported_functions: Rc<HashMap<String, FunctionSignature>>,
    parsed_functions: Rc<IndexMap<String, FunctionDefinition>>,
    custom_types: Rc<IndexMap<String, CustomType>>,
) -> Result<()>
{
    for (import_name, import_sig) in imported_functions.iter() {
        // If a function with the same name as the imports exists, do not expose the function signature instead define the whole function
        // This means that the function has been imported, and we do not need to expose it in the LLVM-IR
        if parsed_functions.contains_key(import_name) {
            continue;
        }

        let mut args = Vec::new();

        for (_, arg_ty) in import_sig.args.arguments.iter() {
            let argument_sig = ty_enum_to_metadata_ty_enum(
                arg_ty
                    .clone()
                    .to_basic_type_enum(ctx, custom_types.clone())?,
            );

            args.push(argument_sig);
        }

        let function_type = match &import_sig.return_type {
            Type::I32 => {
                let return_type = ctx.i32_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            },
            Type::F32 => {
                let return_type = ctx.f32_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            },
            Type::U32 => {
                let return_type = ctx.i32_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            },
            Type::U8 => {
                let return_type = ctx.i32_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            },
            Type::String => {
                let return_type =
                    ctx.ptr_type(AddressSpace::from(DEFAULT_COMPILER_ADDRESS_SPACE_SIZE));

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            },
            Type::Boolean => {
                let return_type = ctx.bool_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            },
            Type::Void => {
                let return_type = ctx.void_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            },
            Type::Struct((_struct_name, struct_inner)) => {
                let return_type = ctx.struct_type(
                    &struct_field_to_ty_list(ctx, struct_inner, custom_types.clone())?,
                    import_sig.args.ellipsis_present,
                );

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            },
            Type::I64 => {
                let return_type = ctx.i64_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            },
            Type::F64 => {
                let return_type = ctx.f32_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            },
            Type::U64 => {
                let return_type = ctx.i64_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            },
            Type::I16 => {
                let return_type = ctx.i16_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            },
            Type::F16 => {
                let return_type = ctx.f16_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            },
            Type::U16 => {
                let return_type = ctx.i16_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            },
            Type::Array(_) => todo!(),
            Type::Pointer(_) => {
                let return_type =
                    ctx.ptr_type(AddressSpace::from(DEFAULT_COMPILER_ADDRESS_SPACE_SIZE));

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            },
            Type::Enum((ty, _)) => {
                let return_type = ty.to_basic_type_enum(&ctx, custom_types.clone())?;

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            },
        };

        module.add_function(import_name, function_type, None);
    }

    Ok(())
}