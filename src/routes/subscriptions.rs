use crate::{models::Subscriptions, schema::subscriptions};
use axum::{extract::State, http::StatusCode, Form};
use diesel::prelude::*;
use diesel_async::{
    pooled_connection::deadpool::Pool, AsyncPgConnection, RunQueryDsl,
};

#[derive(serde::Deserialize)]
pub struct Subscriber {
    name: String,
    email: String,
}

pub async fn subscriptions(
    State(pool): State<Pool<AsyncPgConnection>>,
    Form(subscriber): Form<Subscriber>,
) -> StatusCode {
    let mut connection = pool.get().await.expect("Failed to get connection");
    let subscription_entry =
        Subscriptions::new(subscriber.email, subscriber.name);
    match diesel::insert_into(subscriptions::table)
        .values(subscription_entry)
        .execute(&mut connection)
        .await
    {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            println!("Failed to execute query: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
