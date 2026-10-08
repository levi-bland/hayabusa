// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_view::AccountView,
    address::{address_eq, Address},
    error::ErrorCode,
    hint,
    traits::{ParseSysvar, ToAccountView},
    Result,
};
use core::ops::Deref;
use hayabusa_err_macro::err;
use solana_program_error::ProgramError;

/// Maximum permitted size of account data (10 MiB).
const MAX_PERMITTED_DATA_LENGTH: u64 = 10 * 1024 * 1024;

/// Default rental rate in lamports/byte.
///
/// This calculation is based on:
/// - `10^9` lamports per SOL
/// - `$1` per SOL
/// - `$0.01` per megabyte day
/// - `$7.30` per megabyte
pub const DEFAULT_LAMPORTS_PER_BYTE: u64 = 6960;

/// Account storage overhead for calculation of base rent.
///
/// This is the number of bytes required to store an account with no data. It is
/// added to an accounts data length when calculating [`Rent::minimum_balance`].
pub const ACCOUNT_STORAGE_OVERHEAD: u64 = 128;

/// Maximum lamports per byte value.
const MAX_LAMPORTS_PER_BYTE: u64 = 1_759_197_129_867;

pub mod layout {
    use super::{
        ProgramError, ACCOUNT_STORAGE_OVERHEAD, MAX_LAMPORTS_PER_BYTE, MAX_PERMITTED_DATA_LENGTH,
    };
    use crate::{address, address::Address, hint, impl_sysvar_get, traits::Sysvar, Result};
    /// Rent sysvar data
    #[repr(C)]
    #[derive(Clone, Copy, Debug)]
    pub struct Rent {
        /// Rental rate in lamports per byte.
        lamports_per_byte: u64,
    }

    impl Rent {
        /// The ID of the rent sysvar.
        pub const ID: Address = address!("SysvarRent111111111111111111111111111111111");

        /// Calculates the minimum balance for rent exemption without performing
        /// any validation.
        ///
        /// # Important
        ///
        /// The caller must ensure that `data_len` is within the permitted limit
        /// and the `lamports_per_byte` is within the permitted limit based on
        /// the `exemption_threshold` to avoid overflow.
        ///
        /// # Arguments
        ///
        /// * `data_len` - The number of bytes in the account
        ///
        /// # Returns
        ///
        /// The minimum balance in lamports for rent exemption.
        #[inline(always)]
        pub fn minimum_balance_unchecked(&self, data_len: usize) -> u64 {
            (ACCOUNT_STORAGE_OVERHEAD + data_len as u64) * self.lamports_per_byte
        }

        /// Calculates the minimum balance for rent exemption.
        ///
        /// This method avoids floating-point operations when the
        /// `exemption_threshold` is the default value.
        ///
        /// # Arguments
        ///
        /// * `data_len` - The number of bytes in the account
        ///
        /// # Returns
        ///
        /// The minimum balance in lamports for rent exemption.
        ///
        /// # Errors
        ///
        /// Returns `ProgramError::InvalidArgument` if `data_len` exceeds the
        /// maximum permitted data length or if the `lamports_per_byte` is too
        /// large based on the `exemption_threshold`, which would cause an
        /// overflow.
        #[inline(always)]
        pub fn try_minimum_balance(&self, data_len: usize) -> Result<u64> {
            if hint::unlikely(data_len as u64 > MAX_PERMITTED_DATA_LENGTH) {
                return Err(ProgramError::InvalidArgument);
            }

            // Validate `lamports_per_byte` based on `exemption_threshold` to prevent
            // overflow.

            if hint::unlikely(self.lamports_per_byte > MAX_LAMPORTS_PER_BYTE) {
                return Err(ProgramError::InvalidArgument);
            }

            Ok(self.minimum_balance_unchecked(data_len))
        }
    }

    // Assert that the size of the `Rent` struct is as expected (8 bytes).
    const _ASSERT_STRUCT_LEN: () = assert!(size_of::<Rent>() == 8);

    // Assert that the alignment of the `Rent` struct is as expected (8 byte).
    const _ASSERT_ACCOUNT_ALIGN: () = assert!(align_of::<Rent>() == 8);

    impl Sysvar for Rent {
        impl_sysvar_get!(Rent::ID, 0);
    }
}

pub struct Rent<'view> {
    view: AccountView<'view>,
    rent: layout::Rent,
}

impl Rent<'_> {
    /// The ID of the `Rent` sysvar.
    pub const ID: &'static Address = &layout::Rent::ID;
}

impl<'view> ParseSysvar<'view> for Rent<'view> {
    #[inline(always)]
    fn parse(view: AccountView<'view>) -> Result<Self> {
        if hint::unlikely(!address_eq(view.address(), &Rent::ID)) {
            err!(
                "Rent::try_sysvar_from_account_view: invalid account",
                ErrorCode::InvalidSysvarAccount,
            );
        }

        // SAFETY: runtime guarantees the Rent sysvar account to be valid.
        let rent = unsafe { *(view.data_ptr() as *const layout::Rent) };

        Ok(Rent { view, rent })
    }
}

impl<'view> ToAccountView<'view> for Rent<'view> {
    #[inline(always)]
    fn to_account_view(&self) -> AccountView<'view> {
        self.view
    }
}

impl Deref for Rent<'_> {
    type Target = layout::Rent;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.rent
    }
}
