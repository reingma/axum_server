use axum::extract::FromRef;
use diesel_async::pooled_connection::deadpool::{Pool, PoolError};
use diesel_async::{pooled_connection::deadpool::Object, AsyncPgConnection};

pub mod diesel_configuration;
pub mod queries;

pub type DatabaseConnection = Object<AsyncPgConnection>;
pub type DatabaseConnectionPool = Pool<AsyncPgConnection>;

#[tracing::instrument(
    name = "Retrieving database connection from pool.",
    skip(pool)
)]
pub async fn get_connection(
    pool: Pool<AsyncPgConnection>,
) -> Result<DatabaseConnection, PoolError> {
    match pool.get().await {
        Ok(conn) => {
            tracing::info!("Connection established.");
            Ok(conn)
        }
        Err(e) => {
            tracing::error!(
                "Could not get connection from pool, with error: {:?}",
                e
            );
            Err(e)
        }
    }
}
pub use diesel_configuration::create_connection_pool;

use crate::startup::ApplicationState;

impl FromRef<ApplicationState> for DatabaseConnectionPool {
    fn from_ref(input: &ApplicationState) -> Self {
        input.database_pool.clone()
    }
}
