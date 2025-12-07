use std::{
    collections::HashMap,
    fs::create_dir_all,
    io::Cursor,
    net::{Ipv6Addr, SocketAddr},
    path::PathBuf,
    sync::Arc,
    thread::Thread,
};

use common::{
    anyhow,
    compression::{decompress_bytes, write_zip_to_fs_async, zip_from_bytes},
    crossbeam::channel::{Receiver, bounded},
    dependency::construct_dependency_path,
    dependency_manager::Dependency,
    distributed_compiler::DependencyRequest,
    reqwest::Client,
    rmp_serde,
    tokio::{
        self,
        io::{AsyncReadExt, AsyncWriteExt},
        sync::mpsc::{Sender, channel},
    },
    ty::OrdSet,
};
use dashmap::DashMap;

use crate::{
    net,
    worker::{CompileJob, FinishedJob, JobHandler, JobQueue, ThreadIdentification},
};

#[derive(Debug, Clone)]
pub struct ServerState
{
    pub port: u16,
    pub dependency_manager_url: String,
    pub worker_thread_notifier: HashMap<ThreadIdentification, Thread>,
    pub job_handler: JobHandler,
    pub connected_clients: Arc<DashMap<SocketAddr, String>>,
    pub dependency_folder: Arc<PathBuf>,
    pub thread_error_out: Option<Receiver<(String, ThreadIdentification)>>,
}

impl Default for ServerState
{
    fn default() -> Self
    {
        Self {
            port: 0,
            dependency_manager_url: "http://[::1]:3004".into(),
            worker_thread_notifier: HashMap::new(),
            dependency_folder: Arc::new(PathBuf::new()),
            job_handler: JobHandler::new(Arc::new(JobQueue::new())),
            connected_clients: Arc::new(DashMap::new()),
            thread_error_out: None,
        }
    }
}

impl ServerState
{
    pub fn new(port: u16, dependency_manager_url: String, dependency_folder: Arc<PathBuf>) -> Self
    {
        Self {
            port,
            dependency_folder,
            dependency_manager_url,
            ..Default::default()
        }
    }

    /// Initialize threads for the server
    pub fn initialize_server(&mut self) -> anyhow::Result<()>
    {
        let (thread_out_sender, thread_out_recv) = bounded::<(String, ThreadIdentification)>(255);

        let dep_path = self.dependency_folder.clone();

        let _ = create_dir_all(&*dep_path);

        self.thread_error_out = Some(thread_out_recv);

        let available_cores = std::thread::available_parallelism()?.get();

        let available_cores_left = available_cores.checked_sub(2).unwrap_or(1);

        let outbound_handlers: Arc<DashMap<SocketAddr, Sender<FinishedJob>>> =
            Arc::new(DashMap::new());

        let workers = Arc::new(self.create_workers(
            available_cores_left,
            thread_out_sender.clone(),
            outbound_handlers.clone(),
        )?);

        let current_jobs = self.job_handler.clone();

        let port = self.port;

        let ui_sender_in = thread_out_sender.clone();
        let ui_sender_out = thread_out_sender.clone();

        let connected_clients_handle = self.connected_clients.clone();

        let http_client = Client::new();
        let remote_url = self.dependency_manager_url.clone();

        let ui_sender_out_clone = ui_sender_out.clone();

        tokio::spawn(async move {
            let dep_path = dep_path.clone();
            let http_client = http_client.clone();
            let outbound_handlers = outbound_handlers.clone();

            // Bind listener to local on specified port
            let listener =
                tokio::net::TcpListener::bind((Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), port))
                    .await
                    .unwrap();

            let mut io_thread_idx = 2;

            loop {
                let workers = workers.clone();
                let dep_path = dep_path.clone();
                let remote_url = remote_url.clone();
                let current_jobs = current_jobs.clone();
                let http_client = http_client.clone();

                // Clone sender channel so that we can send messages to the frontend
                let ui_sender_out_clone = ui_sender_out_clone.clone();

                match listener.accept().await {
                    Ok((stream, addr)) => {
                        let outbound_handlers = outbound_handlers.clone();

                        connected_clients_handle.insert(addr, "Some client information".into());

                        let client_addr = addr.clone();

                        let (client_recv, mut client_sender) = tokio::io::split(stream);

                        // Create client sender threads
                        let (sender, mut recv) = channel::<FinishedJob>(255);

                        // Store thread handler channel
                        outbound_handlers.insert(addr, sender);

                        let ui_sender = ui_sender_out_clone.clone();

                        // We need to spawn the client sender before the client handler so as to ensure that the sender is already working by the time the handler processes the request
                        // Spawn client sender
                        tokio::spawn(async move {
                            loop {
                                // Wait for a job to finish with the client
                                match recv.recv().await {
                                    Some(finished_job) => {
                                        // Compiled zip len
                                        let compiled_src =
                                            finished_job.compressed_artifacts_zip.len();

                                        // Send length of the zip
                                        client_sender
                                            .write_all(&(compiled_src as u32).to_be_bytes())
                                            .await
                                            .unwrap();

                                        // Send the actual zip
                                        client_sender
                                            .write_all(&finished_job.compressed_artifacts_zip)
                                            .await
                                            .unwrap();
                                    },
                                    None => {
                                        ui_sender
                                            .send((
                                                format!("Clinet handler for remote `{}` shutting down.", addr),
                                                ThreadIdentification::new(io_thread_idx, crate::worker::ThreadType::IO),
                                            ))
                                            .unwrap();
                                        
                                        break;
                                    },
                                }
                            }
                        });

                        io_thread_idx += 1;

                        // Spawn client handler
                        tokio::spawn(async move {
                            let workers = workers.clone();
                            let current_jobs = current_jobs.clone();
                            let dep_path = dep_path.clone();
                            let remote_url = remote_url.clone();
                            let http_client = http_client.clone();
                            let thread_id = ThreadIdentification::new(
                                io_thread_idx,
                                crate::worker::ThreadType::IO,
                            );
                            let mut client_handle = client_recv;

                            loop {
                                if let Ok(msg_len) = client_handle.read_u32().await {
                                    let mut msg_buf = vec![0; msg_len as usize];

                                    match client_handle.read_exact(&mut msg_buf).await {
                                        // Handle the message sent by the user
                                        Ok(_) => {
                                            match common::rmp_serde::from_slice::<DependencyRequest>(
                                                &msg_buf,
                                            ) {
                                                Ok(request) => {
                                                    let dep_path = construct_dependency_path(
                                                        (*dep_path).clone(),
                                                        request.name.clone(),
                                                        request.version.clone(),
                                                    );

                                                    // Check if we already have the dependency downloaded
                                                    // Implement hash checking so that it enusres that dependencies are always correctly fetched from remotes
                                                    // Preferrably with an api call
                                                    if let Err(_) =
                                                        tokio::fs::metadata(&dep_path).await
                                                    {
                                                        // Send request to server
                                                        let response = net::request_dependency(
                                                            http_client.clone(),
                                                            &remote_url.clone(),
                                                            request.name.clone(),
                                                            request.version.clone(),
                                                        )
                                                        .await
                                                        .unwrap();

                                                        // Get response body from server
                                                        let req_body =
                                                            response.bytes().await.unwrap();

                                                        // Decompress bytes
                                                        let deser_bytes =
                                                            decompress_bytes(&req_body).unwrap();

                                                        // Serialize bytes
                                                        let dependency =
                                                            rmp_serde::from_slice::<Dependency>(
                                                                &deser_bytes,
                                                            )
                                                            .unwrap();

                                                        // Write dependency to folder
                                                        if let Err(err) = write_zip_to_fs_async(
                                                            dep_path.clone(),
                                                            zip_from_bytes(Cursor::new(
                                                                dependency.source,
                                                            ))
                                                            .unwrap(),
                                                        )
                                                        .await
                                                        {
                                                            ui_sender_out_clone.send((format!("An error occured while writing dependency `{}({})` to fs: {err}", request.name.clone(), request.version.clone()), thread_id)).unwrap();
                                                            break;
                                                        };
                                                    }

                                                    current_jobs.in_progress.push(CompileJob {
                                                        remote_address: addr.clone(),
                                                        target_triple: request.target_triple,
                                                        features: OrdSet::from_vec(
                                                            request.features,
                                                        ),
                                                        depdendency_path: dep_path,
                                                        cpu_features: request.cpu_features,
                                                        cpu_name: request.cpu_name,
                                                        flags_passed_in: request.flags_passed_in,
                                                    });

                                                    // Wake workers
                                                    workers.iter().for_each(|worker_handle| {
                                                        worker_handle.1.unpark()
                                                    });
                                                },
                                                Err(error) => {
                                                    ui_sender_out_clone.send((format!("Invalid request body from `{}`: {error}", client_addr), thread_id)).unwrap();
                                                    break;
                                                },
                                            }
                                        },
                                        Err(err) => {
                                            ui_sender_out_clone
                                                .send((err.to_string(), thread_id))
                                                .unwrap();

                                            break;
                                        },
                                    }
                                }
                                else {
                                    ui_sender_out_clone
                                        .send((
                                            format!(
                                                "Failed to receive message from `{}`, disconnecting...",
                                                client_addr
                                            ),
                                            thread_id,
                                        ))
                                        .unwrap();

                                    break;
                                }
                            }
                        });

                        io_thread_idx += 1;
                    },
                    Err(error) => {
                        ui_sender_in
                            .send((
                                error.to_string(),
                                ThreadIdentification::new(0, crate::worker::ThreadType::IO),
                            ))
                            .unwrap();
                    },
                }
            }
        });

        Ok(())
    }
}
