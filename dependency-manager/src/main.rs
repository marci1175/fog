use common::{
    anyhow,
    axum::{
        Router,
        body::Body,
        http::{Request, Response, StatusCode},
        middleware::{self, Next},
        routing::{get, post},
        serve,
    },
    dotenvy,
    tokio::{self, net::TcpListener},
};
use dependency_manager::{
    api::manager::{fetch_dependency_information, fetch_dependency_source, publish_dependency},
    establish_state,
};
use env_logger::Env;
use std::{env::current_dir, fs::create_dir_all, net::SocketAddr, path::PathBuf};

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
    let deps_path = PathBuf::from(format!(
        "{}\\{}",
        current_dir()?.display(),
        std::env::var("DEPENDENCY_PATH")?
    ));

    // Ignore error, since it will return an error if the folder already exists.
    let _ = create_dir_all(&deps_path);

    // Establish connection with the database
    let servere_state = establish_state(&database_url, deps_path)?;

    // Start up the webserver
    let router = Router::new()
        .route("/", get(reply_ok))
        .route(
            "/fetch_dependency_information",
            get(fetch_dependency_information),
        )
        .route("/fetch_dependency", get(fetch_dependency_source))
        .route("/publish_dependency", post(publish_dependency))
        .layer(middleware::from_fn(log_request))
        .with_state(servere_state);

    let listener = TcpListener::bind("[::1]:3004").await?;

    println!("Starting dependency manager service...");

    serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

pub async fn reply_ok() -> StatusCode
{
    StatusCode::OK
}
