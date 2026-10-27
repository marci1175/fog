use std::{
    collections::HashMap,
    fs::{self},
    path::PathBuf,
};

use codegen::{startup::start_codegen, startup::initalize_llvm};
use common::{
    anyhow::{self, Result}, codegen::ty::{OrdSet, Type}, compiler::{BuildCache, BuildManifest, ProjectConfig}, dependency::verify_dependencies_fs, error::{SpannedError, application::ApplicationError, codegen::CodeGenError}, imports::ImportType, inkwell::{
        context::Context,
        passes::PassBuilderOptions,
        targets::{InitializationConfig, RelocMode, Target, TargetMachine},
    }, parser::{GlobalContext, ItemVisibility, Stream, Streamable}, toml, tracing::{error, info},
};
use parser::{parser::Settings, tokenizer::tokenize};

pub struct CompilerInstance
{
    pub is_release: bool,
    pub config: ProjectConfig,
    pub root_dir: PathBuf,
    pub enabled_features: OrdSet<String>,
}

impl CompilerInstance
{
    pub fn new(
        project_root_dir: PathBuf,
        enabled_features: OrdSet<String>,
        optimized: bool,
    ) -> anyhow::Result<Self>
    {
        // Read config file
        let config_file =
            fs::read_to_string(format!("{}\\config.toml", project_root_dir.display()))
                .map_err(|_| ApplicationError::ConfigNotFound(project_root_dir.clone()))?;

        let config =
            toml::from_str::<ProjectConfig>(&config_file).map_err(ApplicationError::ConfigError)?;

        Ok(Self {
            is_release: optimized,
            config,
            root_dir: project_root_dir,
            enabled_features,
        })
    }

    /// Comprehensive function for compiling an entire project.
    /// The function handles generating ASTs, LLVM IR and producing a [`BuildManifest`] that the linker can use to produce a binary.
    pub fn compile(
        &self,
        build_cache: Option<BuildCache>,
    ) -> anyhow::Result<(BuildManifest, Option<BuildCache>)>
    {
        let root_path = vec![self.config.name.clone()];

        // The global context for the whole project
        // This function basically generates all of the ASTs for all of the dependencies and source files and puts them into one global context.
        let mut global_context = self.generate_asts(build_cache.as_ref())?;

        // Ensure that if this project is not a library it has a main function
        if !self.config.is_library {
            // If the project is an application it must have a main function
            let main_fn_name = String::from("main");

            if let Some(sig) = global_context
                .function_signatures
                .get_item(&root_path, &main_fn_name)
            {
                if !(sig.return_type == Type::I32
                    && sig.visibility == ItemVisibility::Public
                    && sig.args.arguments.is_empty()
                    && !sig.args.ellipsis_present)
                {
                    return Err(CodeGenError::InvalidMain.into());
                }
            }
            else {
                return Err(CodeGenError::NoMain.into());
            }
        }

        // Analyze the whole project
        // This includes type resolving (widening type for numbers), type checking, semantic analysis
        let output = analyzer::start_analysis(&mut global_context)?;

        if !output.is_empty() {
            error!(
                "{} errors found.",
                output.iter().map(|(_, errs)| errs.len()).sum::<usize>()
            );

            // Iter over all the outputs, if there are any and stop codegen as the code is invalid.
            // It does not really matter if the fn paths are being stored in an efficient way as code is going to quit anyway after printing out the errors.
            for (path, errors) in output {
                // Print out the errors from a given function
                for err in errors {
                    let error = SpannedError {
                        error: err.inner,
                        file: global_context
                            .parsed_files
                            .get(&path)
                            .cloned()
                            .unwrap_or_default(),
                        span: err.span,
                    };

                    // Display error in cli
                    error!("{error}");

                    // If this is a debug build we should also display the backtrace of the encountered error
                    #[cfg(debug_assertions)]
                    {
                        info!("\n{}", error.error.backtrace());
                    }
                }
            }

            // Return an error and abandon compilation
            return Err(ApplicationError::CompileFail.into());
        }
        else {
            info!("No errors found during analysis.");
        }

        // Generate LLVM IR for the global context and return the output of the generation
        Ok(self.generate_ir(&global_context, build_cache)?)
    }

    fn generate_ir(
        &self,
        global_context: &GlobalContext,
        build_cache: Option<BuildCache>,
    ) -> anyhow::Result<(BuildManifest, Option<BuildCache>)>
    {
        // Fetch the current build cache or create a new empty one
        let mut build_cache = build_cache.unwrap_or_default();
        let target_machine = initalize_llvm()?;

        // Create the llvm context
        let context = Context::create();
        let builder = context.create_builder();
        
        // Generate modules
        let modules = start_codegen(
            &context,
            &builder,
            global_context,
            self.is_release,
            &target_machine,
        )?;

        // Create opt passes list
        let passes = ["globaldce", "sink", "mem2reg"].join(",");
        let passes = passes.as_str();

        // Run optimization passes if the user prompted to
        if self.is_release {
            info!("Running optimization passes: {passes}");
        }

        // Store the canonicalized paths of the
        let mut artifacts = Vec::new();

        for (name, (module, _)) in modules {
            // if self.is_release {
            //     module
            //         .run_passes(passes, &target_machine, PassBuilderOptions::create())
            //         .map_err(|_| CodeGenError::InternalOptimisationPassFailed)?;
            // }

            // Write the generated llvm IR to its designated file
            let path = format!("{}\\{name}.ll", self.config.build_path);
            module.print_to_file(&path)?;

            // Store as a pathbuf and try to canonicalize the path
            artifacts.push(PathBuf::from(path).canonicalize()?);
        }

        let mut cache_modified = false;

        // Generate the compiler's build cache of the project's dependencies
        for (name, info) in self.config.dependencies.iter() {
            // Fetch the last modified time of the folder so that we will know when to recompile
            let metadata =
                fs::metadata(format!("{}\\deps\\{name}", self.root_dir.display()))?;
            let last_modified = metadata.modified()?;

            // Store the compiled library in the cache
            let data = (info.clone(), last_modified);
            cache_modified |= match build_cache.compiled_modules.insert(generate_dependency_name(name.clone(), self.is_release, info.version.clone()), data.clone()) {
                Some(old_item) => {
                    // Only update the cache if the inserted item is different from the old item.
                    old_item != data
                },
                None => true,
            };
        }

        Ok((
            BuildManifest {
                build_arctifact_paths: artifacts,
                additional_linking_material: self.config.additional_linking_material.clone(),
                build_path: self.config.build_path.clone().into(),
                build_type: if self.config.is_library {
                    common::compiler::BuildType::Library
                }
                else {
                    common::compiler::BuildType::Executable
                },
            },
            if cache_modified { Some(build_cache) } else { None },
        ))
    }

    fn generate_asts(&self, build_cache: Option<&BuildCache>) -> Result<GlobalContext>
    {
        let parser_settings = Settings::new(self.config.clone(), self.enabled_features.clone());

        // A global GlobalContext holds all of the context files' items.
        let mut g_context = GlobalContext::new(self.config.name.clone());

        // This source path is always defining the path of the currently parsed file.
        // This is also a way of navigating between imported source files via their relative path.
        let src_path = PathBuf::from(format!("{}\\src\\main.f", self.root_dir.display()));
        let module_path = vec![self.config.name.clone()];

        info!("Compiling `{}`...", self.config.name);

        // Parse "main.f" (The main entrypoint of the project)
        // If "main.f" imports any other files those will get parsed too
        let parsing_result =
            parse_src_file(&parser_settings, &module_path, &mut g_context, src_path);

        // Check if the parsing resulted in any errors, if yes display this error message
        if parsing_result.is_err() {
            error!("Parsing error occured, parsing cannot continue.")
        }

        parsing_result?;

        // Check if the folder is present inside the dependencies folder
        let dependencies = verify_dependencies_fs(
            format!("{}\\deps", self.root_dir.display()).into(),
            &self.config.dependencies,
        )?;

        // Create jobs for the compiler from the dependencies
        for (name, path, last_modified, info) in dependencies {
            // If a cache has been passed in check whether we have to create a job for the dependency
            if let Some(cache) = build_cache {
                // Try getting the dependency's name from the cache
                if let Some((cached_dependency_info, cached_last_modified)) =
                    cache.compiled_modules.get(&generate_dependency_name(name.to_string(), self.is_release, info.version.clone()))
                {
                    // Check if everything matches, if it does we can skip the compilation
                    if &last_modified == cached_last_modified
                        && cached_dependency_info == info
                    {
                        // Inform the user that we are skipping the cached dependency
                        info!("Skipped cached dependency `{name}({})`.", cached_dependency_info.version);

                        continue;
                    }
                }
            }

            // Its safe to unwrap here since the names presented are fetched from the dependency list directly.
            let job = CompilerInstance::new(
                path,
                // Fetch the enabled features from the config.toml
                OrdSet::from_vec(self.config.dependencies.get(name).unwrap().features.clone()),
                self.is_release,
            )?;

            // Create a list of artifacts (these are usually the dependencies of the dependency itself)
            // The function checks for item path collisions
            g_context.append_global_ctx(job.generate_asts(None)?)?;
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
    match parser_settings.parse(&mut tokens, module_path) {
        Ok(ctx) => {
            // Append the functions and items and other important information to the global context
            g_context.append_ctx(&ctx);

            // After parsing insert the src file's path into the list of parsed files
            g_context
                .parsed_files
                .insert(module_path.to_vec(), src_path.clone());

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
            #[cfg(debug_assertions)]
            {
                return Err(error.into());
            }
            #[cfg(not(debug_assertions))]
            {
                // Create a specialized error that also display where the issue is.
                let spanned_err = tokens
                    .get_last_consumed()
                    .map(|tkn| tkn.raise_error(src_path, error))
                    .unwrap();

                return Err(spanned_err.into());
            }
        },
    };

    Ok(())
}

pub fn generate_dependency_name(name: String, is_release: bool, version: String) -> String {
    format!("{name}_{}_{}", if is_release { "release" } else { "debug" }, version)
}