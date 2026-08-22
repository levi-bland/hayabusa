// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

#![allow(unexpected_cfgs)]
#![no_std]

pub mod account_meta;
pub mod account_view;
#[cfg(feature = "bpf")]
pub mod accounts;
pub mod address;
pub mod cpi;
pub mod ctx;
#[cfg(all(feature = "bpf", not(feature = "no-entrypoint")))]
pub mod entrypoint;
pub mod error;
pub mod prelude;
#[cfg(feature = "spl")]
pub mod spl;
pub mod syscalls;
pub mod system_program;
pub mod sysvars;
pub mod traits;
pub mod vec;

pub type Result<T> = core::result::Result<T, solana_program_error::ProgramError>;

/// Module with functions to provide hints to the compiler about how code
/// should be optimized.
pub mod hint {
    /// A "dummy" function with a hint to the compiler that it is unlikely to be
    /// called.
    ///
    /// This function is used as a hint to the compiler to optimize other code paths
    /// instead of the one where the function is used.
    #[cold]
    pub const fn cold_path() {}

    /// Return the given `bool` value with a hint to the compiler that `true` is the
    /// likely case.
    #[inline(always)]
    pub const fn likely(b: bool) -> bool {
        if b {
            true
        } else {
            cold_path();
            false
        }
    }

    /// Return a given `bool` value with a hint to the compiler that `false` is the
    /// likely case.
    #[inline(always)]
    pub const fn unlikely(b: bool) -> bool {
        if b {
            cold_path();
            true
        } else {
            false
        }
    }
}
