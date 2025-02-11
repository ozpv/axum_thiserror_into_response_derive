use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};
use axum_thiserror_intoresponse_derive::IntoResponse;
use thiserror::Error;
use tokio::net::TcpListener;

// sets the default plain text response for StatusCode::INTERNAL_SERVER_ERROR
// #[internal_text = "overridden"]
#[derive(Debug, Error, IntoResponse)]
pub enum AppError {
    // conditionally set StatusCode::INTERNAL_SERVER_ERROR response text with #[text("Something else")]
    // or send the error message in plain text with #[err_text(true)]
    // #[text = "Something else"]
    // #[err_text(true)]
    #[error("This shouldn't show")]
    Internal,
    // automatically send the plain text when the error is set other than internal
    #[status(StatusCode::BAD_REQUEST)]
    #[error("Bad request")]
    ClientError,
}

async fn fail() -> impl IntoResponse {
    AppError::Internal
}

async fn client_fail() -> impl IntoResponse {
    AppError::ClientError
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:3000").await?;

    let routes = Router::new()
        .route("/", get(fail))
        .route("/bad", get(client_fail));

    println!("Listening on http://127.0.0.1:3000/");

    axum::serve(listener, routes).await?;

    Ok(())
}
