use axum::response::Html;

pub async fn greet() -> Html<&'static str> {
    Html("<h1>Hello</h1>")
}
