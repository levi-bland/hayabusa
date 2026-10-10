// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use proc_macro2::TokenStream;
use quote::quote;

use crate::attributes::{AccountArg, AttributeHandler, HandlerCtx, HandlerRole};

pub struct AddressHandler;

impl AttributeHandler for AddressHandler {
    fn key(&self) -> &'static str {
        "address"
    }

    fn role(&self) -> HandlerRole {
        HandlerRole {
            builds_meta: false,
            emits_constraint: true,
        }
    }

    fn constraint_stmts(&self, arg: &AccountArg, ctx: &HandlerCtx) -> syn::Result<TokenStream> {
        let account_ident = &ctx.fields[ctx.field_index].ident;
        let address_expr = arg.as_expr_value().ok_or_else(|| {
            syn::Error::new(
                arg.key().span(),
                "address requires `address = <address_expr>`",
            )
        })?;

        Ok(quote! {
            if ::hayabusa::prelude::hint::unlikely(
                !::hayabusa::prelude::address_eq(#account_ident.address(), #address_expr)
            ) {
                return Err(::hayabusa::prelude::ErrorCode::InvalidAccount.into());
            }
        })
    }
}
