pub mod configuration;
pub mod models;
pub mod routes;
pub mod schema;
pub mod startup;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;
pub type PgPool = Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;
