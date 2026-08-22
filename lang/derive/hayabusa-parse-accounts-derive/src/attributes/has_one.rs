// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use proc_macro2::TokenStream;
use quote::quote;
 
use super::{AccountArg, AttributeHandler, HandlerCtx, HandlerRole};

pub struct HasOneHandler;
 
impl AttributeHandler for HasOneHandler {
    fn key(&self) -> &'static str {
        "has_one"
    }
 
    fn role(&self) -> HandlerRole {
        HandlerRole {
            builds_meta: false,
            emits_constraint: true,
        }
    }
 
    fn needs_field_cast(&self) -> bool {
        true
    }

    fn constraint_stmts(&self, arg: &AccountArg, ctx: &HandlerCtx) -> syn::Result<TokenStream> {
        let account_ident = &ctx.fields[ctx.field_index].ident;
        let sibling_ident = arg.as_ident_value().ok_or_else(|| {
            syn::Error::new(arg.key().span(), "has_one requires `has_one = <field>`")
        })?;

        if is_dyn_account(&ctx.fields[ctx.field_index].ty) {
            return Err(syn::Error::new_spanned(
                account_ident,
                "has_one is not supported for DynAccount type",
            ));
        }

        let cast_ident = quote::format_ident!("__cast_{}", account_ident);

        Ok(quote! {
            if ::hayabusa::hint::unlikely(
                #sibling_ident.address() != &#cast_ident.#sibling_ident,
            ) {
                return Err(::hayabusa::prelude::ErrorCode::InvalidAccount.into());
            }
        })
    }
}

fn is_dyn_account(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        for segment in &type_path.path.segments {
            if segment.ident == "DynAccount" {
                return true;
            }
            // Recurse into generic args like Mut<DynAccount<...>>
            if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                for arg in &args.args {
                    if let syn::GenericArgument::Type(inner_ty) = arg {
                        if is_dyn_account(inner_ty) {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}