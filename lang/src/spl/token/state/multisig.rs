// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use crate::{
    prelude::*,
    spl::token::Token,
    traits::internal::{__AccountDiscriminatorMode, __NoDiscriminator},
};

/// Maximum number of multisignature signers.
pub const MAX_MULTISIG_SIGNERS: usize = 11;

/// Multisignature data.
#[repr(C)]
pub struct Multisig {
    /// Number of signers required
    m: u8,
    /// Number of valid signers
    n: u8,
    /// Is `true` if this structure has been initialized
    is_initialized: u8,
    /// Signer public keys
    signers: [Address; MAX_MULTISIG_SIGNERS],
}

impl Multisig {
    /// Number of signers required to validate the `Multisig` signature.
    #[inline(always)]
    pub const fn required_signers(&self) -> u8 {
        self.m
    }

    /// Number of signer addresses present on the `Multisig`.
    #[inline(always)]
    pub const fn signers_len(&self) -> usize {
        self.n as usize
    }

    /// Return the signer addresses of the `Multisig`.
    #[inline(always)]
    pub fn signers(&self) -> &[Address] {
        // SAFETY: `self.signers` is an array of `Address` with a fixed size of
        // `MAX_MULTISIG_SIGNERS`; `self.signers_len` is always `<=
        // MAX_MULTISIG_SIGNERS` and indicates how many of these signers are
        // valid.
        unsafe { self.signers.get_unchecked(..self.signers_len()) }
    }

    /// Check whether the multisig is initialized or not.
    //
    // It will return a boolean value indicating whether [`self.is_initialized`]
    // is different than `0` or not.
    #[inline(always)]
    pub fn is_initialized(&self) -> bool {
        self.is_initialized != 0
    }
}

unsafe impl __AccountMarker for Multisig {}

#[cfg(feature = "bpf")]
unsafe impl Cast for Multisig {}

impl Owner for Multisig {
    const OWNER: Address = Token::ID;
}

impl Space for Multisig {}

impl __AccountDiscriminatorMode for Multisig {
    type Mode = __NoDiscriminator;
}
