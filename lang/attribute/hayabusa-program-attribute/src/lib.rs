// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use heck::ToUpperCamelCase;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse_macro_input, FnArg, GenericArgument, Ident, Item, ItemFn, ItemMod, Pat, PathArguments,
    Result, Type,
};

#[proc_macro_attribute]
pub fn program(_attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_program(parse_macro_input!(item as ItemMod))
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

fn expand_program(module: ItemMod) -> Result<proc_macro2::TokenStream> {
    let mod_ident = &module.ident;
    let (_, items) = module.clone().content.ok_or(syn::Error::new_spanned(
        module.clone(),
        "#[program] requires an inline module",
    ))?;

    let mut ix_structs = vec![];
    let mut dispatch_arms = vec![];
    let mut preserved_items = vec![];

    for item in items {
        if let Item::Fn(func) = &item {
            extract_instruction(func, &mut ix_structs, &mut dispatch_arms)?;
        }

        preserved_items.push(item);
    }

    Ok(quote! {
        pub mod instruction {
            use super::*;
            #(#ix_structs)*
        }

        #[allow(unexpected_cfgs)]
        #[cfg(not(any(feature = "no-entrypoint", feature = "idl-build")))]
        mod #mod_ident {
            use super::*;
            use super::instruction::*;

            extern crate alloc;
            ::hayabusa::default_allocator!();
            ::hayabusa::nostd_panic_handler!();
            ::hayabusa::program_entrypoint!(dispatcher);

            fn dispatcher(
                program_id: &Address,
                views: &[AccountView],
                ix_data: &[u8],
            ) -> Result<()> {
                if hint::unlikely(!address_eq(program_id, &crate::ID)) {
                    return Err(ProgramError::IncorrectProgramId);
                }

                const DISC_LEN: usize = 8;

                if hint::unlikely(ix_data.len() < DISC_LEN) {
                    return Err(ProgramError::InvalidInstructionData);
                }

                let (disc, rest) = {
                    let ptr = ix_data.as_ptr();
                    (&ix_data[..DISC_LEN], &ix_data[DISC_LEN..])
                };

                match disc {
                    #(#dispatch_arms)*
                    _ => Err(ErrorCode::UnknownInstruction.into()),
                }
            }

            #(#preserved_items)*
        }
    })
}

fn extract_instruction(
    func: &ItemFn,
    ix_structs: &mut Vec<proc_macro2::TokenStream>,
    dispatch_arms: &mut Vec<proc_macro2::TokenStream>,
) -> Result<()> {
    let fn_name = &func.sig.ident;
    let mut fn_name_str = fn_name.to_string().to_upper_camel_case();
    fn_name_str.push_str("Ix");

    let struct_ident = Ident::new(&fn_name_str, Span::mixed_site());

    let mut sig_inputs_iter = func.sig.inputs.iter().cloned();

    let ctx_type = if let Some(ctx_arg) = sig_inputs_iter.next() {
        match ctx_arg {
            syn::FnArg::Typed(pat) => {
                let ty = pat.ty.as_ref();
                extract_ctx_inner(ty)?
            }
            syn::FnArg::Receiver(_) => {
                return Err(syn::Error::new_spanned(
                    ctx_arg,
                    "expected Ctx<T> type as first argument",
                ));
            }
        }
    } else {
        return Err(syn::Error::new_spanned(
            func,
            "expected Ctx<T> type as first argument",
        ));
    };

    let mut fields = vec![];
    let mut arg_idents = vec![];

    for input in sig_inputs_iter {
        let FnArg::Typed(pat) = input.clone() else {
            return Err(syn::Error::new_spanned(
                input.clone(),
                "expected typed argument",
            ));
        };
        let Pat::Ident(pat_ident) = *pat.pat.clone() else {
            return Err(syn::Error::new_spanned(
                input.clone(),
                "expected identifier pattern",
            ));
        };

        let ident = &pat_ident.ident;
        let ty = (*pat.ty).clone();

        fields.push(quote! { pub #ident: #ty });
        arg_idents.push(ident.clone())
    }

    ix_structs.push(quote! {
        #[derive(
            Discriminator,
            ::hayabusa::prelude::BorshDeserialize,
            ::hayabusa::prelude::BorshSerialize,
        )]
        #[borsh(crate = "::hayabusa::prelude::borsh")]
        #[discriminator(namespace = "instruction")]
        pub struct #struct_ident {
            #(#fields),*
        }
    });

    dispatch_arms.push(quote! {
        <#struct_ident as Discriminator>::DISCRIMINATOR => {
            let mut accounts_iter = AccountIter::new(views);

            let ix_data = <#struct_ident as ::hayabusa::prelude::borsh::BorshDeserialize>::try_from_slice(rest)
                .map_err(|_| ProgramError::InvalidInstructionData)?;

            let mut bumps = <<#ctx_type as Bumps>::Bumps as Default>::default();

            let mut accounts = <#ctx_type as ParseAccounts<'_, <#ctx_type as Bumps>::Bumps>>::parse_accounts(&mut accounts_iter, rest, &mut bumps)?;

            let ctx = Ctx::new(program_id, &mut accounts, accounts_iter.remaining(), bumps);

            #fn_name(ctx, #(ix_data.#arg_idents),*)
                .map_err(Into::into)
        }
    });

    Ok(())
}

fn extract_ctx_inner(ty: &Type) -> Result<Type> {
    let Type::Path(type_path) = ty else {
        return Err(syn::Error::new_spanned(ty, "expected path type"));
    };

    let segment =
        type_path.path.segments.last().ok_or_else(|| {
            syn::Error::new_spanned(ty, "expected at least one segment in the path")
        })?;

    if segment.ident != "Ctx" {
        return Err(syn::Error::new_spanned(
            ty,
            "expected Ctx<T> type as first argument",
        ));
    }

    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return Err(syn::Error::new_spanned(
            ty,
            "expected Ctx<T> type as first argument",
        ));
    };

    let GenericArgument::Type(inner) = args
        .args
        .first()
        .ok_or_else(|| syn::Error::new_spanned(ty, "expected Ctx<T> type as first argument"))?
    else {
        return Err(syn::Error::new_spanned(
            ty,
            "expected Ctx<T> type as first argument",
        ));
    };

    Ok(inner.clone())
}
