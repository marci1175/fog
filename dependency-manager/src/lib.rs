use std::path::PathBuf;

use base64::{
    alphabet::{self, Alphabet},
    engine::GeneralPurposeConfig,
};
use common::anyhow::Result;
use diesel::{
    PgConnection,
    r2d2::{self, ConnectionManager},
};

pub mod api;
pub mod models;
pub mod schema;

#[derive(Debug, Clone)]
pub struct ServerState
{
    pub db_connection: r2d2::Pool<ConnectionManager<PgConnection>>,
    pub deps_path: PathBuf,
    pub base64_engine: base64::engine::GeneralPurpose,
}

pub fn establish_state(database_url: &str, deps_path: PathBuf) -> Result<ServerState>
{
    let pg_pool: r2d2::Pool<ConnectionManager<PgConnection>> =
        r2d2::Builder::new().build(ConnectionManager::new(database_url))?;

    Ok(ServerState {
        db_connection: pg_pool,
        deps_path,
        base64_engine: base64::engine::GeneralPurpose::new(
            &alphabet::STANDARD,
            GeneralPurposeConfig::default(),
        ),
    })
}
