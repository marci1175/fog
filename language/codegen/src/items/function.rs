use common::{inkwell::{context::Context, types::FunctionType}, parser::function::FunctionSignature};

pub fn create_function_type<'ctx>(ctx: &'ctx Context, sig: FunctionSignature) -> FunctionType<'ctx> {
    
}