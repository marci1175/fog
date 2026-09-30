use std::collections::HashMap;

use common::{
    anyhow,
    codegen::ty::Type,
    error::{Spanned, analyzer::AnalyzerError},
    imports::ImportType,
    parser::common::{CustomItem, PathMap, StatementVariant},
};

pub fn lookup_ident<'a>(
    imports: Option<&'a HashMap<String, ImportType>>,
    ident: &'a str,
) -> Option<&'a ImportType>
{
    imports.and_then(|imports| imports.get(ident))
}

pub fn resolve_identifiers(
    path: Vec<String>,
    items: &PathMap<Vec<String>, String, CustomItem>,
    imports: Option<&HashMap<String, ImportType>>,
    body: &mut Vec<Spanned<StatementVariant>>,
) -> anyhow::Result<Vec<(Vec<String>, Spanned<anyhow::Error>)>>
{
    let mut errors = Vec::new();
    let mut variables: HashMap<String, Type> = HashMap::new();

    for stmt in body {
        match stmt.get_inner() {
            // Store every variable we have created in the function body.
            // This makes sure that if an ident is referenced we can first check if the reference is to a variable and then check if its something we have imported
            StatementVariant::NewVariable {
                variable_type,
                variable_name,
                ..
            } => {
                variables.insert(variable_name.clone(), variable_type.clone());
            },
            // If the token is anything other then a new variable then we can iterate over that statement and its children statement and create RawReference instances from the identifiers.
            _ => {
                stmt.map_mut_child_statements(&mut |statement| {
                    if let StatementVariant::BasicReference { identifier } = statement.get_inner() {
                        // Lookup identifier in var table
                        if variables.contains_key(identifier) {
                            statement.inner = StatementVariant::RawReference {};
                        }
                        else if let Some(_import) = lookup_ident(imports, identifier) {
                            statement.inner = StatementVariant::RawReference {};
                        }
                        else if let Some(_item) = items.get_item(path.clone(), identifier) {
                            statement.inner = StatementVariant::RawReference {};
                        }
                        else {
                            errors.push((
                                path.clone(),
                                Spanned::new(AnalyzerError::IdentifierNotFound.into(), statement.span),
                            ));
                        }
                    }
                    else if let StatementVariant::StructFieldReference { .. } = statement.get_inner() {
                        return false;
                    }

                    true
                });
            },
        }
    }

    Ok(errors)
}
