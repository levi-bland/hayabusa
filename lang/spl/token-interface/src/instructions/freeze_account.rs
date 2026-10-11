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

/// Freezes an initialized account using the mint's `freeze_authority`
/// (if set).
#[derive(CpiInstruction)]
pub struct FreezeAccount<'view> {
    /// The account to freeze.
    pub account: AccountView<'view>,
    /// The token mint.
    pub mint: AccountView<'view>,
    /// The mint's freeze authority.
    pub authority: AccountView<'view>,
    /// The token program to use for the CPI.
    pub token_program: AccountView<'view>,
}

/// The number of accounts required by the [`FreezeAccount`] instruction.
pub const FREEZE_ACCOUNT_ACCOUNTS_BUFFER_LEN: usize = 3;
/// The length of the [`FreezeAccount`] instruction data.
pub const FREEZE_ACCOUNT_DATA_LEN: usize = 1;
/// The [`FreezeAccount`] instruction discriminator.
pub const FREEZE_ACCOUNT_DISCRIMINATOR: [u8; 1] = [10];

type AccountMetaBuffer<'meta> = [AccountMeta<'meta>; FREEZE_ACCOUNT_ACCOUNTS_BUFFER_LEN];
type CpiAccountBuffer<'view> = [CpiAccount<'view>; FREEZE_ACCOUNT_ACCOUNTS_BUFFER_LEN];
type DataBuffer = [u8; FREEZE_ACCOUNT_DATA_LEN];

impl<'view> CpiSerialize<'view, AccountMetaBuffer<'view>, CpiAccountBuffer<'view>, DataBuffer>
    for FreezeAccount<'view>
{
    #[inline(always)]
    fn serialize(
        &self,
    ) -> Result<
        CpiCtx<'view, 'view, 'view, AccountMetaBuffer<'view>, CpiAccountBuffer<'view>, DataBuffer>,
    > {
        let data = FREEZE_ACCOUNT_DISCRIMINATOR;

        let metas = [
            AccountMeta::writable(self.account.address()),
            AccountMeta::readonly(self.mint.address()),
            AccountMeta::readonly_signer(self.authority.address()),
        ];

        let views = init_cpi_accounts!(self.account, self.mint, self.authority,);

        Ok(CpiCtx::new(
            self.token_program.address(),
            metas,
            views,
            data,
        ))
    }
}
