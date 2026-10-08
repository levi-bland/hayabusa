use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, Result};

use crate::{
    attributes::{find_handler, meta::collect_field_idents, parse_account_arg, AccountArg},
    meta::resolve_meta,
    toposort::toposort,
    AccountField,
};

pub fn expand(input: DeriveInput) -> Result<TokenStream> {
    // -----------------------------------------------------------------------
    // 1. Extract named struct fields
    // -----------------------------------------------------------------------
    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => &f.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    &input,
                    "ParseAccounts requires named fields",
                ))
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                &input,
                "ParseAccounts can only be derived on structs",
            ))
        }
    };

    // -----------------------------------------------------------------------
    // 2. Parse each field + its #[account(...)] attributes
    // -----------------------------------------------------------------------
    let mut account_fields: Vec<AccountField> = Vec::new();

    for field in fields {
        let ident = field.ident.clone().unwrap();
        let ty = field.ty.clone();
        let mut args = Vec::new();

        for attr in &field.attrs {
            if !attr.path().is_ident("account") {
                continue;
            }
            // Parse as a raw token stream so we can use our own arg parser
            // which handles `seeds = [...]` and bare `bump`.
            let parsed_args: Vec<_> = attr.parse_args_with(|input: syn::parse::ParseStream| {
                let mut local = Vec::new();
                while !input.is_empty() {
                    let arg = parse_account_arg(input)?;
                    find_handler(arg.key())?;
                    local.push(arg);
                    if input.is_empty() {
                        break;
                    }
                    input.parse::<syn::Token![,]>()?;
                    // allow trailing comma
                }
                Ok(local)
            })?;
            args.extend(parsed_args);
        }

        account_fields.push(AccountField { ident, ty, args });
    }

    // Collect field names for meta dep resolution
    let field_names_set: std::collections::HashSet<String> =
        account_fields.iter().map(|f| f.ident.to_string()).collect();

    // Fill deps on any KeyMeta args
    for field in &mut account_fields {
        for arg in &mut field.args {
            if let AccountArg::KeyMeta { value, deps, .. } = arg {
                *deps = collect_field_idents(value, &field_names_set);
            }
        }
    }

    // -----------------------------------------------------------------------
    // 3. Toposort
    // -----------------------------------------------------------------------
    let construction_order = toposort(&account_fields)?;

    // -----------------------------------------------------------------------
    // 3b. Rewrite .cast() calls in meta exprs → cached bindings
    // -----------------------------------------------------------------------
    let mut cast_fields: std::collections::HashSet<String> = std::collections::HashSet::new();
    for field in &mut account_fields {
        for arg in &mut field.args {
            if let AccountArg::KeyMeta { value, .. } = arg {
                cast_fields.extend(crate::attributes::meta::rewrite_casts(
                    value,
                    &field_names_set,
                ));
            }
        }
    }

    // -----------------------------------------------------------------------
    // 4. Resolve metas + collect which fields need bump storage
    // -----------------------------------------------------------------------
    let mut extra_items: Vec<TokenStream> = Vec::new();
    let mut find_bump_fields: Vec<&syn::Ident> = Vec::new(); // fields with bare `bump`

    let construction_stmts: Vec<TokenStream> = construction_order
        .iter()
        .map(|&idx| {
            let field = &account_fields[idx];
            let ident = &field.ident;
            let ty = &field.ty;
            let view_ident = format_ident!("{}_view", ident);
            let (meta, extra, preceding_exprs, needs_find_bump) =
                resolve_meta(&account_fields, idx)?;
            extra_items.push(extra);
            if needs_find_bump {
                find_bump_fields.push(ident);
            }

            let mut stmts = quote! {
                #preceding_exprs
                let #ident: #ty = <#ty as ParseAccount<'_>>::parse(#view_ident, &mut #meta)?;
            };

            // Emit a cached cast binding if any meta expression rewrote
            // a .cast()? on this field.
            if cast_fields.contains(&ident.to_string()) {
                let cast_ident = format_ident!("__cast_{}", ident);
                stmts.extend(quote! {
                    let #cast_ident = #ident.cast()?;
                });
            }

            Ok(stmts)
        })
        .collect::<Result<Vec<_>>>()?;

    // -----------------------------------------------------------------------
    // 5. Post-construction: cached casts + constraint statements
    // -----------------------------------------------------------------------

    // Emit one `let __cast_<field> = unsafe { <field>.cast_untracked() };`
    // per field that has at least one constraint handler needing a cast.
    let mut cast_bindings: Vec<TokenStream> = Vec::new();
    let mut cast_emitted = std::collections::HashSet::new();

    for (field_index, field) in account_fields.iter().enumerate() {
        let needs = field.args.iter().any(|arg| {
            find_handler(arg.key())
                .map_or(false, |h| h.role().emits_constraint && h.needs_field_cast())
        });
        // Fields whose meta expressions already produced a `__cast_<field>`
        // binding during construction reuse it instead of rebinding.
        if needs
            && !cast_fields.contains(&field.ident.to_string())
            && cast_emitted.insert(field_index)
        {
            let ident = &field.ident;
            let cast_ident = format_ident!("__cast_{}", ident);
            cast_bindings.push(quote! {
                let #cast_ident = unsafe { #ident.cast_untracked() };
            });
        }
    }

    let constraint_stmts: Vec<TokenStream> = account_fields
        .iter()
        .enumerate()
        .map(|(field_index, field)| {
            let ctx = crate::attributes::HandlerCtx {
                fields: &account_fields,
                field_index,
            };
            let mut stmts = TokenStream::new();
            for arg in &field.args {
                let handler = find_handler(arg.key())?;
                if handler.role().emits_constraint {
                    stmts.extend(handler.constraint_stmts(arg, &ctx)?);
                }
            }
            Ok(stmts)
        })
        .collect::<Result<Vec<_>>>()?;

    // -----------------------------------------------------------------------
    // 6. View bindings (always declaration order)
    // -----------------------------------------------------------------------
    let view_bindings: Vec<TokenStream> = account_fields
        .iter()
        .map(|f| {
            let view_ident = format_ident!("{}_view", f.ident);
            quote! { let #view_ident = unsafe { views.next_unchecked() }; }
        })
        .collect();

    let field_names: Vec<_> = account_fields.iter().map(|f| &f.ident).collect();
    let num_accounts = account_fields.len();
    let struct_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // -----------------------------------------------------------------------
    // 7. Bumps type + impl
    // -----------------------------------------------------------------------
    //
    // If any field has a bare `bump`, generate:
    //   pub struct <Struct>Bumps { pub <field>: u8, ... }
    //   impl Bumps for <Struct><'_> { type Bumps = <Struct>Bumps; }
    //
    // Otherwise:
    //   impl Bumps for <Struct><'_> { type Bumps = (); }
    let (bumps_type, bumps_impl, bumps_param) = if find_bump_fields.is_empty() {
        (
            quote! {},
            quote! {
                #[automatically_derived]
                impl Bumps for #struct_name<'_> {
                    type Bumps = ();
                }
            },
            quote! { () },
        )
    } else {
        let bumps_struct_name = format_ident!("{}Bumps", struct_name);
        let bump_field_defs = find_bump_fields.iter().map(|f| quote! { pub #f: u8 });
        (
            quote! {
                #[derive(Default)]
                #[automatically_derived]
                pub struct #bumps_struct_name {
                    #(#bump_field_defs),*
                }
            },
            quote! {
                #[automatically_derived]
                impl Bumps for #struct_name<'_> {
                    type Bumps = #bumps_struct_name;
                }
            },
            quote! { #bumps_struct_name },
        )
    };

    // -----------------------------------------------------------------------
    // 8. Emit
    // -----------------------------------------------------------------------
    Ok(quote! {
        #(#extra_items)*

        #bumps_type

        #bumps_impl

        #[automatically_derived]
        impl #impl_generics ParseAccounts<'view, #bumps_param> for #struct_name #ty_generics
        #where_clause
        {
            const NUM_ACCOUNTS: usize = #num_accounts;

            fn parse_accounts(
                views: &mut AccountIter<'view>,
                data: &[u8],
                bumps: &mut #bumps_param,
            ) -> Result<Self> {
                views.has_remaining(Self::NUM_ACCOUNTS)?;

                #(#view_bindings)*

                #(#construction_stmts)*

                #(#cast_bindings)*

                #(#constraint_stmts)*

                Ok(Self { #(#field_names),* })
            }
        }
    })
}
