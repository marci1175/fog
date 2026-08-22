/// Checks all types and automatically converts literals to their destined type.
pub mod type_check;

/// Resolves all dependencies for the project.
pub mod dependency_resolver;

/// Semantic analysis, for example checks if the code is actually valid structurally, not just syntactically.
pub mod semantic;
