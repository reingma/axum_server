use axum::http::StatusCode;
use uuid::Uuid;
#[tracing::instrument(
    name = "Health check",
    fields(
        request_id = %Uuid::now_v7(),
    )
)]
pub async fn health_check() -> StatusCode {
    tracing::info!("Health check requested.");
    StatusCode::OK
}
