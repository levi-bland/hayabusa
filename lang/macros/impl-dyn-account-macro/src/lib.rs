// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse::Parse, parse_macro_input, Ident, Token};

#[proc_macro]
pub fn impl_dyn_account_dispatch(input: TokenStream) -> TokenStream {
    let Input {
        dispatch_type,
        trait_name,
        implementing_types,
    } = parse_macro_input!(input as Input);

    let enum_name = Ident::new(&format!("{}AccountType", trait_name), Span::call_site());

    quote! {
        pub struct #dispatch_type;

        #[automatically_derived]
        impl<'view> DynAccountDispatch<'view> for #dispatch_type {
            type Object = dyn #trait_name;
            type AccountType = #enum_name;

            #[inline(always)]
            fn build(
                view: ::hayabusa::accounts::dyn_account::ValidatedView<'view, #enum_name>,
            ) -> Result<Ref<'view, Self::Object>> {
                match view.marker() {
                    #(#enum_name::#implementing_types => {
                        Ok(Ref::map(unsafe { <#implementing_types as Cast>::cast_unchecked(view.to_account_view())? }, |o| o as &dyn #trait_name))
                    })*
                }
            }

            #[inline(always)]
            unsafe fn build_untracked(
                view: ::hayabusa::accounts::dyn_account::ValidatedView<'view, #enum_name>,
            ) -> &'view Self::Object {
                match view.marker() {
                    #(#enum_name::#implementing_types => {
                        <#implementing_types as Cast>::cast_untracked(view.to_account_view()) as &dyn #trait_name
                    })*
                }
            }

            #[inline(always)]
            fn build_mut(
                view: ::hayabusa::accounts::dyn_account::ValidatedView<'view, #enum_name>,
            ) -> Result<RefMut<'view, Self::Object>> {
                match view.marker() {
                    #(#enum_name::#implementing_types => {
                        Ok(RefMut::map(unsafe { <#implementing_types as Cast>::cast_mut_unchecked(view.to_account_view())? }, |o| o as &mut dyn #trait_name))
                    })*
                }
            }

            #[inline(always)]
            unsafe fn build_mut_untracked(
                view: ::hayabusa::accounts::dyn_account::ValidatedView<'view>,
            ) -> &'view mut Self::Object {
                match view.marker() {
                    #(#enum_name::#implementing_types => {
                        <#implementing_types as Cast>::cast_mut_untracked(view.to_account_view()) as &mut dyn #trait_name
                    })*
                }
            }

            #[inline(always)]
            fn validate_account(view: AccountView<'view>) -> Result<::hayabusa::accounts::dyn_account::ValidatedView<'view, #enum_name>> {
                if hint::unlikely(view.data_len() < 8) {
                    return Err(ErrorCode::InvalidAccountDiscriminator.into());
                }

                let discriminator = unsafe { &*(view.data_ptr() as *const [u8; 8]) };

                match discriminator {
                    #(<#implementing_types as Discriminator>::DISCRIMINATOR => {
                        <#implementing_types as Space>::check_space(view.data_len())?;
                        <#implementing_types as Owner>::check_owner(view.owner())?;

                        Ok(unsafe { ::hayabusa::accounts::dyn_account::ValidatedView::new(view, #enum_name::#implementing_types) })
                    })*
                    _ => Err(ErrorCode::InvalidAccountDiscriminator.into()),
                }
            }
        }

        #[derive(Clone, Copy)]
        #[repr(u8)]
        pub enum #enum_name {
            #(#implementing_types,)*
        }
    }
    .into()
}

struct Input {
    pub dispatch_type: Ident,
    pub trait_name: Ident,
    pub implementing_types: Vec<Ident>,
}

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let dispatch_type: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let trait_name: Ident = input.parse()?;

        let mut impls = Vec::new();
        while input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            impls.push(input.parse::<Ident>()?);
        }

        if impls.is_empty() {
            return Err(syn::Error::new(
                trait_name.span(),
                "impl_dyn_account_dispatch! requires at least one implementing type",
            ));
        }

        Ok(Input {
            dispatch_type,
            trait_name,
            implementing_types: impls,
        })
    }
}
