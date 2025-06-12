use std::{
    collections::HashMap,
    fmt::Display,
    ops::{Deref, DerefMut},
};

use indexmap::IndexMap;
use inkwell::{FloatPredicate, IntPredicate};
use strum_macros::Display;

use crate::app::type_system::type_system::{Type, TypeDiscriminant};

use super::error::{ParserError, SyntaxError};

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
    Else,

    Equal,
    NotEqual,
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

    Loop,
    For,
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

#[derive(Debug, Clone, Display, strum_macros::EnumTryAs)]
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

    Comparison(Box<ParsedToken>, Order, Box<ParsedToken>, TypeDiscriminant),

    If(If),

    InitializeStruct(
        IndexMap<String, TypeDiscriminant>,
        IndexMap<String, Box<ParsedToken>>,
    ),

    CodeBlock(Vec<ParsedToken>),

    Loop(Vec<ParsedToken>),
}

#[derive(Debug, Clone, Display)]
pub enum VariableReference {
    /// Variable name, (struct_name, struct_type)
    /// The first item of the StructFieldReference is used to look up the name of the variable which stores the Struct.
    StructFieldReference(
        StructFieldReference,
        (String, IndexMap<String, TypeDiscriminant>),
    ),
    BasicReference(String),
}

/// The first item of the StructFieldReference is used to look up the name of the variable which stores the Struct.
/// The functions which take the iterator of the `field_stack` field should not be passed the first item of the iterator, since the first item is used to look up the name of the variable which stores the struct.
#[derive(Debug, Clone)]
pub struct StructFieldReference {
    /// The name of the fields which get referenced
    pub field_stack: Vec<String>,
}

impl Default for StructFieldReference {
    fn default() -> Self {
        Self::new()
    }
}

impl StructFieldReference {
    /// Creates an instnace from a single entry
    pub fn from_single_entry(field_name: String) -> Self {
        Self {
            field_stack: vec![field_name],
        }
    }

    /// Initializes an instance from a list of field entries
    pub fn from_stack(field_stack: Vec<String>) -> Self {
        Self { field_stack }
    }

    /// Creates an instnace from an empty list
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

#[derive(Debug, Clone)]
pub struct If {
    pub condition: Box<ParsedToken>,

    pub complete_body: Vec<ParsedToken>,
    pub incomplete_body: Vec<ParsedToken>,
}

#[derive(Debug, Clone, Display)]
pub enum Order {
    Equal,
    NotEqual,
    Bigger,
    EqBigger,
    Smaller,
    EqSmaller,
}

impl Order {
    pub fn from_token(token: &Token) -> anyhow::Result<Self> {
        match token {
            Token::Equal => Ok(Self::Equal),
            Token::NotEqual => Ok(Self::Equal),
            Token::Bigger => Ok(Self::Bigger),
            Token::EqBigger => Ok(Self::EqBigger),
            Token::Smaller => Ok(Self::Smaller),
            Token::EqSmaller => Ok(Self::EqSmaller),

            _ => Err(
                ParserError::SyntaxError(SyntaxError::InvalidTokenComparisonUsage(token.clone()))
                    .into(),
            ),
        }
    }
    pub fn into_int_predicate(&self, signed: bool) -> IntPredicate {
        if signed {
            match self {
                Order::Equal => IntPredicate::EQ,
                Order::NotEqual => IntPredicate::NE,
                Order::Bigger => IntPredicate::SGT,
                Order::EqBigger => IntPredicate::SGE,
                Order::Smaller => IntPredicate::SLT,
                Order::EqSmaller => IntPredicate::SLE,
            }
        } else {
            match self {
                Order::Equal => IntPredicate::EQ,
                Order::NotEqual => IntPredicate::NE,
                Order::Bigger => IntPredicate::UGT,
                Order::EqBigger => IntPredicate::UGE,
                Order::Smaller => IntPredicate::ULT,
                Order::EqSmaller => IntPredicate::ULE,
            }
        }
    }

    pub fn into_float_predicate(&self) -> FloatPredicate {
        match self {
            Order::Equal => FloatPredicate::OEQ,
            Order::NotEqual => FloatPredicate::ONE,
            Order::Bigger => FloatPredicate::OGT,
            Order::EqBigger => FloatPredicate::OGE,
            Order::Smaller => FloatPredicate::OLT,
            Order::EqSmaller => FloatPredicate::OLE,
        }
    }
}
