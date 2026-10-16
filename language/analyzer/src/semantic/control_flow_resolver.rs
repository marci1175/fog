use common::{
    anyhow,
    codegen::ty::Type,
    error::{SpanInfo, Spanned, analyzer::AnalyzerError},
    parser::common::StatementVariant,
};

use crate::{VariableMap, semantic::type_resolver::resolve_statement_type};

/// This function only parses and verifies the control flow of the function until a unconditional return statement.
pub fn verify_function_control_flow(
    body: &Vec<Spanned<StatementVariant>>,
    variable_map: &VariableMap,
    errors: &mut Vec<Spanned<anyhow::Error>>,
) -> anyhow::Result<(bool, Vec<Option<Spanned<Type>>>)>
{
    // Collect a list of the returned values
    let mut returned_types = Vec::new();

    // Fetch return type recursively
    let returns_val = fetch_return_type(body, variable_map, &mut returned_types, errors)?;

    Ok((returns_val, returned_types))
}

fn fetch_return_type(
    body: &Vec<Spanned<StatementVariant>>,
    variable_map: &VariableMap,
    returned_types: &mut Vec<Option<Spanned<Type>>>,
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
                returned_types.push(
                    resolve_statement_type(value.get_inner(), variable_map)?
                        .map(|ty| Spanned { inner: ty, span }),
                );

                // Set this flag to true
                value_returned = true;

                // We dont have to parse further since the code will exit here anyway.
                break;
            },
            StatementVariant::Loop(body) => {
                fetch_return_type(&body, variable_map, returned_types, errors)?;
            },
            // Implement the rest of the loop types here
            StatementVariant::If(stmt) => {
                // Is this the last element in the body
                let is_last_elem = idx == body.len() - 1;

                // Try fetching the return type of the true branch
                let true_returned_val = fetch_return_type(
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
                        fetch_return_type(&false_branch.body, variable_map, returned_types, errors)
                    })
                    .unwrap_or(Ok(false))?;

                // If neither of the branches return a value then we can raise an error about the function not returning a value, otherwise we can be more specific and refer to the actual path coverage.
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
