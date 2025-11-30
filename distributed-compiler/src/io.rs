use std::{collections::HashMap, net::{Ipv6Addr, SocketAddr}, sync::Arc, thread::Thread};

use crossbeam::channel::{Sender, bounded};
use dashmap::DashMap;
use common::{anyhow, tokio};

use crate::worker::{FinishedJobQueue, JobHandler, JobQueue, ThreadIdentification};

#[derive(Debug, Clone)]
pub struct ServerState
{
    pub port: u16,
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
            worker_thread_notifier: HashMap::new(),
            job_handler: Arc::new(JobHandler::new(JobQueue::new(), FinishedJobQueue::new())),
            connected_clients: Arc::new(DashMap::new()),
        }
    }
}

impl ServerState
{
    pub fn new(port: u16) -> Self
    {
        Self {
            port,
            ..Default::default()
        }
    }

    /// Initialize threads for the server
    pub fn initialize_server(&mut self) -> anyhow::Result<()>
    {
        let (ui_sender, ui_recv) = bounded::<(String, ThreadIdentification)>(255);

        let available_cores = std::thread::available_parallelism()?.get();

        let available_cores_left = available_cores.checked_sub(2).unwrap_or_else(|| 1);

        self.create_workers(available_cores_left, ui_sender.clone())?;

        let port = self.port;

        let ui_sender_in = ui_sender.clone();
        let ui_sender_out = ui_sender.clone();

        let connected_clients_handle = self.connected_clients.clone();

        // Inbound
        tokio::spawn(async move { loop {
            // Bind listener to local on specified port
            let listener =  tokio::net::TcpListener::bind((Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), port)).await.unwrap();
            
            match listener.accept().await {
                Ok((stream, addr)) => {
                    connected_clients_handle.insert(addr, "Client information".into());

                    // Spawn client handler
                    tokio::spawn(async move {
                        let client_handle = stream;

                        // Handle client requests
                        loop {
                            
                        }
                    });
                },
                Err(error) => {
                    ui_sender_in.send((error.to_string(), ThreadIdentification::new(0))).unwrap();
                },
            }
        } });
        
        // Outbound
        tokio::spawn(async move { loop {} });

        Ok(())
    }
}
