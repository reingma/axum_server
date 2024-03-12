use axum_newsletter::{
    configuration::get_configuration, startup::Application,
    telemetry::setup_tracing,
};
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    setup_tracing("axum_newsletter", "info", std::io::stdout);
    let configuration =
        get_configuration().expect("Could not read configuration file");

    let app = Application::build(configuration).await?;
    app.run_until_stopped().await?;
    Ok(())
}
