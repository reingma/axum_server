use diesel_async::pooled_connection::deadpool::Pool;
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
) -> DatabaseConnection {
    match pool.get().await {
        Ok(conn) => {
            tracing::info!("Connection established.");
            conn
        }
        Err(e) => {
            tracing::error!(
                "Could not get connection from pool, with error: {:?}",
                e
            );
            panic!("Failed to establish connection.");
        }
    }
}
pub use diesel_configuration::create_connection_pool;
