use crate::database::DatabaseConnection;
use crate::domain::NewSubscriber;
use crate::schema;
use crate::{models::Subscriptions, schema::subscriptions};
use diesel::result::Error;
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

#[tracing::instrument(
    name = "Inserting subscriber into databse.",
    skip(subscriber_data, connection)
)]
pub async fn insert_subscriber(
    connection: &mut DatabaseConnection,
    subscriber_data: &NewSubscriber,
) -> Result<Uuid, Error> {
    let subscription_entry = Subscriptions::new(
        subscriber_data.email.as_ref().to_string(),
        subscriber_data.name.as_ref().to_string(),
    );
    match diesel::insert_into(subscriptions::table)
        .values(&subscription_entry)
        .on_conflict(schema::subscriptions::email)
        .do_update()
        .set(schema::subscriptions::email.eq(&subscription_entry.email))
        .returning(schema::subscriptions::id)
        .get_result::<Uuid>(connection)
        .await
    {
        Ok(id) => {
            tracing::info!("New subscriber details have been saved");
            Ok(id)
        }
        Err(e) => {
            tracing::error!("Failed to execute query {:?}", e);
            Err(e)
        }
    }
}
