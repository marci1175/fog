use std::{collections::HashMap, net::SocketAddr};

use common::{
    anyhow::Result,
    compiler::HostInformation,
    dependency::DependencyInfo,
    distributed_compiler::{DependencyRequest, DistributedCompilerWorker},
    error::dependency::DependencyError,
    rmp_serde,
    tokio::{
        self,
        io::{AsyncReadExt, AsyncWriteExt},
        net::{TcpSocket, TcpStream},
        select, spawn,
        sync::mpsc::Sender,
    },
};

pub fn create_remote_list(
    remotes: Vec<DistributedCompilerWorker>,
    host_info: HostInformation,
) -> HashMap<String, (String, Sender<(String, DependencyInfo)>)>
{
    let mut remote_list = HashMap::new();

    for remote in remotes {
        let host_info = host_info.clone();

        let (sender, mut recv) = tokio::sync::mpsc::channel::<(String, DependencyInfo)>(255);

        println!(
            "Contacting remote compiler `{}` at `{}`",
            remote.name.clone(),
            remote.address.clone()
        );

        remote_list.insert(remote.name, (remote.address.clone(), sender));

        // Create a remote handler thread
        spawn(async move {
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
                    // Receive and handle the pre-compiled files
                    Ok(len) = tcp_handle.read_u32() => {

                    }
                }
            }
        });
    }

    remote_list
}

pub fn dependency_requester(
    dependencies: &mut HashMap<String, DependencyInfo>,
    remote_handlers: &HashMap<String, (String, Sender<(String, DependencyInfo)>)>,
) -> Result<()>
{
    for dep in dependencies.drain() {
        request_dependency(dep, remote_handlers)?;
    }

    Ok(())
}

pub fn request_dependency(
    dependency: (String, DependencyInfo),
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
            println!(
                "Requesting dependency `{}({})` from remote compiler `{}`",
                dependency.0.clone(),
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
