use std::{
    collections::HashSet,
    fs::{self},
    path::PathBuf,
    rc::Rc,
};

use common::{
    anyhow::{self, Result},
    compiler::ProjectConfig,
    dependency::verify_dependencies_fs,
    error::{
        application::ApplicationError, codegen::CodeGenError, dependency::DependencyError,
        parser::ParserError,
    },
    imports::ImportType,
    inkwell::{
        context::Context,
        targets::{TargetMachine, TargetTriple},
    },
    linker::BuildManifest,
    parser::common::{GlobalContext, ItemVisibility, Stream, Streamable},
    toml,
    tracing::info,
    ty::{OrdSet, Type},
};
use parser::{parser::Settings, tokenizer::tokenize};

pub struct CompilerJob
{
    pub config: ProjectConfig,
    pub root_dir: PathBuf,
    pub enabled_features: OrdSet<String>,
}

impl CompilerJob
{
    pub fn new(project_root_dir: PathBuf, enabled_features: OrdSet<String>)
    -> anyhow::Result<Self>
    {
        // Read config file
        let config_file =
            fs::read_to_string(format!("{}\\config.toml", project_root_dir.display()))
                .map_err(|_| ApplicationError::ConfigNotFound(project_root_dir.clone()))?;

        let config =
            toml::from_str::<ProjectConfig>(&config_file).map_err(ApplicationError::ConfigError)?;

        Ok(Self {
            config,
            root_dir: project_root_dir,
            enabled_features,
        })
    }

    pub fn generate_asts(
        &self,
        optimization: bool,
        target_triple_name: Option<String>,
    ) -> Result<GlobalContext>
    {
        let parser_settings = Settings::new(self.config.clone(), self.enabled_features.clone());

        // A global GlobalContext holds all of the context files' items.
        let mut g_context = GlobalContext::new(self.config.name.clone());

        // This source path is always defining the path of the currently parsed file.
        // This is also a way of navigating between imported source files via their relative path.
        let src_path = PathBuf::from(format!("{}\\src\\main.f", self.root_dir.display()));
        let module_path = vec![self.config.name.clone()];

        info!("Generating ({})....", self.config.name);

        // Parse "main.f" (The main entrypoint of the project)
        // If "main.f" imports any other files those will get parsed too
        parse_src_file(&parser_settings, &module_path, &mut g_context, src_path)?;

        // Ensure that if this project is not a library it has a main function
        if !self.config.is_library {
            // If the project is an application it must have a main function
            if let Some(main_fn) = g_context
                .functions
                .get_item(Rc::new(module_path), Rc::new(String::from("main")))
            {
                if !(main_fn.signature.return_type == Type::I32
                    && main_fn.visibility == ItemVisibility::Public
                    && main_fn.signature.args.arguments.is_empty()
                    && !main_fn.signature.args.ellipsis_present)
                {
                    return Err(CodeGenError::InvalidMain.into());
                }
            }
            else {
                return Err(CodeGenError::NoMain.into());
            }
        }

        // Check if the folder is present inside the dependencies folder
        let dependencies =
            verify_dependencies_fs(self.root_dir.clone(), &self.config.dependencies)?;

        // Create jobs for the compiler from the dependencies
        for (name, path) in dependencies {
            // Its safe to unwrap here since the names presented are fetched from the dependency list directly.
            let job = CompilerJob::new(
                path,
                // Fetch the enabled features from the config.toml
                OrdSet::from_vec(self.config.dependencies.get(name).unwrap().features.clone()),
            )?;

            // Create a list of artifacts (these are usually the dependencies of the dependency itself)
            g_context
                .append_global_ctx(job.generate_asts(optimization, target_triple_name.clone())?);
        }

        Ok(g_context)
    }
}

fn parse_src_file(
    parser_settings: &Settings,
    module_path: &[String],
    g_context: &mut GlobalContext,
    src_path: PathBuf,
) -> Result<(), anyhow::Error>
{
    // Inform the user that we are parsing the files
    info!("Parsing file `{}`", src_path.display());

    let file_contents = fs::read_to_string(&src_path).map_err(|_| {
        // Check for the edgecase being that the first file read is always for the "main.f" file
        if src_path.ends_with("\\main.f") {
            ApplicationError::CodeGenError(CodeGenError::NoMain.into())
        }
        else {
            ApplicationError::CodeGenError(CodeGenError::SrcFileNotFound(src_path.clone()).into())
        }
    })?;

    // Tokenize raw file
    let mut tokens = Stream::new(tokenize(&file_contents)?);

    // Parse tokenized file
    match parser_settings.parse(&mut tokens, &module_path) {
        Ok(ctx) => {
            // Append the functions and items and other important information to the global context
            g_context.append_ctx(&ctx);

            // After parsing insert the src file's path into the list of parsed files
            g_context.parsed_files.insert(src_path.clone());

            // Evaluate all source file imports
            for (name, import) in ctx.imports.iter() {
                let mut module_path = module_path.to_vec();
                let mut src_path = src_path.clone();

                module_path.push(name.clone());

                // Ensure that this is a file import
                if let ImportType::File(path) = import {
                    // Pop file name of the path
                    src_path.pop();

                    // Append the imported file's path
                    src_path.extend(path);

                    // Parse imported source file
                    parse_src_file(parser_settings, &module_path, g_context, src_path)?;
                }
                else {
                    continue;
                }
            }
        },
        Err(error) => {
            // Create a specialized error that also display where the issue is.
            let spanned_err = tokens
                .get_last_consumed()
                .map(|tkn| tkn.raise_error(src_path, error))
                .unwrap();

            return Err(spanned_err.into());
        },
    };

    Ok(())
}
