use std::{path::PathBuf, rc::Rc};

use codegen::llvm_codegen;
use common::{
    anyhow::Result,
    compiler::ProjectConfig,
    error::codegen::CodeGenError,
    inkwell::{
        context::Context,
        llvm_sys::target::{
            LLVM_InitializeAllAsmParsers, LLVM_InitializeAllAsmPrinters,
            LLVM_InitializeAllTargetInfos, LLVM_InitializeAllTargetMCs, LLVM_InitializeAllTargets,
        },
    },
    linker::BuildManifest,
    ty::{OrdSet, TypeDiscriminant},
};
use imports::list_manager::create_dependency_functions_list;
use parser::{parser_instance::Parser, tokenizer::tokenize};

pub struct CompilerState
{
    pub config: ProjectConfig,
    pub working_dir: PathBuf,
    pub enabled_features: OrdSet<String>,
}

impl CompilerState
{
    pub fn new(
        config: ProjectConfig,
        working_dir: PathBuf,
        enabled_features: OrdSet<String>,
    ) -> Self
    {
        Self {
            config,
            working_dir,
            enabled_features,
        }
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
        target_triple: Option<String>,
        cpu_name: Option<String>,
        cpu_features: Option<String>,
    ) -> Result<BuildManifest>
    {
        println!("Tokenizing...");
        let (tokens, token_ranges, _) = tokenize(file_contents, None)?;

        // for (idx, token) in tokens.iter().enumerate() {
        //     println!(
        //         "{idx} Token: {} | Range: {:?} | Lines: {:?}",
        //         token, token_ranges[idx].char_range, token_ranges[idx].lines
        //     );
        // }

        println!("Creating LLVM context...");
        let context = Context::create();
        let builder = context.create_builder();
        let module = context.create_module("main");

        println!("Initializing LLVM environment...");
        unsafe {
            LLVM_InitializeAllTargetInfos();
            LLVM_InitializeAllTargets();
            LLVM_InitializeAllTargetMCs();
            LLVM_InitializeAllAsmParsers();
            LLVM_InitializeAllAsmPrinters();
        }

        let mut dependency_output_paths = Vec::new();
        let mut additional_linking_material_list = self.config.additional_linking_material.clone();

        println!("Analyzing dependencies...");

        // Create dependency imports
        let dependency_fn_list = create_dependency_functions_list(
            &mut dependency_output_paths,
            &mut additional_linking_material_list,
            self.config.dependencies.clone(),
            PathBuf::from(format!("{}\\deps", self.working_dir.display())),
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
                if fn_sig.function_sig.return_type != TypeDiscriminant::I32
                    || !fn_sig.function_sig.args.arguments_list.is_empty()
                {
                    return Err(CodeGenError::InvalidMain.into());
                }
            }
            else {
                return Err(CodeGenError::NoMain.into());
            }
        }
        else if function_table.contains_key("main") {
            println!("A `main` function has been found, but the library flag is set to `true`.");
        }

        // println!("Recontructed token tree:");
        // let lines = file_contents.lines().collect::<Vec<&str>>();
        // for (fn_name, fn_def) in function_table.iter() {
        //     for psd_tkn in &fn_def.inner {
        //         for (idx, ln_idx) in psd_tkn.debug_information.lines.clone().into_iter().enumerate() {
        //             dbg!(&lines[ln_idx]);

        //             let line_fetch = lines[ln_idx].get(dbg!(psd_tkn.debug_information.char_range[idx].clone()));

        //             println!("{}", line_fetch.unwrap());
        //         }
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
            build_output_paths: dependency_output_paths,
            additional_linking_material: additional_linking_material_list,
            output_path: build_path,
        })
    }
}
