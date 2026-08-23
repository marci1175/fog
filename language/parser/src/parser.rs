use std::{hint::cold_path, path::PathBuf};

use common::{
    anyhow::Result,
    combine_path,
    compiler::ProjectConfig,
    error::{Spanned, parser::ParserError, syntax::SyntaxError},
    parser::{
        common::{Context, Stream, Streamable, child_iterator_until, parse_compiler_instruction},
        function::{CompilerInstruction, parse_function},
        import::parse_import_statement,
        ty::{parse_enum, parse_struct},
    },
    tokenizer::{Token, TokenDiscriminants},
    ty::OrdSet,
};

#[derive(Debug, Clone)]
pub struct Settings
{
    // Project settings
    pub config: ProjectConfig,
    pub enabled_features: OrdSet<String>,
    /// The path to the root of this project.
    /// This is important when we are parsing libraries.
    pub module_path: Vec<String>,
    pub root_path: PathBuf,
}

impl Settings
{
    /*
        TODO: recode importing stuff

        First of all, remove the extra logic from here relating to dependencies
        Also, when parsing the deps make a dependency tree, with the value of `HashMap<&[&str], Dependency>`
        Implement parsing for `foo::bar::x()` type expressions, this will allow us to use functions with the same name on different paths

        Modify the type resolving function to look up dependency items
        Create the `namespace` keyword rework how the dependency paths work
        ```
        namespace backend {
            struct request {};
        }

        use backend::request;
        ```
    */

    /*
        Internal notes:
        imma change some of the syntax for example imma make it so that i can do `import "blabla.f", so that i can bring path into scope.`
    */

    /// Creates a [`Context`] instance by parsing the passed in tokens with the settings provided.
    pub fn parse(&self, tokens: &mut Stream<Spanned<Token>>) -> Result<Context>
    {
        // The first step should be parsing the top level items, such as structs, functions, enums.
        // We will store all the items present, and parse the inner contents of the function later.
        // By doing this, the compiler wont be single pass anymore and the sequence of function declarations wont be important.
        // Im gonna first parse the entire main file and then work out/parse all the other files which were linked.
        let mut ctx = Context::new(self.module_path.clone());

        // Collect the compiler instructions in a list and we can move the instructions to the next item we are parsing.
        let mut item_compiler_instruction: OrdSet<CompilerInstruction> = OrdSet::new();

        // Parse the actual tokens
        while let Some(tkn) = tokens.consume().cloned() {
            match tkn.get_inner() {
                Token::CompilerHintSymbol => {
                    parse_compiler_instruction(&mut item_compiler_instruction, tokens)?;
                },
                Token::ItemVisibility(vis) => {
                    // Type of the item
                    let item_tkn = tokens.try_consume_match(
                        ParserError::ItemTypeExpected,
                        &TokenDiscriminants::TypeDefinition,
                    )?;

                    // This should always follow this path due to the check above.
                    // The only reason the else statement is not `unreachable_unchecked` because im scared of breaking it in future modifications.
                    // Regardless it does not result in any meaningful speedup.
                    if let Token::TypeDefinition(item_type) = item_tkn.get_inner() {
                        // Match the type of the item
                        match item_type {
                            common::tokenizer::TypeToken::Enum => {
                                parse_enum(
                                    &mut ctx,
                                    vis,
                                    tokens,
                                    std::mem::take(&mut item_compiler_instruction),
                                )
                            },
                            common::tokenizer::TypeToken::Struct => {
                                let struct_def = parse_struct(
                                    &mut ctx,
                                    vis,
                                    tokens,
                                    std::mem::take(&mut item_compiler_instruction),
                                )?;

                                ctx.items.insert(
                                    combine_path(ctx.path.clone(), struct_def.name.clone()),
                                    struct_def.name.clone().into(),
                                    common::codegen::CustomItem::Struct(struct_def),
                                );
                            },
                            common::tokenizer::TypeToken::Function => {
                                let function = parse_function(
                                    &ctx,
                                    vis,
                                    tokens,
                                    std::mem::take(&mut item_compiler_instruction),
                                )?;

                                ctx.functions.insert(
                                    combine_path(
                                        function.module_path.clone(),
                                        function.signature.name.clone(),
                                    ),
                                    function.signature.name.clone().into(),
                                    function,
                                );
                            },
                            // We can still return the original error since the item defining tokens are also stored as TypeDefinitions. (They are defined in the TypeToken enum)
                            _ => return Err(ParserError::ItemTypeExpected.into()),
                        }
                    }
                    else {
                        // We can hint the compiler that this path is unlikely to be taken.
                        cold_path();

                        // Panic if we still reach this path.
                        unreachable!("The token matched here is asserted to be a TypeDefinition.")
                    }
                },
                Token::Import => {
                    let mut import_tokens = child_iterator_until(
                        tokens,
                        &TokenDiscriminants::SemiColon,
                        ParserError::SyntaxError(SyntaxError::MissingSemiColon),
                    )?;

                    // Parse improt statement and append it to the list of imports stored
                    parse_import_statement(&mut import_tokens, &mut ctx.imports)?;

                    // Drop child buffer
                    drop(import_tokens);

                    // Consume however many semicolons there are after the import
                    tokens.try_consume_match(
                        ParserError::ExpressionSemicolonMissing,
                        &TokenDiscriminants::SemiColon,
                    )?;
                },
                Token::External => {},

                // If the token was not recognized, return an error.
                _ => return Err(ParserError::ItemRequiresExplicitVisibility.into()),
            }
        }

        Ok(ctx)
    }

    pub fn new(
        config: ProjectConfig,
        module_path: Vec<String>,
        enabled_features: OrdSet<String>,
        root_path: PathBuf,
    ) -> Self
    {
        Self {
            enabled_features,
            config,
            module_path,
            root_path,
        }
    }
}
