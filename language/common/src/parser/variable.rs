use crate::{
    parser::common::ParsedTokenInstance,
    ty::{OrdMap, Type},
};
use strum_macros::Display;

#[derive(Debug, Clone, Display, PartialEq, Eq, Hash)]
pub enum ControlFlowType
{
    Break,
    Continue,
}

#[derive(Debug, Clone, Display, PartialEq, Eq, Hash)]
/// VariableReferences are the lowest layer of referencing a variable. This is enum wrapped in a ParsedToken, consult the documentation of that enum variant for more information.ű
/// VariableReferences should not contain themselves as they are only for referencing a variable, there is not much more to it.
pub enum VariableReference
{
    /// Variable name, (struct_name, struct_type)
    StructFieldReference(StructFieldReference, (String, OrdMap<String, Type>)),
    /// Variable name
    BasicReference(String),
    /// Variable name, array index
    ArrayReference(String, Box<ParsedTokenInstance>),
}

/// The first item of the StructFieldReference is used to look up the name of the variable which stores the Struct.
/// The functions which take the iterator of the `field_stack` field should not be passed the first item of the iterator, since the first item is used to look up the name of the variable which stores the struct.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructFieldReference
{
    /// The name of the fields which get referenced
    pub field_stack: Vec<String>,
}

impl Default for StructFieldReference
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl StructFieldReference
{
    /// Creates an instnace from a single entry
    pub fn from_single_entry(field_name: String) -> Self
    {
        Self {
            field_stack: vec![field_name],
        }
    }

    /// Initializes an instance from a list of field entries
    pub fn from_stack(field_stack: Vec<String>) -> Self
    {
        Self { field_stack }
    }

    /// Creates an instnace from an empty list
    pub fn new() -> Self
    {
        Self {
            field_stack: vec![],
        }
    }
}
