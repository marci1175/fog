use std::{
    collections::HashMap,
    net::Ipv6Addr,
    rc::Rc,
    sync::{Arc, atomic::AtomicUsize},
};

use crossbeam::{
    atomic::AtomicCell,
    channel::{Sender, bounded},
    queue::ArrayQueue,
};
use dashmap::DashMap;
use fog_common::{anyhow, dependency::DependencyInfo, parking_lot::Mutex, tokio};

use crate::worker::{JobHandler, ThreadIdentification};

#[derive(Debug, Clone)]
pub struct ServerState
{
    pub port: u32,
    pub worker_thread_notifier: HashMap<usize, Sender<()>>,
    pub loadbalancer: DashMap<usize, Arc<JobHandler>>,
    pub connected_clients: Arc<DashMap<Ipv6Addr, String>>,
}

impl Default for ServerState
{
    fn default() -> Self
    {
        Self {
            port: 0,
            worker_thread_notifier: HashMap::new(),
            loadbalancer: DashMap::new(),
            connected_clients: Arc::new(DashMap::new()),
        }
    }
}


impl ServerState
{
    pub fn new(port: u32) -> Self
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

        tokio::spawn(async move { loop {} });
        tokio::spawn(async move { loop {} });

        let available_cores_left = available_cores.checked_sub(2).unwrap_or_else(|| 1);

        self.create_workers(available_cores_left, ui_sender)?;

        Ok(())
    }
}
