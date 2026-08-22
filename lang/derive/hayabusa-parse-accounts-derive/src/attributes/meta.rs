// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use crate::attributes::{AccountArg, AttributeHandler, HandlerRole};
use syn::{visit::{self, Visit}, visit_mut::{self, VisitMut}, Expr, Ident};
use std::collections::HashSet;

/// Handler for `#[account(meta = <expr>)]`.
///
/// This attribute is **only valid on types that are not in the built-in
/// account wrapper registry** (i.e. not `Init`, `InitPda`, `Pda`, or their
/// `Mut<…>` variants).  It is the extension point for externally-implemented
/// account wrappers that need to supply a custom meta value to their
/// `ParseAccount` impl.
///
/// Syntax:
/// ```rust
/// #[account(meta = MyMeta { field_1: foo, field_2: bar.address() })]
/// my_account: ExternalWrapper<'view, MyState>,
/// ```
///
/// The expression is emitted verbatim in place of the `NoMeta` that would
/// otherwise be used for plain/unknown field shapes.
pub struct MetaHandler;

impl AttributeHandler for MetaHandler {
    fn key(&self) -> &'static str {
        "meta"
    }

    fn role(&self) -> HandlerRole {
        HandlerRole {
            // Participates in meta construction — it IS the meta.
            builds_meta: true,
            emits_constraint: false,
        }
    }

    /// If the meta expression references other fields by name we cannot
    /// statically know which ones without evaluating arbitrary Rust
    /// expressions, so we conservatively declare no ordering dependencies.
    ///
    /// Users who need ordering must express it through `payer =` or other
    /// attributes that carry explicit field references, or simply declare
    /// fields in dependency order.
    fn dependencies(&self, arg: &AccountArg) -> Vec<String> {
        if let AccountArg::KeyMeta { deps, .. } = arg {
            deps.iter().map(|i| i.to_string()).collect()
        } else {
            vec![]
        }
    }
}

pub fn extract_meta_expr(args: &[AccountArg]) -> Option<&Expr> {
    args.iter().find_map(|a| {
        if let AccountArg::KeyMeta { value, .. } = a { Some(value) } else { None }
    })
}

struct IdentCollector<'a> {
    field_names: &'a HashSet<String>,
    found: Vec<Ident>,
}

impl<'ast> Visit<'ast> for IdentCollector<'_> {
    fn visit_expr_struct(&mut self, node: &'ast syn::ExprStruct) {
        // skip node.path (the struct name) and field keys — only visit values
        for field in &node.fields {
            visit::visit_expr(self, &field.expr);
        }
        if let Some(rest) = &node.rest {
            visit::visit_expr(self, rest);
        }
    }

    fn visit_ident(&mut self, ident: &'ast Ident) {
        let name = ident.to_string();
        if self.field_names.contains(&name)
            && !self.found.iter().any(|i| i == ident)
        {
            self.found.push(ident.clone());
        }
    }
}

pub fn collect_field_idents(expr: &Expr, field_names: &HashSet<String>) -> Vec<Ident> {
    let mut collector = IdentCollector { field_names, found: vec![] };
    visit::visit_expr(&mut collector, expr);
    collector.found
}

struct CastRewriter<'a> {
    field_names: &'a HashSet<String>,
    rewritten: HashSet<String>,
}

impl VisitMut for CastRewriter<'_> {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        // Match `<field>.cast()?` → ExprTry wrapping ExprMethodCall
        if let Expr::Try(try_expr) = expr {
            if let Expr::MethodCall(mc) = &*try_expr.expr {
                if mc.method == "cast" && mc.args.is_empty() {
                    if let Expr::Path(ref p) = *mc.receiver {
                        if let Some(ident) = p.path.get_ident() {
                            let name = ident.to_string();
                            if self.field_names.contains(&name) {
                                self.rewritten.insert(name);
                                let cache_ident = quote::format_ident!("__cast_{}", ident);
                                *expr = syn::parse_quote! { #cache_ident };
                                return;
                            }
                        }
                    }
                }
            }
        }
        visit_mut::visit_expr_mut(self, expr);
    }
}

/// Rewrite every `<field>.cast()?` in `expr` to `__cast_<field>`,
/// returning the set of field names that were rewritten.
pub fn rewrite_casts(expr: &mut Expr, field_names: &HashSet<String>) -> HashSet<String> {
    let mut rewriter = CastRewriter { field_names, rewritten: HashSet::new() };
    visit_mut::visit_expr_mut(&mut rewriter, expr);
    rewriter.rewritten
}