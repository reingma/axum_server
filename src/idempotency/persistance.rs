use crate::database::DatabaseConnection;
use crate::models::{HeaderPair, Idempotency};

use super::IdempotencyKey;
use crate::schema::idempotency::dsl::*;
use axum::body;
use axum::http::{Response, StatusCode};
use diesel::prelude::*;
use diesel::SelectableHelper;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

pub async fn get_saved_response(
    connection: &mut DatabaseConnection,
    key: &IdempotencyKey,
    id: Uuid,
) -> Result<Option<Response<body::Body>>, anyhow::Error> {
    let saved_response: Option<Idempotency> = idempotency
        .filter(idempotency_key.eq(key.as_ref()).and(user_id.eq(id)))
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
        let response: Response<body::Body> =
            response.body(body::Body::from(r.request.response_body))?;
        Ok(Some(response))
    } else {
        Ok(None)
    }
}

pub async fn save_response(
    connection: &mut DatabaseConnection,
    key: &IdempotencyKey,
    id: Uuid,
    response: Response<body::Body>,
) -> Result<Response<body::Body>, anyhow::Error> {
    let (parts, response_body) = response.into_parts();
    let body_bytes = body::to_bytes(response_body, usize::MAX).await?;
    let (header_names, header_values): (Vec<String>, Vec<Vec<u8>>) = parts
        .headers
        .iter()
        .map(|(name, value)| {
            (name.as_str().to_owned(), value.as_bytes().to_owned())
        })
        .unzip();
    let status_code: i16 = parts.status.as_u16().try_into()?;
    // The composite value is assembled in SQL rather than bound through the
    // `ToSql` impl of `HttpRequest`: diesel-async cannot resolve the oid of a
    // custom type nested inside another one (`header_pair[]` inside
    // `http_request`) and fails with "Failed to find a type oid".
    diesel::sql_query(
        "INSERT INTO idempotency (user_id, idempotency_key, request, created_at) \
         VALUES ($1, $2, ROW($3, \
             ARRAY(SELECT ROW(n, v)::header_pair \
                   FROM unnest($4::text[], $5::bytea[]) AS h(n, v)), \
             $6, $7)::http_request, now())",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Text, _>(key.as_ref())
    .bind::<diesel::sql_types::SmallInt, _>(status_code)
    .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(header_names)
    .bind::<diesel::sql_types::Array<diesel::sql_types::Bytea>, _>(
        header_values,
    )
    .bind::<diesel::sql_types::Bytea, _>(body_bytes.as_ref())
    .bind::<diesel::sql_types::Text, _>(format!("{:?}", parts.version))
    .execute(connection)
    .await?;
    Ok(Response::from_parts(parts, body::Body::from(body_bytes)))
}
