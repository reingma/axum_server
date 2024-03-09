use std::sync::Arc;

use axum_newsletter::{
    configuration::get_configuration, database::create_connection_pool,
    email_client::EmailClient, startup::run, telemetry::setup_tracing,
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
    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        &configuration.email_client.base_url,
        configuration
            .email_client
            .sender()
            .expect("Sender email invalid"),
        configuration.email_client.api_token,
        timeout,
    );

    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!(
        "Server started listening on port {}",
        listener.local_addr().unwrap()
    );
    let pool = create_connection_pool(
        configuration.database.connection_string().expose_secret(),
    );

    run(listener, pool, Arc::new(email_client)).await
}
