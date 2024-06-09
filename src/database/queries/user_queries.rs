use crate::schema;
use crate::{database::DatabaseConnection, schema::users::dsl::*};
use anyhow::Context;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

#[tracing::instrument(name = "Get username query", skip(connection))]
pub async fn get_username(
    connection: &mut DatabaseConnection,
    id: Uuid,
) -> Result<String, anyhow::Error> {
    users
        .filter(schema::users::user_id.eq(id))
        .select(schema::users::username)
        .first(connection)
        .await
        .context("Failed to retrieve username from database")
}
