// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    Data, DeriveInput, Expr, ExprStruct, Field, Fields, Ident, Member, Token, Type,
};

/// Derives the hidden constructor used by `cpi!` / `invoke!` / `invoke_signed!`.
///
/// Fields of type `PhantomData<_>` (or fields marked `#[cpi(skip)]`) are filled
/// with `Default::default()` and must not appear at the call site.
#[proc_macro_derive(CpiInstruction, attributes(cpi))]
pub fn derive_cpi_instruction(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    expand_derive(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn is_phantom(ty: &Type) -> bool {
    match ty {
        Type::Path(tp) if tp.qself.is_none() => tp
            .path
            .segments
            .last()
            .is_some_and(|s| s.ident == "PhantomData"),
        _ => false,
    }
}

fn is_skipped(f: &Field) -> syn::Result<bool> {
    if is_phantom(&f.ty) {
        return Ok(true);
    }
    let mut skip = false;
    for attr in &f.attrs {
        if attr.path().is_ident("cpi") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("skip") {
                    skip = true;
                    Ok(())
                } else {
                    Err(meta.error("unknown `cpi` option, expected `skip`"))
                }
            })?;
        }
    }
    Ok(skip)
}

fn marker_ident(field: &Ident) -> Ident {
    Ident::new(&format!("__CPI_FIELD_{field}"), field.span())
}

fn expand_derive(input: DeriveInput) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(n) => &n.named,
            _ => {
                return Err(syn::Error::new(
                    input.ident.span(),
                    "CpiInstruction requires a struct with named fields",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new(
                input.ident.span(),
                "CpiInstruction can only be derived for structs",
            ));
        }
    };

    let mut kept: Vec<(Ident, Type)> = Vec::new();
    let mut skipped: Vec<Ident> = Vec::new();

    for f in fields {
        let ident = f.ident.clone().expect("named field");
        if is_skipped(f)? {
            skipped.push(ident);
        } else {
            kept.push((ident, f.ty.clone()));
        }
    }

    // Canonical order shared with the call-site macros.
    kept.sort_by(|a, b| a.0.to_string().cmp(&b.0.to_string()));

    let arg_names: Vec<&Ident> = kept.iter().map(|(i, _)| i).collect();
    let arg_types: Vec<&Type> = kept.iter().map(|(_, t)| t).collect();
    let markers: Vec<Ident> = arg_names.iter().map(|i| marker_ident(i)).collect();

    Ok(quote! {
        #[automatically_derived]
        #[doc(hidden)]
        #[allow(non_upper_case_globals, clippy::too_many_arguments)]
        impl #impl_generics #name #ty_generics #where_clause {
            #(
                #[doc(hidden)]
                pub const #markers: () = ();
            )*

            #[doc(hidden)]
            #[inline(always)]
            pub fn __cpi_new(#(#arg_names: #arg_types),*) -> Self {
                Self {
                    #(#arg_names,)*
                    #(#skipped: ::core::default::Default::default(),)*
                }
            }
        }
    })
}

struct Call {
    ix: ExprStruct,
    extra: Punctuated<Expr, Token![,]>,
}

impl Parse for Call {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let expr: Expr = input.parse()?;
        let ix = match expr {
            Expr::Struct(s) => s,
            other => {
                return Err(syn::Error::new(
                    other.span(),
                    "expected `Instruction { field: value, .. }`",
                ));
            }
        };
        let extra = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            Punctuated::parse_terminated(input)?
        } else {
            Punctuated::new()
        };
        Ok(Self { ix, extra })
    }
}

/// Expands to a block that evaluates the field expressions in written order,
/// then calls the derived constructor with them in canonical order.
fn build(ix: &ExprStruct) -> syn::Result<TokenStream2> {
    if let Some(rest) = &ix.rest {
        return Err(syn::Error::new(
            rest.span(),
            "`..rest` is not supported in CPI instruction literals",
        ));
    }

    let path = &ix.path;
    let mut lets = Vec::new();
    let mut entries: Vec<(String, Ident)> = Vec::new();

    for fv in &ix.fields {
        let ident = match &fv.member {
            Member::Named(i) => i.clone(),
            Member::Unnamed(i) => {
                return Err(syn::Error::new(i.span(), "tuple fields are not supported"));
            }
        };
        let key = ident.to_string();
        if entries.iter().any(|(k, _)| *k == key) {
            return Err(syn::Error::new(
                ident.span(),
                format!("field `{key}` specified more than once"),
            ));
        }

        let binding = Ident::new(&format!("__cpi_field_{ident}"), ident.span());
        let marker = marker_ident(&ident);
        let value = &fv.expr;

        lets.push(quote! {
            // Points at `#ident` if it is not a settable field of `#path`.
            let _: () = #path::#marker;
            let #binding = #value;
        });
        entries.push((key, binding));
    }

    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let args: Vec<&Ident> = entries.iter().map(|(_, b)| b).collect();

    Ok(quote! {{
        #(#lets)*
        #path::__cpi_new(#(#args),*)
    }})
}

/// Builds the instruction struct only.
#[proc_macro]
pub fn cpi(item: TokenStream) -> TokenStream {
    let call = parse_macro_input!(item as Call);
    match build(&call.ix) {
        Ok(ts) => ts.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

/// Builds, serializes and invokes. Propagates errors with `?`.
#[proc_macro]
pub fn invoke(item: TokenStream) -> TokenStream {
    let call = parse_macro_input!(item as Call);
    let ix = match build(&call.ix) {
        Ok(ts) => ts,
        Err(e) => return e.into_compile_error().into(),
    };
    quote! {{
        let __cpi_ix = #ix;
        __cpi_ix.serialize()?.invoke()?
    }}
    .into()
}

/// Builds, serializes and invokes with signer seeds.
#[proc_macro]
pub fn invoke_signed(item: TokenStream) -> TokenStream {
    let call = parse_macro_input!(item as Call);
    let ix = match build(&call.ix) {
        Ok(ts) => ts,
        Err(e) => return e.into_compile_error().into(),
    };
    let extra = call.extra.iter();
    quote! {{
        let __cpi_ix = #ix;
        __cpi_ix.serialize()?.invoke_signed(#(#extra),*)?
    }}
    .into()
}

/// Like `invoke!`, but yields a `Result` instead of using `?`.
#[proc_macro]
pub fn try_invoke(item: TokenStream) -> TokenStream {
    let call = parse_macro_input!(item as Call);
    let ix = match build(&call.ix) {
        Ok(ts) => ts,
        Err(e) => return e.into_compile_error().into(),
    };
    quote! {{
        let __cpi_ix = #ix;
        __cpi_ix.serialize().and_then(|__ctx| __ctx.invoke())
    }}
    .into()
}

#[proc_macro]
pub fn try_invoke_signed(item: TokenStream) -> TokenStream {
    let call = parse_macro_input!(item as Call);
    let ix = match build(&call.ix) {
        Ok(ts) => ts,
        Err(e) => return e.into_compile_error().into(),
    };
    let extra = call.extra.iter();
    quote! {{
        let __cpi_ix = #ix;
        __cpi_ix.serialize().and_then(|__ctx| __ctx.invoke_signed(#(#extra),*))
    }}
    .into()
}

/// Builds, serializes and invokes with signer seeds.
#[proc_macro]
pub fn invoke_with_signers(item: TokenStream) -> TokenStream {
    let call = parse_macro_input!(item as Call);
    let ix = match build(&call.ix) {
        Ok(ts) => ts,
        Err(e) => return e.into_compile_error().into(),
    };
    let extra = call.extra.iter();
    quote! {{
        let __cpi_ix = #ix;
        __cpi_ix.serialize()?.invoke_with_signers(#(#extra),*)?
    }}
    .into()
}

#[proc_macro]
pub fn invoke_with_signers_with_return_data(item: TokenStream) -> TokenStream {
    let call = parse_macro_input!(item as Call);
    let ix = match build(&call.ix) {
        Ok(ts) => ts,
        Err(e) => return e.into_compile_error().into(),
    };
    let extra = call.extra.iter();
    quote! {{
        let __cpi_ix = #ix;
        __cpi_ix.serialize()?.invoke_with_signers_with_return_data(#(#extra),*)?
    }}
    .into()
}
