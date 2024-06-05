use crate::helpers::{assert_is_redirect_to, spawn_app};

#[tokio::test]
async fn an_error_flash_message_is_set_on_failure() {
    let app = spawn_app(None).await;

    let login_body = serde_json::json!({
        "username":"random_username",
        "password":"random_password"
    });
    let response = app.post_login(&login_body).await;
    assert_is_redirect_to(&response, "/login");

    let page = app.get_login_html().await;
    assert!(page.contains(r#"<p><i>Authentication failed.</i></p>"#));

    //Login page should not have message if we request again.
    let page = app.get_login_html().await;
    assert!(!page.contains(r#"<p><i>Authentication failed.</i></p>"#));
}

#[tokio::test]
async fn redirect_to_dashboard_on_success() {
    let app = spawn_app(None).await;

    let login_body = serde_json::json!({
        "username": &app.test_user.username,
        "password":&app.test_user.password
    });
    let response = app.post_login(&login_body).await;
    assert_is_redirect_to(&response, "/admin/dashboard");

    let page = app.get_admin_dashboard_html().await;
    assert!(page.contains(&format!("Welcome {}", app.test_user.username)));
}
