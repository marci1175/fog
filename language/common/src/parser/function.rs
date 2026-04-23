use std::{collections::HashMap, fmt::Display, hash::Hash, rc::Rc};

use bimap::BiMap;
use strum::EnumDiscriminants;

use crate::{
    anyhow::{self},
    error::{Spanned, parser::ParserError, syntax::SyntaxError},
    indexmap::IndexMap,
    parser::{
        common::{
            Context, ItemVisibility, ParsedToken, Streamable, TokenStream, find_closing_braces,
        },
        ty::parse_type,
        variable::{UniqueId, VARIABLE_ID_SOURCE},
    },
    tokenizer::{Token, TokenDiscriminants},
    ty::{OrdMap, OrdSet, Type},
};

#[derive(Clone, Debug, Default, PartialEq, Hash)]
pub struct UnparsedFunctionDefinition
{
    pub signature: FunctionSignature,
    pub inner: Vec<Token>,

    /// This is used to offset the index when fetching [`DebugInformation`] about [`ParsedToken`]s inside the function.
    pub token_offset: usize,
}

#[derive(Debug, Clone, PartialEq, Hash, Default, Eq)]
pub struct FunctionDefinition
{
    pub signature: FunctionSignature,
    pub body: Vec<Spanned<ParsedToken>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct FunctionSignature
{
    pub name: String,
    pub args: FunctionArguments,
    pub return_type: Type,
    /// Module path does NOT contain function name.
    pub module_path: Vec<String>,
    pub visibility: ItemVisibility,
    pub compiler_instructions: OrdSet<CompilerInstruction>,
    // pub enabling_features: OrdSet<String>,
}

impl Display for FunctionSignature
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_str(&format!("[Function Signature]:\n{:#?}", self))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct FunctionArguments
{
    /// Even though [`UniqueId`]s are truly unique, we still dont want to use them (for now) as a key because strings are unique in this context.
    pub arguments: OrdMap<String, (Type, UniqueId)>,
    pub ellipsis_present: bool,
    /// The map consists of the generic types and their traits.
    /// ie: { "T": {"trait1", "trait2"} }
    pub generics: OrdMap<String, OrdSet<String>>,
    /// This is true if the function references the struct its implemented for ie. using the this keyword.
    /// Obviously this shouldnt be true for an ordinary function since the `this` keyword cannot be used there.
    pub receiver_referenced: bool,
}

impl FunctionArguments
{
    /// We need to implement a Custom eq check on [`FunctionArguments`]s because the use of the [`UniqueId`] they both contain.
    /// If the argument's order and name match this returns `true` otherwise `false`.
    pub fn check_arg_eq(&self, rhs: &Self) -> bool
    {
        self.arguments
            .iter()
            .map(|(name, (ty, _))| (name, ty))
            .collect::<Vec<_>>()
            == rhs
                .arguments
                .iter()
                .map(|(name, (ty, _))| (name, ty))
                .collect::<Vec<_>>()
    }
}

impl FunctionArguments
{
    pub fn new() -> Self
    {
        Self {
            arguments: OrdMap::new(),
            generics: OrdMap::new(),
            ellipsis_present: false,
            receiver_referenced: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, strum_macros::Display, Eq, Hash, EnumDiscriminants)]
#[strum_discriminants(derive(Hash))]
pub enum CompilerInstruction
{
    /// See llvm function attributes
    Cold,
    NoFree,
    Inline,
    NoUnWind,

    /// Feature flag to only enable compilation of the function if a certain function is enabled
    Feature(String),
}

impl From<CompilerInstructionDiscriminants> for CompilerInstruction
{
    fn from(val: CompilerInstructionDiscriminants) -> Self
    {
        match val {
            CompilerInstructionDiscriminants::Cold => CompilerInstruction::Cold,
            CompilerInstructionDiscriminants::NoFree => CompilerInstruction::NoFree,
            CompilerInstructionDiscriminants::Inline => CompilerInstruction::Inline,
            CompilerInstructionDiscriminants::NoUnWind => CompilerInstruction::NoUnWind,
            CompilerInstructionDiscriminants::Feature => {
                CompilerInstruction::Feature(String::new())
            },
        }
    }
}

/// Allows us to create associations based on values.
/// This type stores an internal map, and gives a unique id to every unique value.
#[derive(Debug, Default, Clone)]
pub struct Interner<VALUE: Eq + Hash>
{
    interner: BiMap<VALUE, ID>,
    _internal_counter: usize,
}

impl<VALUE: Eq + Hash> Interner<VALUE>
{
    pub fn new() -> Self
    {
        Self {
            interner: BiMap::new(),
            _internal_counter: 0,
        }
    }

    pub fn lookup_name(&self, value: &VALUE) -> Option<&ID>
    {
        self.interner.get_by_left(value)
    }

    pub fn lookup_id(&self, id: &ID) -> Option<&VALUE>
    {
        self.interner.get_by_right(id)
    }

    pub fn insert_or_get_association(&mut self, value: VALUE) -> ID
    {
        if let Some(right) = self.interner.get_by_left(&value) {
            *right
        }
        else {
            self._internal_counter += 1;

            let curr_id = self._internal_counter;

            self.interner.insert(value, curr_id);

            curr_id
        }
    }

    pub fn remove_association_by_value(&mut self, value: &VALUE) -> Option<(VALUE, usize)>
    {
        self.interner.remove_by_left(value)
    }

    pub fn remove_association_by_id(&mut self, id: &ID) -> Option<(VALUE, usize)>
    {
        self.interner.remove_by_right(id)
    }
}

type ID = usize;

/// This is a custom type which allows two important things. Handling items and their respective scopes.
/// 1. It can look up an item based on its <PATH>.
/// 2. It allows us to check whether a items's name is already present in the map.
#[derive(Debug, Default, Clone)]
pub struct PathMap<PATH: Eq + Hash, NAME: Eq + Hash, ITEM>
{
    /// The function that are contained in this map.
    /// The `PATH` must be unqiue to every function.
    /// A <PATH>'s last item is the function name.
    items: IndexMap<PATH, (ID, ITEM)>,
    /// The namespace map of the functions. This allows us to see how many functions are there in the namespace with the same name.
    namespace_members: HashMap<ID, usize>,

    _interner: Interner<Rc<NAME>>,
}

/// Allows us to specify the method we want to remove a key from a map.
pub enum RemoveType
{
    /// See [`indexmap::IndexMap::swap_remove`] for more documentation.
    Swap,
    /// See [`indexmap::IndexMap::shift_remove`] for more documentation.
    Shift,
}

impl<PATH: Eq + Hash, NAME: Hash + Eq, ITEM> PathMap<PATH, NAME, ITEM>
{
    pub fn new() -> Self
    {
        Self {
            items: IndexMap::new(),
            namespace_members: HashMap::new(),
            _interner: Interner::new(),
        }
    }

    /// If a key is inserted with this method, it first checks if that path is already present in the map.
    /// If it is present it will not overwrite the map's field, instead it will return the passed in function.
    /// The function also increment the function's counter in the namespace map.
    pub fn try_insert(
        &mut self,
        key: PATH,
        value: ITEM,
        name: Rc<NAME>,
    ) -> Option<(PATH, ITEM, Rc<NAME>)>
    {
        let id = self._interner.insert_or_get_association(name.clone());

        if self.items.contains_key(&key) {
            return Some((key, value, name));
        }

        self.increment_namespace(id);

        self.items.insert(key, (id, value));

        None
    }

    /// If a key is inserted with this function, it will automaticly overwrite the value paired to the specified key.
    /// The returned value is the overwritten value of the map.
    /// If the function returns [`None`], it means that the key we inserted was not present in the map.
    /// The function also increment the function's counter in the namespace map.
    pub fn insert(&mut self, key: PATH, name: Rc<NAME>, value: ITEM) -> Option<(ID, ITEM)>
    {
        let id = self._interner.insert_or_get_association(name.clone());

        let insert_result = self.items.insert(key, (id, value));

        if let Some((replaced_id, _)) = &insert_result {
            // If the function this was replaced by does not match the name of the old function we need to update the namespace map.
            self.decrement_namespace(replaced_id);
        }

        self.increment_namespace(id);

        insert_result
    }

    /// This internal function increment the function's count in the namespace.
    /// If the name is not present it creates one.
    fn increment_namespace(&mut self, id: ID)
    {
        // IF the namespace had this value this will return `false` otherwise `true`.
        if let Some(fn_count) = self.namespace_members.get_mut(&id) {
            *fn_count += 1;
        }
        else {
            // We ensure that we only insert if there isnt an existing namespace member with this name.
            self.namespace_members.insert(id, 1);
        }
    }

    pub fn contains_name(&self, name: Rc<NAME>) -> bool
    {
        if let Some(id) = self._interner.lookup_name(&name) {
            return self.namespace_members.contains_key(id);
        }

        false
    }

    pub fn contains_function(&self, path: &PATH) -> bool
    {
        self.items.contains_key(path)
    }

    pub fn get_item(&self, path: &PATH) -> Option<(&Rc<NAME>, &ITEM)>
    {
        self.items
            .get(path)
            .map(|(intern_id, def)| (self._interner.lookup_id(intern_id).unwrap(), def))
    }

    pub fn get_item2(&self, path: &PATH) -> Option<&(ID, ITEM)>
    {
        self.items.get(path)
    }

    pub fn get_item_by_idx(&self, idx: usize) -> Option<(&PATH, (&Rc<NAME>, &ITEM))>
    {
        self.items.get_index(idx).map(|(path, (intern_id, def))| {
            (path, (self._interner.lookup_id(intern_id).unwrap(), def))
        })
    }

    pub fn get_item_by_idx2(&self, idx: usize) -> Option<(&PATH, &(ID, ITEM))>
    {
        self.items.get_index(idx)
    }

    pub fn get_name_from_id(&self, id: &ID) -> Option<&Rc<NAME>>
    {
        self._interner.lookup_id(id)
    }

    pub fn get_item_full(&self, path: &PATH) -> Option<(&ID, &Rc<NAME>, &ITEM)>
    {
        self.items.get(path).map(|(id, def)| {
            let name = self._interner.lookup_id(id).unwrap();

            (id, name, def)
        })
    }

    pub fn remove(&mut self, key: &PATH, remove_type: RemoveType) -> Option<(ID, ITEM)>
    {
        // Remove the function definition on the specified path
        if let Some((id, def)) = {
            // Remove the function the specified way
            match remove_type {
                RemoveType::Swap => self.items.swap_remove(key),
                RemoveType::Shift => self.items.shift_remove(key),
            }
        } {
            // If the function's count is 0, remove the field from the namespace.
            self.decrement_namespace(&id);

            // Reutrn the removed function
            Some((id, def))
        }
        else {
            None
        }
    }

    /// Check how many function with this name are present in the namespace.
    /// Subtract one from the function's counter in the namespace.
    /// Removes the field from the namespace if the counter is 0.
    fn decrement_namespace(&mut self, id: &ID)
    {
        let should_remove = if let Some(fn_count) = self.namespace_members.get_mut(id) {
            // Subtract 1 from the count
            *fn_count -= 1;

            // Check if the function count is 0.
            *fn_count == 0
        }
        else {
            // I was too scared to make this an `unreachable_unchecked` lol
            unreachable!(
                "[INTERNAL ERROR] If you see this, that means ive messed up big time. Please check <FunctionMap> internal behavior."
            )
        };

        // If there are no more function's with this name in the namespace remove the field.
        if should_remove {
            self.namespace_members.remove(id);
            self._interner.remove_association_by_id(id);
        }
    }

    pub fn iter(&self) -> PathMapIterator<'_, PATH, NAME, ITEM>
    {
        PathMapIterator {
            inner_iter: self.items.iter(),
            interner: &self._interner,
        }
    }

    pub fn len(&self) -> usize
    {
        self.items.len()
    }
}

pub struct PathMapIterator<'a, PATH: Eq + Hash, NAME: Eq + Hash, ITEM>
{
    inner_iter: indexmap::map::Iter<'a, PATH, (ID, ITEM)>,
    interner: &'a Interner<Rc<NAME>>,
}

impl<'a, PATH: Eq + Hash, NAME: Eq + Hash, ITEM> Iterator for PathMapIterator<'a, PATH, NAME, ITEM>
{
    type Item = (&'a PATH, &'a Rc<NAME>, &'a ITEM);

    fn next(&mut self) -> Option<Self::Item>
    {
        self.inner_iter
            .next()
            .map(|(path, (id, def))| (path, self.interner.lookup_id(id).unwrap(), def))
    }
}

/// The function parses the entire function, but does not validate the function's body.
/// Syntax of a function:
/// ```
/// <vis> "function" <name> "(" [{<arg>: <type>}] ")" ":" <return type> "{" [{<expr>}] "}"
/// ```
pub fn parse_function(
    ctx: &Context,
    vis: &ItemVisibility,
    tokens: &mut TokenStream<Spanned<Token>>,
    compiler_instructions: OrdSet<CompilerInstruction>,
) -> anyhow::Result<FunctionDefinition>
{
    // Get the function name token
    let function_name_tkn = tokens.try_consume_match(
        ParserError::SyntaxError(SyntaxError::InvalidFunctionName),
        &TokenDiscriminants::Identifier,
    )?;

    // Parse function name, its safe to unwrap here
    let function_name = function_name_tkn
        .try_as_identifier_ref()
        .unwrap()
        .to_owned();

    // This will hold the function's arguments. This variable will get modified later.
    let mut arguments = FunctionArguments::new();

    //Parse the arguments of the function
    // If the first token is a '|' that means the function has generics defined
    // If the first token is a '(' that means that its just a normal function
    if let Some(tkn) = tokens.consume() {
        match tkn.get_inner() {
            // Parse generics before arguments
            Token::BitOr => {
                // Fetch the generics of the function
                arguments.generics = parse_fn_generics(tokens)?;

                // The next token should be a "(" due to the syntax.
                tokens.try_consume_match(
                    ParserError::InvalidSignatureDefinition,
                    &TokenDiscriminants::OpenParentheses,
                )?;

                // Parse the arguments of the function
                arguments.arguments = parse_fn_arguments(tokens)?;
            },
            // Parse arguments
            Token::OpenParentheses => arguments.arguments = parse_fn_arguments(tokens)?,
            _ => return Err(ParserError::InvalidFunctionArgumentDefinition.into()),
        }
    }

    // This should be the ":" character singaling the return type
    tokens.try_consume_match(
        ParserError::SyntaxError(SyntaxError::FunctionRequiresReturn),
        &TokenDiscriminants::Colon,
    )?;

    // Parse the return type of the function
    let return_type = parse_type(tokens)?;

    // The TokenStream should now point to `Token::OpenBraces`
    tokens.try_consume_match(
        ParserError::SyntaxError(SyntaxError::InvalidFunctionBodyStart),
        &TokenDiscriminants::OpenBraces,
    )?;

    // Fetch the function body and increment the tokenstream accordingly.
    let fn_body = parse_fn_body(tokens)?;

    // This should never return an error since we are already checking the closing brace when fetching the fn body.
    tokens.try_consume_match(
        ParserError::SyntaxError(SyntaxError::LeftOpenBraces),
        &TokenDiscriminants::CloseBraces,
    )?;

    Ok(ctx.create_function(
        vis.clone(),
        function_name,
        arguments,
        return_type,
        compiler_instructions,
        fn_body,
    ))
}

/// The function assumes the first token to be the first token in the `|`s.
/// The function does not check or evaluate anything it parses besides syntax checking.
pub fn parse_fn_generics(
    tokens: &mut TokenStream<Spanned<Token>>,
) -> anyhow::Result<OrdMap<String, OrdSet<String>>>
{
    let mut generics: OrdMap<String, OrdSet<String>> = OrdMap::new();

    /*
        Syntax definition:

        {
            <generic> ":" { { <trait> ["+"] } [","] } [","]
        }
    */
    // Lets loop through all the generics
    'main_loop: while let Some(tkn) = tokens.consume() {
        match tkn.get_inner() {
            Token::Identifier(generic_name) => {
                let generic_name = generic_name.clone();

                // Store a checkpoint of the stream so we can load it back later if we need to
                let generic_name_checkpoint = tokens.create_checkpoint();

                // The next token should be a ":" due to syntax
                tokens.try_consume_match(
                    ParserError::SyntaxError(SyntaxError::InvalidFunctionGenericsDefinition),
                    &TokenDiscriminants::Colon,
                )?;

                // Create a new entry for the current generic
                generics.insert(generic_name.clone(), OrdSet::new());
                // We can safely unwrap here because the field is present in the map and return a mutable handle
                let generic_handle = generics.get_mut(&generic_name).unwrap();

                // Loop over the traits
                'trait_loop: while let Some(tkn) = tokens.consume() {
                    if let Token::Identifier(trait_name) = tkn.get_inner() {
                        // Store the trait's name which the user entered for the generic
                        // The ordset for the generic should already be present in the map.
                        generic_handle.insert(trait_name.clone());

                        // Check the next token
                        let next = tokens.consume().ok_or(ParserError::EOF)?;

                        // Match the next token
                        match next.get_inner() {
                            Token::BitOr => break 'main_loop,
                            Token::Addition => continue 'trait_loop,
                            // If we have reached the comma that means that the current trait bound has ended.
                            Token::Comma => break 'trait_loop,
                            _ => {
                                return Err(ParserError::SyntaxError(
                                    SyntaxError::InvalidFunctionGenericsDefinition,
                                )
                                .into());
                            },
                        }
                    }
                    else {
                        return Err(ParserError::SyntaxError(
                            SyntaxError::InvalidFunctionGenericsDefinition,
                        )
                        .into());
                    }
                }

                // Ensure that the trait bound is not empty (although i dont think its possible, but code may change later)
                // If this returned an error load the checkpoint back so that the error will point at the correct generic
                if generic_handle.is_empty() {
                    // Load the position of the cursor at the generic name
                    tokens.load_checkpoint(generic_name_checkpoint);
                    return Err(ParserError::GenericMustHaveAtleastOneTrait.into());
                }
            },
            // If we encounter the closing `|` break the loop
            Token::BitOr => break,

            _ => {
                return Err(ParserError::SyntaxError(
                    SyntaxError::InvalidFunctionGenericsDefinition,
                )
                .into());
            },
        }
    }

    Ok(generics)
}

/// The function assumes the first token to be the first token in the parentheses.
/// Please note that the function does not evaluate anything it parses.
pub fn parse_fn_arguments(
    tokens: &mut TokenStream<Spanned<Token>>,
) -> anyhow::Result<OrdMap<String, (Type, UniqueId)>>
{
    /*
        Arguments are defined like so:
        "(" [{<arg_name> ":" <type>, }] ")"
        The function will be called after the first "(" therefor the function should start parsing from the first arguments name or the closing ")".
    */
    // Create the map of arguments
    let mut arguments: OrdMap<String, (Type, UniqueId)> = OrdMap::new();

    // Loop thorugh all the arguments
    'main_loop: while let Some(tkn) = tokens.consume() {
        // Get the name of the variable
        match tkn.get_inner() {
            Token::Identifier(arg_name) => {
                let arg_name = arg_name.clone();

                // The next token should be a ":" due to syntax
                tokens.try_consume_match(
                    ParserError::InvalidFunctionArgumentDefinition,
                    &TokenDiscriminants::Colon,
                )?;

                // The next token should be a concrete type or an identifier.
                if let Some(ty) = tokens.consume() {
                    // Get the function argument's type
                    let arg_ty = match ty.get_inner() {
                        Token::Identifier(ty_name) => {
                            // Store the type as unresolved, this will be resolved later at the semantic checking process
                            Type::Unresolved(ty_name.clone())
                        },
                        Token::TypeDefinition(ty) => {
                            // Turn the concrete typetoken into a type
                            (ty.clone()).try_into()?
                        },

                        // Invalid syntax, return an error
                        _ => return Err(ParserError::InvalidArgumentType.into()),
                    };

                    // Store the argument
                    let insertion_result = arguments.insert(
                        arg_name.clone(),
                        (arg_ty, VARIABLE_ID_SOURCE.get_unique_id()),
                    );

                    // Check if there are duplicate argument names
                    if insertion_result.is_some() {
                        return Err(ParserError::DuplicateArguments(arg_name.clone()).into());
                    }

                    // Check the next token
                    // If it is a "," that means that there are more arguments or the user just left it in.
                    // If it s a ")" that shows that the all the function arguments have been parsed
                    if let Some(tkn) = tokens.consume() {
                        match tkn.get_inner() {
                            Token::Comma => continue 'main_loop,
                            Token::CloseParentheses => break 'main_loop,
                            _ => return Err(ParserError::InvalidFunctionArgumentDefinition.into()),
                        }
                    }
                }

                // If we didnt break continue or return an error that means that there werent any more tokens left in the stream therefor we can do an EOF.
                return Err(ParserError::EOF.into());
            },
            Token::CloseParentheses => break 'main_loop,
            _ => return Err(ParserError::InvalidFunctionArgumentDefinition.into()),
        }
    }

    Ok(arguments)
}

/// This function will parse the tokens in the body of the function, but it will not check the validness of the tokens themselves.
///
/// The function parses the tokens but does not evaluate them.
pub fn parse_fn_body(
    tokens: &mut TokenStream<Spanned<Token>>,
) -> anyhow::Result<Vec<Spanned<ParsedToken>>>
{
    // Get the index of the closing brace token
    let body_closing_tkn = find_closing_braces(&*tokens)
        .ok_or(ParserError::SyntaxError(SyntaxError::LeftOpenBraces))?;

    // It is safe to unwrap here, since we have already checked if the closing braces would be in the TokenStream
    let mut _fn_body = tokens.child_iterator_bulk(body_closing_tkn).unwrap();

    // Store the parsed tokens somewhere
    let parsed_tokens = Vec::new();

    // parse_tokens(&mut fn_body, &mut parsed_tokens)?;

    Ok(parsed_tokens)
}
