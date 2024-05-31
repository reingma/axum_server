use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Insertable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::subscriptions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Subscriptions {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub status: String,
    pub subscribed_at: DateTime<Utc>,
}

impl Subscriptions {
    pub fn new(email: String, name: String) -> Self {
        Self {
            id: Uuid::now_v7(),
            email,
            name,
            subscribed_at: Utc::now(),
            status: "pending_confirmation".to_string(),
        }
    }
}

#[derive(Insertable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::subscription_tokens)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SubscriptionTokens {
    pub subscriber_id: Uuid,
    pub subscription_token: String,
    pub generated_at: DateTime<Utc>,
}

impl SubscriptionTokens {
    pub fn new(token: &str, sub_id: &Uuid) -> Self {
        Self {
            subscriber_id: *sub_id,
            subscription_token: token.to_string(),
            generated_at: Utc::now(),
        }
    }
}

#[derive(Insertable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Users {
    pub user_id: Uuid,
    pub username: String,
    pub password_hash: String,
}

impl Users {
    pub fn new(user_id: Uuid, username: &str, password_hash: &str) -> Self {
        Self {
            user_id,
            username: username.to_string(),
            password_hash: password_hash.to_string(),
        }
    }
}
