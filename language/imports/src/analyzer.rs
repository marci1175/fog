use std::sync::Arc;

use common::{
    anyhow, compiler::ProjectConfig, dashmap::DashMap, indexmap::IndexMap, parser::FunctionSignature, ty::OrdSet
};
use parser::{parser_instance::Parser, tokenizer::tokenize};

pub fn analyze_dependency(
    source_file_contents: &str,
    deps: Arc<DashMap<Vec<String>, FunctionSignature>>,
    config: ProjectConfig,
    module_path: Vec<String>,
    enabled_features: OrdSet<String>,
) -> anyhow::Result<Parser>
{
    let (tokens, token_ranges, _) = tokenize(source_file_contents, None)?;

    let mut parser = Parser::new(tokens, token_ranges, config, module_path, enabled_features);

    parser.parse(deps)?;

    Ok(parser)
}
