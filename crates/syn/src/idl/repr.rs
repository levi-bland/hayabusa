// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

use hayabusa_idl_types::{IdlRepr, IdlReprModifier};
use syn::{Attribute, Meta};

/// Parses the `#[repr(...)]` attribute from a struct's attributes into an [`IdlRepr`].
/// Defaults to [`IdlRepr::Rust`] with no modifiers if no `repr` attribute is present.
pub fn parse_idl_repr(attrs: &[Attribute]) -> IdlRepr {
    let repr_str = attrs
        .iter()
        .find(|attr| attr.path().is_ident("repr"))
        .and_then(|attr| {
            if let Meta::List(meta) = &attr.meta {
                Some(meta.tokens.to_string())
            } else {
                None
            }
        });

    let Some(repr) = repr_str else {
        return IdlRepr::Rust(IdlReprModifier {
            packed: false,
            align: None,
        });
    };

    // tokenize on commas
    let parts: Vec<&str> = repr.split(',').map(|s| s.trim()).collect();

    if parts.contains(&"transparent") {
        return IdlRepr::Transparent;
    }

    let packed = parts.contains(&"packed");

    let align = parts.iter().find_map(|p| {
        p.strip_prefix("align(")
            .and_then(|s| s.strip_suffix(")"))
            .and_then(|n| n.trim().parse::<usize>().ok())
    });

    let modifier = IdlReprModifier { packed, align };

    if parts.contains(&"C") {
        IdlRepr::C(modifier)
    } else {
        IdlRepr::Rust(modifier)
    }
}
