// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, GenericArgument, Ident, ItemEnum, PathArguments, Type, TypeGenerics,
};

#[proc_macro_derive(EnumAccountDispatch)]
pub fn derive_enum_account_dispatch(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as ItemEnum);
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let mut type_paths = vec![];
    let account_variants = input.variants.iter().map(|variant| {
        let ident = variant.ident.clone();
        let mut fields_iter = variant.fields.iter();
        let account_variant = fields_iter
            .next()
            .expect("EnumAccountDispatch variants must have exactly one field")
            .clone();

        match fields_iter.next() {
            Some(_) => panic!("EnumAccountDispatch variants must have exactly one field"),
            None => (),
        }

        match account_variant.ident {
            Some(_) => panic!("EnumAccountDispatch variants must be tuple variants"),
            None => (),
        }

        match account_variant.ty {
            Type::Path(path) => {
                let last = path
                    .path
                    .segments
                    .last()
                    .expect("EnumAccountDispatch variants must have a last segment");

                if last.ident.to_string() != "Account" {
                    panic!("EnumAccountDispatch variants must have a parameter of type 'Account'");
                }

                match last.arguments.clone() {
                    PathArguments::AngleBracketed(args) => {
                        let mut args_iter = args.args.iter();
                        match args_iter.next() {
                            Some(arg) => {
                                match arg {
                                    GenericArgument::Lifetime(_) => (),
                                    _ => panic!("EnumAccountDispatch variants must have a parameter of type 'Account<'view, T>'"),
                                }
                            }
                            None => panic!("EnumAccountDispatch variants must have a parameter of type 'Account<'view, T>'"),
                        }
                        match args_iter.next() {
                            Some(arg) => {
                                match arg {
                                    GenericArgument::Type(ty) => {
                                        match ty {
                                            Type::Path(path) => {
                                                type_paths.push(path.path.segments.clone());
                                            }
                                            _ => panic!("EnumAccountDispatch variants must have a parameter of type 'Account<'view, T>'"),
                                        }
                                    }
                                    _ => panic!("EnumAccountDispatch variants must have a parameter of type 'Account<'view, T>'"),
                                }
                            }
                            None => panic!("EnumAccountDispatch variants must have a parameter of type 'Account<'view, T>'"),
                        }
                    }
                    _ => panic!("EnumAccountDispatch variants must have angle bracketed generic arguments as such `Account<'view, T>`"),
                }
            }
            _ => panic!("EnumAccountDispatch variants must only contain TypePath's"),
        };
        ident
    })
    .collect::<Vec<_>>();

    unimplemented!();
}
