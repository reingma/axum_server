use axum_newsletter::{configuration::get_configuration, startup::run};
use diesel_async::{
    pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection,
};
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let configuration =
        get_configuration().expect("Could not read configuration file");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = tokio::net::TcpListener::bind(address).await?;
    let pool_manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(
        configuration.database.connection_string(),
    );
    let pool = Pool::builder(pool_manager)
        .build()
        .expect("Connection failed");
    run(listener, pool).await
}
