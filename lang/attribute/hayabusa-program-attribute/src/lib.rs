// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use heck::ToUpperCamelCase;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse_macro_input, visit_mut::VisitMut, Attribute, Expr, ExprLit, FnArg, GenericArgument,
    Ident, Item, ItemFn, ItemMod, Lifetime, Lit, LitInt, Meta, Pat, PathArguments, Result, Type,
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
    let mut seen_discriminators = Vec::new();

    for item in items {
        if let Item::Fn(func) = &item {
            let explicit = explicit_discriminator(func)?;
            if let Some(disc) = &explicit {
                let key = format!("{}{}", disc.lit.suffix(), disc.lit.base10_digits());
                if seen_discriminators.iter().any(|seen| seen == &key) {
                    return Err(syn::Error::new(
                        disc.lit.span(),
                        "duplicate instruction discriminator",
                    ));
                }
                seen_discriminators.push(key);
            }

            extract_instruction(func, explicit.as_ref(), &mut ix_structs, &mut dispatch_arms)?;

            let mut func = func.clone();
            func.attrs
                .retain(|attr| !attr.path().is_ident("discriminator"));
            preserved_items.push(Item::Fn(func));
            continue;
        }

        preserved_items.push(item);
    }

    let dispatch = dispatch_body(&dispatch_arms);

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

                #dispatch
            }

            #(#preserved_items)*
        }
    })
}

struct ExplicitDisc {
    lit: LitInt,
    len: usize,
}

struct DispatchArm {
    disc_len: usize,
    match_arm: proc_macro2::TokenStream,
    prefix_if: proc_macro2::TokenStream,
}

fn dispatch_body(dispatch_arms: &[DispatchArm]) -> proc_macro2::TokenStream {
    let variable_width = dispatch_arms.iter().any(|arm| arm.disc_len != 8);

    if !variable_width {
        let match_arms = dispatch_arms.iter().map(|arm| &arm.match_arm);
        return quote! {
            const DISC_LEN: usize = 8;

            if hint::unlikely(ix_data.len() < DISC_LEN) {
                return Err(ProgramError::InvalidInstructionData);
            }

            let (disc, rest) = {
                let ptr = ix_data.as_ptr();
                (&ix_data[..DISC_LEN], &ix_data[DISC_LEN..])
            };

            match disc {
                #(#match_arms)*
                _ => Err(ErrorCode::UnknownInstruction.into()),
            }
        };
    }

    let mut arms: Vec<&DispatchArm> = dispatch_arms.iter().collect();
    arms.sort_by(|a, b| b.disc_len.cmp(&a.disc_len));
    let min_len = arms.iter().map(|arm| arm.disc_len).min().unwrap_or(8);
    let prefix_ifs = arms.iter().map(|arm| &arm.prefix_if);

    quote! {
        if hint::unlikely(ix_data.len() < #min_len) {
            return Err(ProgramError::InvalidInstructionData);
        }

        #(#prefix_ifs)*

        Err(ErrorCode::UnknownInstruction.into())
    }
}

fn extract_instruction(
    func: &ItemFn,
    explicit: Option<&ExplicitDisc>,
    ix_structs: &mut Vec<proc_macro2::TokenStream>,
    dispatch_arms: &mut Vec<DispatchArm>,
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
                return Err(expected_ctx(&ctx_arg));
            }
        }
    } else {
        return Err(expected_ctx(func));
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

    let struct_def = if let Some(disc) = explicit {
        let lit = &disc.lit;
        let len = disc.len;
        quote! {
            #[derive(
                ::hayabusa::prelude::BorshDeserialize,
                ::hayabusa::prelude::BorshSerialize,
            )]
            #[borsh(crate = "::hayabusa::prelude::borsh")]
            pub struct #struct_ident {
                #(#fields),*
            }

            #[automatically_derived]
            impl ::hayabusa::prelude::Discriminator for #struct_ident {
                const DISCRIMINATOR: &[u8] = {
                    const BYTES: [u8; #len] = #lit.to_le_bytes();
                    &BYTES
                };
            }
        }
    } else {
        quote! {
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
        }
    };
    ix_structs.push(struct_def);

    let body = quote! {
        let mut accounts_iter = AccountIter::new(views);

        let ix_data = <#struct_ident as ::hayabusa::prelude::borsh::BorshDeserialize>::try_from_slice(rest)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        let mut bumps = <<#ctx_type as Bumps>::Bumps as Default>::default();

        let mut accounts = <#ctx_type as ParseAccounts<'_, <#ctx_type as Bumps>::Bumps>>::parse_accounts(&mut accounts_iter, rest, &mut bumps)?;

        let ctx = Ctx::new(program_id, &mut accounts, accounts_iter.remaining(), bumps);

        #fn_name(ctx, #(ix_data.#arg_idents),*)
            .map_err(Into::into)
    };

    dispatch_arms.push(DispatchArm {
        disc_len: explicit.map(|disc| disc.len).unwrap_or(8),
        match_arm: quote! {
            <#struct_ident as Discriminator>::DISCRIMINATOR => {
                #body
            }
        },
        prefix_if: quote! {
            if ix_data.starts_with(<#struct_ident as Discriminator>::DISCRIMINATOR) {
                let rest = &ix_data[<#struct_ident as Discriminator>::DISCRIMINATOR.len()..];
                return { #body };
            }
        },
    });

    Ok(())
}

fn explicit_discriminator(func: &ItemFn) -> Result<Option<ExplicitDisc>> {
    let mut found = None;

    for attr in &func.attrs {
        if !attr.path().is_ident("discriminator") {
            continue;
        }
        if found.is_some() {
            return Err(syn::Error::new_spanned(
                attr,
                "duplicate #[discriminator] attribute",
            ));
        }
        found = Some(parse_discriminator_attr(attr)?);
    }

    Ok(found)
}

fn parse_discriminator_attr(attr: &Attribute) -> Result<ExplicitDisc> {
    let Meta::NameValue(name_value) = &attr.meta else {
        return Err(syn::Error::new_spanned(
            attr,
            "expected #[discriminator = N], for example #[discriminator = 0u8]",
        ));
    };

    let Expr::Lit(ExprLit {
        lit: Lit::Int(lit), ..
    }) = &name_value.value
    else {
        return Err(syn::Error::new_spanned(
            &name_value.value,
            "expected an integer literal with a type suffix, such as `0u8` or `1u32`",
        ));
    };

    let len = match lit.suffix() {
        "u8" => {
            lit.base10_parse::<u8>()?;
            1
        }
        "u16" => {
            lit.base10_parse::<u16>()?;
            2
        }
        "u32" => {
            lit.base10_parse::<u32>()?;
            4
        }
        "u64" => {
            lit.base10_parse::<u64>()?;
            8
        }
        "" => {
            return Err(syn::Error::new_spanned(
                lit,
                "expected an integer literal with a type suffix, such as `0u8` or `1u32`",
            ));
        }
        other => {
            return Err(syn::Error::new_spanned(
                lit,
                format!("unsupported discriminator type `{other}`, expected u8, u16, u32, or u64"),
            ));
        }
    };

    Ok(ExplicitDisc {
        lit: lit.clone(),
        len,
    })
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
        return Err(expected_ctx(ty));
    }

    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return Err(expected_ctx(ty));
    };

    // `Ctx<T>` elides the context lifetime. `Ctx<'view, T<'view>>` names it.
    let mut ctx_lifetime = None;
    let mut inner = None;

    for arg in &args.args {
        match arg {
            GenericArgument::Lifetime(lifetime) if ctx_lifetime.is_none() && inner.is_none() => {
                ctx_lifetime = Some(lifetime);
            }
            GenericArgument::Type(inner_ty) if inner.is_none() => {
                inner = Some(inner_ty.clone());
            }
            _ => return Err(expected_ctx(ty)),
        }
    }

    let Some(mut inner) = inner else {
        return Err(expected_ctx(ty));
    };

    // The handler lifetime is not in scope in the dispatcher. `'_` is inferred
    // from the account views and unified with the handler signature.
    if let Some(lifetime) = ctx_lifetime {
        ElideLifetime { lifetime }.visit_type_mut(&mut inner);
    }

    Ok(inner)
}

fn expected_ctx(tokens: impl quote::ToTokens) -> syn::Error {
    syn::Error::new_spanned(
        tokens,
        "expected Ctx<T> or Ctx<'view, T<'view>> as the first argument",
    )
}

struct ElideLifetime<'a> {
    lifetime: &'a Lifetime,
}

impl VisitMut for ElideLifetime<'_> {
    fn visit_lifetime_mut(&mut self, lifetime: &mut Lifetime) {
        if lifetime.ident == self.lifetime.ident {
            lifetime.ident = Ident::new("_", lifetime.ident.span());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expand(module: ItemMod) -> String {
        expand_program(module).unwrap().to_string()
    }

    #[test]
    fn accepts_elided_ctx() {
        let expanded = expand(syn::parse_quote! {
            pub mod counter {
                pub fn initialize(ctx: Ctx<InitializeCounter>) -> Result<()> {
                    Ok(())
                }
            }
        });

        assert!(
            expanded.contains("InitializeCounter as ParseAccounts"),
            "{expanded}"
        );
        assert!(!expanded.contains("InitializeCounter < '_ >"), "{expanded}");
    }

    #[test]
    fn accepts_explicit_ctx_lifetime() {
        let expanded = expand(syn::parse_quote! {
            pub mod counter {
                pub fn increment<'view>(
                    ctx: Ctx<'view, IncrementCounter<'view>>,
                    amount: u64,
                ) -> Result<()> {
                    Ok(())
                }
            }
        });

        assert!(
            expanded.contains("IncrementCounter < '_ > as ParseAccounts"),
            "{expanded}"
        );
        assert!(
            expanded.contains("ctx : Ctx < 'view , IncrementCounter < 'view > >"),
            "{expanded}"
        );
    }

    #[test]
    fn custom_discriminator_overrides_hash() {
        let expanded = expand(syn::parse_quote! {
            pub mod counter {
                #[discriminator = 0u8]
                pub fn test_instruction(ctx: Ctx<TestAccounts>) -> Result<()> {
                    Ok(())
                }

                pub fn hashed(ctx: Ctx<HashedAccounts>) -> Result<()> {
                    Ok(())
                }
            }
        });

        assert!(
            expanded.contains("const BYTES : [u8 ; 1usize] = 0u8 . to_le_bytes ()"),
            "{expanded}"
        );
        assert!(expanded.contains("ix_data . starts_with"), "{expanded}");
        assert!(!expanded.contains("discriminator ="), "{expanded}");
        assert!(
            expanded.contains("namespace = \"instruction\""),
            "{expanded}"
        );
    }

    #[test]
    fn rejects_unsuffixed_discriminator() {
        let err = expand_program(syn::parse_quote! {
            pub mod counter {
                #[discriminator = 0]
                pub fn test_instruction(ctx: Ctx<TestAccounts>) -> Result<()> {
                    Ok(())
                }
            }
        })
        .unwrap_err()
        .to_string();

        assert!(err.contains("type suffix"), "{err}");
    }

    #[test]
    fn rejects_duplicate_discriminator() {
        let err = expand_program(syn::parse_quote! {
            pub mod counter {
                #[discriminator = 1u8]
                pub fn first(ctx: Ctx<First>) -> Result<()> {
                    Ok(())
                }

                #[discriminator = 1u8]
                pub fn second(ctx: Ctx<Second>) -> Result<()> {
                    Ok(())
                }
            }
        })
        .unwrap_err()
        .to_string();

        assert!(err.contains("duplicate instruction discriminator"), "{err}");
    }
}
