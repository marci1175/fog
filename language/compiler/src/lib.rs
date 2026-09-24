use std::{
    fs::{self},
    path::PathBuf,
};

use codegen::irgen::start_codegen;
use common::{
    anyhow::{self, Result},
    codegen::ty::{OrdSet, Type},
    compiler::ProjectConfig,
    dependency::verify_dependencies_fs,
    error::{application::ApplicationError, codegen::CodeGenError},
    imports::ImportType,
    inkwell::{
        context::Context,
        passes::PassBuilderOptions,
        targets::{InitializationConfig, RelocMode, Target, TargetMachine},
    },
    linker::BuildManifest,
    parser::common::{GlobalContext, ItemVisibility, Stream, Streamable},
    toml,
    tracing::info,
};
use parser::{parser::Settings, tokenizer::tokenize};

pub struct CompilerInstance
{
    pub optimized: bool,
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
            optimized,
            config,
            root_dir: project_root_dir,
            enabled_features,
        })
    }

    /// Comprehensive function for compiling an entire project.
    /// The function handles generating ASTs, LLVM IR and producing a [`BuildManifest`] that the linker can use to produce a binary.
    pub fn compile(&self) -> anyhow::Result<BuildManifest>
    {
        let root_path = vec![self.config.name.clone()];

        // The global context for the whole project
        // This function basically generates all of the ASTs for all of the dependencies and source files and puts them into one global context.
        let mut global_context = self.generate_asts()?;

        // Ensure that if this project is not a library it has a main function
        if !self.config.is_library {
            // If the project is an application it must have a main function
            if let Some(main_fn) = global_context
                .functions
                .get_item(root_path, String::from("main"))
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

        // Analyze the whole project
        // This includes type resolving (widening type for numbers), type checking, semantic analysis
        analyzer::start_analysis(&mut global_context)?;

        // Generate LLVM IR for the global context
        self.generate_ir(&global_context)?;

        Ok(todo!())
    }

    fn generate_ir(&self, global_context: &GlobalContext) -> anyhow::Result<()>
    {
        info!("Initalizing LLVM....");

        // Initalize target
        Target::initialize_x86(&InitializationConfig::default());
        let target_triple = TargetMachine::get_default_triple();

        // Create target
        let target = Target::from_triple(&target_triple)
            .map_err(|_| common::anyhow::Error::from(CodeGenError::FaliedToAcquireTargetTriple))?;

        // Create target machine
        let target_machine = target
            .create_target_machine(
                &target_triple,
                &TargetMachine::get_host_cpu_name().to_string(),
                &TargetMachine::get_host_cpu_features().to_string(),
                common::inkwell::OptimizationLevel::Aggressive,
                RelocMode::Default,
                common::inkwell::targets::CodeModel::Default,
            )
            .unwrap();

        // Create the llvm context
        let context = Context::create();
        let builder = context.create_builder();

        // Generate modules
        let modules = start_codegen(
            &context,
            &builder,
            global_context,
            self.optimized,
            &target_machine,
        )?;

        // Create opt passes list
        let passes = ["globaldce", "sink", "mem2reg"].join(",");
        let passes = passes.as_str();

        // Run optimization passes if the user prompted to
        if self.optimized {
            info!("Running optimization passes: {passes}...");
        }

        for (name, (module, _)) in modules {
            if self.optimized {
                module
                    .run_passes(passes, &target_machine, PassBuilderOptions::create())
                    .map_err(|_| CodeGenError::InternalOptimisationPassFailed)?;
            }

            // Write the generated llvm IR to its designated file
            module.print_to_file(format!("{name}.ll"))?;
        }

        Ok(())
    }

    fn generate_asts(&self) -> Result<GlobalContext>
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

        // Check if the folder is present inside the dependencies folder
        let dependencies =
            verify_dependencies_fs(self.root_dir.clone(), &self.config.dependencies)?;

        // Create jobs for the compiler from the dependencies
        for (name, path) in dependencies {
            // Its safe to unwrap here since the names presented are fetched from the dependency list directly.
            let job = CompilerInstance::new(
                path,
                // Fetch the enabled features from the config.toml
                OrdSet::from_vec(self.config.dependencies.get(name).unwrap().features.clone()),
                self.optimized,
            )?;

            // Create a list of artifacts (these are usually the dependencies of the dependency itself)
            // The function checks for item path collisions
            g_context.append_global_ctx(job.generate_asts()?)?;
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
    match parser_settings.parse(&mut tokens, module_path) {
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
