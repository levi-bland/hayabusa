// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use proc_macro::TokenStream;
use quote::quote;
use sha2::{Digest, Sha256};
use syn::{parse_macro_input, DeriveInput, LitStr};

/// Usage:
///   #[derive(Discriminator)]                    -> defaults to "account:<Name>"
///   #[discriminator(namespace = "account")]     -> "account:<Name>"
///   #[discriminator(namespace = "instruction")] -> "instruction:<Name>"
///   #[discriminator(namespace = "event")]       -> "event:<Name>"
#[proc_macro_derive(Discriminator, attributes(discriminator))]
pub fn derive_discriminator(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let namespace = extract_namespace(&input.attrs).unwrap_or_else(|| "account".to_string());
    let name_str = name.to_string();
    let hash_input = format!("{}:{}", namespace, name_str);
    let discriminator = compute_discriminator(&hash_input);

    TokenStream::from(quote! {
        #[automatically_derived]
        impl #impl_generics ::hayabusa::traits::Discriminator for #name #ty_generics #where_clause {
            const DISCRIMINATOR: &[u8; 8] = &[#(#discriminator),*];
        }
    })
}

/// Parses `#[discriminator(namespace = "...")]` off the struct attrs.
/// Returns None if the attribute is absent (caller should default).
fn extract_namespace(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if !attr.path().is_ident("discriminator") {
            continue;
        }

        let mut namespace = None;

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("namespace") {
                let value: LitStr = meta.value()?.parse()?;
                let ns = value.value();
                match ns.as_str() {
                    "account" | "instruction" | "event" => namespace = Some(ns),
                    other => {
                        return Err(meta.error(format!(
                            "unknown discriminator namespace `{}`, expected one of: account, instruction, event",
                            other
                        )));
                    }
                }
            } else {
                return Err(meta.error("unknown key, expected `namespace`"));
            }
            Ok(())
        })
        .expect("failed to parse #[discriminator(...)] attribute");

        return namespace;
    }

    None
}

fn compute_discriminator(input: &str) -> [u8; 8] {
    let hash = Sha256::digest(input.as_bytes());
    let mut disc = [0u8; 8];
    disc.copy_from_slice(&hash[..8]);
    disc
}
