// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use quote::quote;
use std::collections::BTreeMap;
use syn::{
    Error, Fields, FieldsNamed, FieldsUnnamed, GenericArgument, Item, PathArguments, Result, Type,
    TypePath,
};

pub fn idl_collect_rust_types(item: &Item) -> Result<BTreeMap<String, Type>> {
    let mut types = BTreeMap::new();

    match item {
        Item::Struct(s) => collect_from_fields(&s.fields, &mut types)?,
        Item::Enum(e) => {
            for variant in &e.variants {
                collect_from_fields(&variant.fields, &mut types)?;
            }
        }
        Item::Type(t) => {
            collect_from_type(&t.ty, &mut types)?;
        }
        _ => {}
    }

    Ok(types)
}

fn collect_from_fields(fields: &Fields, out: &mut BTreeMap<String, Type>) -> Result<()> {
    match fields {
        Fields::Named(FieldsNamed { named, .. }) => {
            for f in named {
                collect_from_type(&f.ty, out)?;
            }
        }
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
            for f in unnamed {
                collect_from_type(&f.ty, out)?;
            }
        }
        Fields::Unit => {}
    }

    Ok(())
}

fn collect_from_type(ty: &Type, out: &mut BTreeMap<String, Type>) -> Result<()> {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            for seg in &path.segments {
                if let PathArguments::AngleBracketed(ab) = &seg.arguments {
                    for arg in &ab.args {
                        if let GenericArgument::Type(inner) = arg {
                            collect_from_type(inner, out)?;
                        }
                    }
                }
            }
            let key = quote!(#ty).to_string();
            out.entry(key).or_insert_with(|| ty.clone());
        }
        Type::Reference(r) => {
            return Err(Error::new_spanned(
                r,
                "references are not IDL compatible; use an owned type instead",
            ));
        }
        Type::Ptr(p) => {
            return Err(Error::new_spanned(
                p,
                "raw pointers are not IDL compatible; use an owned type instead",
            ));
        }
        Type::Array(a) => collect_from_type(&a.elem, out)?,
        Type::Slice(s) => {
            return Err(Error::new_spanned(
                s,
                "slices are not IDL compatible; use an owned type instead",
            ));
        }
        Type::Tuple(t) => {
            for elem in &t.elems {
                collect_from_type(elem, out)?;
            }
        }
        _ => {}
    }
    Ok(())
}
