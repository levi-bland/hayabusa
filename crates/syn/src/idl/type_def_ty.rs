// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

use crate::docs::parse_docs;
use hayabusa_idl_types::{
    IdlArrayLen, IdlDefinedFields, IdlEnumVariant, IdlField, IdlGenericArg, IdlType, IdlTypeDefTy,
};
use syn::{Error, Fields, GenericArgument, Item, PathArguments, Result, Type, Variant};

pub fn parse_idl_type_def_ty(item: &Item) -> Result<IdlTypeDefTy> {
    match item {
        Item::Struct(s) => Ok(IdlTypeDefTy::Struct {
            fields: parse_idl_defined_fields(&s.fields)?,
        }),
        Item::Enum(e) => Ok(IdlTypeDefTy::Enum {
            variants: e
                .variants
                .iter()
                .map(parse_idl_enum_variant)
                .collect::<Result<_>>()?,
        }),
        Item::Type(t) => Ok(IdlTypeDefTy::Type {
            alias: parse_idl_type(&t.ty)?,
        }),
        _ => Err(Error::new_spanned(
            item,
            "expected struct, enum, or type alias",
        )),
    }
}

fn parse_idl_defined_fields(fields: &Fields) -> Result<Option<IdlDefinedFields>> {
    match fields {
        Fields::Unit => Ok(None),
        Fields::Named(named) => {
            let fields = named
                .named
                .iter()
                .map(|f| {
                    Ok(IdlField {
                        name: f.ident.as_ref().unwrap().to_string(),
                        ty: parse_idl_type(&f.ty)?,
                        docs: parse_docs(&f.attrs).0,
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(Some(IdlDefinedFields::Named(fields)))
        }
        Fields::Unnamed(unnamed) => {
            let types = unnamed
                .unnamed
                .iter()
                .map(|f| parse_idl_type(&f.ty))
                .collect::<Result<Vec<_>>>()?;
            Ok(Some(IdlDefinedFields::Tuple(types)))
        }
    }
}

fn parse_idl_enum_variant(variant: &Variant) -> Result<IdlEnumVariant> {
    Ok(IdlEnumVariant {
        name: variant.ident.to_string(),
        fields: parse_idl_defined_fields(&variant.fields)?,
    })
}

fn parse_idl_type(ty: &Type) -> Result<IdlType> {
    match ty {
        Type::Path(tp) => {
            let segment = tp
                .path
                .segments
                .last()
                .ok_or_else(|| Error::new_spanned(tp, "empty type path"))?;
            let name = segment.ident.to_string();

            match name.as_str() {
                "u8" => return Ok(IdlType::U8),
                "u16" => return Ok(IdlType::U16),
                "u32" => return Ok(IdlType::U32),
                "u64" => return Ok(IdlType::U64),
                "u128" => return Ok(IdlType::U128),
                "i8" => return Ok(IdlType::I8),
                "i16" => return Ok(IdlType::I16),
                "i32" => return Ok(IdlType::I32),
                "i64" => return Ok(IdlType::I64),
                "i128" => return Ok(IdlType::I128),
                "f32" => return Ok(IdlType::F32),
                "f64" => return Ok(IdlType::F64),
                "bool" => return Ok(IdlType::Bool),
                "Address" => return Ok(IdlType::Pubkey),
                "String" | "Vec" => {
                    return Err(Error::new_spanned(
                        tp,
                        format!("`{}` is not supported in this IDL spec", name),
                    ))
                }
                _ => {}
            }

            match name.as_str() {
                "Option" => {
                    let inner = extract_single_generic_type(&segment.arguments)?;
                    return Ok(IdlType::Option(Box::new(parse_idl_type(inner)?)));
                }
                _ => {}
            }

            let generics = match &segment.arguments {
                PathArguments::AngleBracketed(args) => args
                    .args
                    .iter()
                    .map(parse_idl_generic_arg)
                    .collect::<Result<Vec<_>>>()?,
                PathArguments::None => vec![],
                PathArguments::Parenthesized(_) => {
                    return Err(Error::new_spanned(tp, "fn pointer types are not supported"))
                }
            };

            Ok(IdlType::Defined { name, generics })
        }
        Type::Array(a) => {
            let ty = parse_idl_type(&a.elem)?;
            let len = parse_array_len(&a.len)?;
            Ok(IdlType::Array(Box::new(ty), len))
        }
        Type::Reference(r) => {
            // &[u8] => Bytes
            if let Type::Slice(s) = r.elem.as_ref() {
                if let Type::Path(p) = s.elem.as_ref() {
                    if p.path.is_ident("u8") {
                        return Ok(IdlType::Bytes);
                    }
                }
            }
            Err(Error::new_spanned(ty, "unsupported reference type"))
        }
        Type::Slice(s) => {
            // [u8] => Bytes (without the reference)
            if let Type::Path(p) = s.elem.as_ref() {
                if p.path.is_ident("u8") {
                    return Ok(IdlType::Bytes);
                }
            }
            Err(Error::new_spanned(
                ty,
                "bare slice types are not supported, use &[u8] for Bytes",
            ))
        }
        _ => Err(Error::new_spanned(ty, "unsupported type")),
    }
}

fn parse_idl_generic_arg(arg: &GenericArgument) -> Result<IdlGenericArg> {
    match arg {
        GenericArgument::Type(ty) => Ok(IdlGenericArg::Type {
            ty: parse_idl_type(ty)?,
        }),
        GenericArgument::Const(expr) => Ok(IdlGenericArg::Const {
            value: quote::quote!(#expr).to_string(),
        }),
        _ => Err(Error::new_spanned(arg, "unsupported generic argument")),
    }
}

fn parse_array_len(expr: &syn::Expr) -> Result<IdlArrayLen> {
    match expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(n),
            ..
        }) => Ok(IdlArrayLen::Value(n.base10_parse()?)),
        syn::Expr::Path(p) if p.path.segments.len() == 1 => Ok(IdlArrayLen::Generic(
            p.path.segments.first().unwrap().ident.to_string(),
        )),
        _ => Err(Error::new_spanned(
            expr,
            format!("unsupported array length: {}", quote::quote!(#expr)),
        )),
    }
}

fn extract_single_generic_type(args: &PathArguments) -> Result<&Type> {
    match args {
        PathArguments::AngleBracketed(ab) => {
            let arg = ab
                .args
                .first()
                .ok_or_else(|| Error::new_spanned(args, "expected generic argument"))?;
            match arg {
                GenericArgument::Type(ty) => Ok(ty),
                _ => Err(Error::new_spanned(arg, "expected type generic argument")),
            }
        }
        _ => Err(Error::new_spanned(
            args,
            "expected angle bracketed generics",
        )),
    }
}
