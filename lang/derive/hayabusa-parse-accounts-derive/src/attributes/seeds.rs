// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

use crate::attributes::{AccountArg, AttributeHandler, HandlerRole};

pub struct SeedsHandler;

impl AttributeHandler for SeedsHandler {
    fn key(&self) -> &'static str { "seeds" }

    fn role(&self) -> HandlerRole {
        // Seeds feed into __PdaByteSliceMeta, which is assembled by meta.rs.
        // This handler is a tag — the actual codegen is in meta::resolve_meta.
        HandlerRole { builds_meta: true, emits_constraint: false }
    }
}

/// Extract the seed expression list from a field's args.
pub fn extract_seeds(args: &[AccountArg]) -> Option<&[syn::Expr]> {
    args.iter()
        .find(|a| a.key() == "seeds")
        .and_then(|a| a.as_bracket_list())
}