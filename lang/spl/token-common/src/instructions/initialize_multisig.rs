// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use core::{array::from_fn, marker::PhantomData};

use crate::TokenProgram;
use hayabusa_common::{
    Result,
    account_meta::AccountMeta,
    account_view::AccountView,
    cpi::{CpiAccount, CpiCtx},
    traits::CpiSerialize,
};
use hayabusa_invoke_cpi_macros::CpiInstruction;
use solana_program_error::ProgramError;

/// Initializes a multisig account with `N` provided signers, `M` of
/// which are required to validate it.
///
/// Unlike the other instructions in this module the account list is
/// variable length, so the buffers are sized by the `ACCOUNTS` const
/// parameter: `ACCOUNTS == 2 + N`, i.e. the multisig account, the Rent
/// sysvar, and one entry per signer. [`Self::serialize`] fails if
/// `signers` does not hold exactly `ACCOUNTS - 2` entries.
#[derive(CpiInstruction)]
pub struct InitializeMultisig<'view, 'signers, const ACCOUNTS: usize, P: TokenProgram> {
    /// The multisig account to initialize.
    pub multisig: AccountView<'view>,
    /// The Rent sysvar.
    pub rent_sysvar: AccountView<'view>,
    /// The signer accounts, `1 <= N <= 11`.
    pub signers: &'signers [AccountView<'view>],
    /// The number of signers (M) required to validate this multisig.
    pub m: u8,
    /// Signifies which token program to use for the CPI.
    __phantom: PhantomData<P>,
}

impl<'view, 'signers, const ACCOUNTS: usize, P: TokenProgram>
    InitializeMultisig<'view, 'signers, ACCOUNTS, P>
{
    pub fn new(
        multisig: AccountView<'view>,
        rent_sysvar: AccountView<'view>,
        signers: &'signers [AccountView<'view>],
        m: u8,
    ) -> Self {
        Self {
            multisig,
            rent_sysvar,
            signers,
            m,
            __phantom: PhantomData,
        }
    }
}

/// The minimum number of multisig signers (min N).
pub const MIN_SIGNERS: usize = 1;
/// The maximum number of multisig signers (max N).
pub const MAX_SIGNERS: usize = 11;
/// The number of non-signer accounts in the [`InitializeMultisig`] account list.
pub const INITIALIZE_MULTISIG_FIXED_ACCOUNTS: usize = 2;
/// The length of the [`InitializeMultisig`] instruction data.
pub const INITIALIZE_MULTISIG_DATA_LEN: usize = 2;
/// The [`InitializeMultisig`] instruction discriminator.
pub const INITIALIZE_MULTISIG_DISCRIMINATOR: [u8; 1] = [2];

type AccountMetaBuffer<'meta, const ACCOUNTS: usize> = [AccountMeta<'meta>; ACCOUNTS];
type CpiAccountBuffer<'view, const ACCOUNTS: usize> = [CpiAccount<'view>; ACCOUNTS];
type DataBuffer = [u8; INITIALIZE_MULTISIG_DATA_LEN];

impl<'view, const ACCOUNTS: usize, P: TokenProgram>
    CpiSerialize<
        'view,
        AccountMetaBuffer<'view, ACCOUNTS>,
        CpiAccountBuffer<'view, ACCOUNTS>,
        DataBuffer,
    > for InitializeMultisig<'view, '_, ACCOUNTS, P>
{
    #[inline(always)]
    fn serialize(
        &self,
    ) -> Result<
        CpiCtx<
            'view,
            'view,
            'static,
            AccountMetaBuffer<'view, ACCOUNTS>,
            CpiAccountBuffer<'view, ACCOUNTS>,
            DataBuffer,
        >,
    > {
        const {
            assert!(
                ACCOUNTS >= INITIALIZE_MULTISIG_FIXED_ACCOUNTS + MIN_SIGNERS
                    && ACCOUNTS <= INITIALIZE_MULTISIG_FIXED_ACCOUNTS + MAX_SIGNERS,
                "InitializeMultisig requires 1..=11 signer accounts",
            );
        }

        let signers = ACCOUNTS - INITIALIZE_MULTISIG_FIXED_ACCOUNTS;

        if self.signers.len() != signers || self.m == 0 || self.m as usize > signers {
            return Err(ProgramError::InvalidArgument.into());
        }

        let data = [INITIALIZE_MULTISIG_DISCRIMINATOR[0], self.m];

        let metas = from_fn(|i| match i {
            0 => AccountMeta::writable(self.multisig.address()),
            1 => AccountMeta::readonly(self.rent_sysvar.address()),
            _ => AccountMeta::readonly(self.signers[i - 2].address()),
        });

        let views = from_fn(|i| match i {
            0 => CpiAccount::from(self.multisig),
            1 => CpiAccount::from(self.rent_sysvar),
            _ => CpiAccount::from(self.signers[i - 2]),
        });

        Ok(CpiCtx::new(&P::ID, metas, views, data))
    }
}
