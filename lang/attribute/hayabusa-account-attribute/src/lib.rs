// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, ItemStruct, LitStr, Result};

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

        #[automatically_derived]
        impl #impl_generics Owner for #ident #ty_generics #where_clause {
            const OWNER: &'static Address = crate::ID;
        }

        #[automatically_derived]
        unsafe impl #impl_generics ::hayabusa::prelude::__AccountDiscriminatorMode for #ident #ty_generics #where_clause {
            type Mode = __WithDiscriminator;
        }

        #[automatically_derived]
        unsafe impl #impl_generics __AccountMarker for #ident #ty_generics #where_clause {}

        #[automatically_derived]
        unsafe impl #impl_generics ::hayabusa::prelude::Ownership for #ident #ty_generics #where_clause {
            type OwnerType = ::hayabusa::prelude::__Owner;

            #[inline(always)]
            fn check_ownership(view: AccountView<'_>) -> Result<()> {
                Self::check_owner(view)
            }
        }

        #[automatically_derived]
        impl<'view> #impl_generics ::hayabusa::prelude::AccountInit<'view> for #ident #ty_generics #where_clause {
            type Meta<'a> = ::hayabusa::prelude::__AccountInitMeta<'view, 'a> where 'view: 'a;
            type PdaMeta<'a, 'b, 'c> = ::hayabusa::prelude::__AccountPdaInitMeta<'view, 'a, 'b, 'c> where 'view: 'c, 'c: 'b, 'b: 'a;

            #[inline(never)]
            fn init<'a>(view: AccountView<'view>, meta: &Self::Meta<'a>) -> Result<()>
            where
                'view: 'a,
            {
                let _: ::hayabusa::prelude::Signer<'view> = <::hayabusa::prelude::Signer as ::hayabusa::prelude::ParseAccount<'view>>::parse(view, &mut ::hayabusa::prelude::NoMeta)?;

                ::hayabusa::prelude::create_or_allocate_account(view, meta.payer.to_account_view(), crate::ID, Self::SPACE)?;

                let mut borrow = view.try_borrow_mut()?;
                let disc_bytes = &mut borrow[..Self::DISCRIMINATOR.len()];
                disc_bytes.copy_from_slice(Self::DISCRIMINATOR);

                Ok(())
            }

            #[inline(never)]
            fn init_pda<'a, 'b, 'c>(view: AccountView<'view>, meta: &mut Self::PdaMeta<'a, 'b, 'c>) -> Result<()>
            where
                'view: 'c,
                'c: 'b,
                'b: 'a,
            {
                match &mut meta.signer {
                    ::hayabusa::prelude::SignerBumpness::With(signer) => {
                        ::hayabusa::prelude::create_or_allocate_pda(meta.payer.to_account_view(), view, crate::ID, &[*signer], Self::SPACE)?;
                    }
                    ::hayabusa::prelude::SignerBumpness::Without(signer, bump_slot) => {
                        let (_, bump) = ::hayabusa::prelude::try_find_program_address(signer.as_slice_of_slices(), crate::ID)?;

                        let slot: &'c mut u8 = bump_slot.take().ok_or(ErrorCode::MetaAlreadyConsumed)?;
                        *slot = bump;
                        let bump_ref: &'c u8 = slot; // move the &mut, downgrade to shared for all of 'c
                        signer.push(Seed::from(core::slice::from_ref(bump_ref)))?; // 'c: 'b, so it coerces

                        ::hayabusa::prelude::create_or_allocate_pda(meta.payer.to_account_view(), view, crate::ID, &[signer.as_signer()], Self::SPACE)?;
                    }
                }

                let mut borrow = view.try_borrow_mut()?;
                let disc_bytes = &mut borrow[..Self::DISCRIMINATOR.len()];
                disc_bytes.copy_from_slice(Self::DISCRIMINATOR);

                Ok(())
            }
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
