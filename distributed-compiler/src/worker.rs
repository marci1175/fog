use std::{sync::Arc, thread::Thread};

use common::{anyhow, dependency::DependencyInfo};
use crossbeam::{
    channel::{Sender, bounded},
    deque,
    queue::ArrayQueue,
};

use crate::io::ServerState;

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
}

impl ThreadIdentification
{
    pub fn new(id: usize) -> Self
    {
        Self { id }
    }
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
        for thread_idx in 2..available_cores + 2 {
            // Create thread identificator
            let thread_id = ThreadIdentification::new(thread_idx);

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
