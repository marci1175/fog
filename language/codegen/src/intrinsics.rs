use common::{
    anyhow,
    codegen::ty::OrdSet,
    error::codegen::CodeGenError,
    inkwell::{attributes::Attribute, context::Context, values::FunctionValue},
    parser::CompilerInstruction,
};

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
