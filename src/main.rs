use axum_newsletter::{
    configuration::get_configuration, startup::run, telemetry::setup_tracing,
};
use diesel_async::{
    pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection,
};
use secrecy::ExposeSecret;
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    setup_tracing("info", std::io::stdout);
    let configuration =
        get_configuration().expect("Could not read configuration file");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!(
        "Server started listening on port {}",
        listener.local_addr().unwrap()
    );
    let pool_manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(
        configuration.database.connection_string().expose_secret(),
    );
    let pool = Pool::builder(pool_manager)
        .build()
        .expect("Connection failed");
    run(listener, pool).await
}
