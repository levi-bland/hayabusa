// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use hayabusa_syn::{
    docs::parse_docs,
    idl::{parse_idl_repr, parse_idl_type_def_generics, parse_idl_type_def_ty},
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Error, Ident, Item, ItemStruct, Result};

pub fn generate_struct_type_idl_emitter(input: &ItemStruct) -> Result<TokenStream> {
    let ItemStruct {
        attrs,
        ident,
        generics,
        semi_token,
        ..
    } = input;

    if semi_token.is_some() {
        return Err(Error::new_spanned(
            ident,
            "#[account] does not support tuple/unit structs",
        ));
    }

    let ident_string = ident.to_string();

    let idl_type_function_ident = Ident::new(
        &format!("__idl_type_{}", ident_string.to_lowercase()),
        Span::call_site(),
    );

    let idl_type_docs = parse_docs(attrs);
    let idl_type_repr = parse_idl_repr(attrs);
    let idl_type_generics = parse_idl_type_def_generics(&generics.params);
    let idl_type_def_ty = parse_idl_type_def_ty(&Item::from(input.clone()))?;

    let idl_type_printer = quote! {
        #[cfg(feature = "idl-build")]
        #[test]
        fn #idl_type_function_ident() {
            use ::hayabusa::idl::*;

            let idl_type = IdlTypeDef {
                name: #ident_string.to_string(),
                docs: #idl_type_docs,
                serialization: IdlSerialization::Bytemuck, // bytemuck is the only "serialization" format we support
                repr: Some(#idl_type_repr),
                generics: #idl_type_generics,
                ty: #idl_type_def_ty,
            };

            idl_type.print("__IDL_TYPE: ");
        }
    };

    Ok(idl_type_printer)
}
