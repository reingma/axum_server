use crate::helpers::{
    assert_is_redirect_to, spawn_app, ConfirmationLinks, TestApp,
};
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

    app.login_test_user().await;

    let newsletter_request_body = serde_json::json!({
        "title":"Newsletter title",
        "content_text": "Newsletter body as plaintext",
        "content_html":"<p>Newsletter body as HTML</p>"
    });
    let response = app.post_newsletter(&(newsletter_request_body)).await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    let html_page = app.get_newsletter_html().await;
    assert!(
        html_page.contains("<p><i>Newsletter delivered successfully</i></p>")
    );
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

    app.login_test_user().await;

    let newsletter_request_body = serde_json::json!({
        "title":"Newsletter title",
        "content_text": "Newsletter body as plaintext",
        "content_html":"<p>Newsletter body as HTML</p>"
    });
    let response = app.post_newsletter(&(newsletter_request_body)).await;

    assert_is_redirect_to(&response, "/admin/newsletters");

    let html_page = app.get_newsletter_html().await;
    assert!(
        html_page.contains("<p><i>Newsletter delivered successfully</i></p>")
    );
}

#[tokio::test]
async fn redirect_with_message_on_invalid_data_email_not_delivered() {
    let app = spawn_app(None).await;
    create_confirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    app.login_test_user().await;

    let newsletter_request_body = serde_json::json!({
        "title":"Newsletter title",
        "content_html":"<p>Newsletter body as HTML</p>"
    });
    let response = app.post_newsletter(&newsletter_request_body).await;

    assert_is_redirect_to(&response, "/admin/newsletters");

    let html_page = app.get_newsletter_html().await;
    assert!(html_page.contains("<p><i>Invalid newsletter body.</i></p>"));
}

#[tokio::test]
async fn newsletter_succeds_with_message_on_valid_data() {
    let app = spawn_app(None).await;

    app.login_test_user().await;

    let body = serde_json::json!({
        "title":"Newsletter title",
        "content_text": "Newsletter body as plaintext",
        "content_html":"<p>Newsletter body as HTML</p>"
    });
    let response = app.post_newsletter(&body).await;

    assert_is_redirect_to(&response, "/admin/newsletters");

    let html_page = app.get_newsletter_html().await;
    assert!(
        html_page.contains("<p><i>Newsletter delivered successfully</i></p>")
    );
}

#[tokio::test]
async fn you_must_be_logged_in_to_post_newsletter() {
    let app = spawn_app(None).await;
    let body = serde_json::json!({
        "title":"Newsletter title",
        "content_text": "Newsletter body as plaintext",
        "content_html":"<p>Newsletter body as HTML</p>"
    });
    let response = app.post_newsletter(&body).await;
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn you_must_be_logged_in_to_see_newsletter_page() {
    let app = spawn_app(None).await;
    let response = app.get_newsletter().await;
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn newsletter_creation_is_idempotent() {
    let app = spawn_app(None).await;
    create_confirmed_subscriber(&app).await;
    app.login_test_user().await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let body = serde_json::json!({
        "title":"Newsletter title",
        "content_text": "Newsletter body as plaintext",
        "content_html":"<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::now_v7().to_string()
    });
    let response = app.post_newsletter(&body).await;

    assert_is_redirect_to(&response, "/admin/newsletters");

    let html_page = app.get_newsletter_html().await;
    assert!(
        html_page.contains("<p><i>Newsletter delivered successfully</i></p>")
    );

    //Deliver again.
    let response = app.post_newsletter(&body).await;

    assert_is_redirect_to(&response, "/admin/newsletters");

    let html_page = app.get_newsletter_html().await;
    assert!(
        html_page.contains("<p><i>Newsletter delivered successfully</i></p>")
    );
}
