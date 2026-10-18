use common::{
    anyhow,
    codegen::ty::Type,
    error::{SpanInfo, Spanned, analyzer::AnalyzerError},
    parser::common::StatementVariant,
};

use crate::{VariableMap, semantic::type_resolver::resolve_statement_type};

/// This function only parses and verifies the control flow of the function body until an unconditional return statement.
pub fn verify_control_flow(
    body: &Vec<Spanned<StatementVariant>>,
    variable_map: &VariableMap,
    errors: &mut Vec<Spanned<anyhow::Error>>,
    target_return_ty: Option<Type>,
) -> anyhow::Result<(bool, Type)>
{
    // Collect a list of the returned values
    let mut returned_types = Vec::new();

    // Fetch return type recursively
    let returns_val = fetch_returned_type(body, variable_map, &mut returned_types, errors)?;

    Ok((
        returns_val,
        // Verify that all of the returned types are uniform
        // If the return types do not match, this function returns an error
        verify_uniform_return_type(
            &returned_types,
            errors,
            target_return_ty.unwrap_or(
                returned_types
                    .first()
                    .map(|inner| inner.get_inner().clone())
                    .unwrap_or(Type::Void),
            ),
        ).unwrap_or(Type::Placeholder),
    ))
}

fn verify_uniform_return_type(
    returned_types: &[Spanned<Type>],
    errors: &mut Vec<Spanned<anyhow::Error>>,
    correct_ret_ty: Type,
) -> anyhow::Result<Type>
{
    let mismatching_types = returned_types
        .iter()
        .filter(|ret_ty| ret_ty.get_inner() != &correct_ret_ty);

    let mut mismatch_occured = false;

    // Iter over all the mismatching return types
    for mismatching_type in mismatching_types {
        mismatch_occured = true;

        errors.push(mismatching_type.rewrap_value(AnalyzerError::HeterogeneousReturnType.into()));
    }

    // If there were any mismatching types then we can return an error here
    if mismatch_occured {
        return Err(AnalyzerError::HeterogeneousReturnType.into());
    }

    return Ok(correct_ret_ty);
}

fn fetch_returned_type(
    body: &Vec<Spanned<StatementVariant>>,
    variable_map: &VariableMap,
    returned_types: &mut Vec<Spanned<Type>>,
    errors: &mut Vec<Spanned<anyhow::Error>>,
) -> anyhow::Result<bool>
{
    let mut value_returned = false;

    for (idx, stmt) in body.iter().enumerate() {
        // Iterate over all the if statements and such
        let span = stmt.get_span().clone();
        let stmt = stmt.get_inner();

        match stmt {
            StatementVariant::ReturnValue { value } => {
                // Store the returned type
                returned_types.push(Spanned {
                    inner: resolve_statement_type(value.get_inner(), variable_map, errors)?,
                    span,
                });

                // Set this flag to true
                value_returned = true;

                // We dont have to parse further since the code will exit here anyway.
                break;
            },
            StatementVariant::Loop(body) => {
                // We may just assume its not returning
                value_returned = fetch_returned_type(&body, variable_map, returned_types, errors)?;
            },
            // Implement the rest of the loop types here
            StatementVariant::If(stmt) => {
                // Is this the last element in the body
                let is_last_elem = idx == body.len() - 1;

                // Try fetching the return type of the true branch
                let true_returned_val = fetch_returned_type(
                    &stmt.true_branch.body,
                    variable_map,
                    returned_types,
                    errors,
                )?;

                // Try fetching the return type of the false branch if it exists
                let false_returned_val = stmt
                    .false_branch
                    .as_ref()
                    .map(|false_branch| {
                        fetch_returned_type(
                            &false_branch.body,
                            variable_map,
                            returned_types,
                            errors,
                        )
                    })
                    .unwrap_or(Ok(false))?;

                // If neither of the branches return a value then we can signal the function not returning a value, otherwise we can be more specific and refer to the actual path coverage.
                if !true_returned_val && !false_returned_val {
                    return Ok(false);
                }

                // If this is the last element, and not all paths are returning a value, then we can store an error about insufficient path coverage.
                if is_last_elem && !(false_returned_val && true_returned_val) {
                    errors.push(Spanned {
                        inner: AnalyzerError::InsufficientReturnPathCoverage.into(),
                        span,
                    });
                }

                // If either of the cases return atleast one value then we can return a true since we are technically returning a value both not on all paths
                if false_returned_val || true_returned_val {
                    return Ok(true);
                }
            },
            _ => (),
        }
    }

    Ok(value_returned)
}
