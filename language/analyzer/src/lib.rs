use std::collections::HashMap;

use common::{
    anyhow,
    codegen::ty::Type,
    error::{Spanned, analyzer::AnalyzerError},
    imports::{ExternalDeclerationType, ImportType},
    parser::{
        common::{CustomItem, GlobalContext, PathMap},
        function::FunctionSignature,
    },
    tracing::info,
};

use crate::semantic::{
    control_flow_resolver::verify_control_flow,
    type_resolver::resolve_numerical_size_in_fn_body,
    variable_resolver::{IdentifierMap, resolve_identifiers},
};

pub type TypeMap = PathMap<Vec<String>, String, CustomItem>;
pub type FnSigMap = PathMap<Vec<String>, String, FunctionSignature>;
pub type ExternalDeclMap = PathMap<Vec<String>, String, ExternalDeclerationType>;
pub type ImportMap = HashMap<String, ImportType>;
pub type VariableMap = HashMap<String, Type>;

/// Checks all types and automatically converts literals to their destined type.
// pub mod ty;

/// Semantic analysis, for example checks if the code is actually valid structurally, not just syntactically.
pub mod semantic;

/// The function can return `Ok` while still encountering errors during analysis. These errors are collected and dispalyed after the analysis has ended.
pub fn start_analysis(
    g_ctx: &mut GlobalContext,
) -> anyhow::Result<Vec<(Vec<String>, Vec<Spanned<anyhow::Error>>)>>
{
    info!("Analyzing ({})...", g_ctx.name);

    // The errors encountered during analysis
    let mut errors: Vec<(Vec<String>, Vec<Spanned<anyhow::Error>>)> = Vec::with_capacity(6);

    // Errors encountered in a given function
    let mut function_errors: Vec<Spanned<anyhow::Error>> = Vec::with_capacity(6);

    // Iter over all of the functions in the global context
    for (path, name, def) in g_ctx.function_definitions.iter_mut() {
        // Resolve imports and identifiers (function names, variable names)
        let imports = g_ctx.ctx_imports.get(path);

        // Get the current function's signature
        let this_sig = g_ctx.function_signatures.get_item(path, name).unwrap();

        // Create a variable map so that we can check the type of the variable later.
        // (If the identifiers are resolved, then this map can be used regardless of where we are trying to resolve the type of a specific variable
        // since the validity of the usage of the variable is verified when creating this list. `resolve_identifiers`)
        let mut variable_map = HashMap::from_iter(
            this_sig
                .args
                .arguments
                .iter()
                .map(|(name, (ty, _))| (name.clone(), ty.clone())),
        );

        // Create a map of all the available identifiers, its easier to handle in this format
        let identifier_map = IdentifierMap {
            items: &g_ctx.items,
            function_sigs: &g_ctx.function_signatures,
            external_declerations: &g_ctx.external_declerations,
            imports,
            variable_map: &mut variable_map,
            path,
        };

        // Resolve items present in context (functions, structs, etc)
        // Analyze and modify the identifiers and return the list of errors
        // Extend the list of errors with the analyzation step
        resolve_identifiers(identifier_map, &mut def.body, &mut function_errors)?;

        // Resolve types in the function bodies
        // This checks the intended type of a value and if that value is a literal, it resizes the literal to the desired type (size)
        resolve_numerical_size_in_fn_body(&mut def.body, &mut function_errors, this_sig)?;

        // Verify the function return paths
        let returned_type = verify_control_flow(
            &def.body,
            &variable_map,
            &mut function_errors,
            Some(this_sig.return_type.clone()),
        )?;

        // If the function doesnt return anything and the return type is not void (meaning we should be returning smth) raise an error.
        if this_sig.return_type != Type::Void && returned_type == Type::Void {
            function_errors.push(Spanned {
                inner: AnalyzerError::FunctionRequiresReturn.into(),
                span: def.span,
            });
        }

        if !function_errors.is_empty() {
            errors.push((path.clone(), std::mem::take(&mut function_errors)));
        }
    }

    // First of all we should resolve everything we can that is produced by the AST parser in an unfinished state. (Such as resolving the type of numeric literals)
    Ok(errors)
}
