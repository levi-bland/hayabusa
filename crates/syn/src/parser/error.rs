// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use syn::{
    ext::IdentExt,
    parse::{Error as ParserError, Parse, ParseStream, Result},
    Expr, Ident, ItemEnum, Lit, LitInt, Meta, Token,
};

pub fn parse(error_enum: &mut ItemEnum, args: Option<ErrorArgs>) -> Error {
    let ident = error_enum.ident.clone();
    let mut last_discriminant = 0;
    let codes: Vec<ErrorCode> = error_enum
        .variants
        .iter_mut()
        .map(|variant| {
            let msg = parse_error_attribute(variant);
            let ident = variant.ident.clone();
            let id = match &variant.discriminant {
                None => last_discriminant,
                Some((_, disc)) => match disc {
                    Expr::Lit(expr_lit) => match &expr_lit.lit {
                        Lit::Int(int) => {
                            int.base10_parse::<u32>().expect("Must be a base 10 number")
                        }
                        _ => panic!("Invalid error discriminant"),
                    },
                    _ => panic!("Invalid error discriminant"),
                },
            };
            last_discriminant = id + 1;

            // remove any non-doc attributes on the variant
            variant
                .attrs
                .retain(|attr| attr.path().segments[0].ident == "doc");

            ErrorCode { id, ident, msg }
        })
        .collect();

    Error {
        name: error_enum.ident.to_string(),
        raw_enum: error_enum.clone(),
        ident,
        codes,
        args,
    }
}

fn parse_error_attribute(variant: &syn::Variant) -> Option<String> {
    let attrs = variant
        .attrs
        .iter()
        .filter(|attr| attr.path().segments[0].ident != "doc")
        .collect::<Vec<_>>();
    match attrs.len() {
        0 => None,
        1 => {
            let attr = &attrs[0];
            let attr_str = attr.path().segments[0].ident.to_string();
            assert!(
                &attr_str == "msg",
                "Use #[msg(...)] to specify error strings"
            );
            let msg = match &attr.meta {
                Meta::List(meta_list) => {
                    let mut tts = meta_list.tokens.clone().into_iter();
                    match tts.next().expect("Must specify a message string") {
                        proc_macro2::TokenTree::Literal(lit) => lit.to_string().replace('\"', ""),
                        _ => panic!("Invalid syntax"),
                    }
                }
                _ => panic!("Invalid syntax"),
            };
            Some(msg)
        }
        _ => {
            panic!("Too many attributes found. Use `#[msg(...)]` to specify error strings");
        }
    }
}

pub struct ErrorInput {
    pub error_code: Expr,
}

impl Parse for ErrorInput {
    fn parse(stream: ParseStream) -> Result<Self> {
        let error_code = stream.call(Expr::parse)?;
        Ok(Self { error_code })
    }
}

pub struct Error {
    pub name: String,
    pub raw_enum: ItemEnum,
    pub ident: Ident,
    pub codes: Vec<ErrorCode>,
    pub args: Option<ErrorArgs>,
}

pub struct ErrorCode {
    pub id: u32,
    pub ident: Ident,
    pub msg: Option<String>,
}

pub struct ErrorArgs {
    pub offset: LitInt,
}

impl Parse for ErrorArgs {
    fn parse(stream: ParseStream) -> Result<Self> {
        let offset_span = stream.span();
        let offset: Ident = stream.call(IdentExt::parse_any)?;

        if offset.to_string().as_str() != "offset" {
            return Err(ParserError::new(offset_span, "expected keyword offset"));
        }
        stream.parse::<Token![=]>()?;
        let offset: LitInt = stream.parse()?;
        Ok(ErrorArgs { offset })
    }
}
