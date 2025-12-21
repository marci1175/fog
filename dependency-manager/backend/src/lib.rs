use std::path::PathBuf;

use common::{
    anyhow::Result,
    base64::{
        self,
        alphabet::{self},
        engine::GeneralPurposeConfig,
    },
    dependency_manager::ServerState,
};
use diesel::{
    PgConnection,
    r2d2::{self, ConnectionManager},
};

pub mod api;
pub mod models;
pub mod schema;

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
