use std::net::SocketAddr;

use axum::{Router, body::Body, http::{Request, Response, StatusCode}, middleware::{self, Next}, serve};
use common::{anyhow, tokio::{self, net::TcpListener}};
use env_logger::Env;

async fn log_request(request: Request<Body>, next: Next) -> Result<Response<Body>, StatusCode> {
    let method = request.method().clone();
    let uri = request.uri().clone();

    println!("> Incoming: {} {}", method, uri);
    println!("> Headers: {:?}", request.headers());

    let response = next.run(request).await;

    println!("< Response status: {}", response.status());

    Ok(response)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Establish connection with the database
    let servere_state = establish_state();

    // Start up the webserver
    let router = Router::new()
        .layer(middleware::from_fn(log_request))
        .with_state(servere_state);

    let listener = TcpListener::bind("[::1]:3004").await?;

    serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

#[derive(Debug, Clone)]
pub struct ServerState {  }

fn establish_state() -> ServerState {
    ServerState {  }
}