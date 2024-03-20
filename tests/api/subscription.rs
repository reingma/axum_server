use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::spawn_app;

#[tokio::test]
async fn subscribe_returns_200_for_valid_form() {
    let test_app = spawn_app().await;

    let body = "name=Gabriel%20Aguiar&email=gabriel.masarin.aguiar%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.email_server)
        .await;

    let response = test_app
        .subscribe(body.to_string())
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());
}
#[tokio::test]
async fn subscribe_persists_subcriber_data() {
    let test_app = spawn_app().await;

    let body = "name=Gabriel%20Aguiar&email=gabriel.masarin.aguiar%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.email_server)
        .await;

    let _ = test_app
        .subscribe(body.to_string())
        .await
        .expect("Failed to execute request.");

    let mut connection =
        test_app.pool.get().await.expect("Could not get connection");
    let results = crate::helpers::check_subscriber_existance(
        &mut connection,
        "gabriel.masarin.aguiar@gmail.com",
    )
    .await;
    assert_eq!(results.len(), 1);
    let particular = results.first().unwrap();

    assert_eq!(&particular.name, "Gabriel Aguiar");
    assert_eq!(&particular.email, "gabriel.masarin.aguiar@gmail.com");
    assert_eq!(&particular.status, "pending_confirmation");
}
#[tokio::test]
async fn subscribe_returns_422_when_data_is_missing() {
    let test_app = spawn_app().await;

    let test_cases = vec![
        ("name=Gabriel%20Aguiar", "missing email"),
        ("email=gabriel.masarin.aguiar%40gmail.com", "missing name"),
        ("", "missing both"),
    ];

    for (body, message) in test_cases {
        let response = test_app
            .subscribe(body.to_string())
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            422,
            response.status().as_u16(),
            "The API did not fail with 422 Bad Request when the payload was {}",
            message
        );
    }
}
#[tokio::test]
async fn subscribe_returns_400_when_fields_are_present_but_empty() {
    let test_app = spawn_app().await;

    let test_cases = vec![
        (
            "name= &email=gabriel.masarin.aguiar%40gmail.com",
            "missing name",
        ),
        ("name=Gabriel%20Aguiar&email=", "missing email"),
        ("name=person&email=totallynotemail", "missing both"),
    ];

    for (body, message) in test_cases {
        let response = test_app
            .subscribe(body.to_string())
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}",
            message
        );
    }
}
#[tokio::test]
async fn subscribe_sends_a_confirmation_email_for_valid_sub() {
    let test_app = spawn_app().await;

    let body = "name=Gabriel%20Aguiar&email=gabriel.masarin.aguiar%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    let response = test_app
        .subscribe(body.into())
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
    let test_app = spawn_app().await;

    let body = "name=Gabriel%20Aguiar&email=gabriel.masarin.aguiar%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.email_server)
        .await;

    let _ = test_app.subscribe(body.into()).await.unwrap();

    let email_requests =
        &test_app.email_server.received_requests().await.unwrap();
    let email_request = email_requests.first().unwrap();

    let confirmation_links =
        test_app.get_confirmation_links(email_request).await;

    assert_eq!(
        confirmation_links.html.as_str(),
        confirmation_links.plain_text.as_str()
    );
}
