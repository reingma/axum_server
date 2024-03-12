use crate::helpers::spawn_app;
use axum_newsletter::models::Subscriptions;
use diesel::prelude::*;
use diesel::SelectableHelper;
use diesel_async::RunQueryDsl;

#[tokio::test]
async fn subscribe_returns_200_for_valid_form() {
    use axum_newsletter::schema::subscriptions::dsl::*;
    let test_app = spawn_app().await;

    let body = "name=Gabriel%20Aguiar&email=gabriel.masarin.aguiar%40gmail.com";
    let response = test_app
        .subscribe(body.to_string())
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());
    let mut connection =
        test_app.pool.get().await.expect("Could not get connection");
    let results = subscriptions
        .limit(1)
        .filter(name.eq("Gabriel Aguiar"))
        .select(Subscriptions::as_select())
        .load(&mut connection)
        .await
        .expect("Failed to read query");
    assert_eq!(results.len(), 1);
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
