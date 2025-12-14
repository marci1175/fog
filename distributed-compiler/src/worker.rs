use std::{
    collections::HashMap,
    fs::{self, create_dir, create_dir_all},
    io::{self, Write, stdout},
    mem,
    net::SocketAddr,
    path::PathBuf,
    ptr::{self, replace},
    sync::Arc,
};

use crate::io::ServerState;
use common::{
    anyhow,
    compiler::ProjectConfig,
    compression::{compress_bytes, zip_folder},
    crossbeam::{channel::Sender, deque, queue::ArrayQueue},
    distributed_compiler::{CompileJob, FinishedJob},
    error::codegen::CodeGenError,
    linker::BuildManifest,
    serde::{Deserialize, Serialize},
    tokio, toml,
    ty::OrdSet,
};
use compiler::CompilerState;
use dashmap::DashMap;

pub type JobQueue = deque::Injector<CompileJob>;

#[derive(Debug, Clone)]
pub struct JobHandler
{
    /// Compilation tasks which are in progress
    pub in_progress: Arc<JobQueue>,
}

impl JobHandler
{
    pub fn new(in_progress: Arc<deque::Injector<CompileJob>>) -> Self
    {
        Self { in_progress }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ThreadIdentification
{
    pub id: usize,
    pub thread_type: ThreadType,
}

impl ThreadIdentification
{
    pub fn new(id: usize, thread_type: ThreadType) -> Self
    {
        Self { id, thread_type }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThreadType
{
    IO,
    Worker,
}

impl ServerState
{
    pub fn create_workers(
        &mut self,
        available_cores: usize,
        ui_sender: Sender<(String, ThreadIdentification)>,
        outbound_handlers: Arc<DashMap<SocketAddr, tokio::sync::mpsc::Sender<FinishedJob>>>,
    ) -> Result<HashMap<ThreadIdentification, std::thread::Thread>, anyhow::Error>
    {
        let mut worker_thread_notifier: HashMap<ThreadIdentification, std::thread::Thread> =
            HashMap::new();

        // Increment the thread idx for the identification because the first two a reseverved for io
        for thread_idx in 0..available_cores {
            // Create thread identificator
            let thread_id = ThreadIdentification::new(thread_idx, ThreadType::Worker);
            let outbound_handlers = outbound_handlers.clone();

            // Clone Ui handle for frontend
            let ui_sender = ui_sender.clone();
            let job_handler = self.job_handler.clone();

            // Start the thread
            let thread_handle = std::thread::spawn(move || {
                // Store information about the thread
                let thread_id = thread_id;
                let job_queue = job_handler;
                let ui_sender = ui_sender;

                loop {
                    // Fetch the latest job from the job queue, if we couldnt that means we were notified too early.
                    if let Some(job) = job_queue.in_progress.steal().success() {
                        match compile_job(job.clone(), ui_sender.clone(), thread_id) {
                            Ok((path_to_output_artifacts, project_config, build_manifest)) => {
                                let zipped_artifacts = zip_folder(
                                    fs::read_dir(path_to_output_artifacts).unwrap(),
                                    None,
                                )
                                .unwrap();

                                let zip = zipped_artifacts.finish().unwrap().into_inner();

                                match outbound_handlers.get(&job.remote_address) {
                                    Some(handler) => {
                                        let channel = handler.value();

                                        channel
                                            .try_send(FinishedJob {
                                                info: job.dependency_information,
                                                artifacts_zip_bytes: zip,
                                                dependency_config: project_config,
                                                build_manifest,
                                            })
                                            .unwrap();
                                    },
                                    None => {
                                        ui_sender
                                            .send((
                                                format!("Outbound handler does not exist for remote `{}`", job.remote_address),
                                                thread_id,
                                            ))
                                            .unwrap();
                                    },
                                }
                            },
                            Err(error) => {
                                ui_sender
                                    .send((
                                        format!("Error occured when compiling job `{}`", error),
                                        thread_id,
                                    ))
                                    .unwrap();
                            },
                        }
                    }
                    else {
                        std::thread::park();
                    }
                }
            });

            // Store the thread handle
            worker_thread_notifier.insert(thread_id, thread_handle.thread().clone());
        }

        Ok(worker_thread_notifier)
    }
}

fn compile_job(
    job: CompileJob,
    ui_sender: Sender<(String, ThreadIdentification)>,
    thread_id: ThreadIdentification,
) -> anyhow::Result<(PathBuf, ProjectConfig, BuildManifest)>
{
    let compiler_state = CompilerState::new(job.depdendency_path.clone(), job.features).unwrap();

    // Send message that we have received a job
    ui_sender
        .send((
            format!(
                "Received job `{}`({}).",
                compiler_state.config.name.clone(),
                compiler_state.config.version.clone()
            ),
            thread_id,
        ))
        .unwrap();

    let source_file =
        fs::read_to_string(format!("{}\\src\\main.f", job.depdendency_path.display()))
            .map_err(|_| CodeGenError::NoMain)?;

    let build_arctifacts_path = format!(
        "{}\\{}",
        job.depdendency_path.display(),
        compiler_state.config.build_path
    );

    let build_artifact_name = format!(
        "{}\\{}",
        build_arctifacts_path,
        compiler_state.config.name.clone()
    );

    let _ = create_dir_all(&build_arctifacts_path);
    let _ = create_dir_all(PathBuf::from(format!(
        "{}\\deps",
        compiler_state.root_dir.display()
    )));

    let target_ir_path = PathBuf::from(format!("{build_artifact_name}.ll"));

    let target_o_path = PathBuf::from(format!("{build_artifact_name}.obj"));

    let build_path = PathBuf::from(format!("{build_artifact_name}.exe"));

    let build_manifest_path = PathBuf::from(format!("{build_artifact_name}.manifest"));

    let build_manifest = compiler_state.compilation_process(
        &source_file,
        target_ir_path,
        target_o_path,
        build_path,
        true,
        true,
        &format!("{}\\src", job.depdendency_path.display()),
        &job.flags_passed_in,
        Some(job.target_triple),
        job.cpu_name,
        job.cpu_features,
    )?;

    fs::write(build_manifest_path, toml::to_string(&build_manifest)?)?;

    ui_sender
        .send((
            format!(
                "Compiled job `{}`({}).",
                compiler_state.config.name.clone(),
                compiler_state.config.version.clone()
            ),
            thread_id,
        ))
        .unwrap();

    Ok((
        job.depdendency_path,
        compiler_state.config,
        build_manifest.localize_paths(compiler_state.root_dir),
    ))
}
