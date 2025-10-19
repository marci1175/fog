use fog_common::{
    anyhow::Result,
    codegen::{CustomType, struct_field_to_ty_list, ty_enum_to_metadata_ty_enum},
    indexmap::IndexMap,
    inkwell::{AddressSpace, context::Context, module::Module},
    parser::{FunctionDefinition, FunctionSignature},
    ty::TypeDiscriminant,
};
use std::{collections::HashMap, rc::Rc, sync::Arc};

pub fn import_user_lib_functions<'a>(
    ctx: &'a Context,
    module: &Module<'a>,
    imported_functions: Rc<HashMap<String, FunctionSignature>>,
    parsed_functions: Rc<IndexMap<String, FunctionDefinition>>,
    custom_types: Arc<IndexMap<String, CustomType>>,
) -> Result<()>
{
    for (import_name, import_sig) in imported_functions.iter() {
        // If a function with the same name as the imports exists, do not expose the function signature instead define the whole function
        // This means that the function has been imported, and we do not need to expose it in the LLVM-IR
        if parsed_functions.contains_key(import_name) {
            continue;
        }

        let mut args = Vec::new();

        for (_, arg_ty) in import_sig.args.arguments_list.iter() {
            let argument_sig = ty_enum_to_metadata_ty_enum(
                arg_ty
                    .clone()
                    .to_basic_type_enum(ctx, custom_types.clone())?,
            );

            args.push(argument_sig);
        }

        let function_type = match &import_sig.return_type {
            TypeDiscriminant::I32 => {
                let return_type = ctx.i32_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            },
            TypeDiscriminant::F32 => {
                let return_type = ctx.f32_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            },
            TypeDiscriminant::U32 => {
                let return_type = ctx.i32_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            },
            TypeDiscriminant::U8 => {
                let return_type = ctx.i32_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            },
            TypeDiscriminant::String => {
                let return_type = ctx.ptr_type(AddressSpace::default());

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            },
            TypeDiscriminant::Boolean => {
                let return_type = ctx.bool_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            },
            TypeDiscriminant::Void => {
                let return_type = ctx.void_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            },
            TypeDiscriminant::Struct((_struct_name, struct_inner)) => {
                let return_type = ctx.struct_type(
                    &struct_field_to_ty_list(ctx, struct_inner, custom_types.clone())?,
                    import_sig.args.ellipsis_present,
                );

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            },
            TypeDiscriminant::I64 => {
                let return_type = ctx.i64_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            },
            TypeDiscriminant::F64 => {
                let return_type = ctx.f32_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            },
            TypeDiscriminant::U64 => {
                let return_type = ctx.i64_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            },
            TypeDiscriminant::I16 => {
                let return_type = ctx.i16_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            },
            TypeDiscriminant::F16 => {
                let return_type = ctx.f16_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            },
            TypeDiscriminant::U16 => {
                let return_type = ctx.i16_type();

                return_type.fn_type(&args, import_sig.args.ellipsis_present)
            },
            TypeDiscriminant::Array(_) => todo!(),
        };

        module.add_function(import_name, function_type, None);
    }

    Ok(())
}
