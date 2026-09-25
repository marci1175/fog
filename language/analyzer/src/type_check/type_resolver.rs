use common::{
    anyhow,
    codegen::ty::{Type, Value},
};

pub fn resolve_type(_destination_type: Type, _value: &mut Value) -> anyhow::Result<()>
{
    Ok(())
}
