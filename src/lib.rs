#![no_std]

extern crate alloc;
extern crate proc_macro;

use alloc::{string::String, vec::Vec};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Expr, Fields, Lit, Meta};

#[proc_macro_derive(IntoResponse, attributes(internal_text, status))]
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

            // make sure fields on the enum variants are matched
            let fields = match &variant.fields {
                Fields::Named(_) => quote! {{..}},
                Fields::Unit => quote! {},
                Fields::Unnamed(fields) => {
                    let all_fields = fields
                        .unnamed
                        .iter()
                        .map(|_| quote! {_}.into())
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
        let ser = quote! {
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
        };

        expanded.extend([ser]);
    }

    expanded.into()
}
