// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Attribute, Expr, Lit, Meta};

pub struct DocsVec(pub Vec<String>);

pub fn parse_docs(attrs: &[Attribute]) -> DocsVec {
    DocsVec(
        attrs
            .iter()
            .filter(|attr| attr.path().is_ident("doc"))
            .filter_map(|attr| {
                if let Meta::NameValue(meta) = &attr.meta {
                    if let Expr::Lit(expr_lit) = &meta.value {
                        if let Lit::Str(s) = &expr_lit.lit {
                            return Some(s.value().trim().to_string());
                        }
                    }
                }
                None
            })
            .collect(),
    )
}

impl ToTokens for DocsVec {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let items = &self.0;
        tokens.extend(quote! { vec![#(#items.to_string()),*] });
    }
}
