use std::collections::HashMap;

use common::parser::common::Context;

pub struct DependencyVault
{
    pub vault: HashMap<(), Context>,
}
