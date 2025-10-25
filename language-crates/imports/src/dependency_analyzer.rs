use std::collections::HashMap;

use fog_common::{anyhow, compiler::ProjectConfig, indexmap::IndexMap, parser::FunctionSignature};
use fog_parser::{parser_instance::Parser, tokenizer::tokenize};

pub fn analyze_dependency(
    source_file_contents: &str,
    deps: HashMap<String, IndexMap<String, FunctionSignature>>,
    config: ProjectConfig,
    module_path: Vec<String>,
) -> anyhow::Result<Parser>
{
    let (tokens, _) = tokenize(source_file_contents, None)?;

    let mut parser = Parser::new(tokens, config, module_path);

    parser.parse(deps)?;

    Ok(parser)
}
