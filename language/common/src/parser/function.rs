use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    hash::Hash,
    ops::Add,
    rc::Rc,
};

use bimap::BiMap;

use crate::{
    anyhow::{self, Result},
    codegen::{CustomItem, FunctionArgumentIdentifier},
    error::{SpanInfo, parser::ParserError, syntax::SyntaxError},
    indexmap::IndexMap,
    parser::{
        common::{
            ItemVisibility, ParsedToken, ParsedTokenInstance, find_closing_comma, find_closing_paren, find_next_bitor
        },
        value::parse_value,
        variable::{UniqueId, VARIABLE_ID_SOURCE, VariableReference},
    },
    tokenizer::Token,
    ty::{OrdMap, OrdSet, Type, ty_from_token},
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
    pub inner: Vec<ParsedTokenInstance>,
    /// This is used to offset the index when fetching [`DebugInformation`] about [`ParsedToken`]s inside the function.
    pub token_offset: usize,
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
    pub compiler_hints: OrdSet<CompilerHint>,
    pub enabling_features: OrdSet<String>,
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
    /// Even though [`UniqueId`]s are truly unique, we still dont want to use them (for now) as a key because strings are quniue in this context.
    pub arguments: OrdMap<String, (Type, UniqueId)>,
    pub ellipsis_present: bool,
    /// The map consists of the generic types and their traits.
    /// ie: { "T": {"trait1", "trait2"} }
    pub generics: OrdMap<String, OrdSet<Vec<String>>>,
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

#[derive(Debug, Clone, PartialEq, strum_macros::Display, Eq, Hash)]
pub enum CompilerHint
{
    /// See llvm function attributes
    Cold,
    NoFree,
    Inline,
    NoUnWind,

    /// Feature flag to only enable compilation of the function if a certain function is enabled
    Feature,
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
    pub fn insert(&mut self, key: PATH, value: ITEM, name: Rc<NAME>) -> Option<(ID, ITEM)>
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

/// The slice should startwith the first token from inside the Parentheses.
/// This function quits at the ")".
pub fn parse_function_call_args(
    tokens: &[Token],
    function_tokens_offset: usize,
    mut origin_token_idx: usize,
    debug_infos: &[SpanInfo],
    variable_scope: &mut IndexMap<String, (Type, UniqueId)>,
    mut this_function_args: FunctionArguments,
    function_signatures: Rc<PathMap<Vec<String>, String, UnparsedFunctionDefinition>>,
    imported_functions: Rc<HashMap<String, FunctionSignature>>,
    custom_items: Rc<IndexMap<String, CustomItem>>,
    receiver: Option<(&VariableReference, Type, usize)>,
    module_path: Vec<String>,
) -> anyhow::Result<(
    OrdMap<FunctionArgumentIdentifier<String, usize>, (ParsedTokenInstance, (Type, UniqueId))>,
    usize,
)>
{
    let mut tokens_idx = 0;

    let args_list_len = tokens[tokens_idx..].len() + tokens_idx;

    // Arguments which will passed in to the function
    let mut arguments: OrdMap<
        FunctionArgumentIdentifier<String, usize>,
        (
            // The parsed token of the argument
            ParsedTokenInstance,
            (
                // Parsed argument value type
                Type,
                // Unique ID of the type itself
                UniqueId,
            ),
        ),
    > = OrdMap::new();

    if this_function_args.receiver_referenced {
        // Check if the receiver is a Some
        // It must be since there is a receiver referenced in the args `this`
        let (receiver, recv_type, recv_id) =
            receiver.ok_or(ParserError::VariableNotFound(String::from("this")))?;

        // Manually insert a reference of the original struct into the the function call
        arguments.insert(
            FunctionArgumentIdentifier::Identifier(String::from("this")),
            (
                ParsedTokenInstance {
                    inner: ParsedToken::VariableReference(receiver.clone()),
                    debug_information: SpanInfo::default(),
                },
                (recv_type, recv_id),
            ),
        );
    }

    // If there are no arguments just return everything as is
    if tokens.is_empty() {
        if !this_function_args.arguments.is_empty() {
            return Err(ParserError::InvalidFunctionArgumentCount.into());
        }

        return Ok((arguments, tokens_idx));
    }

    while tokens_idx < tokens.len() {
        let current_token = tokens[tokens_idx].clone();

        if let Token::Identifier(arg_name) = current_token.clone() {
            if let Some(Token::SetValue) = tokens.get(tokens_idx + 1) {
                let (argument_type, argument_variable_id) = this_function_args
                    .arguments
                    .get(&arg_name)
                    .ok_or(ParserError::ArgumentError(arg_name.clone()))?;

                tokens_idx += 2;

                let closing_idx = find_closing_comma(&tokens[tokens_idx..])? + tokens_idx;

                let (parsed_argument, jump_idx, arg_ty) = parse_value(
                    &tokens[tokens_idx..closing_idx],
                    function_tokens_offset,
                    debug_infos,
                    origin_token_idx + tokens_idx,
                    function_signatures.clone(),
                    variable_scope,
                    Some(argument_type.clone()),
                    imported_functions.clone(),
                    custom_items.clone(),
                    module_path.clone(),
                )?;

                tokens_idx += jump_idx;

                let argmuent_id = *argument_variable_id;

                // Remove tha argument from the argument list so we can parse unnamed arguments easier
                this_function_args.arguments.shift_remove(&arg_name);

                arguments.insert(
                    FunctionArgumentIdentifier::Identifier(arg_name.clone()),
                    (parsed_argument, (arg_ty, argmuent_id)),
                );

                continue;
            }
        }
        else if Token::CloseParentheses == current_token {
            break;
        }
        else if Token::Comma == current_token {
            tokens_idx += 1;

            continue;
        }

        let mut token_buf = Vec::new();
        let mut bracket_counter: i32 = 0;

        // Update the value of the origin_token_idx
        origin_token_idx += tokens_idx;

        // We should start by finding the comma and parsing the tokens in between the current idx and the comma
        while tokens_idx < args_list_len {
            let token = &tokens[tokens_idx];

            if *token == Token::OpenParentheses {
                bracket_counter += 1;
            }
            else if *token == Token::CloseParentheses {
                bracket_counter -= 1;
            }

            // If a comma is found parse the tokens from the slice
            if (*token == Token::Comma && bracket_counter == 0) || tokens_idx == args_list_len - 1 {
                if tokens_idx == args_list_len - 1 {
                    token_buf.push(token.clone());
                }

                let fn_argument = this_function_args.arguments.first_entry();

                if let Some(fn_argument) = fn_argument {
                    let (arg_ty, arg_id) = fn_argument.get();
                    let (parsed_argument, _jump_idx, arg_ty) = parse_value(
                        &token_buf,
                        function_tokens_offset,
                        debug_infos,
                        tokens_idx,
                        function_signatures.clone(),
                        variable_scope,
                        Some(arg_ty.clone()),
                        imported_functions.clone(),
                        custom_items.clone(),
                        module_path.clone(),
                    )?;

                    tokens_idx += 1;

                    token_buf.clear();

                    arguments.insert(
                        FunctionArgumentIdentifier::Identifier(fn_argument.key().clone()),
                        (parsed_argument, (arg_ty, *arg_id)),
                    );

                    // Remove the argument from the argument list
                    fn_argument.shift_remove();
                }
                // If an argument is passed into a function which takes a variable amount of arguments, it wont be found in the fn argument list
                // We can allocate a new variable id to the argument passed in this way
                else {
                    let (parsed_argument, _jump_idx, arg_ty) = parse_value(
                        &token_buf,
                        function_tokens_offset + tokens_idx,
                        debug_infos,
                        origin_token_idx,
                        function_signatures.clone(),
                        variable_scope,
                        None,
                        imported_functions.clone(),
                        custom_items.clone(),
                        module_path.clone(),
                    )?;

                    tokens_idx += 1;

                    token_buf.clear();

                    let nth_argument = arguments.len();

                    arguments.insert(
                        FunctionArgumentIdentifier::Index(nth_argument),
                        (
                            parsed_argument,
                            (arg_ty, VARIABLE_ID_SOURCE.get_unique_id()),
                        ),
                    );
                }

                continue;
            }
            else {
                token_buf.push(token.clone());
            }

            tokens_idx += 1;
        }
    }

    if !this_function_args.arguments.is_empty() {
        return Err(ParserError::InvalidFunctionArgumentCount.into());
    }

    Ok((arguments, tokens_idx))
}

pub fn parse_signature_args(
    tokens: &[Token],
    custom_types: &IndexMap<String, CustomItem>,
    is_struct_implementation: bool,
    function_generics: &OrdMap<String, OrdSet<Vec<String>>>,
) -> Result<FunctionArguments>
{
    Ok(todo!())
}

pub fn parse_signature_argument_tokens(
    tokens: &[Token],
    custom_types: &IndexMap<String, CustomItem>,
    is_struct_implementation: bool,
    function_generics: OrdMap<String, OrdSet<Vec<String>>>,
) -> Result<(usize, FunctionArguments)>
{
    let bracket_closing_idx =
        find_closing_paren(tokens, 0).map_err(|_| ParserError::InvalidSignatureDefinition)?;

    let mut args = FunctionArguments::new();

    if bracket_closing_idx != 0 {
        args = parse_signature_args(
            &tokens[..bracket_closing_idx],
            custom_types,
            is_struct_implementation,
            &function_generics,
        )?;
    }

    args.generics = function_generics;

    Ok((bracket_closing_idx, args))
}

/// Parses the function generics definitions and modifies the provided [`OrdMap`].
/// The first token of the provided slice should be the first token after the opening `|`.
/// Only traits can be implmeneted / required for a generic for now, all of the trait names are looked up to verify them.
/// The function returns the amount of indexes we have incremented
pub fn parse_fn_generics(
    // Parsed tokens
    tokens: &[Token],
    // Curerently available custom types / items
    custom_types: &IndexMap<String, CustomItem>,
    // The list of function generics
    function_generics: &mut OrdMap<String, OrdSet<Vec<String>>>,
) -> anyhow::Result<usize>
{
    let function_g_closing_idx = find_next_bitor(tokens)
        .map_err(|_| ParserError::SyntaxError(crate::error::syntax::SyntaxError::LeftOpenBitOr))?;

    let traits_slice = &tokens[..function_g_closing_idx];

    let mut idx = 0;

    'generics_loop: while idx < traits_slice.len() {
        if let Some(Token::Identifier(generic_name)) = traits_slice.get(idx) {
            // Insert a new field into the function's generics
            let insertion_result = function_generics.insert(generic_name.clone(), OrdSet::new());

            // If there has already been a generic with this name inserted, return an error
            if insertion_result.is_some() {
                return Err(ParserError::DuplicateGenerics(generic_name.clone()).into());
            }

            // Return a mutable reference to the newly inserted generic's trait impls
            let trait_list = function_generics.get_mut(generic_name).unwrap();

            // Match syntax
            if let Some(Token::LeftArrow) = traits_slice.get(idx + 1) {
                // Move index to the beginning of the Traits list
                idx += 2;

                // In this loop we increment the index until the comma, which closes the traits list
                'traits_loop: while idx < traits_slice.len() {
                    if let Some(Token::Identifier(trait_name)) = traits_slice.get(idx) {
                        idx += 1;

                        // Check if its a valid trait
                        match custom_types
                            .get(trait_name)
                            .ok_or(ParserError::CustomItemNotFound(trait_name.clone()))?
                        {
                            CustomItem::Enum(_) | CustomItem::Struct(_) => {
                                return Err(ParserError::CustomItemUnavailableForGenerics(
                                    trait_name.clone(),
                                )
                                .into());
                            },
                            // We just have to check if its a trait or not
                            CustomItem::Trait { access_path, .. } => {
                                // Store trait name, into the mutable reference
                                // Check if the trait is already required
                                if !trait_list.insert(access_path.clone()) {
                                    return Err(ParserError::TraitAlreadyRequiredForGeneric(
                                        generic_name.clone(),
                                        trait_name.clone(),
                                    )
                                    .into());
                                }

                                // Check syntax
                                match traits_slice.get(idx) {
                                    // If there is an addition token that means that there are more traits for this generic to implement
                                    Some(&Token::Addition) => {
                                        // Consume plus sign
                                        idx += 1;

                                        // Check if there are more tokens to parse after the `+`, if not we should raise an error
                                        if idx >= traits_slice.len() {
                                            return Err(ParserError::SyntaxError(
                                                SyntaxError::InvalidFunctionGenericsDefinition(
                                                    Token::Addition,
                                                ),
                                            )
                                            .into());
                                        }

                                        continue 'traits_loop;
                                    },
                                    // If there was a comma, we should stop parsing the traits for this generic, and parse the next generic
                                    Some(&Token::Comma) => {
                                        // Consume comma
                                        idx += 1;

                                        continue 'generics_loop;
                                    },
                                    // If there is a different token that means that the syntax doesnt match
                                    Some(tkn) => {
                                        return Err(
                                            SyntaxError::InvalidFunctionGenericsDefinition(
                                                tkn.clone(),
                                            )
                                            .into(),
                                        );
                                    },
                                    _ => break 'traits_loop,
                                }
                            },
                        }
                    }

                    // Check if we have mentioned atleast one trait to implement for the last generic
                    // If not we should raise an error
                    if trait_list.is_empty() {
                        return Err(ParserError::GenericMustHaveAtleastOneTrait(
                            generic_name.clone(),
                        )
                        .into());
                    }

                    // Check syntax validity
                    match traits_slice.get(idx) {
                        Some(tkn) => {
                            return Err(SyntaxError::InvalidFunctionGenericsDefinition(
                                tkn.clone(),
                            )
                            .into());
                        },
                        _ => break 'traits_loop,
                    }
                }
            }

            idx += 1;
        }

        match traits_slice.get(idx) {
            Some(_) => {
                return Err(ParserError::SyntaxError(
                    SyntaxError::MissingCommaAtGenericsDefinition,
                )
                .into());
            },
            _ => break,
        }
    }

    Ok(idx)
}
