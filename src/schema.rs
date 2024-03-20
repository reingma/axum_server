// @generated automatically by Diesel CLI.

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

diesel::joinable!(subscription_tokens -> subscriptions (subscriber_id));

diesel::allow_tables_to_appear_in_same_query!(
    subscription_tokens,
    subscriptions,
);
