// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "bpf")]
use hayabusa_common::traits::Cast;
use hayabusa_common::{
    Result,
    account_view::AccountView,
    accounts::account::Account,
    address::Address,
    error::ErrorCode,
    hint,
    traits::{
        __AccountDiscriminatorMode, __AccountMarker, __Owner, __ParseAccountDispatch, __SplAccount,
        __SplAccountMarker, Id, Owner, Ownership, Space,
    },
};
use hayabusa_err_macro::err;
use solana_program_error::ProgramError;

use crate::Token;

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

unsafe impl __SplAccountMarker for Multisig {}

#[cfg(feature = "bpf")]
unsafe impl Cast for Multisig {}

impl Owner for Multisig {
    const OWNER: &Address = Token::ID;
}

unsafe impl Ownership for Multisig {
    type OwnerType = __Owner;

    #[inline(always)]
    fn check_ownership(view: AccountView<'_>) -> Result<()> {
        Self::check_owner(view)
    }
}

impl Space for Multisig {}

unsafe impl __AccountDiscriminatorMode for Multisig {
    type Mode = __SplAccount;
}

impl<'view>
    __ParseAccountDispatch<
        'view,
        <Multisig as __AccountDiscriminatorMode>::Mode,
        Account<'view, Multisig>,
    > for Multisig
{
    #[inline(always)]
    fn dispatch(view: AccountView<'view>) -> Result<Account<'view, Multisig>> {
        Self::check_owner(view)?;
        Self::check_space(view)?;

        // SAFETY: `Self::cast_unchecked` is safe because we have already checked the owner and space.
        if hint::unlikely(!unsafe { Self::cast_unchecked(view)? }.is_initialized()) {
            err!(
                "Multisig::dispatch: account is not initialized",
                ErrorCode::InvalidAccount,
            );
        }

        // SAFETY: `Account::new` is safe because we have already checked the owner and space.
        Ok(unsafe { Account::new(view) })
    }
}
