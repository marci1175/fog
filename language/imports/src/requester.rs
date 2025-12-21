use std::{collections::HashMap, fs, io::Cursor, path::PathBuf, sync::Arc};

use common::{
    anyhow::Result,
    compiler::{HostInformation, ProjectConfig},
    compression::{decompress_bytes, unzip_from_bytes, write_zip_to_fs},
    dashmap::DashMap,
    dependency::DependencyInfo,
    distributed_compiler::{DependencyRequest, DistributedCompilerWorker, FinishedJob},
    error::dependency::DependencyError,
    parking_lot::Mutex,
    parser::FunctionSignature,
    rmp_serde,
    tokio::{
        self,
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpStream,
        select, spawn,
        sync::mpsc::Sender,
        task::JoinHandle,
    },
    tracing::info,
    ty::OrdSet,
};
use parser::{parser_instance::Parser, tokenizer::tokenize};

pub fn create_remote_list(
    remotes: Vec<DistributedCompilerWorker>,
    host_info: HostInformation,
    deps: Arc<DashMap<Vec<String>, FunctionSignature>>,
    root_path: PathBuf,
) -> (
    HashMap<String, (String, Sender<(String, DependencyInfo)>)>,
    Vec<JoinHandle<()>>,
)
{
    let mut threads = Vec::new();
    let mut remote_list = HashMap::new();

    for remote in remotes {
        let host_info = host_info.clone();

        let (sender, mut recv) = tokio::sync::mpsc::channel::<(String, DependencyInfo)>(255);

        info!(
            "Contacting remote compiler `{}` at `{}`",
            remote.name.clone(),
            remote.address.clone()
        );

        remote_list.insert(remote.name, (remote.address.clone(), sender));

        let root_path = root_path.clone();

        // Create a remote handler thread
        let thread_handle = spawn(async move {
            let mut tcp_handle = TcpStream::connect(remote.address).await.unwrap();
            let host_info = host_info.clone();

            loop {
                select! {
                    Some((name, info)) = recv.recv() => {
                        // Send the request to the remote
                        let packet = rmp_serde::to_vec(&DependencyRequest {name, version: info.version, features: info.features, target_triple: host_info.target_triple.clone(), cpu_features: host_info.cpu_features.clone(), cpu_name: host_info.cpu_name.clone(), flags_passed_in: String::new() }).unwrap();

                        // Send len
                        tcp_handle.write_all(&(packet.len() as u32).to_be_bytes()).await.unwrap();

                        // Send packet
                        tcp_handle.write_all(&packet).await.unwrap();
                    }
                    // Receive and handle the pre-compiled files and break the loop
                    Ok(len) = tcp_handle.read_u32() => {
                        let mut packet_buf = vec![0; len as usize];

                        tcp_handle.read_exact(&mut packet_buf).await.unwrap();

                        let decomp_bytes = decompress_bytes(&packet_buf).unwrap();

                        let finished_job = rmp_serde::from_slice::<FinishedJob>(&decomp_bytes).unwrap();

                        let zip = unzip_from_bytes(Cursor::new(finished_job.artifacts_zip_bytes)).unwrap();

                        // Construct zip fs path
                        let dependency_path = format!("{}\\remote_compile\\{}\\", root_path.display(), finished_job.info.dependency_name.clone());

                        // Write zip contents to fs
                        write_zip_to_fs(&(dependency_path.clone().into()), zip).unwrap();

                        // Quit thread
                        break;
                    }
                }
            }
        });

        threads.push(thread_handle);
    }
    (remote_list, threads)
}

pub fn dependency_requester(
    dependencies: &HashMap<String, DependencyInfo>,
    remote_handlers: &HashMap<String, (String, Sender<(String, DependencyInfo)>)>,
) -> Result<()>
{
    for dep in dependencies.iter() {
        request_dependency(dep, remote_handlers)?;
    }

    Ok(())
}

pub fn request_dependency(
    dependency: (&String, &DependencyInfo),
    remote_handlers: &HashMap<String, (String, Sender<(String, DependencyInfo)>)>,
) -> Result<()>
{
    // If there was no remote compiler set and the folder was not found return that the dependency is not found.
    let remote_compiler = dependency
        .1
        .remote
        .as_ref()
        .ok_or(DependencyError::DependencyNotFound(dependency.0.clone()))?
        .clone();

    match remote_handlers.get(&remote_compiler) {
        Some((peer_addr, thread)) => {
            let dependency = (dependency.0.clone(), dependency.1.clone());

            info!(
                "Requesting dependency `{}({})` from remote compiler `{}`",
                dependency.0,
                dependency.1.version.clone(),
                peer_addr
            );

            // If we cant send a message the thread either panicked to some kind of error occured in the io
            thread
                .try_send(dependency.clone())
                .map_err(|_| DependencyError::FailedConnectingToRemote(remote_compiler))?;
        },
        None => {
            return Err(DependencyError::InvalidRemoteCompiler(remote_compiler).into());
        },
    }

    Ok(())
}
