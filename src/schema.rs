// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "http_request"))]
    pub struct HttpRequest;
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "header_pair"))]
    pub struct HeaderPair;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::HttpRequest;

    idempotency (user_id, idempotency_key) {
        user_id -> Uuid,
        idempotency_key -> Text,
        request -> HttpRequest,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    subscription_tokens (subscription_token) {
        subscription_token -> Text,
        subscriber_id -> Uuid,
        generated_at -> Timestamptz,
    }
}

diesel::table! {
    subscriptions (id) {
        id -> Uuid,
        email -> Text,
        name -> Text,
        subscribed_at -> Timestamptz,
        status -> Text,
    }
}

diesel::table! {
    users (user_id) {
        user_id -> Uuid,
        username -> Text,
        password_hash -> Text,
    }
}

diesel::joinable!(idempotency -> users (user_id));
diesel::joinable!(subscription_tokens -> subscriptions (subscriber_id));

diesel::allow_tables_to_appear_in_same_query!(
    idempotency,
    subscription_tokens,
    subscriptions,
    users,
);
