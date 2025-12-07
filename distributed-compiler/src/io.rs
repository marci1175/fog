use std::{
    collections::HashMap,
    net::{Ipv6Addr, SocketAddr},
    sync::Arc,
    thread::Thread,
};

use common::{
    anyhow, crossbeam::channel::bounded, distributed_compiler::DependencyRequest, tokio::{self, io::AsyncReadExt, select}
};
use dashmap::DashMap;

use crate::worker::{FinishedJobQueue, JobHandler, JobQueue, ThreadIdentification};

#[derive(Debug, Clone)]
pub struct ServerState
{
    pub port: u16,
    pub dependency_manager_url: String,
    pub worker_thread_notifier: HashMap<ThreadIdentification, Thread>,
    pub job_handler: Arc<JobHandler>,
    pub connected_clients: Arc<DashMap<SocketAddr, String>>,
}

impl Default for ServerState
{
    fn default() -> Self
    {
        Self {
            port: 0,
            dependency_manager_url: "http://[::1]:3004".into(),
            worker_thread_notifier: HashMap::new(),
            job_handler: Arc::new(JobHandler::new(JobQueue::new(), FinishedJobQueue::new())),
            connected_clients: Arc::new(DashMap::new()),
        }
    }
}

impl ServerState
{
    pub fn new(port: u16, dependency_manager_url: String) -> Self
    {
        Self {
            port,
            dependency_manager_url,
            ..Default::default()
        }
    }

    /// Initialize threads for the server
    pub fn initialize_server(&mut self) -> anyhow::Result<()>
    {
        let (ui_sender, ui_recv) = bounded::<(String, ThreadIdentification)>(255);

        let available_cores = std::thread::available_parallelism()?.get();

        let available_cores_left = available_cores.checked_sub(2).unwrap_or(1);

        self.create_workers(available_cores_left, ui_sender.clone())?;

        let port = self.port;

        let ui_sender_in = ui_sender.clone();
        let ui_sender_out = ui_sender.clone();

        let connected_clients_handle = self.connected_clients.clone();

        let ui_sender_out_clone = ui_sender_out.clone();
        // Inbound
        tokio::spawn(async move {
            loop {
                // Bind listener to local on specified port
                let listener =
                    tokio::net::TcpListener::bind((Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), port))
                        .await
                        .unwrap();

                // Clone sender channel so that we can send messages to the frontend
                let ui_sender_out_clone = ui_sender_out_clone.clone();

                match listener.accept().await {
                    Ok((stream, addr)) => {
                        connected_clients_handle.insert(addr, "Client information".into());

                        // Spawn client handler
                        tokio::spawn(async move {
                            let mut client_handle = stream;

                            // Handle client requests
                            loop {
                                select! {
                                    Ok(msg_len) = client_handle.read_u32() => {
                                        let mut msg_buf = vec![0; msg_len as usize];

                                        match client_handle.read_exact(&mut msg_buf).await  {
                                            // Handle the message sent by the user
                                            Ok(_) => {
                                                let request = common::rmp_serde::from_slice::<DependencyRequest>(&msg_buf);

                                                
                                            },
                                            Err(err) => {
                                                ui_sender_out_clone.send((err.to_string(), ThreadIdentification::new(0))).unwrap();
                                            },
                                        }
                                    }
                                }
                            }
                        });
                    },
                    Err(error) => {
                        ui_sender_in
                            .send((error.to_string(), ThreadIdentification::new(0)))
                            .unwrap();
                    },
                }
            }
        });

        // Outbound
        tokio::spawn(async move { loop {} });

        Ok(())
    }
}
