use anyhow::Context;
use axum::{
    body::Body,
    extract::State,
    http::{Response, StatusCode},
    response::IntoResponse,
};
use tracing::instrument;

use crate::{
    database::queries::get_username, session_state::TypedSession,
    startup::ApplicationState, TEMPLATES,
};

#[instrument(skip(app_state, session))]
pub async fn admin_dashboard(
    State(app_state): State<ApplicationState>,
    session: TypedSession,
) -> Result<Response<Body>, DashboardError> {
    let mut connection =
        crate::database::get_connection(app_state.database_pool)
            .await
            .context("Could not get database pool")?;
    let username = if let Some(user_id) = session
        .get_user_id()
        .await
        .context("Failed to get user_id")?
    {
        get_username(&mut connection, user_id).await?
    } else {
        todo!()
    };
    let mut tera_context = tera::Context::new();
    tera_context.insert("user_name", &username.to_string());
    let html_body = TEMPLATES
        .render("pages/admin_dashboard.html", &tera_context)
        .context("Could not render login page.")?;
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(html_body))
        .context("Could not render html.")?)
}

#[derive(thiserror::Error, Debug)]
pub enum DashboardError {
    #[error("Something went wrong.")]
    UnexpectedError(#[from] anyhow::Error),
}

impl IntoResponse for DashboardError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{} Reason {:?}", self, self);
        match self {
            Self::UnexpectedError(_) => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Something went wrong.".into())
                .unwrap(),
        }
    }
}
