use axum::{response::IntoResponse, routing::get, Router};
use axum_error_into_response_derive::IntoResponse;
use thiserror::Error;
use tokio::net::TcpListener;

#[derive(Debug, Error, IntoResponse)]
pub enum AppError {
    #[error("This shouldn't show")]
    Error,
}

async fn fail() -> impl IntoResponse {
    AppError::Error
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    let routes = Router::new().route("/fail", get(fail));

    println!("Listening on http://127.0.0.1:3000/");

    axum::serve(listener, routes).await?;

    Ok(())
}
