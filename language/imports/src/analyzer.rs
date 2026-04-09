use std::rc::Rc;

use common::{
    anyhow, compiler::ProjectConfig, dashmap::DashMap, parser::function::FunctionSignature,
    ty::OrdSet,
};
use parser::{parser_instance::ParserSettings, tokenizer::tokenize};

pub fn analyze_dependency(
    source_file_contents: &str,
    deps: Rc<DashMap<Vec<String>, FunctionSignature>>,
    config: ProjectConfig,
    module_path: Vec<String>,
    enabled_features: OrdSet<String>,
) -> anyhow::Result<ParserSettings>
{
    let (tokens) = tokenize(source_file_contents)?;

    let mut parser = ParserSettings::new(config, module_path, enabled_features);

    // parser.parse(deps)?;

    Ok(parser)
}
