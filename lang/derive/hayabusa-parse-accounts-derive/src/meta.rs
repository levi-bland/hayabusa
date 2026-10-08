// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{GenericArgument, PathArguments, Type};

use crate::{
    attributes::{
        bump::{classify_bump, require_bump_kind_find, BumpKind},
        payer::extract_payer,
        seeds::extract_seeds,
        spl::spl_account_assert,
        AccountArg,
    },
    AccountField,
};

// ---------------------------------------------------------------------------
// Wrapper detection
// ---------------------------------------------------------------------------

/// The relevant structural shape of a field type.
pub enum FieldShape {
    /// No init/pda wrapper — plain parse.
    Plain,
    /// `Init<Account<'_, T>>` — account initialisation.
    Init,
    /// `Init<Pda<Account<'_, T>>>` — PDA account initialisation.
    InitPda,
    /// `Pda<Account<'_, T>>` or `Mut<Pda<Account<'_, T>>>` — existing PDA validation.
    Pda,
}

pub fn detect_shape(ty: &Type) -> FieldShape {
    match outermost_ident(ty).as_deref() {
        Some("Init") => {
            if peel_single_generic(ty, "Init")
                .and_then(|t| outermost_ident(t))
                .as_deref()
                == Some("Pda")
            {
                FieldShape::InitPda
            } else {
                FieldShape::Init
            }
        }
        Some("Pda") => FieldShape::Pda,
        Some("Mut") => {
            // Mut<Pda<...>>
            if peel_single_generic(ty, "Mut")
                .and_then(|t| outermost_ident(t))
                .as_deref()
                == Some("Pda")
            {
                FieldShape::Pda
            } else {
                FieldShape::Plain
            }
        }
        _ => FieldShape::Plain,
    }
}

fn outermost_ident(ty: &Type) -> Option<String> {
    if let Type::Path(tp) = ty {
        tp.path.segments.last().map(|s| s.ident.to_string())
    } else {
        None
    }
}

fn peel_single_generic<'t>(ty: &'t Type, wrapper: &str) -> Option<&'t Type> {
    if let Type::Path(tp) = ty {
        if let Some(last) = tp.path.segments.last() {
            if last.ident == wrapper {
                if let PathArguments::AngleBracketed(ref ab) = last.arguments {
                    if let Some(GenericArgument::Type(inner)) = ab.args.first() {
                        return Some(inner);
                    }
                }
            }
        }
    }
    None
}

/// Extract the inner `T` from `Account<'_, T>`, peeling any number of
/// `Init`, `Pda`, `Mut` wrappers first.
pub fn extract_inner_t(ty: &Type) -> Option<&Type> {
    let mut cur = ty;
    loop {
        match outermost_ident(cur).as_deref() {
            Some("Init") | Some("Pda") | Some("Mut") => {
                cur = peel_single_generic(cur, outermost_ident(cur).unwrap().as_str())?;
            }
            Some("Account") => {
                if let Type::Path(tp) = cur {
                    if let Some(last) = tp.path.segments.last() {
                        if let PathArguments::AngleBracketed(ref ab) = last.arguments {
                            if let Some(GenericArgument::Type(t)) = ab.args.iter().nth(1) {
                                return Some(t);
                            }
                        }
                    }
                }
                return None;
            }
            _ => return None,
        }
    }
}

// ---------------------------------------------------------------------------
// SPL detection
// ---------------------------------------------------------------------------

fn is_spl_token_init(field: &AccountField) -> bool {
    let has = |k: &str| field.args.iter().any(|a| a.key() == k);
    has("payer") && has("mint") && has("owner")
}

// ---------------------------------------------------------------------------
// Bump helpers
// ---------------------------------------------------------------------------

/// Build the `__PdaByteSliceMeta::new(seeds, bump_ptr)` expression for a
/// `Pda` field and return any bump_ptr argument for the bumps struct.
///
/// Returns `(meta_expr, needs_find_bump)` where `needs_find_bump` signals
/// that this field participates in the generated `Bumps` struct.
fn pda_meta(
    field: &AccountField,
    _field_index: usize,
    _fields: &[AccountField],
) -> syn::Result<(TokenStream, bool)> {
    let seeds = extract_seeds(&field.args).ok_or_else(|| {
        syn::Error::new(
            field.ident.span(),
            "Pda account requires `#[account(seeds = [...])]`",
        )
    })?;

    let bump_kind = classify_bump(&field.args, &field.ident);
    let field_ident = &field.ident;
    let view_ident = format_ident!("{}_view", field_ident);

    if !is_pda(&field.ty) {
        return Err(syn::Error::new_spanned(
            field_ident,
            "type must include `Pda<T>`",
        ));
    }

    // Build seed slice elements, appending the bump as a final `&[bump]`
    // when it is a known value.
    let mut seed_elems: Vec<TokenStream> = seeds.iter().map(|e| quote! { #e }).collect();

    let (bump_ptr_arg, needs_find_bump) = match bump_kind {
        None => {
            // No bump attribute at all, `bump_ptr: None`, no extra seed
            (quote! { None }, false)
        }

        Some(BumpKind::Find) => {
            // `bump` bare, find the PDA, store bump into bumps.<field>
            (quote! { Some(&mut bumps.#field_ident) }, true)
        }

        Some(BumpKind::SelfRef { subfield }) => {
            // `bump = <this_field>.<subfield>`, read from on-chain data
            // We need the inner T to call T::cast(view)?
            let inner_t = extract_inner_t(&field.ty).ok_or_else(|| {
                syn::Error::new(
                    field.ident.span(),
                    "could not extract inner T for bump cast",
                )
            })?;
            seed_elems.push(quote! {
                &[#inner_t::cast(#view_ident)?.#subfield]
            });
            (quote! { None }, false)
        }

        Some(BumpKind::Expr { expr }) => {
            // `bump = <arbitrary expr>`
            seed_elems.push(quote! { &[#expr] });
            (quote! { None }, false)
        }
    };

    let meta = quote! {
        ::hayabusa::prelude::__PdaByteSliceMeta::new(&[#(#seed_elems),*], #bump_ptr_arg)
    };

    Ok((meta, needs_find_bump))
}

fn pda_init_meta<'a>(field: &'a AccountField) -> syn::Result<(TokenStream, TokenStream)> {
    let seeds = extract_seeds(&field.args).ok_or_else(|| {
        syn::Error::new(
            field.ident.span(),
            "Pda account requires `#[account(seeds = [...])]`",
        )
    })?;

    require_bump_kind_find(&field.args);
    let field_ident = &field.ident;

    if !is_pda(&field.ty) {
        return Err(syn::Error::new_spanned(
            field_ident,
            "type must include `Pda<T>`",
        ));
    }

    let seed_elems: Vec<TokenStream> = seeds.iter().map(|e| quote! { #e }).collect();
    let initial_len = seed_elems.len();

    let ts = quote! {
        let mut __seeds_buffer = ::hayabusa::prelude::seeds!(#(#seed_elems),*, ::hayabusa::prelude::Seed::from(&[][..]));
        let __growable_signer = ::hayabusa::prelude::GrowableSigner::try_new(&mut __seeds_buffer, #initial_len)?;
    };

    Ok((quote! { Some(&mut bumps.#field_ident) }, ts))
}

fn is_pda(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        for segment in &type_path.path.segments {
            if segment.ident == "Pda" {
                return true;
            }
            // Recurse into generic args like Mut<DynAccount<...>>
            if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                for arg in &args.args {
                    if let syn::GenericArgument::Type(inner_ty) = arg {
                        if is_pda(inner_ty) {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Returns `(meta_expr, extra_items, preceeding_expressions, needs_find_bump)`.
///
/// `extra_items` - top-level `const _` assertion blocks (SPL trait checks).
/// `preceeding_expressions` - any expressions that must be evaluated before the meta expression.
/// `needs_find_bump` - true when this field's bump should appear in the
///                     generated `Bumps` struct.
pub fn resolve_meta(
    fields: &[AccountField],
    field_index: usize,
) -> syn::Result<(TokenStream, TokenStream, TokenStream, bool)> {
    let field = &fields[field_index];

    // Guard: `meta =` is only valid on external wrappers (FieldShape::Plain
    // on a non-built-in type). Reject it on all built-in shapes up front.
    if let Some(arg) = field
        .args
        .iter()
        .find(|a| matches!(a, AccountArg::KeyMeta { .. }))
    {
        if crate::registry::is_known_wrapper(&field.ty) {
            return Err(syn::Error::new(
                arg.key().span(),
                "`meta` is not allowed on built-in account wrappers",
            ));
        }
    }

    match detect_shape(&field.ty) {
        FieldShape::Plain => match crate::attributes::meta::extract_meta_expr(&field.args) {
            Some(expr) => Ok((quote! { #expr }, quote! {}, quote! {}, false)),
            None => Ok((quote! { NoMeta }, quote! {}, quote! {}, false)),
        },

        FieldShape::Pda => {
            let (meta, needs_find_bump) = pda_meta(field, field_index, fields)?;
            Ok((meta, quote! {}, quote! {}, needs_find_bump))
        }

        FieldShape::Init => {
            let payer = require_payer(field)?;
            if is_spl_token_init(field) {
                let mint = require_arg(field, "mint")?;
                let owner = require_arg(field, "owner")?;
                let inner_t = require_inner_t(field)?;
                let assertion = spl_account_assert(inner_t);
                Ok((
                    quote! {
                        ::hayabusa_spl_token::state::token_account::__TokenAccountInitMeta {
                            payer: &#payer,
                            mint:  &#mint,
                            owner: #owner.address(),
                        }
                    },
                    assertion,
                    quote! {},
                    false,
                ))
            } else {
                Ok((
                    quote! { __AccountInitMeta { payer: &#payer } },
                    quote! {},
                    quote! {},
                    false,
                ))
            }
        }

        FieldShape::InitPda => {
            let payer = require_payer(field)?;
            if is_spl_token_init(field) {
                let mint = require_arg(field, "mint")?;
                let owner = require_arg(field, "owner")?;
                let inner_t = require_inner_t(field)?;
                let assertion = spl_account_assert(inner_t);
                Ok((
                    quote! {
                        ::hayabusa_spl_token::state::token_account::__TokenAccountPdaInitMeta::new(
                            &#payer,
                            &#mint,
                            &#owner,
                            __cpi_signer,
                            __bump_ref,
                        )
                    },
                    assertion,
                    quote! {},
                    false,
                ))
            } else {
                let (bump_ts, signer_ts) = pda_init_meta(field)?;
                Ok((
                    quote! {
                        ::hayabusa::prelude::__AccountPdaInitMeta::new(
                            &#payer,
                            ::hayabusa::prelude::SignerBumpness::without(__growable_signer, #bump_ts),
                        )
                    },
                    quote! {},
                    signer_ts,
                    true,
                ))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn require_payer(field: &AccountField) -> syn::Result<&syn::Ident> {
    extract_payer(&field.args).ok_or_else(|| {
        syn::Error::new(
            field.ident.span(),
            "Init account requires `#[account(payer = <field>)]`",
        )
    })
}

fn require_arg<'f>(field: &'f AccountField, key: &str) -> syn::Result<&'f syn::Ident> {
    field
        .args
        .iter()
        .find(|a| a.key() == key)
        .and_then(|a| a.as_ident_value())
        .ok_or_else(|| {
            syn::Error::new(
                field.ident.span(),
                format!("this account type requires `#[account({key} = <field>)]`"),
            )
        })
}

fn require_inner_t(field: &AccountField) -> syn::Result<&Type> {
    extract_inner_t(&field.ty).ok_or_else(|| {
        syn::Error::new(
            field.ident.span(),
            "could not extract inner type T from Account<'_, T>",
        )
    })
}
