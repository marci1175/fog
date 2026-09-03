use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anyhow::Result;
use strum_macros::Display;

use crate::{
    codegen::{
        CustomItem, FunctionArgumentIdentifier, If, LogicalOperator, Order, StructAttributes,
        StructDefinition,
    },
    error::{Spanned, parser::ParserError, syntax::SyntaxError},
    imports::{FFIDeclType, ImportType},
    parser::{
        function::{
            CompilerInstruction, CompilerInstructionDiscriminants, FunctionArguments,
            FunctionDefinition, FunctionSignature, PathMap,
        },
        numeric_value::MathematicalSymbol,
        variable::{ControlFlowType, UniqueId},
    },
    tokenizer::{Token, TokenDiscriminants},
    ty::{OrdMap, OrdSet, Type, Value},
};

/// Helper trait for types lookingto implement a buffer-like stream.
pub trait Streamable<T>
{
    /// Peeks the nth next token from the stream.
    /// Since nth is an [`isize`] it can peek both backwards and forwards.
    fn peek(&self, nth: isize) -> Option<&T>;
    fn peek_next(&self) -> Option<&T>;
    fn len(&self) -> usize;
    fn stream_idx(&self) -> usize;

    /// Returns the next item from the stream.
    fn consume(&mut self) -> Option<&T>;

    /// This function only returns the item, if it equals the discriminant. If it does not it returns the error provided.
    fn try_consume_match<E: Clone, D>(&mut self, error: E, discriminant: &D) -> Result<&T, E>
    where
        T: PartialEq<D>;

    /// Returns true if the pattern matches the next `n` tokens in the stream.
    fn try_match_pattern<D>(&self, pattern: &[D]) -> bool
    where
        T: PartialEq<D>;

    /// The fetching should be non-inclusive.
    /// The function should consume the `nth` next tokens.
    fn consume_bulk(&mut self, nth: usize) -> Option<&[T]>;

    /// The fetching should be non-inclusive.
    /// The function should return the `nth` next tokens, but shouldnt increment the internal index.
    fn peek_bulk(&self, nth: usize) -> Option<&[T]>;

    fn decrement_cursor(&mut self, num: usize);

    /// Peeks the rest of the stream.
    fn peek_remainder(&self) -> Option<&[T]>;

    /// Returns the last consumed item of the stream.
    fn get_last_consumed(&self) -> Option<&T>;

    /// Calls the closure passed in, if that closure returns true, the stream will return the index of the item the closure returned true to.
    /// The function does not consume tokens.
    fn map_next_pos<'a, F: FnMut(&'a T) -> bool>(&'a self, check: F) -> Option<usize>
    where
        T: 'a;

    /// Create a child iterator, which has its own internal index and holds a reference for their owner's index.
    /// When incrementing the child's index it also increments the parent's index. However, the child only holds the amount of tokens it was provided with.
    fn child_iterator_bulk<'child>(&'child mut self, nth: usize) -> Option<StreamChild<'child, T>>;
}

/// Creates a child iterator until the entered [`TokenDiscriminantsBase`] matches.
pub fn child_iterator_until<'child, T: Streamable<Spanned<Token>>>(
    s: &'child mut T,
    until: &TokenDiscriminants,
    err: ParserError,
) -> Result<StreamChild<'child, Spanned<Token>>>
{
    Ok(s.child_iterator_bulk(
        s.map_next_pos(|element| element.get_inner() == until)
            .ok_or(err)?,
    )
    .ok_or(ParserError::EOF)?)
}

/// Stores the index of the cursor in the time this checkpoint was captured.
pub struct StreamCheckpoint
{
    idx: usize,
}

#[derive(Debug, Clone, Default)]
pub struct Stream<T>
{
    buffer: Vec<T>,
    idx_stack: Rc<RefCell<Vec<usize>>>,
}

impl<T> Stream<T>
{
    pub fn new(tokens: Vec<T>) -> Self
    {
        Self {
            buffer: tokens,
            idx_stack: Rc::new(RefCell::new(vec![0])),
        }
    }

    pub fn create_checkpoint(&self) -> StreamCheckpoint
    {
        StreamCheckpoint {
            idx: self.idx_stack.borrow()[0],
        }
    }

    pub fn load_checkpoint(&mut self, checkpoint: StreamCheckpoint)
    {
        self.idx_stack.borrow_mut()[0] = checkpoint.idx;
    }

    pub fn tokens_mut(&mut self) -> &mut Vec<T>
    {
        &mut self.buffer
    }

    pub const fn len(&self) -> usize
    {
        self.buffer.len()
    }

    pub const fn is_empty(&self) -> bool
    {
        self.buffer.is_empty()
    }

    fn idx(&self) -> usize
    {
        self.idx_stack.borrow()[0]
    }

    fn increment(&self, amount: usize)
    {
        self.idx_stack.borrow_mut()[0] += amount;
    }

    fn decrement(&self, amount: usize)
    {
        let mut stack = self.idx_stack.borrow_mut();
        stack[0] = stack[0].saturating_sub(amount);
    }
}

impl<T> Streamable<T> for Stream<T>
{
    fn len(&self) -> usize
    {
        self.buffer.len()
    }

    fn stream_idx(&self) -> usize
    {
        self.idx()
    }

    /// Decrements iternal cursor, if the result would be a negative the index is zeroed.
    fn decrement_cursor(&mut self, num: usize)
    {
        self.decrement(num);
    }

    fn peek(&self, nth: isize) -> Option<&T>
    {
        self.idx()
            .checked_add_signed(nth)
            .and_then(|idx| self.buffer.get(idx))
    }

    fn peek_next(&self) -> Option<&T>
    {
        self.buffer.get(self.idx())
    }

    /// This does not remove the token from the list, therefor it is O(1).
    /// The function only increments an internal index.
    fn consume(&mut self) -> Option<&T>
    {
        let query = self.buffer.get(self.idx())?;
        self.increment(1);
        Some(query)
    }

    /// This does not remove the token from the list, therefor it is O(1).
    /// The function only increments an internal index.
    /// If the tokenstream does not have any more items left, this function will return the provided error.
    fn try_consume_match<E: Clone, D>(&mut self, error: E, discriminant: &D) -> Result<&T, E>
    where
        T: PartialEq<D>,
    {
        let query = self.buffer.get(self.idx()).ok_or(error.clone())?;

        self.increment(1);

        if query != discriminant {
            return Err(error);
        }

        Ok(query)
    }

    /// Returns true if the pattern matches the next `n` tokens in the stream.
    fn try_match_pattern<D>(&self, pattern: &[D]) -> bool
    where
        T: PartialEq<D>,
    {
        pattern
            .iter()
            .enumerate()
            .all(|(idx, tkn)| self.buffer.get(self.idx() + idx).is_some_and(|x| x == tkn))
    }

    /// This does not remove the token from the list, therefor it is O(1).
    /// The function only increments an internal index.
    /// The fetching is non-inclusive.
    fn consume_bulk(&mut self, nth: usize) -> Option<&[T]>
    {
        let query = self.buffer.get(self.idx()..self.idx() + nth)?;
        self.increment(nth);
        Some(query)
    }

    fn peek_bulk(&self, nth: usize) -> Option<&[T]>
    {
        let query = self.buffer.get(self.idx()..self.idx() + nth)?;
        Some(query)
    }

    /// Decrement the cursor by `num`. If `num > self.idx` the internal index is zeroed.
    // fn decrement_cursor(&mut self, num: usize)
    // {
    //     self.idx = self.idx.saturating_sub(num);
    // }

    /// Peeks the rest of the [`TokenStream`].
    fn peek_remainder(&self) -> Option<&[T]>
    {
        self.buffer.get(self.idx()..)
    }
    /// Returns None if none were consumed or if there arent any tokens left in the buffer.
    fn get_last_consumed(&self) -> Option<&T>
    {
        let idx = self.idx().checked_sub(1)?;
        self.buffer.get(idx)
    }

    /// Calls the closure passed in, if that closure returns true, the stream will return the index of the item the closure returned true to.
    /// The function does not consume tokens.
    fn map_next_pos<'a, F: FnMut(&'a T) -> bool>(&'a self, mut check: F) -> Option<usize>
    {
        for (idx, e) in self.buffer.iter().skip(self.idx()).enumerate() {
            if (check)(e) {
                return Some(idx);
            }
        }

        None
    }

    /// Create a child iterator, which has its own internal index and holds a reference for their owner's index.
    /// When incrementing the child's index it also increments the parent's index. However, the child only holds the amount of tokens it was provided with.
    fn child_iterator_bulk<'child>(&'child mut self, nth: usize) -> Option<StreamChild<'child, T>>
    {
        let start = self.idx();
        self.buffer.get(start..start + nth).map(|buffer| {
            self.idx_stack.borrow_mut().push(0);
            let depth = self.idx_stack.borrow().len() - 1;
            StreamChild {
                buffer,
                depth,
                idx_stack: Rc::clone(&self.idx_stack),
            }
        })
    }
}

#[derive(Debug)]
pub struct StreamChild<'owner, T>
{
    buffer: &'owner [T],
    depth: usize,
    idx_stack: Rc<RefCell<Vec<usize>>>,
}

impl<'owner, T> StreamChild<'owner, T>
{
    fn local_idx(&self) -> usize
    {
        self.idx_stack.borrow()[self.depth]
    }

    fn increment_all(&self, amount: usize)
    {
        let mut stack = self.idx_stack.borrow_mut();
        for i in 0..=self.depth {
            stack[i] += amount;
        }
    }

    fn decrement_all(&self, amount: usize)
    {
        let mut stack = self.idx_stack.borrow_mut();
        for i in 0..=self.depth {
            stack[i] = stack[i].saturating_sub(amount);
        }
    }

    pub fn owner_idx(&self) -> usize
    {
        // depth - 1 is the immediate parent
        if self.depth > 0 {
            self.idx_stack.borrow()[self.depth - 1]
        }
        else {
            self.idx_stack.borrow()[0]
        }
    }
}

impl<'owner, T> Drop for StreamChild<'owner, T>
{
    fn drop(&mut self)
    {
        self.idx_stack.borrow_mut().pop();
    }
}

impl<'owner, T> Streamable<T> for StreamChild<'owner, T>
{
    fn len(&self) -> usize
    {
        self.buffer.len()
    }

    fn stream_idx(&self) -> usize
    {
        self.local_idx()
    }

    fn peek(&self, nth: isize) -> Option<&T>
    {
        self.local_idx()
            .checked_add_signed(nth)
            .and_then(|idx| self.buffer.get(idx))
    }

    fn peek_next(&self) -> Option<&T>
    {
        self.buffer.get(self.local_idx())
    }

    /// This does not remove the token from the list, therefor it is O(1).
    /// The function only increments an internal index.
    fn consume(&mut self) -> Option<&T>
    {
        let query = self.buffer.get(self.local_idx())?;
        self.increment_all(1);
        Some(query)
    }

    /// This does not remove the token from the list, therefor it is O(1).
    /// The function only increments an internal index.
    /// If the tokenstream does not have any more items left, this function will return the provided error.
    fn try_consume_match<E: Clone, D>(&mut self, error: E, discriminant: &D) -> Result<&T, E>
    where
        T: PartialEq<D>,
    {
        let query = self.buffer.get(self.local_idx()).ok_or(error.clone())?;

        self.increment_all(1);

        if query != discriminant {
            return Err(error);
        }

        Ok(query)
    }

    /// Returns true if the pattern matches the next `n` tokens in the stream.
    fn try_match_pattern<D>(&self, pattern: &[D]) -> bool
    where
        T: PartialEq<D>,
    {
        let idx = self.local_idx();
        pattern
            .iter()
            .enumerate()
            .all(|(i, tkn)| self.buffer.get(idx + i).is_some_and(|x| x == tkn))
    }

    /// This does not remove the token from the list, therefor it is O(1).
    /// The function only increments an internal index.
    /// The fetching is non-inclusive.
    fn consume_bulk(&mut self, nth: usize) -> Option<&[T]>
    {
        let idx = self.local_idx();
        let query = self.buffer.get(idx..idx + nth)?;
        self.increment_all(nth);
        Some(query)
    }

    fn peek_bulk(&self, nth: usize) -> Option<&[T]>
    {
        let idx = self.local_idx();
        self.buffer.get(idx..idx + nth)
    }

    /// Decrement the cursor by `num`. If `num > self.idx` the internal index is zeroed.
    fn decrement_cursor(&mut self, num: usize)
    {
        self.decrement_all(num);
    }

    /// Peeks the rest of the [`TokenStream`].
    fn peek_remainder(&self) -> Option<&[T]>
    {
        self.buffer.get(self.local_idx()..)
    }

    /// Returns None if none were consumed or if there arent any tokens left in the buffer.
    fn get_last_consumed(&self) -> Option<&T>
    {
        let idx = self.local_idx().checked_sub(1)?;
        self.buffer.get(idx)
    }

    /// Calls the closure passed in, if that closure returns true, the stream will return the index of the item the closure returned true to.
    /// The function does not consume tokens.
    fn map_next_pos<'a, F: FnMut(&'a T) -> bool>(&'a self, mut check: F) -> Option<usize>
    where
        T: 'a,
    {
        for (idx, e) in self.buffer.iter().skip(self.local_idx()).enumerate() {
            if (check)(e) {
                return Some(idx);
            }
        }

        None
    }

    /// Create a child iterator, which has its own internal index and holds a reference for their owner's index.
    /// When incrementing the child's index it also increments the parent's index. However, the child only holds the amount of tokens it was provided with.
    fn child_iterator_bulk<'child>(&'child mut self, nth: usize) -> Option<StreamChild<'child, T>>
    {
        let start = self.local_idx();
        self.buffer.get(start..start + nth).map(|buffer| {
            self.idx_stack.borrow_mut().push(0);
            let depth = self.idx_stack.borrow().len() - 1;
            StreamChild {
                buffer,
                depth,
                idx_stack: Rc::clone(&self.idx_stack),
            }
        })
    }
}

#[derive(Debug, Clone, Display, strum_macros::EnumTryAs, PartialEq, Eq, Hash)]
pub enum StatementVariant
{
    NewVariable
    {
        variable_name: String,
        variable_type: Type,
        variable_value: Box<Spanned<StatementVariant>>,
        variable_id: UniqueId,
        is_mutable: bool,
    },

    /// This is the token for referencing a basic variable (by name only). This is the lowest layer of referencing a variable.
    BasicReference
    {
        variable_name: String,
    },
    ArrayReference
    {
        variable_reference: Box<Spanned<StatementVariant>>,
        index: Box<Spanned<StatementVariant>>,
    },
    StructFieldReference
    {
        variable_reference: Box<Spanned<StatementVariant>>,
        field_name: String,
    },

    Value(Value),

    TypeCast(Box<Spanned<StatementVariant>>, Type),

    MathematicalExpression
    {
        lhs: Box<Spanned<StatementVariant>>,
        symbol: MathematicalSymbol,
        rhs: Box<Spanned<StatementVariant>>,
    },

    NegateValue(Box<Spanned<StatementVariant>>),

    Brackets(Vec<Spanned<StatementVariant>>, Type),

    FunctionCall
    {
        // This will get resolved later
        // signature: FunctionSignature,
        identifier: Box<Spanned<StatementVariant>>,

        arguments: OrdMap<
            // A function's arguments can be identified by its position in the function call, or if the argument is named
            FunctionArgumentIdentifier<String, usize>,
            Spanned<StatementVariant>,
        >,
    },

    /// The first ParsedToken is the parsedtoken referencing some kind of variable reference (Does not need to be a `VariableReference`), basicly anything.
    /// The second is the value we are setting this variable.
    SetValue
    {
        receiver: Box<Spanned<StatementVariant>>,
        value: Box<Spanned<StatementVariant>>,
    },

    ModifyValueArithmetic
    {
        receiver: Box<Spanned<StatementVariant>>,
        symbol: MathematicalSymbol,
        value: Box<Spanned<StatementVariant>>,
    },

    ReturnValue
    {
        value: Box<Spanned<StatementVariant>>,
    },

    Comparison
    {
        lhs: Box<Spanned<StatementVariant>>,
        ord: Order,
        rhs: Box<Spanned<StatementVariant>>,
    },

    /// Both lhs and rhs must resolve to a boolean value
    LogicalOperation
    {
        lhs: Box<Spanned<StatementVariant>>,
        op: LogicalOperator,
        rhs: Box<Spanned<StatementVariant>>,
    },

    If(If),

    CodeBlock(Vec<StatementVariant>),

    Grouping
    {
        inner_expr: Box<Spanned<StatementVariant>>,
    },

    Loop(Vec<Spanned<StatementVariant>>),

    ControlFlow(ControlFlowType),

    ArrayInitialization
    {
        values: Vec<Spanned<StatementVariant>>,
    },

    GetPointerTo(Box<Spanned<StatementVariant>>),

    DerefPointer(Box<Spanned<StatementVariant>>),
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub enum ItemVisibility
{
    /// Not available to any scopes besides the file it was created in
    #[default]
    Private, // priv
    /// Is exposed as a function to import
    Public, // pub
    /// Can only be accessed from the same library it was created in
    PublicLibrary, // publib
    /// Branches are parsed like function, and this type is supposed to indicate that the function is actually a branch.
    /// A branch does not have any visibility, it is only for debugging.
    Branch,
}

/// A [`Context`] instance represents one module/scope.
/// The instance has its own imports and external declerations (`extern import`).
/// The simplest way to explain a context is basically a source file, as one source file has one context assigned to it.
#[derive(Clone, Debug)]
pub struct Context
{
    /// This field stores all the functions created for this context (/ scope, basically in this module).
    /// `PATH` contains the full access path to the function including the name of the function.
    /// `NAME` contains the plain name of the function.
    pub functions: PathMap<Vec<String>, String, FunctionDefinition>,

    /// This field stores all the items created for this context (/ scope, basically in this module).
    /// `PATH` contains the full access path to the item including the name of the item.
    /// `NAME` contains the plain name of the item. Two different items cannot share the same name, thus the same `PATH`.
    pub items: PathMap<Vec<String>, String, CustomItem>,

    /// Imports defined in the source code. These can be either source code imports or dependency imports.
    pub imports: HashMap<String, ImportType>,

    /// FFI declerations are raw ffi function definitions, which are valid when the right object files are linked with the project. (such as libc when linking with clang)
    pub ffi_declerations: HashMap<String, FFIDeclType>,

    /// Path to the source file this context represents.
    pub path: Vec<String>,
}

impl Context
{
    pub fn new(path: Vec<String>) -> Self
    {
        Self {
            functions: PathMap::new(),
            items: PathMap::new(),
            imports: HashMap::new(),
            ffi_declerations: HashMap::new(),
            path,
        }
    }

    pub fn create_function(
        &self,
        vis: ItemVisibility,
        name: String,
        arguments: FunctionArguments,
        return_type: Type,
        compiler_instructions: OrdSet<CompilerInstruction>,
        body: Vec<Spanned<StatementVariant>>,
        enabling_features: OrdSet<String>,
    ) -> FunctionDefinition
    {
        FunctionDefinition {
            signature: FunctionSignature {
                name,
                args: arguments,
                return_type,
            },
            module_path: self.path.clone(),
            visibility: vis,
            compiler_instructions,
            enabling_features,
            body,
        }
    }

    pub fn create_struct(
        &self,
        vis: ItemVisibility,
        name: String,
        fields: OrdMap<String, Type>,
        generics: OrdMap<String, OrdSet<String>>,
        attributes: StructAttributes,
    ) -> StructDefinition
    {
        StructDefinition {
            visibility: vis,
            name,
            fields,
            generics,
            attributes,
        }
    }
}

pub fn find_closing_paren<S: Streamable<Spanned<Token>>>(tokens: &S) -> Option<usize>
{
    tokens.peek_remainder().and_then(|tkns| {
        let mut parentheses_counter: usize = 1;

        for (idx, token) in tkns.iter().enumerate() {
            if token.get_inner() == &Token::OpenParentheses {
                parentheses_counter += 1;
            }
            else if token.get_inner() == &Token::CloseParentheses {
                parentheses_counter -= 1;
            }

            if parentheses_counter == 0 {
                return Some(idx);
            }
        }

        None
    })
}

/// This function will return the idx of the earliest occurence of a `|` in the provided slice.
pub fn find_next_bitor(bitor_start_slice: &[Token]) -> Result<usize>
{
    let iter = bitor_start_slice.iter().enumerate();

    for (idx, token) in iter {
        match token {
            Token::BitOr => return Ok(idx),
            _ => continue,
        }
    }

    Err(ParserError::SyntaxError(SyntaxError::LeftOpenParentheses).into())
}

pub fn find_closing_braces<S: Streamable<Spanned<Token>>>(tokens: &S) -> Option<usize>
{
    tokens.peek_remainder().and_then(|tkns| {
        let mut braces_counter: usize = 1;

        for (idx, token) in tkns.iter().enumerate() {
            if token.get_inner() == &Token::OpenBraces {
                braces_counter += 1;
            }
            else if token.get_inner() == &Token::CloseBraces {
                braces_counter -= 1;
            }

            if braces_counter == 0 {
                return Some(idx);
            }
        }

        None
    })
}

pub fn find_closing_comma(slice: &[Token]) -> Result<usize>
{
    let mut paren_level = 0;

    for (idx, item) in slice.iter().enumerate() {
        if *item == Token::OpenParentheses {
            paren_level += 1;
        }
        else if *item == Token::CloseParentheses {
            paren_level -= 1;
        }

        if *item == Token::Comma && paren_level == 0 || slice.len() - 1 == idx {
            return Ok(idx);
        }
    }

    Err(ParserError::InvalidFunctionCallArguments.into())
}

pub fn find_next_comma(slice: &[Token]) -> Result<usize>
{
    for (idx, item) in slice.iter().enumerate() {
        if *item == Token::Comma {
            return Ok(idx);
        }
    }

    Err(ParserError::SyntaxError(SyntaxError::CommaNotFound).into())
}

pub fn parse_compiler_instruction(
    instr_buf: &mut OrdSet<CompilerInstruction>,
    tokens: &mut Stream<Spanned<Token>>,
) -> anyhow::Result<()>
{
    if let Some(tkn) = tokens.consume() {
        match tkn.get_inner() {
            Token::CompilerInstruction(instr) => {
                // If this is a feature that means the next token should be a string referencing the feature name.
                if instr == &CompilerInstructionDiscriminants::Feature {
                    // Its safe to unwrap since we are already checking inside the try consume
                    let feature_name = tokens
                        .try_consume_match(
                            ParserError::InvalidFunctionFeature,
                            &TokenDiscriminants::Literal,
                        )?
                        .try_as_literal_ref()
                        .and_then(|val| val.try_as_string_ref())
                        .ok_or(ParserError::InvalidFunctionFeature)?;

                    instr_buf.insert(CompilerInstruction::Feature(feature_name.clone()));
                }
                // If its not a feature we can just store the instruction as is.
                else {
                    instr_buf.insert((*instr).into());
                }
            },
            _ => {
                return Err(ParserError::SyntaxError(
                    SyntaxError::CompilerInstructionRequiredAfterSymbol,
                )
                .into());
            },
        }
    }
    else {
        return Err(
            ParserError::SyntaxError(SyntaxError::CompilerInstructionRequiredAfterSymbol).into(),
        );
    }

    Ok(())
}
