use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::spawn_app;

#[tokio::test]
async fn confirmations_without_token_are_rejected_with_a_400() {
    let test_app = spawn_app().await;

    let response =
        reqwest::get(&format!("{}/subscriptions/confirm", test_app.address))
            .await
            .unwrap();
    assert_eq!(response.status().as_u16(), 400);
}
#[tokio::test]
async fn link_on_subscribe_returns_200_when_clicked() {
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

    let confirmation_link =
        test_app.get_confirmation_links(email_request).await;

    let response = reqwest::get(confirmation_link.html).await.unwrap();

    assert_eq!(response.status().as_u16(), 200);
}
#[tokio::test]
async fn clicking_on_confirmation_link_sets_subscriber_status_to_confirmed() {
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

    let confirmation_link =
        test_app.get_confirmation_links(email_request).await;

    reqwest::get(confirmation_link.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    let mut connection =
        test_app.pool.get().await.expect("Could not get connection");
    let results = crate::helpers::check_subscriber_existance(
        &mut connection,
        "gabriel.masarin.aguiar@gmail.com",
    )
    .await;
    assert_eq!(results.len(), 1);
    let subscriber_data = results.first().unwrap();
    assert_eq!(subscriber_data.status, "confirmed");
}
