use std::collections::HashMap;

use common::{anyhow, codegen::ty::Type, error::Spanned, imports::ImportType, parser::common::StatementVariant};

pub fn lookup_ident<'a>(imports: Option<&'a HashMap<String, ImportType>>, ident: &'a str) -> Option<&'a ImportType> {
    imports.map(|imports| imports.get(ident)).flatten()
}

pub fn resolve_identifiers(imports: Option<&HashMap<String, ImportType>>, body: &mut Vec<Spanned<StatementVariant>>) -> anyhow::Result<Vec<anyhow::Error>> {
    let mut errors = Vec::new();
    let mut variables: HashMap<String, Type> = HashMap::new();

    for stmt in body {
        match stmt.get_inner() {
            // Store every variable we have created in the function body.
            // This makes sure that if an ident is referenced we can first check if the reference is to a variable and then check if its something we have imported
            StatementVariant::NewVariable { variable_name, variable_type, .. } => {
                variables.insert(variable_name.clone(), variable_type.clone());
            }
            _ => continue,
        }
    }

    Ok(errors)
}