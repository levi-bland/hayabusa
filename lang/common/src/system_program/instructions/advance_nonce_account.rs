// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_meta::AccountMeta,
    account_view::AccountView,
    cpi::{CpiAccount, CpiCtx},
    init_cpi_accounts,
    system_program::System,
    traits::{CpiSerialize, Id},
    Result,
};
use hayabusa_invoke_cpi_macros::CpiInstruction;

/// Consumes a stored nonce, replacing it with a successor.
#[derive(CpiInstruction)]
pub struct AdvanceNonceAccount<'view> {
    /// Nonce account
    pub account: AccountView<'view>,
    /// Recent blockhash sysvar
    pub recent_blockhash_sysvar: AccountView<'view>,
    /// Nonce authority
    pub authority: AccountView<'view>,
}

/// The number of accounts required by the [`AdvanceNonceAccount`] system program instruction
pub const ADVANCE_NONCE_ACCOUNT_ACCOUNTS_BUFFER_LEN: usize = 3;
/// The length of the [`AdvanceNonceAccount`] instruction data
pub const ADVANCE_NONCE_ACCOUNT_DATA_LEN: usize = 1;
/// The [`AdvanceNonceAccount`] system program instruction discriminator
pub const ADVANCE_NONCE_ACCOUNT_DISCRIMINATOR: [u8; 1] = [4];

type AccountMetaBuffer<'meta> = [AccountMeta<'meta>; ADVANCE_NONCE_ACCOUNT_ACCOUNTS_BUFFER_LEN];
type CpiAccountBuffer<'view> = [CpiAccount<'view>; ADVANCE_NONCE_ACCOUNT_ACCOUNTS_BUFFER_LEN];
type DataBuffer = [u8; ADVANCE_NONCE_ACCOUNT_DATA_LEN];

impl<'view> CpiSerialize<'view, AccountMetaBuffer<'view>, CpiAccountBuffer<'view>, DataBuffer>
    for AdvanceNonceAccount<'view>
{
    #[inline(always)]
    fn serialize(
        &self,
    ) -> Result<
        CpiCtx<
            'view,
            'view,
            'static,
            AccountMetaBuffer<'view>,
            CpiAccountBuffer<'view>,
            DataBuffer,
        >,
    > {
        let metas = [
            AccountMeta::writable(self.account.address()),
            AccountMeta::readonly(self.recent_blockhash_sysvar.address()),
            AccountMeta::readonly_signer(self.authority.address()),
        ];

        let views = init_cpi_accounts!(self.account, self.recent_blockhash_sysvar, self.authority);

        Ok(CpiCtx::new(
            &System::ID,
            metas,
            views,
            ADVANCE_NONCE_ACCOUNT_DISCRIMINATOR,
        ))
    }
}
