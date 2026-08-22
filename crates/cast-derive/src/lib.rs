// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Cast)]
pub fn derive_cast(stream: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(stream as DeriveInput);

    let ident = derive_input.ident.clone();

    let output = quote! {
        impl Cast for #ident {}
    };

    TokenStream::from(output)
}

#[proc_macro_derive(FromBytesUnchecked)]
pub fn derive_from_bytes_unchecked(stream: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(stream as DeriveInput);
    let ident = derive_input.ident.clone();

    let output = quote! {
        impl FromBytesUnchecked for #ident {}
    };

    TokenStream::from(output)
}
