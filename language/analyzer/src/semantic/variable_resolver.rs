use crate::{ExternalDeclMap, FnSigMap, ImportMap, TypeMap, VariableMap};
use common::{
    anyhow,
    error::{SpanInfo, Spanned, analyzer::AnalyzerError},
    imports::{ExternalDeclerationType, ImportType},
    parser::common::{
        ResolvedItemReference::{self},
        StatementVariant,
    },
};

/// An identifier map is only valid for a scope. It does not contain the global list of identifiers.
pub struct IdentifierMap<'a>
{
    pub items: &'a TypeMap,
    pub function_sigs: &'a FnSigMap,
    pub external_declerations: &'a ExternalDeclMap,
    pub imports: Option<&'a ImportMap>,
    pub variable_map: &'a mut VariableMap,
    pub path: &'a Vec<String>,
}

impl<'a> IdentifierMap<'a>
{
    fn lookup_ident(
        &self,
        errors: &mut Vec<Spanned<anyhow::Error>>,
        ident_span: SpanInfo,
        identifier: &String,
    ) -> ResolvedItemReference
    {
        // Lookup identifier in the variable table
        if self.variable_map.contains_key(identifier) {
            common::parser::common::ResolvedItemReference::Variable
        }
        // Lookup item the locally available items.
        else if let Some(item) = self.items.get_item(self.path, identifier) {
            common::parser::common::ResolvedItemReference::Type(item.clone())
        }
        // Search ident in external declerations
        else if let Some(external_decl) =
            self.external_declerations.get_item(self.path, identifier)
        {
            match external_decl {
                ExternalDeclerationType::Static(static_var_ty) => {
                    ResolvedItemReference::Static(static_var_ty.clone())
                },
                ExternalDeclerationType::Function(sig) => {
                    ResolvedItemReference::Function(sig.clone())
                },
            }
        }
        // Lookup from inside the imported items too
        else if let Some(import) = lookup_import(self.imports, identifier) {
            match import {
                ImportType::File(_) => {
                    errors.push(Spanned::new(
                        AnalyzerError::ItemNotCallableAsFunction.into(),
                        ident_span,
                    ));

                    ResolvedItemReference::Placeholder
                },
                ImportType::Dependency(imported_path) => {
                    let scope = imported_path[..imported_path.len() - 1].to_vec();

                    if let Some(imported_type) = self
                        .items
                        .get_item(&scope, &imported_path[imported_path.len() - 1])
                    {
                        common::parser::common::ResolvedItemReference::Type(imported_type.clone())
                    }
                    else if let Some(imported_fn) = self
                        .function_sigs
                        .get_item(&scope, &imported_path[imported_path.len() - 1])
                    {
                        common::parser::common::ResolvedItemReference::Function(imported_fn.clone())
                    }
                    else {
                        errors.push(Spanned::new(
                            {
                                if !(self.function_sigs.contains_scope(&scope)
                                    || self.items.contains_scope(&scope))
                                {
                                    AnalyzerError::PathModuleNotFound.into()
                                }
                                else {
                                    AnalyzerError::IdentifierNotFound.into()
                                }
                            },
                            ident_span,
                        ));

                        ResolvedItemReference::Placeholder
                    }
                },
            }
        }
        // If we could not find anything for the given identifier, we can return an error
        else {
            errors.push(Spanned::new(
                AnalyzerError::IdentifierNotFound.into(),
                ident_span,
            ));

            // Insert a placeholder so that we can continue parsing
            ResolvedItemReference::Placeholder
        }
    }
}

#[inline(always)]
pub fn lookup_import<'a>(imports: Option<&'a ImportMap>, ident: &'a str) -> Option<&'a ImportType>
{
    imports.and_then(|imports| imports.get(ident))
}

pub fn resolve_identifiers(
    identifier_map: IdentifierMap,
    body: &mut Vec<Spanned<StatementVariant>>,
    errors: &mut Vec<Spanned<anyhow::Error>>,
) -> anyhow::Result<()>
{
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
                identifier_map
                    .variable_map
                    .insert(variable_name.clone(), variable_type.clone());
            },
            // If the token is anything other then a new variable
            // then we can iterate over that statement and its children statement and
            // create RawReference instances from the identifiers.
            _ => {
                stmt.map_mut_child_statements(&mut |statement| {
                    // Maps all the identifiers even in function (names/identifiers)
                    // There shouldnt be any exceptions where we should stop parsing as everything (that involves variables) resolves directly to a BasicReference
                    if let StatementVariant::BasicReference { identifier } = statement.get_inner() {
                        statement.inner = StatementVariant::InternalCheckedReference {
                            identifier: identifier.clone(),
                            referenced_item: identifier_map.lookup_ident(
                                errors,
                                *statement.get_span(),
                                identifier,
                            ),
                        };
                    }

                    true
                });
            },
        }
    }

    Ok(())
}
