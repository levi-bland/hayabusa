// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

mod attributes;
mod codegen;
mod meta;
mod registry;
mod toposort;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(ParseAccounts, attributes(account))]
pub fn derive_parse_accounts(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match codegen::expand(input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// One field in the annotated struct, fully parsed.
struct AccountField {
    /// The field name, e.g. `counter`
    pub ident: syn::Ident,
    /// The full type as written, e.g. `Init<Account<'view, Counter>>`
    pub ty: syn::Type,
    /// Parsed attribute arguments from all `#[account(...)]` attrs on this field
    pub args: Vec<crate::attributes::AccountArg>,
}