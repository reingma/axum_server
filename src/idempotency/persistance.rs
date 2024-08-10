use crate::database::DatabaseConnection;
use crate::models::{HeaderPair, Idempotency};

use super::IdempotencyKey;
use crate::schema::idempotency::dsl::*;
use axum::http::{Response, StatusCode};
use diesel::prelude::*;
use diesel::SelectableHelper;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

pub async fn get_saved_response(
    connection: &mut DatabaseConnection,
    key: IdempotencyKey,
    id: Uuid,
) -> Result<Option<Response<Vec<u8>>>, anyhow::Error> {
    let key: String = String::from(key);
    let saved_response: Option<Idempotency> = idempotency
        // .filter(diesel::BoolExpressionMethods::and(
        //     idempotency_key.eq(key.into()),
        //     user_id.eq(id),
        // ))
        .filter(idempotency_key.eq(key).and(user_id.eq(id)))
        .select(Idempotency::as_select())
        .first(connection)
        .await
        .optional()?;
    if let Some(r) = saved_response {
        let status_code =
            StatusCode::from_u16(r.request.response_status_code.try_into()?)?;
        let mut response = Response::builder().status(status_code);
        for HeaderPair { name, value } in r.request.response_headers {
            response = response.header(name, value);
        }
        let response = response.body(r.request.response_body)?;
        Ok(Some(response))
    } else {
        Ok(None)
    }
}
