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
    pub status: Option<String>,
    pub subscribed_at: DateTime<Utc>,
}

impl Subscriptions {
    pub fn new(email: String, name: String) -> Self {
        Self {
            id: Uuid::now_v7(),
            email,
            name,
            subscribed_at: Utc::now(),
            status: Some("confirmed".to_string()),
        }
    }
}
