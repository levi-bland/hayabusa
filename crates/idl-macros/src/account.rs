// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

use hayabusa_syn::{
    docs::parse_docs,
    idl::{idl_collect_rust_types, parse_idl_repr, parse_idl_type_def_ty},
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, Item, ItemStruct, Result};

pub fn generate_account_idl_build_impl(input: &ItemStruct) -> Result<TokenStream> {
    let ItemStruct {
        ident,
        semi_token,
        attrs,
        ..
    } = input.clone();

    if semi_token.is_some() {
        return Err(Error::new_spanned(
            ident,
            "Account structs must be named structs, not tuple/unit structs",
        ));
    }

    let ident_string = ident.to_string();
    let struct_docs = parse_docs(&attrs);
    let struct_repr = parse_idl_repr(&attrs);
    let idl_type_def_ty = parse_idl_type_def_ty(&Item::from(input.clone()))?;

    let insert_types_calls = idl_collect_rust_types(&Item::from(input.clone()))?
        .values()
        .map(|ty| {
            quote! {
                <#ty as hayabusa::idl::IdlBuild>::insert_types(types);
            }
        })
        .collect::<Vec<_>>();

    let idl_impl = quote! {
        #[cfg(feature = "idl-build")]
        impl hayabusa::idl::IdlBuild for #ident {
            fn create_type() -> Option<hayabusa::idl::IdlTypeDef> {
                use hayabusa::idl::*;
                Some(IdlTypeDef {
                    name: #ident_string.to_string(),
                    docs: #struct_docs,
                    serialization: IdlSerialization::Bytemuck,
                    repr: #struct_repr,
                    generics: vec![], // accounts should not have generics
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

    Ok(idl_impl)
}
