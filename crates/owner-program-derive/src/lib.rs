// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(OwnerProgram)]
pub fn derive_owner_program(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        impl OwnerProgram for #name {
            const OWNER: Address = crate::ID;

            fn owner() -> Address {
                Self::OWNER
            }
        }
    };

    TokenStream::from(expanded)
}
