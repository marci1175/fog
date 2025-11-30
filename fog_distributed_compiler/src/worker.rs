use std::sync::Arc;

use crossbeam::{channel::{Sender, bounded}, queue::ArrayQueue};
use fog_common::{anyhow, dependency::DependencyInfo};

use crate::io::ServerState;

pub type JobQueue = ArrayQueue<JobId>;
pub type FinishedJobQueue = ArrayQueue<FinishedJobId>;

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
pub struct JobId
{
    pub idx: usize,
    pub dependency_name: String,
    pub dependency_information: DependencyInfo,
    pub target: String,
}

#[derive(Debug)]
pub struct FinishedJobId
{
    pub job_id: JobId,
    pub compilation_result: Arc<anyhow::Result<Vec<u8>>>,
}

#[derive(Debug, Clone, Copy)]
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
        for thread_idx in 0..available_cores {
            // Clone Ui handle for frontend
            let ui_sender = ui_sender.clone();

            // Create notifier channels
            let (sender, receiver) = bounded(65_535);

            // Store the channel
            self.worker_thread_notifier.insert(thread_idx, sender);

            // Create a new JobQueue for the thread
            let in_progress = JobQueue::new(65_535);
            let finished = FinishedJobQueue::new(65_535);

            let job_handler = Arc::new(JobHandler::new(in_progress, finished));

            // Store the JobQueue handle
            self.loadbalancer.insert(thread_idx, job_handler.clone());

            // Start the thread
            std::thread::spawn(move || {
                // Store information about the thread
                let thread_id = ThreadIdentification::new(thread_idx);
                let job_queue = job_handler;
                let ui_sender = ui_sender;

                loop {
                    // Thread will be block until a new task comes.
                    let notification = receiver.recv();

                    // If we failed receiving the notification send the error to the ui.
                    if let Err(err) = notification {
                        // If we cant send it to the ui just panic, since the main thread probably panicked too.
                        ui_sender.send((err.to_string(), thread_id)).unwrap();
                        break;
                    }

                    // Fetch the latest job from the job queue, if we couldnt that means we were notified too early.
                    if let Some(job) = job_queue.in_progress.pop() {
                    }
                    else {
                        ui_sender.send((String::from("Failed fetching job, worker was notified too early. Quitting...."), thread_id)).unwrap();
                        break;
                    }
                }
            });
        }

        Ok(())
    }
}
