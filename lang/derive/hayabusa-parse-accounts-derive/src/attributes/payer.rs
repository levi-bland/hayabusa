// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0
 
use super::{AccountArg, AttributeHandler, HandlerRole};
 
pub struct PayerHandler;
 
impl AttributeHandler for PayerHandler {
    fn key(&self) -> &'static str { "payer" }
 
    fn role(&self) -> HandlerRole {
        HandlerRole { builds_meta: true, emits_constraint: false }
    }
 
    fn dependencies(&self, arg: &AccountArg) -> Vec<String> {
        // payer field must be constructed before this field
        arg.as_ident_value()
            .map(|v| vec![v.to_string()])
            .unwrap_or_default()
    }
}
 
pub fn extract_payer(args: &[AccountArg]) -> Option<&syn::Ident> {
    args.iter()
        .find(|a| a.key() == "payer")
        .and_then(|a| a.as_ident_value())
}