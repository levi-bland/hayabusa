// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

pub mod bump;
pub mod has_one;
pub mod meta;
pub mod payer;
pub mod seeds;
pub mod spl;

use proc_macro2::TokenStream;
use syn::{
    bracketed,
    parse::ParseStream,
    punctuated::Punctuated,
    token, Expr, Ident, Result, Token,
};
use has_one::HasOneHandler;
use payer::PayerHandler;
use spl::{MintHandler, OwnerHandler};
use seeds::SeedsHandler;
use bump::BumpHandler;

use crate::attributes::meta::MetaHandler;

#[derive(Clone)]
pub enum AccountArg {
    /// `key = value`  where value is a single ident  (e.g. `payer = signer`)
    KeyValue { key: Ident, value: Ident },
 
    /// `key = <expr>` where value is an arbitrary expression
    /// Used by `bump = counter.bump` or `bump = some::expr()`
    KeyExpr { key: Ident, value: Expr },
 
    /// `key = [expr, expr, ...]`  (e.g. `seeds = [Counter::SEED, x.as_ref()]`)
    BracketList { key: Ident, exprs: Vec<Expr> },
 
    /// `key`  bare flag with no value  (e.g. `bump`)
    BareFlag { key: Ident },

    /// `meta = <expr>` — stores the expression plus all idents from it
    /// that matched known field names, for toposort dependency resolution.
    KeyMeta { key: Ident, value: Expr, deps: Vec<Ident> },
}
 
impl AccountArg {
    pub fn key(&self) -> &Ident {
        match self {
            AccountArg::KeyValue   { key, .. } => key,
            AccountArg::KeyExpr    { key, .. } => key,
            AccountArg::BracketList{ key, .. } => key,
            AccountArg::BareFlag   { key }     => key,
            AccountArg::KeyMeta    { key, ..} => key,
        }
    }
 
    /// Returns the single-ident value for `KeyValue`, or `None`.
    pub fn as_ident_value(&self) -> Option<&Ident> {
        match self {
            AccountArg::KeyValue { value, .. } => Some(value),
            _ => None,
        }
    }
 
    /// Returns the expression for `KeyExpr`, or `None`.
    pub fn as_expr_value(&self) -> Option<&Expr> {
        match self {
            AccountArg::KeyExpr { value, .. } => Some(value),
            AccountArg::KeyMeta { value, .. } => Some(value),
            _ => None,
        }
    }
 
    /// Returns the expression list for `BracketList`, or `None`.
    pub fn as_bracket_list(&self) -> Option<&[Expr]> {
        match self {
            AccountArg::BracketList { exprs, .. } => Some(exprs),
            _ => None,
        }
    }
 
    pub fn is_bare_flag(&self) -> bool {
        matches!(self, AccountArg::BareFlag { .. })
    }
}
 
/// Parse one argument inside `#[account(...)]`.
///
/// Grammar:
///   arg = IDENT ( "=" ( "[" expr,* "]" | expr ) )?
///
/// A bare IDENT (no `=`) becomes `BareFlag`.
/// `IDENT = [...]` becomes `BracketList`.
/// `IDENT = single_ident` becomes `KeyValue` (enables dep-tracking by name).
/// `IDENT = <other expr>` becomes `KeyExpr`.
pub fn parse_account_arg(input: ParseStream) -> Result<AccountArg> {
    let key: Ident = input.parse()?;
 
    if !input.peek(Token![=]) {
        return Ok(AccountArg::BareFlag { key });
    }
 
    let _eq: Token![=] = input.parse()?;
 
    // `seeds = [...]`
    if input.peek(token::Bracket) {
        let content;
        bracketed!(content in input);
        let exprs = Punctuated::<Expr, Token![,]>::parse_terminated(&content)?
            .into_iter()
            .collect();
        return Ok(AccountArg::BracketList { key, exprs });
    }

    if key == "meta" {
        let value: Expr = input.parse()?;
        return Ok(AccountArg::KeyMeta { key, value, deps: vec![] });
    }
 
    // `key = <expr>` — try to detect single-ident vs arbitrary expression
    let expr: Expr = input.parse()?;
    if let Expr::Path(ref p) = expr {
        if let Some(ident) = p.path.get_ident() {
            return Ok(AccountArg::KeyValue {
                key,
                value: ident.clone(),
            });
        }
    }
 
    Ok(AccountArg::KeyExpr { key, value: expr })
}
 
// ---------------------------------------------------------------------------
// 2. Handler trait
// ---------------------------------------------------------------------------
 
pub struct HandlerCtx<'a> {
    pub fields: &'a [crate::AccountField],
    pub field_index: usize,
}
 
pub struct HandlerRole {
    /// Participates in building the Meta passed to `ParseAccount::parse`.
    pub builds_meta: bool,
    /// Emits statements after all accounts are constructed (runtime checks).
    pub emits_constraint: bool,
}

#[allow(unused)]
pub trait AttributeHandler: Send + Sync {
    fn key(&self) -> &'static str;
    fn role(&self) -> HandlerRole;
 
    /// Field names that must be *constructed* before this field.
    /// Only consulted when `builds_meta` is true.
    fn dependencies(&self, arg: &AccountArg) -> Vec<String> {
        let _ = arg;
        vec![]
    }
 
    /// Meta expression contributed by this handler, if any.
    /// Only called when `builds_meta` is true.
    fn meta_expr(&self, arg: &AccountArg, ctx: &HandlerCtx) -> Result<Option<TokenStream>> {
        let _ = (arg, ctx);
        Ok(None)
    }

    /// Whether this handler's constraint code needs a cached
    /// `cast_untracked()` binding for the owning field.
    fn needs_field_cast(&self) -> bool {
        false
    }
 
    /// Post-construction constraint statements.
    /// Only called when `emits_constraint` is true.
    fn constraint_stmts(&self, arg: &AccountArg, ctx: &HandlerCtx) -> Result<TokenStream> {
        let _ = (arg, ctx);
        Ok(TokenStream::new())
    }
}
 
pub static HANDLERS: &[&(dyn AttributeHandler + Sync)] = &[
    &PayerHandler,
    &HasOneHandler,
    &SeedsHandler,
    &BumpHandler,
    &MintHandler,
    &OwnerHandler,
    &MetaHandler,
];
 
pub fn find_handler(key: &Ident) -> Result<&'static (dyn AttributeHandler + Sync + 'static)> {
    let s = key.to_string();
    HANDLERS
        .iter()
        .find(|h| h.key() == s)
        .copied()
        .ok_or_else(|| syn::Error::new(key.span(), format!("unknown account attribute `{s}`")))
}