use std::{
    collections::HashMap,
    fmt::Display,
    ops::{Deref, DerefMut},
};

use indexmap::IndexMap;
use strum_macros::Display;

use crate::app::type_system::type_system::{Type, TypeDiscriminant};

use super::error::ParserError;

#[derive(Debug, Clone, PartialEq, strum_macros::Display)]
pub enum Token {
    Literal(Type),

    UnparsedLiteral(String),

    TypeDefinition(TypeDiscriminant),
    As,

    Identifier(String),
    Comment(String),

    Struct,
    Extend,
    Function,
    Return,

    Multiplication,
    Division,
    Addition,
    Subtraction,
    Modulo,
    SetValueMultiplication,
    SetValueDivision,
    SetValueAddition,
    SetValueSubtraction,
    SetValueModulo,

    And,
    Or,
    Not,

    If,

    Equals,
    Bigger,
    EqBigger,
    Smaller,
    EqSmaller,

    OpenParentheses,
    CloseParentheses,
    OpenBraces,
    CloseBraces,

    LineBreak,
    Comma,
    DoubleColon,
    Colon,
    Dot,

    SetValue,

    BitAnd,
    BitOr,
    BitLeft,
    BitRight,

    Import,
}

#[derive(Debug, Clone)]
pub enum MathematicalSymbol {
    Addition,
    Subtraction,
    Division,
    Multiplication,
    Modulo,
}

impl TryInto<MathematicalSymbol> for Token {
    type Error = ParserError;

    fn try_into(self) -> Result<MathematicalSymbol, Self::Error> {
        let expr = match self {
            Self::Addition => MathematicalSymbol::Addition,
            Self::Subtraction => MathematicalSymbol::Subtraction,
            Self::Division => MathematicalSymbol::Division,
            Self::Multiplication => MathematicalSymbol::Multiplication,
            Self::Modulo => MathematicalSymbol::Modulo,

            _ => return Err(ParserError::InternalVariableError),
        };

        Ok(expr)
    }
}

#[derive(Debug, Clone, Display)]
pub enum ParsedToken {
    NewVariable(String, TypeDiscriminant, Box<ParsedToken>),

    VariableReference(VariableReference),

    Literal(Type),

    TypeCast(Box<ParsedToken>, TypeDiscriminant),

    MathematicalExpression(Box<ParsedToken>, MathematicalSymbol, Box<ParsedToken>),

    Brackets(Vec<ParsedToken>, TypeDiscriminant),

    FunctionCall((FunctionSignature, String), IndexMap<String, ParsedToken>),

    SetValue(VariableReference, Box<ParsedToken>),

    MathematicalBlock(Box<ParsedToken>),

    ReturnValue(Box<ParsedToken>),

    // Const(TypeDiscriminants),
    If(If),

    InitializeStruct(
        IndexMap<String, TypeDiscriminant>,
        IndexMap<String, Box<ParsedToken>>,
    ),
}

#[derive(Debug, Clone, Display)]
pub enum VariableReference {
    /// Variable name, struct_type, field_name
    StructFieldReference(
        StructFieldReference,
        (String, IndexMap<String, TypeDiscriminant>),
    ),
    BasicReference(String),
}

#[derive(Debug, Clone)]
pub struct StructFieldReference {
    pub field_stack: Vec<String>,
}

impl Default for StructFieldReference {
    fn default() -> Self {
        Self::new()
    }
}

impl StructFieldReference {
    pub fn from_single_entry(field_name: String) -> Self {
        Self {
            field_stack: vec![field_name],
        }
    }

    pub fn from_stack(field_stack: Vec<String>) -> Self {
        Self { field_stack }
    }

    pub fn new() -> Self {
        Self {
            field_stack: vec![],
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct UnparsedFunctionDefinition {
    pub function_sig: FunctionSignature,
    pub inner: Vec<Token>,
}

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub function_sig: FunctionSignature,
    pub inner: Vec<ParsedToken>,
}

#[derive(Debug, Clone, Default)]
pub struct FunctionSignature {
    pub args: indexmap::IndexMap<String, TypeDiscriminant>,
    pub return_type: TypeDiscriminant,
}

impl Display for FunctionSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "Arguments: {:?}, Return type: {}",
            self.args, self.return_type
        ))
    }
}

/// All of the custom types implemented by the User are defined here
#[derive(Debug, Clone, PartialEq, Display)]
pub enum CustomType {
    Struct((String, IndexMap<String, TypeDiscriminant>)),
    Enum(IndexMap<String, TypeDiscriminant>),
    // First argument is the struct's name which the Extend extends
    // The second argument is the list of functions the stuct is being extended with
    // Extend(String, IndexMap<String, FunctionDefinition>),
}

#[derive(Debug, Clone)]
pub struct If {
    cmp: Comparison,

    inner: Vec<ParsedToken>,
}

#[derive(Debug, Clone)]
pub struct Comparison {
    rhs: Type,

    lhs: Type,

    cmp: Cmp,
}

#[derive(Debug, Clone)]
pub struct Cmp {}

/// These are used to define Imports.
/// Function symbols are manually defined to be imported.
#[derive(Debug, Clone, Default)]
pub struct Imports(HashMap<String, FunctionSignature>);

impl DerefMut for Imports {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for Imports {
    type Target = HashMap<String, FunctionSignature>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
