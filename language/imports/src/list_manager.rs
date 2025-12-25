use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
    rc::Rc,
    sync::Arc,
};

use codegen::llvm_codegen;
use common::{
    anyhow::{self, ensure},
    compiler::{HostInformation, ProjectConfig},
    dashmap::DashMap,
    dependency::DependencyInfo,
    distributed_compiler::DistributedCompilerWorker,
    error::dependency::DependencyError,
    futures,
    indexmap::IndexSet,
    inkwell::{builder::Builder, context::Context, module::Module, targets::TargetTriple},
    parser::FunctionSignature,
    tokio, toml,
    tracing::info,
    ty::OrdSet,
};

use crate::{
    analyzer::analyze_dependency,
    requester::{create_remote_list, dependency_requester},
};

/// Creates a dependency list from the path provided, by reading in all the folder names and libraries.
pub fn create_dependency_functions_list<'ctx>(
    dependency_output_path_list: &mut Vec<PathBuf>,
    // All of the additional linking stuff is put into this list here.
    additional_linking_material_list: &mut Vec<PathBuf>,
    mut dependency_list: HashMap<String, DependencyInfo>,
    remote_workers: Option<Vec<DistributedCompilerWorker>>,
    deps_path: PathBuf,
    root_dir: PathBuf,
    optimization: bool,
    context: &'ctx Context,
    builder: &'ctx Builder<'ctx>,
    root_module: &Module<'ctx>,
    flags_passed_in: &str,
    target_triple: Arc<TargetTriple>,
    cpu_name: Option<String>,
    cpu_features: Option<String>,
) -> anyhow::Result<Arc<DashMap<Vec<String>, FunctionSignature>>>
{
    let deps: Arc<DashMap<Vec<String>, FunctionSignature>> = Arc::new(DashMap::new());

    let mut module_path = vec![];

    // This will panic if the deps folder is not found
    let mut dir_entries = fs::read_dir(&deps_path)?;

    scan_dependencies(
        dependency_output_path_list,
        additional_linking_material_list,
        &mut dependency_list,
        optimization,
        context,
        builder,
        root_module,
        deps.clone(),
        &mut module_path,
        &mut dir_entries,
        flags_passed_in,
        target_triple.clone(),
        cpu_name.clone(),
        cpu_features.clone(),
    )?;

    // Request remaining dependencies from package handler server
    if let Some(remotes) = remote_workers {
        let host_information = HostInformation::new(
            cpu_features.clone(),
            cpu_name.clone(),
            Some(flags_passed_in.to_string()),
            target_triple.as_str().to_string_lossy().to_string(),
        );

        // Create a map of the remotes' thread handlers
        let (remote_handlers, thread_handles) =
            create_remote_list(remotes, host_information, deps.clone(), root_dir.clone());

        // Request the dependencies from those remotes
        dependency_requester(&dependency_list, &remote_handlers)?;

        // Wait for the threads to finish
        tokio::runtime::Handle::current().block_on(async move {
            futures::future::join_all(thread_handles).await;
        });
    }
    else if !dependency_list.is_empty() {
        return Err(DependencyError::MissingDependencies(dependency_list).into());
    }

    let mut dir_entries_remote = fs::read_dir(format!("{}\\remote_compile", root_dir.display()))?;

    // Scan and parse downloaded dependencies
    scan_dependencies(
        dependency_output_path_list,
        additional_linking_material_list,
        &mut dependency_list,
        optimization,
        context,
        builder,
        root_module,
        deps.clone(),
        &mut module_path,
        &mut dir_entries_remote,
        flags_passed_in,
        target_triple.clone(),
        cpu_name.clone(),
        cpu_features.clone(),
    )?;

    Ok(deps)
}

fn scan_dependencies<'ctx>(
    dependency_output_path_list: &mut Vec<PathBuf>,
    additional_linking_material_list: &mut Vec<PathBuf>,
    dependency_list: &mut HashMap<String, DependencyInfo>,
    optimization: bool,
    context: &'ctx Context,
    builder: &'ctx Builder<'ctx>,
    root_module: &Module<'ctx>,
    deps: Arc<DashMap<Vec<String>, FunctionSignature>>,
    module_path: &mut Vec<String>,
    dir_entries: &mut fs::ReadDir,
    flags_passed_in: &str,
    target_triple: Arc<TargetTriple>,
    cpu_name: Option<String>,
    cpu_features: Option<String>,
) -> Result<(), anyhow::Error>
{
    // Scan the dependencies which are present in the dependencies' folder
    while let Some(Ok(dir_entry)) = dir_entries.next() {
        let metadata = dir_entry
            .metadata()
            .map_err(|err| DependencyError::FileError(err.into()))?;

        // Dont do anything with files
        // From this point on assume everything is a project folder
        if metadata.is_file() {
            continue;
        }

        let mut dependency_path = dir_entry.path();

        scan_dependency(
            dependency_output_path_list,
            additional_linking_material_list,
            dependency_list,
            deps.clone(),
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
    additional_linking_material_list: &mut Vec<PathBuf>,
    dependency_list: &mut HashMap<String, DependencyInfo>,
    deps: Arc<DashMap<Vec<String>, FunctionSignature>>,
    dependency_path: &mut PathBuf,
    optimization: bool,
    context: &'ctx Context,
    builder: &'ctx Builder<'ctx>,
    root_module: &Module<'ctx>,
    module_path: &mut Vec<String>,
    flags_passed_in: &str,
    target_triple: Arc<TargetTriple>,
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

            if dependency_config.remote_compiler_workers.is_some() {
                info!(
                    "WARNING: Dependency {} has set a remote compiler worker. The attribute will be ignored.",
                    dependency_config.name
                );
            }

            // Remove the library which was found already, so that ideally the dep list will be empty after this function ran.
            // Match version number
            if let Some(project_dependency) = dependency_list.remove(&dependency_config.name) {
                ensure!(
                    project_dependency.version.clone() == dependency_config.version,
                    DependencyError::MismatchedVersion(
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

                dependency_path.push("deps");

                fs::create_dir_all(&dependency_path)?;

                // Parse the library's dependecies
                // We pass in the things mutable because this is how we are checking that every dependency is covered. (See: create_dependency_functions_list)
                scan_dependencies(
                    dependency_output_path_list,
                    additional_linking_material_list,
                    &mut dependency_config.dependencies,
                    optimization,
                    context,
                    builder,
                    root_module,
                    deps.clone(),
                    module_path,
                    &mut fs::read_dir(dependency_path.clone())?,
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

                // Store the public functions in the main dep list.
                for (path, sig) in parser_state.library_public_function_table().to_owned() {
                    deps.insert(path, sig);
                }

                // Specific the paths of the additional linking material and store it
                additional_linking_material_list.extend(
                    dependency_config
                        .additional_linking_material
                        .iter()
                        .map(|path| {
                            PathBuf::from(format!(
                                "{}\\{}",
                                original_dep_path_root.display(),
                                path.display()
                            ))
                        }),
                );

                let imported_functions = Rc::new(parser_state.imported_functions().clone());

                // Generate LLVM-IR for the dependency
                let target_ir_path = PathBuf::from(format!(
                    "{}\\{}\\{}.ll",
                    original_dep_path_root.display(),
                    dependency_config.build_path.clone(),
                    dependency_config.name
                ));

                // Only generate llvm-ir files if they dont exist for the dependency
                if !fs::exists(&target_ir_path)? {
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
                }

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
