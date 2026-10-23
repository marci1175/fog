pub mod control_flow_resolver;

/// Checks all types and automatically converts literals to their destined type.
pub mod type_resolver;

/// This resolve imports and checks whether variables are available when they are referenced
pub mod variable_resolver;
