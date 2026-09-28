use common::{anyhow, parser::common::GlobalContext};

use crate::semantic::variable_resolver::resolve_identifiers;

/// Checks all types and automatically converts literals to their destined type.
pub mod ty;

/// Semantic analysis, for example checks if the code is actually valid structurally, not just syntactically.
pub mod semantic;

/// The function can return `Ok` while still encountering errors during analysis. These errors are collected and dispalyed after the analysis has ended.
pub fn start_analysis(g_ctx: &mut GlobalContext) -> anyhow::Result<Vec<anyhow::Error>>
{
    // The errors encountered during analysis
    let mut errors: Vec<anyhow::Error> = Vec::new();

    // Iter over all of the functions in the global context
    for (path, _name, def) in g_ctx.functions.iter_mut() {
        // Resolve imports and identifiers (function names, variable names)
        let imports = g_ctx.ctx_imports.get(path);

        // Analyze and modify the identifiers and return the list of errors
        let outputs = resolve_identifiers(imports, &mut def.body)?;
    }

    // First of all we should resolve everything we can that is produced by the AST parser in an unfinished state. (Such as resolving the type of numeric literals)
    Ok(errors)
}
