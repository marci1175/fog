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
}

pub fn establish_state(database_url: &str) -> Result<ServerState>
{
    let pg_pool: r2d2::Pool<ConnectionManager<PgConnection>> =
        r2d2::Builder::new().build(ConnectionManager::new(database_url))?;

    Ok(ServerState {
        db_connection: pg_pool,
    })
}
