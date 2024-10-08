use crate::{
    domain::{NewSubscriber, SubscriptionToken},
    TEMPLATES,
};

use super::EmailClient;

#[tracing::instrument(
    name = "Send a confirmation email to the new subscriber",
    skip(email_client, new_subscriber)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
    token: &SubscriptionToken,
) -> Result<(), SendEmailError> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url,
        token.as_ref()
    );
    let mut tera_context = tera::Context::new();
    tera_context.insert("link", &confirmation_link);
    let html_body =
        TEMPLATES.render("emails/subscription_email.html", &tera_context)?;
    let plain_text_body = format!("Welcome to reingma's newsletter!\nVisit {} to confirm your subscription.",
                        confirmation_link);
    email_client
        .send_email(
            &new_subscriber.email,
            &plain_text_body,
            &html_body,
            "Welcome to reingma's newsletter!",
        )
        .await?;

    tracing::info!("Email sent to subscriber.");
    Ok(())
}
#[derive(Debug, thiserror::Error)]
pub enum SendEmailError {
    #[error("Could not render email template.")]
    TemplateRenderError(#[from] tera::Error),
    #[error("Could not deliver email.")]
    DeliveryError(#[from] reqwest::Error),
}
