use crate::helpers::{assert_is_redirect_to, spawn_app};
use uuid::Uuid;

#[tokio::test]
async fn you_must_be_logged_in_to_see_change_password_page() {
    let app = spawn_app(None).await;
    let response = app.get_change_password().await;
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn you_must_be_logged_in_to_change_password() {
    let app = spawn_app(None).await;
    let new_password = Uuid::now_v7();
    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": app.test_user.password,
            "new_password": new_password,
            "new_password_check": new_password
        }))
        .await;
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn new_password_fields_must_match() {
    let app = spawn_app(None).await;
    let new_password = Uuid::now_v7();
    let another_password = Uuid::now_v7();
    app.post_login(&serde_json::json!({
        "username": app.test_user.username,
        "password":app.test_user.password,
    }))
    .await;
    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": app.test_user.password,
            "new_password": new_password,
            "new_password_check": another_password
        }))
        .await;
    assert_is_redirect_to(&response, "/admin/password");

    let html_page = app.get_change_password_html().await;
    assert!(html_page.contains(
        "<p><i>You entered two different new passwords - \
        the field values must match.</i></p>"
    ));
}
#[tokio::test]
async fn current_password_must_be_valid() {
    let app = spawn_app(None).await;
    let new_password = Uuid::now_v7();
    app.post_login(&serde_json::json!({
        "username": app.test_user.username,
        "password":app.test_user.password,
    }))
    .await;
    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": Uuid::now_v7(),
            "new_password": new_password,
            "new_password_check": new_password
        }))
        .await;
    assert_is_redirect_to(&response, "/admin/password");

    let html_page = app.get_change_password_html().await;
    assert!(
        html_page.contains("<p><i>The current password is incorrect.</i></p>")
    );
}

#[tokio::test]
async fn short_password_is_rejected() {
    let app = spawn_app(None).await;
    let new_password = "1234";
    app.post_login(&serde_json::json!({
        "username": app.test_user.username,
        "password":app.test_user.password,
    }))
    .await;
    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": Uuid::now_v7(),
            "new_password": new_password,
            "new_password_check": new_password
        }))
        .await;
    assert_is_redirect_to(&response, "/admin/password");

    let html_page = app.get_change_password_html().await;
    assert!(html_page.contains("<p><i>Password is too short.</i></p>"));
}

#[tokio::test]
async fn too_large_password_is_rejected() {
    let app = spawn_app(None).await;
    let new_password = "a".repeat(134);
    app.post_login(&serde_json::json!({
        "username": app.test_user.username,
        "password":app.test_user.password,
    }))
    .await;
    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": Uuid::now_v7(),
            "new_password": new_password,
            "new_password_check": new_password
        }))
        .await;
    assert_is_redirect_to(&response, "/admin/password");

    let html_page = app.get_change_password_html().await;
    assert!(html_page.contains("<p><i>Password is too long.</i></p>"));
}
