use std::collections::HashMap;

use fog_common::{anyhow::Result, dependency::DependencyInfo};

pub fn dependency_requester(dependencies: &mut HashMap<String, DependencyInfo>) -> Result<()> {
    for dep in dependencies.drain() {
        request_dependency(dep)?;
    }

    Ok(())
}

pub fn request_dependency((name, dependency_info): (String, DependencyInfo)) -> Result<()> {
    
    Ok(())
}