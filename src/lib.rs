#![no_std]

extern crate alloc;
extern crate proc_macro;

use alloc::{string::String, vec::Vec};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Expr, Lit, Meta};

#[proc_macro_derive(IntoResponse, attributes(internal_text, status, text))]
pub fn derive_into_response(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let attrs = &input.attrs;

    // parse the internal_text attribute
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

    // parse the attributes for status code override (if any)
    let mut variant_overrides = Vec::new();

    if let Data::Enum(data) = &input.data {
        for variant in &data.variants {
            let name = &variant.ident;

            let attr = variant
                .attrs
                .iter()
                .find(|attr| attr.path().is_ident("status"));

            if let Some(attr) = attr {
                if let Meta::List(list) = &attr.meta {
                    let status = &list.tokens;

                    let status = quote! {
                        Self::#name => #status,
                    };

                    variant_overrides.push(status);
                }
            }
        }
    } else {
        panic!("IntoResponse can only be derived on an Enum");
    }

    // build the impl
    let expanded = quote! {
        impl ::axum::response::IntoResponse for #name {
            fn into_response(self) -> ::axum::response::Response {
                let status = match self {
                    #(#variant_overrides),*
                    _ => ::axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                };

                let text = {
                    if status == ::axum::http::StatusCode::INTERNAL_SERVER_ERROR {
                        #internal_text.to_string()
                    } else {
                        self.to_string()
                    }
                };

                ::axum::response::IntoResponse::into_response((status, text))
            }
        }
    };

    expanded.into()
}
