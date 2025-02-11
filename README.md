# axum_thiserror_intoresponse_derive

Helpful `IntoResponse` derive macro for the thiserror crate

To derive `IntoResponse`, the error type must implement `Debug` and `thiserror::Error`

Bringing your own implementation of `Debug` allows for more flexibilty

By default, errors will have a 500 status code (`INTERNAL_SERVER_ERROR`) and return a plain text
response of "Something went wrong"

You can change the default text response of a 500 status code using the `#[internal_text = "..."]` attribute

Default behavior can be overridden on certain fields using the `#[status(...)]` macro

When overridden, the server will respond with the custom status and plain text according to your `Debug` implementation

If you'd like a Json response, enable the crate's serde feature, and wrap the enum in `Json(...)`

To display the internal error with tracing, enable the crate's tracing feature

## Example

```rust
use axum_thiserror_intoresponse_derive::IntoResponse;
use thiserror::Error;

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
```

A working example with all of the features enabled can be viewed [in the repo](https://github.com/ozpv/axum_thiserror_intoresponse_derive/blob/main/example/src/main.rs)

## Contributing

All contributions are welcome. Just open a pull request in the repo.

## License

This lib is licensed under the [MIT license](https://github.com/ozpv/axum_thiserror_intoresponse_derive/blob/main/LICENSE)
