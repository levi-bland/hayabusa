// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Ident, ItemStruct};

#[proc_macro_attribute]
pub fn meta(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    let name = input.ident.clone();
    let generics = input.generics.clone();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let fields = input.fields.clone();
    let args: Vec<proc_macro2::TokenStream> = fields
        .iter()
        .map(|field| {
            let ident = &field.ident;
            let ty = &field.ty;
            quote! { #ident: #ty }
        })
        .collect();

    let idents: Vec<Ident> = fields
        .iter()
        .map(|field| field.ident.clone().expect("expected a named field"))
        .collect();

    quote! {
        #input

        impl #impl_generics #name #ty_generics #where_clause {
            pub fn new(#(#args),*) -> Self {
                Self {
                    #(#idents),*
                }
            }
        }
    }
    .into()
}
