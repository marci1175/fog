use axum::{
    Router,
    body::Body,
    http::{Request, Response, StatusCode},
    middleware::{self, Next},
    response::{Html, Redirect},
    routing::{get, post},
    serve,
};
use common::{
    anyhow, dotenvy, tokio::{self, net::TcpListener}
};
use dependency_manager::{
    api::manager::{fetch_dependency_information, fetch_dependency_source},
    establish_state,
};
use env_logger::Env;
use std::net::SocketAddr;

async fn log_request(request: Request<Body>, next: Next) -> Result<Response<Body>, StatusCode>
{
    let method = request.method().clone();
    let uri = request.uri().clone();

    println!("> Incoming: {} {}", method, uri);
    println!("> Headers: {:?}", request.headers());

    let response = next.run(request).await;

    println!("< Response status: {}", response.status());

    Ok(response)
}

#[tokio::main]
async fn main() -> anyhow::Result<()>
{
    dotenvy::dotenv()?;

    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let database_url = std::env::var("DATABASE_URL")?;

    // Establish connection with the database
    let servere_state = establish_state(&database_url)?;

    // Start up the webserver
    let router = Router::new()
        .route("/", get(redirect_to_project))
        .route(
            "/api/fetch_dependency_info",
            get(fetch_dependency_information),
        )
        .route("/api/fetch_dependency", get(fetch_dependency_source))
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

pub async fn redirect_to_project() -> Result<Redirect, StatusCode>
{
    Ok(Redirect::permanent("https://github.com/marci1175/fog"))
}
