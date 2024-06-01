use axum::http::StatusCode;
#[tracing::instrument(name = "Health check")]
pub async fn health_check() -> StatusCode {
    tracing::info!("Health check requested.");
    StatusCode::OK
}
