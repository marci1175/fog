use std::collections::HashMap;

use common::{
    anyhow, codegen::ty::Type, error::{Spanned, analyzer::AnalyzerError}, imports::{FFIDeclType, ImportType}, indexmap::IndexMap, parser::{
        common::{
            CustomItem, PathMap,
            ResolvedItemReference::{self, Unresolved},
            StatementVariant,
        },
        function::{FunctionDefinition, FunctionSignature},
    },
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
    function_sigs: &PathMap<Vec<String>, String, FunctionSignature>,
    imports: Option<&HashMap<String, ImportType>>,
    mut variable_map: HashMap<String, Type>,

    body: &mut Vec<Spanned<StatementVariant>>,
) -> anyhow::Result<Vec<(Vec<String>, Spanned<anyhow::Error>)>>
{
    let mut errors = Vec::new();

    for stmt in body {
        match stmt.get_inner() {
            // Store every variable we have created in the function body.
            // This makes sure that if an ident is referenced
            // we can first check if the reference is to a variable and then check if its something we have imported
            StatementVariant::NewVariable {
                variable_type,
                variable_name,
                ..
            } => {
                variable_map.insert(variable_name.clone(), variable_type.clone());
            },
            // If the token is anything other then a new variable
            // then we can iterate over that statement and its children statement and
            // create RawReference instances from the identifiers.
            _ => {
                stmt.map_mut_child_statements(&mut |statement| {
                    if let StatementVariant::BasicReference { identifier } = statement.get_inner() {
                        statement.inner = StatementVariant::RawReference {
                            identifier: identifier.clone(),
                            referenced_item: {
                                // Lookup identifier in the variable table
                                if variable_map.contains_key(identifier) {
                                    common::parser::common::ResolvedItemReference::Variable
                                }
                                // else if let Some(external_decl) = ...
                                // Lookup item the locally available items.
                                else if let Some(item) = items.get_item(&path, identifier) {
                                    common::parser::common::ResolvedItemReference::Type(item.clone())
                                }
                                // Lookup from inside the imported items too
                                else if let Some(import) = lookup_ident(imports, identifier) {
                                    match import {
                                        ImportType::File(_) => {
                                            errors.push((
                                                path.clone(),
                                                Spanned::new(
                                                    AnalyzerError::ItemNotCallableAsFunction.into(),
                                                    statement.span,
                                                ),
                                            ));

                                            ResolvedItemReference::Unresolved
                                        },
                                        ImportType::Dependency(imported_path) => {
                                            let scope = imported_path[..imported_path.len() - 1].to_vec();
                                            
                                            if let Some(imported_type) = items.get_item(
                                                &scope,
                                                &imported_path[imported_path.len() - 1],
                                            ) {
                                                common::parser::common::ResolvedItemReference::Type(
                                                    imported_type.clone(),
                                                )
                                            }
                                            else if let Some(imported_fn) = function_sigs
                                                .get_item(
                                                    &scope,
                                                    &imported_path[imported_path.len() - 1],
                                                )
                                            {
                                                common::parser::common::ResolvedItemReference::Function(
                                                    imported_fn.clone(),
                                                )
                                            }
                                            else {
                                                errors.push((
                                                    path.clone(),
                                                    Spanned::new(
                                                        {
                                                            if !(function_sigs.contains_scope(&scope) || items.contains_scope(&scope)) {
                                                                AnalyzerError::PathModuleNotFound.into()
                                                            }
                                                            else {
                                                                AnalyzerError::IdentifierNotFound.into()
                                                            }
                                                        },
                                                        statement.span,
                                                    ),
                                                ));

                                                ResolvedItemReference::Unresolved
                                            }
                                        },
                                    }
                                }
                                else {
                                    errors.push((
                                        path.clone(),
                                        Spanned::new(
                                            AnalyzerError::IdentifierNotFound.into(),
                                            statement.span,
                                        ),
                                    ));

                                    ResolvedItemReference::Unresolved
                                }
                            },
                        };
                    }
                    else if let StatementVariant::StructFieldReference { .. } =
                        statement.get_inner()
                    {
                        return false;
                    }

                    true
                });
            },
        }
    }

    Ok(errors)
}
