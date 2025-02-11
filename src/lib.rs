//! Helpful `IntoResponse` derive macro for the thiserror crate
//!
//! To derive `IntoResponse`, the error type must implement `Debug` and `thiserror::Error`
//!
//! Bringing your own implementation of `Debug` allows for more flexibilty
//!
//! By default, errors will have a 500 status code (`INTERNAL_SERVER_ERROR`) and return a plain text
//! response of "Something went wrong"
//!
//! You can change the default text response of a 500 status code using the `#[internal_text = "..."]` attribute
//!
//! Default behavior can be overridden on certain fields using the `#[status(...)]` macro
//! When overridden, the server will respond with the custom status and plain text according to your `Debug` implementation
//!
//! If you'd like a Json response, enable the crate's serde feature, and wrap the enum in `Json(...)`
//!
//! To display the internal error with tracing, enable the crate's tracing feature
//!
//! # Example
//!
//! ```rust
//! use axum_thiserror_intoresponse_derive::IntoResponse;
//! use thiserror::Error;
//!
//! #[derive(Debug, Error, IntoResponse)]
//! // set the default response for StatusCode::INTERNAL_SERVER_ERROR
//! #[internal_text = "overridden"]
//! pub enum AppError {
//!     #[error("This shouldn't show in the response, but will in tracing")]
//!     Internal,
//!     // automatically sends the error text in the response
//!     // when the status is set other than StatusCode::INTERNAL_SERVER_ERROR
//!     #[status(StatusCode::BAD_REQUEST)]
//!     #[error("Bad request")]
//!     ClientError,
//!     // keep the magic of fields
//!     #[status(StatusCode::UNAUTHORIZED)]
//!     #[error("Error: {0}")]
//!     AuthError(&'static str),
//!     // multiple fields
//!     #[status(StatusCode::BAD_REQUEST)]
//!     #[error("Error: {0} {1}")]
//!     AnotherError(&'static str, &'static str),
//! }
//! ```
//!
//! A working example with all of the features enabled can be viewed [in the repo](https://github.com/ozpv/axum_thiserror_intoresponse_derive/blob/main/example/src/main.rs)
//!
//! # Contributing
//!
//! All contributions are welcome. Just open a pull request in the repo.
//!
//! # License
//!
//! This lib is licensed under the [MIT license](https://github.com/ozpv/axum_thiserror_intoresponse_derive/blob/main/LICENSE)

#![no_std]

extern crate alloc;
extern crate proc_macro;

use alloc::{string::String, vec::Vec};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Expr, Fields, Lit, Meta};

/// # Panics
///
/// if the type isn't an enum
#[proc_macro_derive(IntoResponse, attributes(internal_text, status))]
pub fn derive_into_response(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let attrs = &input.attrs;

    // parse the internal_text attribute
    let internal_text = attrs
        .iter()
        .find(|attr| attr.path().is_ident("internal_text"))
        .and_then(|attr| {
            if let Meta::NameValue(meta) = &attr.meta {
                if let Expr::Lit(expr) = &meta.value {
                    if let Lit::Str(lit_str) = &expr.lit {
                        return Some(lit_str.value());
                    }
                }
            }
            None
        })
        .unwrap_or_else(|| String::from("Something went wrong"));

    // parse the attributes for status code override (if any)
    let mut variant_overrides = Vec::new();

    if let Data::Enum(data) = &input.data {
        for variant in &data.variants {
            let name = &variant.ident;

            // make sure fields on the enum variants are matched
            let fields = match &variant.fields {
                Fields::Named(_) => quote! {{..}},
                Fields::Unit => quote! {},
                Fields::Unnamed(fields) => {
                    let all_fields = fields
                        .unnamed
                        .iter()
                        .map(|_| quote! {_})
                        .collect::<Vec<proc_macro2::TokenStream>>();

                    quote! {
                        (#(#all_fields),*)
                    }
                }
            };

            let attr = variant
                .attrs
                .iter()
                .find(|attr| attr.path().is_ident("status"));

            if let Some(attr) = attr {
                // extract the status and build the tokens
                if let Meta::List(list) = &attr.meta {
                    let status = &list.tokens;

                    let status = quote! {
                        Self::#name #fields => ::axum::http::status::#status,
                    };

                    variant_overrides.push(status);
                }
            }
        }
    } else {
        panic!("IntoResponse can only be derived on an Enum");
    }

    let tracing = {
        #[allow(unused)]
        let mut stream = proc_macro2::TokenStream::new();
        #[cfg(feature = "tracing")]
        {
            let err = quote! {
                let internal_err = self.to_string();
                ::tracing::error!("{internal_err}");
            };

            stream = err;
        }
        stream
    };

    // build the impl
    #[allow(unused_mut)]
    let mut expanded = quote! {
        #[automatically_derived]
        impl ::axum::response::IntoResponse for #name {
            fn into_response(self) -> ::axum::response::Response {
                let status = match self {
                    #(#variant_overrides)*
                    _ => ::axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                };

                let text = {
                    if status == ::axum::http::StatusCode::INTERNAL_SERVER_ERROR {
                        #tracing
                        #internal_text.to_string()
                    } else {
                        self.to_string()
                    }
                };

                ::axum::response::IntoResponse::into_response((status, text))
            }
        }
    };

    #[cfg(feature = "serde")]
    {
        let ser = serde_derive(&name, &variant_overrides, &internal_text);
        expanded.extend([ser]);
    }

    expanded.into()
}

#[cfg(feature = "serde")]
fn serde_derive(
    name: &proc_macro2::Ident,
    variant_overrides: &Vec<proc_macro2::TokenStream>,
    internal_text: &str,
) -> proc_macro2::TokenStream {
    quote! {
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for #name {
            fn serialize<__S>(&self, __serializer: __S) -> Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let status = match self {
                    #(#variant_overrides)*
                    _ => ::axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                }.as_u16();

                let text = {
                    if status == ::axum::http::StatusCode::INTERNAL_SERVER_ERROR {
                        #internal_text.to_string()
                    } else {
                        self.to_string()
                    }
                };

                let mut __serde_state = _serde::Serializer::serialize_struct(__serializer, "", false as usize + 1 + 1)?;
                _serde::ser::SerializeStruct::serialize_field(&mut __serde_state, "status", &status)?;
                _serde::ser::SerializeStruct::serialize_field(&mut __serde_state, "error", &text)?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    }
}
