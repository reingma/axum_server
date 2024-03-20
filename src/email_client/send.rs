use crate::domain::NewSubscriber;

use super::EmailClient;

#[tracing::instrument(
    name = "Send a confirmation email to the new subscriber",
    skip(email_client, new_subscriber)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
    token: &str,
) -> Result<(), String> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, token
    );
    let plain_text_body = format!("Welcome to reingma's newsletter!\nVisit {} to confirm your subscription.",
                        confirmation_link);
    let html_body = format!("Welcome to reingma's newsletter!<br />\
                        Click <a href=\"{}\">here</a> to confirm your subscription.", 
                        confirmation_link);
    email_client
        .send_email(
            new_subscriber.email,
            &plain_text_body,
            &html_body,
            "Welcome to reingma's newsletter!",
        )
        .await
}
