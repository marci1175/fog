use std::path::PathBuf;

use common::{
    anyhow::{self, Result},
    combine_path,
    compiler::ProjectConfig,
    error::{Spanned, parser::ParserError},
    parser::{
        common::{Context, Streamable, TokenStream, parse_compiler_instruction},
        function::{CompilerInstruction, CompilerInstructionDiscriminants, parse_function},
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
        imma change some of the syntax for example imma make it so that i can do `pub import "blabla.f", so that i can bring path into scope.`
    */

    pub fn parse(&self, tokens: &mut TokenStream<Spanned<Token>>) -> Result<Context>
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

                    // Match the type of the item
                    match item_tkn.get_inner() {
                        Token::TypeDefinition(item_type) => {
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
                                            function.signature.module_path.clone(),
                                            function.signature.name.clone(),
                                        ),
                                        function.signature.name.clone().into(),
                                        function,
                                    );
                                },
                                _ => return Err(ParserError::ItemTypeExpected.into()),
                            }
                        },

                        _ => return Err(ParserError::ItemTypeExpected.into()),
                    }
                },

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