// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

mod account;
mod idl_build_derive;
mod ty;

use account::generate_account_idl_build_impl;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Error, Ident, ItemStruct, Result};

#[proc_macro_derive(IdlBuild)]
pub fn idl_build(input: TokenStream) -> TokenStream {
    idl_build_derive::idl_build_derive_handler(input)
}

#[proc_macro_attribute]
pub fn idl(attr: TokenStream, input: TokenStream) -> TokenStream {
    let ident = parse_macro_input!(attr as Ident);

    let parse_type = match IdlParseType::try_from(ident) {
        Ok(parse_type) => parse_type,
        Err(e) => return e.to_compile_error().into(),
    };

    let suffix = match parse_type {
        IdlParseType::Account => {
            let input = input.clone();
            let input = parse_macro_input!(input as ItemStruct);
            match generate_account_idl_build_impl(&input) {
                Ok(g) => g,
                Err(e) => return e.to_compile_error().into(),
            }
        }
        _ => todo!(),
    };

    let input2 = proc_macro2::TokenStream::from(input.clone());

    quote! {
        #suffix
    }
    .into()
}

enum IdlParseType {
    Account,
    Event,
    Type,
    Instruction,
}

impl TryFrom<Ident> for IdlParseType {
    type Error = Error;

    fn try_from(ident: Ident) -> Result<Self> {
        match ident.to_string().as_str() {
            "account" => Ok(IdlParseType::Account),
            "event" => Ok(IdlParseType::Event),
            "type" => Ok(IdlParseType::Type),
            "instruction" => Ok(IdlParseType::Instruction),
            _ => Err(Error::new_spanned(ident, "Invalid identifier")),
        }
    }
}
