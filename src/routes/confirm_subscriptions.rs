use axum::{
    extract::{Query, State},
    http::StatusCode,
};

use crate::database::{
    queries::{confirm_subscriber, get_subscriber_id_for_token},
    DatabaseConnectionPool,
};

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(
    name = "Confirming Pending Subscriber",
    skip(parameters, database_pool)
)]
pub async fn confirm(
    parameters: Query<Parameters>,
    State(database_pool): State<DatabaseConnectionPool>,
) -> StatusCode {
    let mut connection = crate::database::get_connection(database_pool).await;
    let subscriber_id = match get_subscriber_id_for_token(
        &mut connection,
        &parameters.subscription_token,
    )
    .await
    {
        Ok(subscriber_id) => subscriber_id,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };
    match subscriber_id {
        None => StatusCode::UNAUTHORIZED,
        Some(id) => {
            if confirm_subscriber(&mut connection, &id).await.is_err() {
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
            StatusCode::OK
        }
    }
}
