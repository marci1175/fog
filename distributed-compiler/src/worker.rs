use std::sync::Arc;

use crate::io::ServerState;
use common::{
    anyhow,
    crossbeam::{channel::Sender, deque},
    dependency::DependencyInfo,
};

pub type JobQueue = deque::Injector<CompileJob>;
pub type FinishedJobQueue = deque::Injector<FinishedJobId>;

#[derive(Debug)]
pub struct JobHandler
{
    /// Compilation tasks which are in progress
    pub in_progress: JobQueue,

    /// Compilation tasks which have been finished
    pub finished: FinishedJobQueue,
}

impl JobHandler
{
    pub fn new(in_progress: JobQueue, finished: FinishedJobQueue) -> Self
    {
        Self {
            in_progress,
            finished,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompileJob
{
    pub dependency_name: String,
    pub dependency_information: DependencyInfo,
    pub target: String,
}

#[derive(Debug)]
pub struct FinishedJobId
{
    pub job_id: CompileJob,
    pub compilation_result: Arc<anyhow::Result<Vec<u8>>>,
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
    ) -> Result<(), anyhow::Error>
    {
        // Increment the thread idx for the identification because the first two a reseverved for io
        for thread_idx in 0..available_cores {
            // Create thread identificator
            let thread_id = ThreadIdentification::new(thread_idx, ThreadType::Worker);

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
                        // Send message that we have received a job
                        ui_sender
                            .send((
                                format!(
                                    "Received job `{}`({}).",
                                    job.dependency_name.clone(),
                                    job.dependency_information.version.clone()
                                ),
                                thread_id,
                            ))
                            .unwrap();
                    }
                    else {
                        std::thread::park();
                    }
                }
            });

            // Store the thread handle
            self.worker_thread_notifier
                .insert(thread_id, thread_handle.thread().clone());
        }

        Ok(())
    }
}
