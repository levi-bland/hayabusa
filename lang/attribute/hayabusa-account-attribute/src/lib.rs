// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, Fields, Ident, ItemStruct, LitStr, Result};

#[proc_macro_attribute]
pub fn account(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !proc_macro2::TokenStream::from(attr.clone()).is_empty() {
        return syn::Error::new_spanned(
            proc_macro2::TokenStream::from(attr),
            "#[account] does not take arguments",
        )
        .to_compile_error()
        .into();
    }

    let input = parse_macro_input!(item as ItemStruct);

    match expand_account(input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn zero_copy(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !proc_macro2::TokenStream::from(attr.clone()).is_empty() {
        return syn::Error::new_spanned(
            proc_macro2::TokenStream::from(attr),
            "#[zero_copy] does not take arguments",
        )
        .to_compile_error()
        .into();
    }

    let input = parse_macro_input!(item as ItemStruct);

    match expand_zero_copy(input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn seed(attr: TokenStream, item: TokenStream) -> TokenStream {
    let ts = proc_macro2::TokenStream::from(item.clone());
    let seed_str = parse_macro_input!(attr as LitStr);
    let ItemStruct {
        ident, generics, ..
    } = parse_macro_input!(item as ItemStruct);

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        #ts

        impl #impl_generics #ident #ty_generics #where_clause {
            pub const SEED: &[u8] = #seed_str.as_bytes();
        }
    }
    .into()
}

fn expand_account(input: ItemStruct) -> Result<proc_macro2::TokenStream> {
    let ItemStruct {
        attrs,
        vis,
        ident,
        generics,
        fields,
        semi_token,
        ..
    } = input;

    if semi_token.is_some() {
        return Err(syn::Error::new_spanned(
            ident,
            "#[account] does not support tuple/unit structs",
        ));
    }

    let preserved_struct_attrs = strip_account_attr(&attrs);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let bump_offset_impl = derive_bump_offset(ident.clone(), fields.clone());

    Ok(quote! {
        #(#preserved_struct_attrs)*
        #[derive(
            ::hayabusa::prelude::Pod,
            ::hayabusa::prelude::Zeroable,
            Discriminator,
            Copy,
            Clone,
        )]
        #[repr(C)]
        #[bytemuck(crate = "::hayabusa::prelude::bytemuck")]
        #vis struct #ident #ty_generics #fields #where_clause

        #bump_offset_impl

        #[automatically_derived]
        impl #impl_generics Owner for #ident #ty_generics #where_clause {
            const OWNER: Address = crate::ID;
        }

        #[automatically_derived]
        impl #impl_generics ::hayabusa::traits::internal::__AccountDiscriminatorMode for #ident #ty_generics #where_clause {
            type Mode = ::hayabusa::traits::internal::__WithDiscriminator;
        }

        #[automatically_derived]
        unsafe impl #impl_generics __AccountMarker for #ident #ty_generics #where_clause {}
    })
}

fn derive_bump_offset(name: Ident, fields: Fields) -> Option<proc_macro2::TokenStream> {
    let _bump_field = fields
        .iter()
        .find(|f| f.ident.as_ref().is_some_and(|id| id == "bump"))?;

    Some(quote! {
        #[automatically_derived]
        impl BumpOffset for #name {
            const BUMP_OFFSET: usize = ::core::mem::offset_of!(#name, bump);
        }
    })
}

fn strip_account_attr(attrs: &[Attribute]) -> Vec<Attribute> {
    attrs
        .iter()
        .filter(|attr| !attr.path().is_ident("zero_copy"))
        .cloned()
        .collect()
}

fn expand_zero_copy(input: ItemStruct) -> Result<proc_macro2::TokenStream> {
    let ItemStruct {
        attrs,
        vis,
        ident,
        generics,
        fields,
        semi_token,
        ..
    } = input;

    if semi_token.is_some() {
        return Err(syn::Error::new_spanned(
            ident,
            "#[zero_copy] does not support tuple/unit structs",
        ));
    }

    let preserved_struct_attrs = strip_account_attr(&attrs);
    let (_impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        #(#preserved_struct_attrs)*
        #[derive(
            ::hayabusa::prelude::Pod,
            ::hayabusa::prelude::Zeroable,
            Copy,
            Clone,
        )]
        #[repr(C)]
        #[bytemuck(crate = "::hayabusa::prelude::bytemuck")]
        #vis struct #ident #ty_generics #fields #where_clause
    })
}
