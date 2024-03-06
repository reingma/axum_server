use axum_newsletter::{
    configuration::get_configuration, database::create_connection_pool,
    startup::run, telemetry::setup_tracing,
};
use secrecy::ExposeSecret;
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    setup_tracing("axum_newsletter", "info", std::io::stdout);
    let configuration =
        get_configuration().expect("Could not read configuration file");
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!(
        "Server started listening on port {}",
        listener.local_addr().unwrap()
    );
    tracing::info!(
        "Database connected to: {}",
        configuration.database.connection_string().expose_secret()
    );
    let pool = create_connection_pool(
        configuration.database.connection_string().expose_secret(),
    );

    run(listener, pool).await
}
