// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use crate::TokenProgram;
use core::marker::PhantomData;
use hayabusa_common::{
    Result,
    account_meta::AccountMeta,
    account_view::AccountView,
    cpi::{CpiAccount, CpiCtx},
    init_cpi_accounts,
    traits::CpiSerialize,
};
use hayabusa_invoke_cpi_macros::CpiInstruction;

/// Thaws a frozen account using the mint's `freeze_authority` (if set).
#[derive(CpiInstruction)]
pub struct ThawAccount<'view, P: TokenProgram> {
    /// The account to thaw.
    pub account: AccountView<'view>,
    /// The token mint.
    pub mint: AccountView<'view>,
    /// The mint's freeze authority.
    pub authority: AccountView<'view>,
    /// Signifies which token program to use for the CPI.
    __phantom: PhantomData<P>,
}

impl<'view, P: TokenProgram> ThawAccount<'view, P> {
    pub fn new(
        account: AccountView<'view>,
        mint: AccountView<'view>,
        authority: AccountView<'view>,
    ) -> Self {
        Self {
            account,
            mint,
            authority,
            __phantom: PhantomData,
        }
    }
}

/// The number of accounts required by the [`ThawAccount`] instruction.
pub const THAW_ACCOUNT_ACCOUNTS_BUFFER_LEN: usize = 3;
/// The length of the [`ThawAccount`] instruction data.
pub const THAW_ACCOUNT_DATA_LEN: usize = 1;
/// The [`ThawAccount`] instruction discriminator.
pub const THAW_ACCOUNT_DISCRIMINATOR: [u8; 1] = [11];

impl<'view, P: TokenProgram> CpiSerialize<'view> for ThawAccount<'view, P> {
    type Metas = [AccountMeta<'view>; THAW_ACCOUNT_ACCOUNTS_BUFFER_LEN];
    type CpiAccounts = [CpiAccount<'view>; THAW_ACCOUNT_ACCOUNTS_BUFFER_LEN];
    type Data = [u8; THAW_ACCOUNT_DATA_LEN];

    #[inline(always)]
    fn serialize(
        &self,
    ) -> Result<CpiCtx<'view, 'view, 'static, Self::Metas, Self::CpiAccounts, Self::Data>> {
        let data = THAW_ACCOUNT_DISCRIMINATOR;

        let metas = [
            AccountMeta::writable(self.account.address()),
            AccountMeta::readonly(self.mint.address()),
            AccountMeta::readonly_signer(self.authority.address()),
        ];

        let views = init_cpi_accounts!(self.account, self.mint, self.authority,);

        Ok(CpiCtx::new(&P::ID, metas, views, data))
    }
}
