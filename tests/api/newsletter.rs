use crate::helpers::{spawn_app, ConfirmationLinks, TestApp};
use wiremock::matchers::{any, method, path};
use wiremock::{Mock, ResponseTemplate};

async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
    let body = "name=gabriel%20aguiar&email=gabriel.aguiar%40gmail.com";
    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;
    app.subscribe(body.into())
        .await
        .expect("Request failed.")
        .error_for_status()
        .unwrap();
    let email_request = &app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();
    app.get_confirmation_links(email_request).await
}

async fn create_confirmed_subscriber(app: &TestApp) {
    let confirmation_link = create_unconfirmed_subscriber(app).await;
    reqwest::get(confirmation_link.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

#[tokio::test]
async fn newsletter_are_not_delivered_to_uncofirmed_subscribers() {
    let app = spawn_app(None).await;
    create_unconfirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "title":"Newsletter title",
        "content": {
            "text":"Newsletter body as plaintext",
            "html":"<p>Newsletter body as HTML</p>"
        }
    });
    let response = app.post_newsletter(newsletter_request_body).await;
    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn newsletter_are_delivered_to_confirmed_subscribers() {
    let app = spawn_app(None).await;
    create_confirmed_subscriber(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "title":"Newsletter title",
        "content": {
            "text":"Newsletter body as plaintext",
            "html":"<p>Newsletter body as HTML</p>"
        }
    });
    let response = app.post_newsletter(newsletter_request_body).await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn newsletter_returns_400_on_invalid_data() {
    let app = spawn_app(None).await;
    let cases = vec![
        (
            serde_json::json!({
                "content": {
                    "text": "Body as plain text",
                    "html": "<p> Body as html </p>"
                }
            }),
            "missing title",
        ),
        (
            serde_json::json!({
                "title":"Newsletter"
            }),
            "missing content",
        ),
        (
            serde_json::json!({
                "content": {
                    "text": "Body as plain text",
                },
                "title":"Newsletter"
            }
            ),
            "missing html content",
        ),
        (
            serde_json::json!({
                "content": {
                    "html": "<p> Body as html </p>",
                },
                "title":"Newsletter"
            }
            ),
            "missing plain text content",
        ),
    ];
    for (body, error_message) in cases {
        let response = app.post_newsletter(body).await;

        assert_eq!(
            response.status().as_u16(),
            422,
            "The API did not respond with 422 Unprocessable Content when the payload was {}",
            error_message
        )
    }
}

#[tokio::test]
async fn newsletter_returns_200_on_valid_data() {
    let app = spawn_app(None).await;
    let body = serde_json::json!({
        "content": {
            "text": "Body as plain text",
            "html": "<p> Body as html </p>"
        },
        "title": "Newsletter"
    });
    let response = app.post_newsletter(body).await;

    assert_eq!(response.status().as_u16(), 200)
}
