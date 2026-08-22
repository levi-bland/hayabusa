// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;
 
use crate::attributes::{AccountArg, AttributeHandler, HandlerRole};
 
pub struct MintHandler;
pub struct OwnerHandler;
 
/// Implement trivially for `mint =` and `owner =` — they carry no
/// independent behaviour.  All SPL-specific logic (meta construction +
/// the `SplAccount` assertion) lives in `meta::resolve_meta`, which
/// detects the combination of payer + mint + owner being present.
macro_rules! passthrough_handler {
    ($handler:ty, $key:literal) => {
        impl AttributeHandler for $handler {
            fn key(&self) -> &'static str {
                $key
            }
 
            fn role(&self) -> HandlerRole {
                HandlerRole {
                    builds_meta: true,   // participates in meta construction
                    emits_constraint: false,
                }
            }
 
            // No ordering dependency — the referenced field just needs to
            // exist, not be constructed first.  (mint/owner are read-only
            // references, not cloned accounts that must be live.)
            fn dependencies(&self, _arg: &AccountArg) -> Vec<String> {
                vec![]
            }
        }
    };
}
 
passthrough_handler!(MintHandler,  "mint");
passthrough_handler!(OwnerHandler, "owner");
 
// ---------------------------------------------------------------------------
// Assertion helper — called by meta::resolve_meta
// ---------------------------------------------------------------------------
 
/// Emit a `const _` block that fails at compile time if `inner_t` does not
/// implement `::hayabusa::SplAccount`.
///
/// Because `SplAccount` is a sealed trait (private supertrait in the runtime
/// crate), any user-defined type named `TokenAccount` will produce:
///
/// ```text
/// error[E0277]: the trait bound `TokenAccount: __SplAccountMarker` is not satisfied
/// ```
pub fn spl_account_assert(inner_t: &Type) -> TokenStream {
    quote! {
        const _: () = {
            const fn __assert_spl_account<__T: ::hayabusa::traits::__SplAccountMarker>() {}
            __assert_spl_account::<#inner_t>();
        };
    }
}