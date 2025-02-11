#![no_std]

extern crate alloc;
extern crate proc_macro;

use alloc::string::String;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Expr, Lit, Meta};

#[proc_macro_derive(IntoResponse, attributes(internal_text, status, text))]
pub fn derive_into_response(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let attrs = &input.attrs;

    let internal_text = attrs
        .iter()
        .find(|attr| attr.path().is_ident("internal_text"))
        .map(|attr| {
            if let Meta::NameValue(meta) = &attr.meta {
                if let Expr::Lit(expr) = &meta.value {
                    if let Lit::Str(lit_str) = &expr.lit {
                        return Some(lit_str.value());
                    }
                }
            }
            None
        })
        .flatten()
        .unwrap_or_else(|| String::from("Something went wrong"));

    let expanded = quote! {
        impl ::axum::response::IntoResponse for #name {
            fn into_response(self) -> ::axum::response::Response {
                let status = match self {
                    _ => ::axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                };

                let text = {
                    if status == ::axum::http::StatusCode::INTERNAL_SERVER_ERROR {
                        String::from(#internal_text)
                    } else {
                        self.to_string()
                    }
                };

                ::axum::response::IntoResponse::into_response((status, text))
            }
        }
    };

    TokenStream::from(expanded)
}
