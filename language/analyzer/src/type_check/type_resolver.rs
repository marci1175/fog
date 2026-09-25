use common::{
    anyhow, codegen::ty::{Type, Value}, error::analyzer::AnalyzerError,
};

/// This function is only ran on values which do not have a type cast. (ie. the user did not specifically cast the value to a specific type.)
pub fn resolve_numerical_size(dest_ty: Type, value: &mut Value) -> anyhow::Result<()>
{
    // Get original value's type
    let original_ty = value.get_type();
    
    // If the original value is not a number return an error
    if !(original_ty.is_float() || original_ty.is_int() || original_ty.is_uint()) {
        return Err(AnalyzerError::InternalValueNotNumerical(original_ty).into());
    }

    // Only the numerical value's size needs to be figured out as the numbers type is resolved unlike their size. (Except int to uint) 
    match value.clone() {
        Value::F64(_) => {
            // If a value is parsed as an f64 it cannot be casted to anything else without data loss.
            if original_ty != dest_ty {
                return Err(AnalyzerError::InvalidAutomaticTypeSizeResolve(value.clone(), dest_ty).into());
            }
        }
        Value::U64(val) => {
            if dest_ty.is_int() {
                // Modify the value passed in
                *value = Value::I64(val as i64);

                return Ok(());
            }
            else {
                return Err(AnalyzerError::InvalidAutomaticTypeSizeResolve(value.clone(), dest_ty).into());
            }
        }
        Value::F32(val) => {}
        Value::U32(val) => {}
        Value::F16(val) => {}
        Value::U16(val) => {}
        Value::U8(val) => {}

        // These are usually not emitted by the AST parser.
        // Negative numbers are parsed as NegateValue(uint)
        Value::I64(val) => return Ok(()),
        Value::I32(val) => {}
        Value::I16(val) => {}

        _ => return Err(AnalyzerError::InternalValueNotNumerical(original_ty).into()),
    }


    Ok(())
}
