use std::{
    fs::{self, create_dir_all},
    path::PathBuf,
    rc::Rc,
};

use codegen::llvm_codegen;
use common::{
    anyhow::{self, Result},
    compiler::ProjectConfig,
    error::{application::ApplicationError, codegen::CodeGenError},
    inkwell::{
        context::Context,
        llvm_sys::target::{
            LLVM_InitializeAllAsmParsers, LLVM_InitializeAllAsmPrinters,
            LLVM_InitializeAllTargetInfos, LLVM_InitializeAllTargetMCs, LLVM_InitializeAllTargets,
        },
        targets::{TargetMachine, TargetTriple},
    },
    linker::BuildManifest,
    toml,
    tracing::{debug, info},
    ty::{OrdSet, Type},
};
use imports::list_manager::create_dependency_functions_list;
use parser::{parser_instance::Parser, tokenizer::tokenize};

pub struct CompilerState
{
    pub config: ProjectConfig,
    pub root_dir: PathBuf,
    pub enabled_features: OrdSet<String>,
}

impl CompilerState
{
    pub fn new(root_dir: PathBuf, enabled_features: OrdSet<String>) -> anyhow::Result<Self>
    {
        // Read config file
        let config_file = fs::read_to_string(format!("{}\\config.toml", root_dir.display()))
            .map_err(|_| ApplicationError::ConfigNotFound(root_dir.clone()))?;

        let config =
            toml::from_str::<ProjectConfig>(&config_file).map_err(ApplicationError::ConfigError)?;

        Ok(Self {
            config,
            root_dir,
            enabled_features,
        })
    }

    pub fn compilation_process(
        &self,
        file_contents: &str,
        target_ir_path: PathBuf,
        target_o_path: PathBuf,
        build_path: PathBuf,
        optimization: bool,
        is_lib: bool,
        path_to_src: &str,
        flags_passed_in: &str,
        target_triple_name: Option<String>,
        cpu_name: Option<String>,
        cpu_features: Option<String>,
    ) -> Result<BuildManifest>
    {
        let target_triple = Rc::new(
            if let Some(target_triple_name) = target_triple_name {
                TargetTriple::create(&target_triple_name)
            }
            else {
                TargetMachine::get_default_triple()
            },
        );

        info!("Tokenizing...");
        let (tokens, token_ranges, _) = tokenize(file_contents, None)?;

        // for (idx, token) in tokens.iter().enumerate() {
        //     info!(
        //         "{idx} Token: {} | Range: {:?} | Lines: {:?}",
        //         token, token_ranges[idx].char_range, token_ranges[idx].lines
        //     );
        // }

        info!("Creating LLVM context...");
        let context = Context::create();
        let builder = context.create_builder();
        let module = context.create_module("main");

        info!("Initializing LLVM environment...");
        unsafe {
            LLVM_InitializeAllTargetInfos();
            LLVM_InitializeAllTargets();
            LLVM_InitializeAllTargetMCs();
            LLVM_InitializeAllAsmParsers();
            LLVM_InitializeAllAsmPrinters();
        }

        let mut dependency_output_paths = Vec::new();
        let deps_path = PathBuf::from(format!("{}\\deps", self.root_dir.display()));

        info!("Analyzing dependencies...");

        // Create an extern libs folder which we will store all the external (pre compiled) deps in
        let extern_libs_path = PathBuf::from(format!("{}\\extern_libs", self.config.build_path));

        let _ = create_dir_all(&extern_libs_path);

        let mut additional_linking_material_list: Vec<PathBuf> = Vec::new();

        // Move all of the external dep files to the folder
        for origin_path in &self.config.additional_linking_material {
            let mut extern_libs_path = extern_libs_path.clone();

            // Modify path with the file name
            extern_libs_path.push(
                origin_path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
            );

            fs::copy(origin_path, &extern_libs_path)?;

            additional_linking_material_list.push(extern_libs_path);
        }

        // Create dependency imports
        let dependency_fn_list = create_dependency_functions_list(
            &mut dependency_output_paths,
            &mut additional_linking_material_list,
            self.config.dependencies.clone(),
            self.config.remote_compiler_workers.clone(),
            deps_path.clone(),
            self.root_dir.clone(),
            optimization,
            &context,
            &builder,
            &module,
            flags_passed_in,
            target_triple.clone(),
            cpu_name.clone(),
            cpu_features.clone(),
        )?;

        let mut parser = Parser::new(
            tokens,
            token_ranges,
            self.config.clone(),
            vec![self.config.name.clone()],
            self.enabled_features.clone(),
        );

        parser.parse(dependency_fn_list)?;

        let function_table = parser.function_table();
        let imported_functions = parser.imported_functions().clone();

        if !is_lib {
            if let Some(fn_sig) = function_table.get("main") {
                if fn_sig.signature.return_type != Type::I32
                    || !fn_sig.signature.args.arguments.is_empty()
                {
                    return Err(CodeGenError::InvalidMain.into());
                }
            }
            else {
                return Err(CodeGenError::InvalidMain.into());
            }
        }
        else if function_table.contains_key("main") {
            info!("A `main` function has been found, but the library flag is set to `true`.");
        }

        // This does NOT work with structs and comments
        // check function token offset and custom types offsetting tokens
        // debug!("Recontructed token tree:");
        // let lines = file_contents.lines().collect::<Vec<&str>>();
        // for (fn_name, fn_def) in function_table.iter() {
        //     for psd_tkn in &fn_def.inner {
        //         println!("{fn_name}: tkn: {}  str: {}", psd_tkn.inner, &lines[dbg!(psd_tkn.debug_information.char_start.line)][dbg!(psd_tkn.debug_information.char_start.column)..dbg!(psd_tkn.debug_information.char_end.column - 1)])
        //     }
        // }

        llvm_codegen(
            target_ir_path.clone(),
            target_o_path,
            optimization,
            parser.clone(),
            function_table,
            Rc::new(imported_functions),
            &context,
            &builder,
            module,
            path_to_src,
            flags_passed_in,
            target_triple,
            cpu_name,
            cpu_features,
        )?;

        // Linking the object file
        // link_llvm_to_target(&module, target, target_o_path)?;
        dependency_output_paths.push(target_ir_path.clone());

        Ok(BuildManifest {
            // Localize path for later use, if we cannot strip it, it means that the path is already a stripped version, therefor we can skip that
            build_output_paths: dependency_output_paths,
            additional_linking_material: additional_linking_material_list,
            // Localize path for later use
            output_path: build_path,
        })
    }
}
