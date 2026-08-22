// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use hayabusa_idl_types::IdlTypeDefGeneric;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{punctuated::Punctuated, token::Comma, GenericParam};

pub struct IdlTypeDefGenericVec(Vec<IdlTypeDefGeneric>);

pub fn parse_idl_type_def_generics(
    generics: &Punctuated<GenericParam, Comma>,
) -> IdlTypeDefGenericVec {
    IdlTypeDefGenericVec(
        generics
            .iter()
            .map(|generic| match generic {
                GenericParam::Type(t) => IdlTypeDefGeneric::Type {
                    name: t.ident.to_string(),
                },
                GenericParam::Const(c) => IdlTypeDefGeneric::Const {
                    name: c.ident.to_string(),
                    ty: c.ty.to_token_stream().to_string(),
                },
                GenericParam::Lifetime(_) => {
                    panic!("Lifetime generics are not supported on IDL type definitions")
                }
            })
            .collect(),
    )
}

impl ToTokens for IdlTypeDefGenericVec {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let items = &self.0;
        tokens.extend(quote! { vec![#(#items),*] });
    }
}
