// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use hayabusa_syn::{
    docs::parse_docs,
    idl::{idl_collect_rust_types, parse_idl_repr, parse_idl_type_def_ty},
};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Item};

pub(crate) fn idl_build_derive_handler(input: TokenStream) -> TokenStream {
    let item_token_stream = input.clone();
    let item_input = parse_macro_input!(item_token_stream as Item);

    let derive_token_stream = input.clone();
    let derive_input = parse_macro_input!(derive_token_stream as DeriveInput);

    let ident = derive_input.ident.clone();
    let ident_string = derive_input.ident.to_string();
    let docs = parse_docs(&derive_input.attrs);
    let repr = parse_idl_repr(&derive_input.attrs);
    let idl_type_def_ty = match parse_idl_type_def_ty(&item_input) {
        Ok(t) => t,
        Err(e) => return e.into_compile_error().into(),
    };

    let insert_types_calls = match idl_collect_rust_types(&item_input) {
        Ok(m) => m
            .values()
            .map(|ty| {
                quote! {
                    <#ty as hayabusa::idl::IdlBuild>::insert_types(types);
                }
            })
            .collect::<Vec<_>>(),
        Err(e) => return e.into_compile_error().into(),
    };

    let (impl_generics, ty_generics, where_clause) = derive_input.generics.split_for_impl();

    let generated = quote! {
        #[cfg(feature = "idl-build")]
        impl #impl_generics ::hayabusa::idl::IdlBuild for #ident #ty_generics #where_clause {
            fn create_type() -> Option<hayabusa::idl::IdlTypeDef> {
                Some(::hayabusa::idl::IdlTypeDef {
                    name: #ident_string.to_string(),
                    docs: #docs,
                    serialization: ::hayabusa::idl::IdlSerialization::Bytemuck, // TODO: fix this
                    repr: #repr,
                    ty: #idl_type_def_ty,
                })
            }

            fn insert_types(types: &mut std::collections::BTreeMap<String, hayabusa::idl::IdlTypeDef>) {
                if let Some(ty) = Self::create_type() {
                    types.insert(Self::get_full_path(), ty);
                }

                #(#insert_types_calls)*
            }
        }
    };

    generated.into()
}
