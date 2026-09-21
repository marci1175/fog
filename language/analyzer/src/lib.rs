use common::{anyhow, parser::common::GlobalContext};

/// Checks all types and automatically converts literals to their destined type.
pub mod type_check;

/// Semantic analysis, for example checks if the code is actually valid structurally, not just syntactically.
pub mod semantic;

pub fn start_analysis(g_ctx: &mut GlobalContext) -> anyhow::Result<()>
{
    Ok(())
}
