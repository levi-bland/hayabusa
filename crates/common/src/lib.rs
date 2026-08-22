// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

#![no_std]

pub use solana_account_view::{self as account_view, AccountView, Ref, RefMut};
pub use solana_address::{address, address_eq, declare_id, Address, ADDRESS_BYTES};

pub trait IdlTypeName {
    fn idl_type() -> &'static str;
}

macro_rules! impl_idl_type {
    ($($rust_ty:ty => $idl_str:literal),* $(,)?) => {
        $(
            impl IdlTypeName for $rust_ty {
                fn idl_type() -> &'static str { $idl_str }
            }
        )*
    };
}

impl_idl_type! {
    // bool => "bool" --- bool is not Pod
    u8 => "u8",
    u16 => "u16",
    u32 => "u32",
    u64 => "u64",
    u128 => "u128",
    i8 => "i8",
    i16 => "i16",
    i32 => "i32",
    i64 => "i64",
    i128 => "i128",
    // f32 => "f32" --- this is commented because i decided to ban floats
    // f64 => "f64"
}

impl IdlTypeName for Address {
    fn idl_type() -> &'static str {
        "pubkey"
    }
}

/// Trait for types representing a single program.
///
/// # Example
/// ```ignore
/// impl ProgramId for SystemProgram {
///     const ID: Address = system_program::ID;
/// }
///
/// // Use in validation
/// if account.owner() != T::ID {
///     return Err(ProgramError::InvalidAccountOwner);
/// }
/// ```
pub trait ProgramId {
    /// The program's public key.
    const ID: Address;
}

/// Trait for types representing multiple valid programs.
///
/// Used when an account can be one of several programs (e.g. Token or Token2022).
///
/// # Example
/// ```ignore
/// impl ProgramIds for TokenInterface {
///     const IDS: &'static [Address] = &[
///         token::ID,
///         token_2022::ID,
///     ];
/// }   
/// ```
pub trait ProgramIds {
    /// Slice of valid program addresses.
    const IDS: &'static [Address];
}
