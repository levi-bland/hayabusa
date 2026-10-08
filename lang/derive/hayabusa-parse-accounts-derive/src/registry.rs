// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use syn::Type;

/// Every first-party account wrapper shipped with the runtime.
///
/// `Mut` and `Pda` are **transparent** — they are peeled before the registry
/// lookup so that e.g. `Mut<Signer<'_>>` resolves to `Signer` (known) and
/// `Mut<ExternalWrapper<'_>>` resolves to `ExternalWrapper` (unknown).
///
/// `Init` and `InitPda` are handled by `detect_shape` before `resolve_meta`
/// ever reaches the registry, so they do not need to appear here.
const KNOWN_WRAPPERS: &[&str] = &[
    "Account",
    "DynAccount",
    "Program",
    "Signer",
    "SystemAccount",
    "Sysvar",
    "UncheckedAccount",
    "AccountView",
];

/// Returns `true` when `ty` (after peeling any number of transparent `Mut`
/// and `Pda` layers) names a built-in account wrapper.
///
/// External wrappers — anything not in [`KNOWN_WRAPPERS`] — return `false`.
pub fn is_known_wrapper(ty: &Type) -> bool {
    match effective_ident(ty) {
        Some(name) => KNOWN_WRAPPERS.contains(&name.as_str()),
        None => false,
    }
}

/// Peel `Mut<…>` and `Pda<…>` transparent wrappers, then return the
/// outermost ident of whatever remains.
///
/// Returns `None` for non-path types (references, tuples, …).
fn effective_ident(ty: &Type) -> Option<String> {
    let mut cur = ty;
    loop {
        let ident = outermost_ident(cur)?;
        if ident == "Mut" || ident == "Pda" {
            cur = peel_single_generic(cur, &ident)?;
        } else {
            return Some(ident);
        }
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
    use syn::{GenericArgument, PathArguments};
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
