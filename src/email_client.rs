use crate::domain::SubscriberEmail;
use reqwest::Client;
use secrecy::{ExposeSecret, Secret};
pub struct EmailClient {
    api_token: Secret<String>,
    sender: SubscriberEmail,
    http_client: Client,
    base_url: reqwest::Url,
}

impl EmailClient {
    pub fn new(
        base_url: &str,
        sender: SubscriberEmail,
        api_token: Secret<String>,
    ) -> Self {
        Self {
            base_url: reqwest::Url::try_from(base_url).expect("Invalid url!"),
            sender,
            http_client: Client::new(),
            api_token,
        }
    }
    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        text_content: &str,
        html_content: &str,
        subject: &str,
    ) -> Result<(), String> {
        let url = reqwest::Url::join(&self.base_url, "/email")
            .map_err(|_| "Invalid uri")?;
        let request_body = SendEmailRequest {
            from: self.sender.as_ref().to_owned(),
            to: recipient.as_ref().to_owned(),
            subject: subject.to_owned(),
            html_body: html_content.to_owned(),
            text_body: text_content.to_owned(),
        };

        let _builder = self
            .http_client
            .post(url)
            .header("X-Postmark-Server-Token", self.api_token.expose_secret())
            .json(&request_body)
            .send()
            .await
            .map_err(|_| "Could not send request")?;
        Ok(())
    }
}

#[derive(serde::Serialize)]
struct SendEmailRequest {
    from: String,
    to: String,
    subject: String,
    html_body: String,
    text_body: String,
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use secrecy::Secret;
    use wiremock::matchers::any;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::try_from(SafeEmail().fake::<String>())
            .expect("Email should be valid");
        let email_client = EmailClient::new(
            &mock_server.uri(),
            sender,
            Secret::new(Faker.fake()),
        );

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email =
            SubscriberEmail::try_from(SafeEmail().fake::<String>())
                .expect("Email should be valid");
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        let _ = email_client
            .send_email(subscriber_email, &content, &content, &subject)
            .await;
    }
}
