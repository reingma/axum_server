use axum::response::Redirect;
use axum_extra::extract::SignedCookieJar;
use cookie::Cookie;

pub fn redirect_with_flash(
    uri: &str,
    e: anyhow::Error,
    jar: SignedCookieJar,
) -> (SignedCookieJar, Redirect) {
    tracing::error!("{} Reason {:?}", e, e);
    let cookie = Cookie::build(("_flash", e.to_string()))
        .path("/")
        .secure(true);
    (jar.add(cookie), Redirect::to(uri))
}

pub fn get_flash_error(jar: SignedCookieJar) -> (SignedCookieJar, String) {
    if let Some(error) =
        jar.get("_flash").map(|cookie| cookie.value().to_owned())
    {
        (jar.remove(Cookie::from("_flash")), error.to_string())
    } else {
        (jar, "".to_string())
    }
}
