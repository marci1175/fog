use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
    rc::Rc,
};

use fog_codegen::llvm_codegen;
use fog_common::{
    anyhow::{self, ensure},
    compiler::ProjectConfig,
    dependency::DependencyInfo,
    error::{codegen::CodeGenError, dependency::DependencyError},
    indexmap::{IndexMap, IndexSet},
    inkwell::{builder::Builder, context::Context, module::Module},
    parser::FunctionSignature,
    toml,
    ty::OrdSet,
};

use crate::dependency_analyzer::analyze_dependency;

/// Creates a dependency list from the path provided, by reading in all the folder names and libraries.
pub fn create_dependency_functions_list<'ctx>(
    dependency_output_path_list: &mut Vec<PathBuf>,
    mut dependency_list: HashMap<String, DependencyInfo>,
    deps_path: PathBuf,
    optimization: bool,
    context: &'ctx Context,
    builder: &'ctx Builder<'ctx>,
    root_module: &Module<'ctx>,
    flags_passed_in: &str,
    target_triple: Option<String>,
    cpu_name: Option<String>,
    cpu_features: Option<String>,
) -> anyhow::Result<HashMap<String, IndexMap<String, FunctionSignature>>>
{
    let mut deps = HashMap::new();

    let mut module_path = vec![];

    let dir_entries = fs::read_dir(deps_path)?;

    scan_dependencies(
        dependency_output_path_list,
        &mut dependency_list,
        optimization,
        context,
        builder,
        root_module,
        &mut deps,
        &mut module_path,
        dir_entries,
        flags_passed_in,
        target_triple.clone(),
        cpu_name.clone(),
        cpu_features.clone(),
    )?;

    if !dependency_list.is_empty() {
        return Err(DependencyError::MissingDependencies(dependency_list.clone()).into());
    }

    Ok(deps)
}

fn scan_dependencies<'ctx>(
    dependency_output_path_list: &mut Vec<PathBuf>,
    dependency_list: &mut HashMap<String, DependencyInfo>,
    optimization: bool,
    context: &'ctx Context,
    builder: &'ctx Builder<'ctx>,
    root_module: &Module<'ctx>,
    deps: &mut HashMap<String, IndexMap<String, FunctionSignature>>,
    module_path: &mut Vec<String>,
    mut dir_entries: fs::ReadDir,
    flags_passed_in: &str,
    target_triple: Option<String>,
    cpu_name: Option<String>,
    cpu_features: Option<String>,
) -> Result<(), anyhow::Error>
{
    while let Some(Ok(dir_entry)) = dir_entries.next() {
        let metadat = dir_entry
            .metadata()
            .map_err(|err| DependencyError::FileError(err.into()))?;

        // Dont do anything with files
        // From this point on assume everything is a project folder
        if metadat.is_file() {
            continue;
        }

        let mut dependency_path = dir_entry.path();

        scan_dependency(
            dependency_output_path_list,
            dependency_list,
            deps,
            &mut dependency_path,
            optimization,
            context,
            builder,
            root_module,
            module_path,
            flags_passed_in,
            target_triple.clone(),
            cpu_name.clone(),
            cpu_features.clone(),
        )?;
    }

    Ok(())
}

fn scan_dependency<'ctx>(
    dependency_output_path_list: &mut Vec<PathBuf>,
    dependency_list: &mut HashMap<String, DependencyInfo>,
    deps: &mut HashMap<String, IndexMap<String, FunctionSignature>>,
    dependency_path: &mut PathBuf,
    optimization: bool,
    context: &'ctx Context,
    builder: &'ctx Builder<'ctx>,
    root_module: &Module<'ctx>,
    module_path: &mut Vec<String>,
    flags_passed_in: &str,
    target_triple: Option<String>,
    cpu_name: Option<String>,
    cpu_features: Option<String>,
) -> Result<(), anyhow::Error>
{
    let mut project_dir = fs::read_dir(dependency_path.clone())
        .map_err(|err| DependencyError::FileError(err.into()))?;

    let project_directory_entry =
        project_dir.find(|entry| entry.as_ref().is_ok_and(|e| e.file_name() == "config.toml"));

    match project_directory_entry {
        Some(config_file) => {
            let cfg_file = config_file.map_err(|err| DependencyError::FileError(err.into()))?;

            let config_file_content = fs::read_to_string(cfg_file.path())
                .map_err(|err| DependencyError::FileError(err.into()))?;

            let mut dependency_config = toml::from_str::<ProjectConfig>(&config_file_content)?;

            // Remove the library which was found already, so that ideally the dep list will be empty after this function ran.
            // Match version number
            if let Some(project_dependency) = dependency_list.remove(&dependency_config.name) {
                ensure!(
                    project_dependency.version.clone() == dependency_config.version,
                    DependencyError::MismatchedVersionNumber(
                        dependency_config.name,
                        project_dependency.version.clone(),
                        dependency_config.version
                    )
                );

                let dep_features_enabled = project_dependency.features.clone();

                if !dependency_config.is_library {
                    return Err(
                        DependencyError::InvalidDependencyType(dependency_config.name).into(),
                    );
                }

                if let Some(dep_available_features) = &dependency_config.features {
                    // If there are more features enabled than there is available throw an error about invalid features.
                    if !HashSet::<String>::from_iter(dep_features_enabled.iter().cloned())
                        .is_subset(&HashSet::from_iter(dep_available_features.iter().cloned()))
                    {
                        return Err(DependencyError::InvalidDependencyFeature(
                            dependency_config.name,
                            dep_available_features.clone(),
                            dep_features_enabled,
                        )
                        .into());
                    }
                }

                let lib_src_file_content =
                    fs::read_to_string(format!("{}\\src\\main.f", dependency_path.display()))
                        .map_err(|err| DependencyError::FileError(err.into()))?;

                // Create context for the dependency
                let lib_module = context.create_module(&dependency_config.name);

                module_path.push(dependency_config.name.clone());

                let current_module_path = module_path.clone();

                let original_dep_path_root = dependency_path.clone();

                dependency_path.push(format!("deps"));

                // Parse the library's dependecies
                // We pass in the things mutable because this is how we are checking that every dependency is covered. (See: create_dependency_functions_list)
                scan_dependencies(
                    dependency_output_path_list,
                    &mut dependency_config.dependencies,
                    optimization,
                    context,
                    builder,
                    root_module,
                    deps,
                    module_path,
                    fs::read_dir(dependency_path.clone())?,
                    flags_passed_in,
                    target_triple.clone(),
                    cpu_name.clone(),
                    cpu_features.clone(),
                )?;

                if !dependency_config.dependencies.is_empty() {
                    // If the error was thrown here it means that a library has a missing dependency. Shame on the developer for not checking their library.
                    return Err(DependencyError::MissingDependencies(
                        dependency_config.dependencies.clone(),
                    )
                    .into());
                }

                // Parse library for public items
                let parser_state = analyze_dependency(
                    &lib_src_file_content,
                    deps.clone(),
                    dependency_config.clone(),
                    current_module_path.clone(),
                    OrdSet::wrap(IndexSet::from_iter(dep_features_enabled.iter().cloned())),
                )?;

                deps.insert(
                    dependency_config.name.clone(),
                    parser_state.library_public_function_table().clone(),
                );

                let imported_functions = Rc::new(parser_state.imported_functions().clone());

                // Generate LLVM-IR for the dependency
                let target_ir_path = PathBuf::from(format!(
                    "{}\\{}\\{}.ll",
                    original_dep_path_root.display(),
                    dependency_config.build_path.clone(),
                    dependency_config.name
                ));

                llvm_codegen(
                    target_ir_path.clone(),
                    PathBuf::from(format!(
                        "{}\\{}\\{}.o",
                        original_dep_path_root.display(),
                        dependency_config.build_path.clone(),
                        dependency_config.name
                    )),
                    optimization,
                    parser_state.clone(),
                    parser_state.function_table(),
                    imported_functions,
                    context,
                    builder,
                    lib_module.clone(),
                    &format!("{}\\src", dependency_path.display()),
                    flags_passed_in,
                    target_triple,
                    cpu_name,
                    cpu_features,
                )?;

                dependency_output_path_list.push(target_ir_path);

                root_module
                    .link_in_module(lib_module)
                    .map_err(|err| DependencyError::ModuleLinkingFailed(err.to_string()))?;
            }
        },
        None => {
            return Err(DependencyError::DependencyMissingConfig(dependency_path.clone()).into());
        },
    };

    Ok(())
}
