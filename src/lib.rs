extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(IntoResponse)]
pub fn derive_into_response(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let expanded = quote! {
        impl ::axum::response::IntoResponse for #name {
            fn into_response(self) -> ::axum::response::Response {
                let status = match self {
                    _ => ::axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                };

                let text = {
                    if status == ::axum::http::StatusCode::INTERNAL_SERVER_ERROR {
                        String::from("Something went wrong")
                    } else {
                        self.to_string()
                    }
                };

                (status, text).into_response()
            }
        }
    };

    TokenStream::from(expanded)
}
