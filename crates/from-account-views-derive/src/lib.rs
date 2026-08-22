// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, parse_quote, punctuated::Punctuated, spanned::Spanned, Data, DeriveInput,
    Fields, Ident, Path, PathArguments, PathSegment, Result, Token, Type,
};

#[proc_macro_derive(FromAccountViews, attributes(meta))]
pub fn derive_from_account_views(input: TokenStream) -> TokenStream {
    let derive_token_stream = input.clone();
    let derive_input = parse_macro_input!(derive_token_stream as DeriveInput);

    let struct_name = &derive_input.ident;

    // Require exactly one lifetime parameter.
    let info_lt = match derive_input
        .generics
        .lifetimes()
        .collect::<Vec<_>>()
        .as_slice()
    {
        [lt] => &lt.lifetime,
        [] => {
            return syn::Error::new(
                derive_input.span(),
                "FromAccountViews requires exactly one lifetime parameter",
            )
            .to_compile_error()
            .into();
        }
        _ => {
            return syn::Error::new(
                derive_input.span(),
                "FromAccountViews supports exactly one lifetime parameter",
            )
            .to_compile_error()
            .into();
        }
    };

    let fields = match &derive_input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(n) => &n.named,
            _ => {
                return syn::Error::new(
                    s.fields.span(),
                    "FromAccountViews supports named fields only",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new(
                derive_input.span(),
                "FromAccountViews can only be derived for structs",
            )
            .to_compile_error()
            .into();
        }
    };

    let (impl_generics, ty_generics, where_clause) = derive_input.generics.split_for_impl();

    let mut bindings = Vec::new();
    let mut field_idents = Vec::new();

    for field in fields {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;

        field_idents.push(ident);

        let meta_expr = match parse_meta(&field.attrs, ty) {
            Ok(m) => m,
            Err(e) => return e.to_compile_error().into(),
        };

        // NOTE: `data` is in scope in the generated method, so meta values can reference it.
        bindings.push(quote! {
            let #ident =
                <#ty as FromAccountView<#info_lt>>::try_from_account_view(
                    account_views.next()?,
                    #meta_expr,
                )?;
        });
    }

    let ix_data_type_path = make_instruction_type(struct_name);

    let expanded = quote! {
        impl #impl_generics FromAccountViews<#info_lt>
            for #struct_name #ty_generics #where_clause
        {
            type IxData = #ix_data_type_path;

            #[inline(always)]
            fn try_from_account_views(
                account_views: &mut AccountCursor<#info_lt>,
                data: &Self::IxData,
            ) -> Result<Self> {
                // keep `data` available for #[meta(...)] expressions
                let _ = data;

                #(#bindings)*

                Ok(Self {
                    #(#field_idents,)*
                })
            }
        }
    };

    expanded.into()
}

fn parse_meta(attrs: &[syn::Attribute], ty: &Type) -> Result<proc_macro2::TokenStream> {
    for attr in attrs {
        if attr.path().is_ident("meta") {
            // Parse: writable = true, owner = foo, ...
            let args = attr
                .parse_args_with(Punctuated::<syn::MetaNameValue, Token![,]>::parse_terminated)?;

            // Build initializer tokens: writable: true, owner: foo, ...
            let inits = args
                .iter()
                .map(|kv| {
                    let path = &kv.path;
                    let value = &kv.value;

                    if path.get_ident().is_none() {
                        return Err(syn::Error::new(
                            path.span(),
                            "meta keys must be simple identifiers (e.g. writable = true)",
                        ));
                    }

                    Ok(quote! { #path: #value })
                })
                .collect::<Result<Vec<_>>>()?;

            // Compute nominal meta type path: foo::Signer<'ix> -> foo::SignerMeta
            let meta_path = meta_type_path_from_account_type(ty)?;

            return Ok(quote! {
                #meta_path { #(#inits,)* }
            });
        }
    }

    Ok(quote! { NoMeta })
}

fn meta_type_path_from_account_type(ty: &Type) -> Result<proc_macro2::TokenStream> {
    let type_path = match ty {
        Type::Path(p) if p.qself.is_none() => &p.path,
        _ => {
            return Err(syn::Error::new(
                ty.span(),
                "account field type must be a path type (e.g. Signer<'ix> or foo::Signer<'ix>), so the macro can derive SignerMeta",
            ));
        }
    };

    let mut path = type_path.clone();
    let last = path
        .segments
        .last_mut()
        .ok_or_else(|| syn::Error::new(ty.span(), "empty type path"))?;

    // Rename last segment: Signer -> SignerMeta
    let base = last.ident.to_string();
    last.ident = Ident::new(&format!("{base}Meta"), last.ident.span());

    // Drop generic args for struct literal construction.
    last.arguments = syn::PathArguments::None;

    Ok(quote! { #path })
}

fn make_instruction_type(input_ty: &Ident) -> Path {
    #[allow(clippy::redundant_static_lifetimes)]
    const SUFFIX: &'static str = "Ix";
    let generated = format_ident!("{}{}", input_ty, SUFFIX, span = Span::call_site());

    let mut path: Path = parse_quote!(crate::instruction);
    path.segments.push(PathSegment {
        ident: generated,
        arguments: PathArguments::None,
    });

    path
}
