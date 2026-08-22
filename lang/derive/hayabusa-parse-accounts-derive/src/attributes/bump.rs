// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

use crate::attributes::{AccountArg, AttributeHandler, HandlerRole};

pub struct BumpHandler;

impl AttributeHandler for BumpHandler {
    fn key(&self) -> &'static str { "bump" }

    fn role(&self) -> HandlerRole {
        HandlerRole { builds_meta: true, emits_constraint: false }
    }
}

/// The resolved form of a `bump` argument.
pub enum BumpKind<'a> {
    /// `bump` — bare flag.  The PDA bump is found at parse time and stored
    /// into the generated `Bumps` struct field.
    Find,

    /// `bump = <this_field>.<subfield>` — self-referencing shorthand.
    /// Expands to `T::cast(<this_field>_view)?.<subfield>` and is appended
    /// as the last seed slice element (`&[<value>]`).
    SelfRef { subfield: &'a syn::Ident },

    /// `bump = <arbitrary expr>` — passed verbatim as the bump byte,
    /// appended as `&[<expr>]` to the seeds.
    Expr { expr: &'a syn::Expr },
}

/// Classify the `bump` arg for `field` at `field_index`.
///
/// `field_ident` is the name of the field being processed, used to detect
/// the self-referencing pattern `bump = <field_ident>.<something>`.
pub fn classify_bump<'a>(
    args: &'a [AccountArg],
    field_ident: &syn::Ident,
) -> Option<BumpKind<'a>> {
    let arg = args.iter().find(|a| a.key() == "bump")?;

    if arg.is_bare_flag() {
        return Some(BumpKind::Find);
    }

    // `bump = <expr>` — detect `<field_ident>.<subfield>` self-ref
    if let Some(expr) = arg.as_expr_value() {
        if let syn::Expr::Field(ref ef) = expr {
            if let syn::Expr::Path(ref base) = *ef.base {
                if base.path.is_ident(field_ident) {
                    if let syn::Member::Named(ref subfield) = ef.member {
                        return Some(BumpKind::SelfRef { subfield });
                    }
                }
            }
        }
        return Some(BumpKind::Expr { expr });
    }

    // `bump = single_ident` — treat as KeyValue, self-ref check still applies
    if let Some(ident) = arg.as_ident_value() {
        // A bare ident can't be `<field>.<subfield>` so it's always an expr
        let _ = ident;
    }

    None
}

pub fn require_bump_kind_find<'a>(
    args: &'a [AccountArg],
) -> BumpKind<'a> {
    let arg = args.iter().find(|a| a.key() == "bump").expect("bump attribute is required");

    if !arg.is_bare_flag() {
        panic!("bump must be a bare flag");
    }
    
    BumpKind::Find
}