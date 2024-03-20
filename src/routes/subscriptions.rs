use crate::{
    database::queries, database::queries::insert_subscriber,
    domain::NewSubscriber, email_client::send::send_confirmation_email,
    startup::ApplicationState,
};
use axum::{extract::State, http::StatusCode, Form};
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::AsyncConnection;
use rand::distributions::Alphanumeric;
use rand::thread_rng;
use rand::Rng;

#[derive(serde::Deserialize)]
pub struct Subscriber {
    pub name: String,
    pub email: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(subscriber,app_state),
    fields(
        subscriber_email = %subscriber.email,
        subscriber_name = %subscriber.name
    )
)]
pub async fn subscriptions(
    State(app_state): State<ApplicationState>,
    Form(subscriber): Form<Subscriber>,
) -> StatusCode {
    let new_subscriber: NewSubscriber = match subscriber.try_into() {
        Ok(new_subscriber) => new_subscriber,
        Err(_) => return StatusCode::BAD_REQUEST,
    };
    tracing::info!("Adding a new subscriber to the database.");

    let mut connection =
        crate::database::get_connection(app_state.database_pool).await;

    let subscription_token = generate_subscription_token();
    let copy = subscription_token.clone();
    if connection
        .transaction::<_, diesel::result::Error, _>(|conn| {
            async move {
                let subscriber_id =
                    insert_subscriber(conn, &new_subscriber).await?;

                queries::store_token(conn, &copy, &subscriber_id).await?;
                send_confirmation_email(
                    &app_state.email_client,
                    new_subscriber,
                    &app_state.base_url,
                    &subscription_token,
                )
                .await
                .map_err(|_| diesel::result::Error::RollbackTransaction)?;
                Ok(())
            }
            .scope_boxed()
        })
        .await
        .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    return StatusCode::OK;
}

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}
