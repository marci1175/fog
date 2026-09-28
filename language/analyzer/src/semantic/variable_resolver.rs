use std::collections::HashMap;

use common::{anyhow, error::Spanned, imports::ImportType, parser::common::StatementVariant};

pub fn resolve_identifiers(imports: Option<&HashMap<String, ImportType>>, body: &mut Vec<Spanned<StatementVariant>>) -> anyhow::Result<Vec<anyhow::Error>> {
    let mut errors = Vec::new();

    for stmt in body {
        
    }

    Ok(errors)
}