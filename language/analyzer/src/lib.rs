use std::collections::HashMap;

use common::{anyhow, error::Spanned, parser::common::GlobalContext, tracing::info};

use crate::semantic::variable_resolver::resolve_identifiers;

/// Checks all types and automatically converts literals to their destined type.
pub mod ty;

/// Semantic analysis, for example checks if the code is actually valid structurally, not just syntactically.
pub mod semantic;

/// The function can return `Ok` while still encountering errors during analysis. These errors are collected and dispalyed after the analysis has ended.
pub fn start_analysis(
    g_ctx: &mut GlobalContext,
) -> anyhow::Result<Vec<(Vec<String>, Spanned<anyhow::Error>)>>
{
    info!("Analyzing ({})...", g_ctx.name);

    // The errors encountered during analysis
    let mut errors = Vec::with_capacity(12);

    // Iter over all of the functions in the global context
    for (path, name, def) in g_ctx.function_definitions.iter_mut() {
        // Resolve imports and identifiers (function names, variable names)
        let imports = g_ctx.ctx_imports.get(path);
        
        // Get the external decls made in this context
        // let extern_decls = g_ctx.external_declerations.get_scope(path);
        
        // Get the current function's signature
        let this_sig = g_ctx.function_signatures.get_item(path, name).unwrap();

        // Resolve items present in context (functions, structs, etc)
        // Analyze and modify the identifiers and return the list of errors
        // Extend the list of errors with the analyzation step
        errors.extend(resolve_identifiers(
            path.to_vec(),
            &g_ctx.items,
            &g_ctx.function_signatures,
            imports,
            // extern_decls,

            // Collect the arguments of this function and load the arguments into the pre-existing variable map
            HashMap::from_iter(this_sig.args.arguments.iter().map(|(name, (ty, _))| { (name.clone(), ty.clone()) })),

            &mut def.body,
        )?);
    }

    // First of all we should resolve everything we can that is produced by the AST parser in an unfinished state. (Such as resolving the type of numeric literals)
    Ok(errors)
}
