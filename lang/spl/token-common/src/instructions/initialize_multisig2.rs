// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use core::{array::from_fn, marker::PhantomData};

use super::initialize_multisig::{MAX_SIGNERS, MIN_SIGNERS};
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

/// Like [`super::InitializeMultisig`], but does not require the Rent
/// sysvar to be provided.
///
/// The account list is variable length, so the buffers are sized by the
/// `ACCOUNTS` const parameter: `ACCOUNTS == 1 + N`, i.e. the multisig
/// account plus one entry per signer. [`Self::serialize`] fails if
/// `signers` does not hold exactly `ACCOUNTS - 1` entries.
#[derive(CpiInstruction)]
pub struct InitializeMultisig2<'view, 'signers, const ACCOUNTS: usize, P: TokenProgram> {
    /// The multisig account to initialize.
    pub multisig: AccountView<'view>,
    /// The signer accounts, `1 <= N <= 11`.
    pub signers: &'signers [AccountView<'view>; ACCOUNTS],
    /// The number of signers (M) required to validate this multisig.
    pub m: u8,
    /// Signifies which token program to use for the CPI.
    __phantom: PhantomData<P>,
}

impl<'view, 'signers, const ACCOUNTS: usize, P: TokenProgram>
    InitializeMultisig2<'view, 'signers, ACCOUNTS, P>
{
    pub fn new(
        multisig: AccountView<'view>,
        signers: &'signers [AccountView<'view>; ACCOUNTS],
        m: u8,
    ) -> Self {
        Self {
            multisig,
            signers,
            m,
            __phantom: PhantomData,
        }
    }
}

/// The number of non-signer accounts in the [`InitializeMultisig2`] account list.
pub const INITIALIZE_MULTISIG2_FIXED_ACCOUNTS: usize = 1;
/// The length of the [`InitializeMultisig2`] instruction data.
pub const INITIALIZE_MULTISIG2_DATA_LEN: usize = 2;
/// The [`InitializeMultisig2`] instruction discriminator.
pub const INITIALIZE_MULTISIG2_DISCRIMINATOR: [u8; 1] = [19];

type AccountMetaBuffer<'meta, const ACCOUNTS: usize> = [AccountMeta<'meta>; ACCOUNTS];
type CpiAccountBuffer<'view, const ACCOUNTS: usize> = [CpiAccount<'view>; ACCOUNTS];
type DataBuffer = [u8; INITIALIZE_MULTISIG2_DATA_LEN];

impl<'view, const ACCOUNTS: usize, P: TokenProgram>
    CpiSerialize<
        'view,
        AccountMetaBuffer<'view, ACCOUNTS>,
        CpiAccountBuffer<'view, ACCOUNTS>,
        DataBuffer,
    > for InitializeMultisig2<'view, '_, ACCOUNTS, P>
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
                ACCOUNTS >= INITIALIZE_MULTISIG2_FIXED_ACCOUNTS + MIN_SIGNERS
                    && ACCOUNTS <= INITIALIZE_MULTISIG2_FIXED_ACCOUNTS + MAX_SIGNERS,
                "InitializeMultisig2 requires 1..=11 signer accounts",
            );
        }

        let signers = ACCOUNTS - INITIALIZE_MULTISIG2_FIXED_ACCOUNTS;

        if self.signers.len() != signers || self.m == 0 || self.m as usize > signers {
            return Err(ProgramError::InvalidArgument.into());
        }

        let data = [INITIALIZE_MULTISIG2_DISCRIMINATOR[0], self.m];

        let metas = from_fn(|i| match i {
            0 => AccountMeta::writable(self.multisig.address()),
            _ => AccountMeta::readonly(self.signers[i - 1].address()),
        });

        let views = from_fn(|i| match i {
            0 => CpiAccount::from(self.multisig),
            _ => CpiAccount::from(self.signers[i - 1]),
        });

        Ok(CpiCtx::new(&P::ID, metas, views, data))
    }
}
