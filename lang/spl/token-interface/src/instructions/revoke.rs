// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use hayabusa_common::{
    Result,
    account_meta::AccountMeta,
    account_view::AccountView,
    cpi::{CpiAccount, CpiCtx},
    init_cpi_accounts,
    traits::CpiSerialize,
};
use hayabusa_invoke_cpi_macros::CpiInstruction;

/// Revokes the delegate's authority over the source account.
#[derive(CpiInstruction)]
pub struct Revoke<'view> {
    /// The source account.
    pub source: AccountView<'view>,
    /// The source account's owner.
    pub owner: AccountView<'view>,
    /// The token program to use for the CPI.
    pub token_program: AccountView<'view>,
}

/// The number of accounts required by the [`Revoke`] instruction.
pub const REVOKE_ACCOUNTS_BUFFER_LEN: usize = 2;
/// The length of the [`Revoke`] instruction data.
pub const REVOKE_DATA_LEN: usize = 1;
/// The [`Revoke`] instruction discriminator.
pub const REVOKE_DISCRIMINATOR: [u8; 1] = [5];

type AccountMetaBuffer<'meta> = [AccountMeta<'meta>; REVOKE_ACCOUNTS_BUFFER_LEN];
type CpiAccountBuffer<'view> = [CpiAccount<'view>; REVOKE_ACCOUNTS_BUFFER_LEN];
type DataBuffer = [u8; REVOKE_DATA_LEN];

impl<'view> CpiSerialize<'view, AccountMetaBuffer<'view>, CpiAccountBuffer<'view>, DataBuffer>
    for Revoke<'view>
{
    #[inline(always)]
    fn serialize(
        &self,
    ) -> Result<
        CpiCtx<'view, 'view, 'view, AccountMetaBuffer<'view>, CpiAccountBuffer<'view>, DataBuffer>,
    > {
        let data = REVOKE_DISCRIMINATOR;

        let metas = [
            AccountMeta::writable(self.source.address()),
            AccountMeta::readonly_signer(self.owner.address()),
        ];

        let views = init_cpi_accounts!(self.source, self.owner,);

        Ok(CpiCtx::new(
            self.token_program.address(),
            metas,
            views,
            data,
        ))
    }
}
