// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use serde::{Deserialize, Serialize};
use solana_address::Address;
use std::{collections::BTreeMap, str::FromStr};

pub trait IdlBuild {
    fn create_type() -> Option<IdlTypeDef> {
        None
    }
    #[allow(unused_variables)]
    fn insert_types(types: &mut BTreeMap<String, IdlTypeDef>) {}
    fn get_full_path() -> String {
        String::from(module_path!())
    }
}

macro_rules! impl_idl_build {
    ($($ty:ident),+ $(,)?) => {
        $(impl IdlBuild for $ty {})+
    };
}

impl_idl_build!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, bool, Address);

pub trait IdlPrint: Serialize {
    fn print(&self, prefix: &str) {
        let json = serde_json::to_string(self)
            .expect("IdlPrintError panic: could not convert IDL item to JSON");
        println!("{}{}", prefix, json);
    }
}

macro_rules! impl_idl_print {
    ($($ty:ident),+ $(,)?) => {
        $(impl IdlPrint for $ty {})+
    };
}

impl_idl_print!(
    Idl,
    IdlMetadata,
    IdlInstruction,
    IdlInstructionAccount,
    IdlAccount,
    IdlEvent,
    IdlErrorCode,
    IdlField,
    IdlTypeDef,
    IdlType,
);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Idl {
    pub address: String,
    pub metadata: IdlMetadata,
    #[serde(default, skip_serializing_if = "is_default")]
    pub docs: Vec<String>,
    pub instructions: Vec<IdlInstruction>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub accounts: Vec<IdlAccount>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub events: Vec<IdlEvent>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub errors: Vec<IdlErrorCode>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub types: Vec<IdlTypeDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdlMetadata {
    pub name: String,
    pub version: String,
    pub spec: String,
    #[serde(skip_serializing_if = "is_default")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdlInstruction {
    pub name: String,
    #[serde(default, skip_serializing_if = "is_default")]
    pub docs: Vec<String>,
    pub discriminator: IdlDiscriminator,
    pub accounts: Vec<IdlInstructionAccount>,
    pub args: Vec<IdlField>,
    #[serde(skip_serializing_if = "is_default")]
    pub returns: Option<IdlType>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdlInstructionAccount {
    pub name: String,
    #[serde(default, skip_serializing_if = "is_default")]
    pub docs: Vec<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub writable: bool,
    #[serde(default, skip_serializing_if = "is_default")]
    pub signer: bool,
    #[serde(skip_serializing_if = "is_default")]
    pub address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdlAccount {
    pub name: String,
    pub discriminator: IdlDiscriminator,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdlEvent {
    pub name: String,
    pub discriminator: IdlDiscriminator,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdlErrorCode {
    pub code: u32,
    pub name: String,
    #[serde(skip_serializing_if = "is_default")]
    pub msg: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdlField {
    pub name: String,
    #[serde(default, skip_serializing_if = "is_default")]
    pub docs: Vec<String>,
    #[serde(rename = "type")]
    pub ty: IdlType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdlTypeDef {
    pub name: String,
    #[serde(default, skip_serializing_if = "is_default")]
    pub docs: Vec<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub serialization: IdlSerialization,
    #[serde(skip_serializing_if = "is_default")]
    pub repr: Option<IdlRepr>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub generics: Vec<IdlTypeDefGeneric>,
    #[serde(rename = "type")]
    pub ty: IdlTypeDefTy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum IdlType {
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    F32,
    F64,
    Bool,
    Pubkey,
    Bytes,
    Option(Box<IdlType>),
    Vec(Box<IdlType>),
    Array(Box<IdlType>, IdlArrayLen),
    Defined {
        name: String,
        #[serde(default, skip_serializing_if = "is_default")]
        generics: Vec<IdlGenericArg>,
    },
    Generic(String),
}

impl FromStr for IdlType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s = s.to_owned();
        s.retain(|c| !c.is_whitespace());

        let idl_type = match s.as_str() {
            "u8" => IdlType::U8,
            "u16" => IdlType::U16,
            "u32" => IdlType::U32,
            "u64" => IdlType::U64,
            "u128" => IdlType::U128,
            "i8" => IdlType::I8,
            "i16" => IdlType::I16,
            "i32" => IdlType::I32,
            "i64" => IdlType::I64,
            "i128" => IdlType::I128,
            "f32" => IdlType::F32,
            "f64" => IdlType::F64,
            "bool" => IdlType::Bool,
            "Address" => IdlType::Pubkey, // Address is used instead of Pubkey in hayabusa
            "&[u8]" => IdlType::Bytes,
            _ => {
                if s.starts_with("[") {
                    fn array_from_str(inner: &str) -> IdlType {
                        match inner.strip_suffix("]") {
                            Some(nested_inner) => array_from_str(&nested_inner[1..]),
                            None => {
                                let (raw_type, raw_length) = inner.rsplit_once(";").unwrap();
                                let ty = IdlType::from_str(raw_type).unwrap();
                                let len = match raw_length.replace("_", "").parse::<usize>() {
                                    Ok(len) => IdlArrayLen::Value(len),
                                    Err(_) => IdlArrayLen::Generic(raw_length.to_owned()),
                                };
                                IdlType::Array(Box::new(ty), len)
                            }
                        }
                    }

                    return Ok(array_from_str(&s));
                }

                let (name, generics) = if let Some(i) = s.find("<") {
                    (
                        s.get(..i).unwrap().to_owned(),
                        s.get(i + 1..)
                            .unwrap()
                            .strip_suffix('>')
                            .unwrap()
                            .split(',')
                            .map(|g| g.trim().to_owned())
                            .map(|g| {
                                if g.parse::<bool>().is_ok()
                                    || g.parse::<u128>().is_ok()
                                    || g.parse::<i128>().is_ok()
                                    || g.parse::<char>().is_ok()
                                {
                                    Ok(IdlGenericArg::Const { value: g })
                                } else {
                                    Self::from_str(&g).map(|ty| IdlGenericArg::Type { ty })
                                }
                            })
                            .collect::<Result<Vec<_>, _>>()?,
                    )
                } else {
                    (s.to_owned(), vec![])
                };

                IdlType::Defined { name, generics }
            }
        };

        Ok(idl_type)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "lowercase")]
#[non_exhaustive]
pub enum IdlRepr {
    Rust(IdlReprModifier),
    C(IdlReprModifier),
    Transparent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdlReprModifier {
    #[serde(default, skip_serializing_if = "is_default")]
    pub packed: bool,
    #[serde(default, skip_serializing_if = "is_default")]
    pub align: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum IdlSerialization {
    #[default]
    Bytemuck,
    BytemuckUnsafe,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum IdlTypeDefGeneric {
    Type {
        name: String,
    },
    Const {
        name: String,
        #[serde(rename = "type")]
        ty: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum IdlTypeDefTy {
    Struct {
        #[serde(skip_serializing_if = "is_default")]
        fields: Option<IdlDefinedFields>,
    },
    Enum {
        variants: Vec<IdlEnumVariant>,
    },
    Type {
        alias: IdlType,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum IdlDefinedFields {
    Named(Vec<IdlField>),
    Tuple(Vec<IdlType>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdlEnumVariant {
    pub name: String,
    #[serde(skip_serializing_if = "is_default")]
    pub fields: Option<IdlDefinedFields>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum IdlArrayLen {
    Generic(String),
    #[serde(untagged)]
    Value(usize),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum IdlGenericArg {
    Type {
        #[serde(rename = "type")]
        ty: IdlType,
    },
    Const {
        value: String,
    },
}

pub type IdlDiscriminator = Vec<u8>;

/// Get whether the given data is the default of its type.
fn is_default<T: Default + PartialEq>(it: &T) -> bool {
    *it == T::default()
}

impl ToTokens for IdlRepr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            IdlRepr::Rust(m) => quote! { ::hayabusa::idl::IdlRepr::Rust(#m) },
            IdlRepr::C(m) => quote! { ::hayabusa::idl::IdlRepr::C(#m) },
            IdlRepr::Transparent => quote! { ::hayabusa::idl::IdlRepr::Transparent },
        });
    }
}

impl ToTokens for IdlReprModifier {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let packed = self.packed;
        let align = match self.align {
            Some(n) => quote! { Some(#n) },
            None => quote! { None },
        };
        tokens.extend(quote! {
            hayabusa::idl::IdlReprModifier {
                packed: #packed,
                align: #align,
            }
        });
    }
}

impl ToTokens for IdlTypeDefGeneric {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            IdlTypeDefGeneric::Type { name } => {
                quote! { ::hayabusa::idl::IdlTypeDefGeneric::Type { name: #name.to_string() } }
            }
            IdlTypeDefGeneric::Const { name, ty } => {
                quote! { ::hayabusa::idl::IdlTypeDefGeneric::Const { name: #name.to_string(), ty: #ty.to_string() } }
            }
        });
    }
}

impl ToTokens for IdlTypeDefTy {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            IdlTypeDefTy::Struct { fields } => {
                if let Some(fields) = fields {
                    quote! {
                        ::hayabusa::idl::IdlTypeDefTy::Struct { fields: Some(#fields) }
                    }
                } else {
                    quote! {
                        ::hayabusa::idl::IdlTypeDefTy::Struct { fields: None }
                    }
                }
            }
            IdlTypeDefTy::Enum { variants } => {
                quote! {
                    ::hayabusa::idl::IdlTypeDefTy::Enum { variants: vec![#(#variants),*] }
                }
            }
            IdlTypeDefTy::Type { alias } => {
                quote! {
                    ::hayabusa::idl::IdlTypeDefTy::Type { alias: #alias }
                }
            }
        });
    }
}

impl ToTokens for IdlEnumVariant {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = self.name.clone();
        let fields = self.fields.clone();

        tokens.extend(quote! {
            ::hayabusa::idl::IdlEnumVariant {
                name: #name.to_string(),
                fields: #fields,
            }
        });
    }
}

impl ToTokens for IdlDefinedFields {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            IdlDefinedFields::Named(f) => {
                quote! {
                    ::hayabusa::idl::IdlDefinedFields::Named(vec![#(#f),*])
                }
            }
            IdlDefinedFields::Tuple(t) => {
                quote! {
                    ::hayabusa::idl::IdlDefinedFields::Tuple(vec![#(#t),*])
                }
            }
        });
    }
}

impl ToTokens for IdlField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = self.name.clone();
        let docs = &self.docs;
        let ty = self.ty.clone();

        let extension = quote! {
            ::hayabusa::idl::IdlField {
                name: #name.to_string(),
                docs: vec![#(#docs.to_string()),*],
                ty: #ty,
            }
        };

        tokens.extend(extension);
    }
}

impl ToTokens for IdlType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            IdlType::U8 => quote! { ::hayabusa::idl::IdlType::U8 },
            IdlType::U16 => quote! { ::hayabusa::idl::IdlType::U16 },
            IdlType::U32 => quote! { ::hayabusa::idl::IdlType::U32 },
            IdlType::U64 => quote! { ::hayabusa::idl::IdlType::U64 },
            IdlType::U128 => quote! { ::hayabusa::idl::IdlType::U128 },
            IdlType::I8 => quote! { ::hayabusa::idl::IdlType::I8 },
            IdlType::I16 => quote! { ::hayabusa::idl::IdlType::I16 },
            IdlType::I32 => quote! { ::hayabusa::idl::IdlType::I32 },
            IdlType::I64 => quote! { ::hayabusa::idl::IdlType::I64 },
            IdlType::I128 => quote! { ::hayabusa::idl::IdlType::I128 },
            IdlType::F32 => quote! { ::hayabusa::idl::IdlType::F32 },
            IdlType::F64 => quote! { ::haybausa::idl::IdlType::F64 },
            IdlType::Bool => quote! { ::hayabusa::idl::IdlType::Bool },
            IdlType::Bytes => quote! { ::hayabusa::idl::IdlType::Bytes },
            IdlType::Pubkey => quote! { ::hayabusa::idl::IdlType::Pubkey },
            IdlType::Generic(g) => quote! { ::hayabusa::idl::IdlType::Generic(#g.to_string()) },
            IdlType::Array(ty, len) => quote! { ::hayabusa::idl::IdlType::Array(#ty, #len) },
            IdlType::Vec(ty) => quote! { ::hayabusa::idl::IdlType::Vec(#ty) },
            IdlType::Option(ty) => quote! { ::hayabusa::idl::IdlType::Option(#ty) },
            IdlType::Defined { name, generics } => quote! { ::hayabusa::idl::IdlType::Defined { name: #name.to_string(), generics: vec![#(#generics),*] } },
        });
    }
}

impl ToTokens for IdlArrayLen {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            IdlArrayLen::Generic(g) => {
                quote! { ::hayabusa::idl::IdlArrayLen::Generic(#g.to_string()) }
            }
            IdlArrayLen::Value(n) => quote! { ::hayabusa::idl::IdlArrayLen::Value(#n) },
        })
    }
}

impl ToTokens for IdlGenericArg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            IdlGenericArg::Type { ty } => {
                quote! { ::hayabusa::idl::IdlGenericArg::Type { ty: #ty } }
            }
            IdlGenericArg::Const { value } => {
                quote! { ::hayabusa::idl::IdlGenericArg::Const { value: #value.to_string() } }
            }
        });
    }
}
