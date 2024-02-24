use axum::response::Html;
use uuid::Uuid;

#[tracing::instrument(
    name = "User greeting",
    fields(
        request_id = %Uuid::now_v7(),
    )
)]
pub async fn greet() -> Html<&'static str> {
    tracing::info!("Greet requested");
    Html("<h1>Hello</h1>")
}
