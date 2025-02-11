use axum::{response::IntoResponse, routing::get, Json, Router};
use axum_thiserror_intoresponse_derive::IntoResponse;
use thiserror::Error;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

#[derive(Debug, Error, IntoResponse)]
// set the default response for StatusCode::INTERNAL_SERVER_ERROR
#[internal_text = "overridden"]
pub enum AppError {
    #[error("This shouldn't show in the response, but will in tracing")]
    Internal,
    // automatically sends the error text in the response
    // when the status is set other than StatusCode::INTERNAL_SERVER_ERROR
    #[status(StatusCode::BAD_REQUEST)]
    #[error("Bad request")]
    ClientError,
    // keep the magic of fields
    #[status(StatusCode::UNAUTHORIZED)]
    #[error("Error: {0}")]
    AuthError(&'static str),
    // multiple fields
    #[status(StatusCode::BAD_REQUEST)]
    #[error("Error: {0} {1}")]
    AnotherError(&'static str, &'static str),
}

async fn fail() -> impl IntoResponse {
    AppError::Internal
}

async fn client_fail() -> impl IntoResponse {
    AppError::ClientError
}

async fn unauthorized() -> impl IntoResponse {
    AppError::AuthError("Not allowed to access this endpoint")
}

async fn as_json() -> Json<AppError> {
    Json(AppError::ClientError)
}

async fn multiple_fields() -> impl IntoResponse {
    AppError::AnotherError("This", "is two fields")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let listener = TcpListener::bind("127.0.0.1:3000").await?;

    let routes = Router::new()
        .route("/", get(fail))
        .route("/bad_request", get(client_fail))
        .route("/json", get(as_json))
        .route("/multiple_fields", get(multiple_fields))
        .route("/unauthorized", get(unauthorized))
        .layer(TraceLayer::new_for_http());

    tracing::info!("Listening on http://127.0.0.1:3000/");

    axum::serve(listener, routes).await?;

    Ok(())
}
