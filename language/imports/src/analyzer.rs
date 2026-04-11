use std::rc::Rc;

use common::{
    anyhow, compiler::ProjectConfig, dashmap::DashMap, parser::function::FunctionSignature,
    ty::OrdSet,
};
use parser::{parser::Settings, tokenizer::tokenize};

pub fn analyze_dependency(
    source_file_contents: &str,
    deps: Rc<DashMap<Vec<String>, FunctionSignature>>,
    config: ProjectConfig,
    module_path: Vec<String>,
    enabled_features: OrdSet<String>,
) -> anyhow::Result<Settings>
{
    let tokens = tokenize(source_file_contents)?;

    let mut parser = Settings::new(config, module_path, enabled_features, todo!());

    // parser.parse(deps)?;

    Ok(parser)
}
