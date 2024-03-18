use crate::helpers::spawn_app;

#[tokio::test]
async fn subscribe_returns_200_for_valid_form() {
    let test_app = spawn_app().await;

    let body = "name=Gabriel%20Aguiar&email=gabriel.masarin.aguiar%40gmail.com";
    let response = test_app
        .subscribe(body.to_string())
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());
    let mut connection =
        test_app.pool.get().await.expect("Could not get connection");
    let results = crate::helpers::check_subscriber_existance(
        &mut connection,
        "gabriel.masarin.aguiar@gmail.com",
    )
    .await;
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
