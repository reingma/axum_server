use chrono::{DateTime, Utc};
use diesel::{
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::Pg,
    prelude::*,
    serialize::{ToSql, WriteTuple},
    sql_types::{Array, Bytea, Record, SmallInt, Text},
};
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

#[derive(Insertable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::idempotency)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Idempotency {
    pub user_id: Uuid,
    pub idempotency_key: String,
    pub request: HttpRequest,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromSqlRow, AsExpression, Clone)]
#[diesel(sql_type = crate::schema::sql_types::HeaderPair)]
pub struct HeaderPair {
    pub name: String,
    pub value: Vec<u8>,
}
impl FromSql<crate::schema::sql_types::HeaderPair, Pg> for HeaderPair {
    fn from_sql(
        bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
    ) -> diesel::deserialize::Result<Self> {
        let (name, value) =
            FromSql::<Record<(Text, Bytea)>, Pg>::from_sql(bytes)?;
        let pair = HeaderPair { name, value };
        Ok(pair)
    }
}
impl ToSql<crate::schema::sql_types::HeaderPair, Pg> for HeaderPair {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        WriteTuple::<(Text, Bytea)>::write_tuple(
            &(self.name.clone(), self.value.clone()),
            out,
        )
    }
}

#[derive(Debug, FromSqlRow, AsExpression)]
#[diesel(sql_type = crate::schema::sql_types::HttpRequest)]
pub struct HttpRequest {
    pub response_status_code: i16,
    pub response_headers: Vec<HeaderPair>,
    pub response_body: Vec<u8>,
    pub http_version: String,
}

impl FromSql<crate::schema::sql_types::HttpRequest, Pg> for HttpRequest {
    fn from_sql(
        bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
    ) -> diesel::deserialize::Result<Self> {
        let (
            response_status_code,
            response_headers,
            response_body,
            http_version,
        ) = FromSql::<
            Record<(
                SmallInt,
                Array<crate::schema::sql_types::HeaderPair>,
                Bytea,
                Text,
            )>,
            Pg,
        >::from_sql(bytes)?;
        Ok(HttpRequest {
            response_status_code,
            response_headers,
            response_body,
            http_version,
        })
    }
}
impl ToSql<crate::schema::sql_types::HttpRequest, Pg> for HttpRequest {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        WriteTuple::<(
            SmallInt,
            Array<crate::schema::sql_types::HeaderPair>,
            Bytea,
            Text,
        )>::write_tuple(
            &(
                self.response_status_code,
                self.response_headers.clone(),
                self.response_body.clone(),
                self.http_version.clone(),
            ),
            out,
        )
    }
}
