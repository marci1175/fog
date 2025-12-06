use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct DistributedCompilerWorker
{
    /// The user needs to name their remote workers.
    pub name: String,

    /// Remote address the client is connecting to.
    pub address: String,
    // / The libraries which should be compiled remotely.
    // pub libraries: HashMap<String, DependencyInfo>,
}
