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

/// Given a wrapped / native token account (a token account holding
/// SOL), updates its `amount` field based on the account's underlying
/// lamports. Useful after moving lamports into a wrapped SOL account
/// with a system transfer.
#[derive(CpiInstruction)]
pub struct SyncNative<'view, P: TokenProgram> {
    /// The native token account to sync with its underlying lamports.
    pub account: AccountView<'view>,
    /// Signifies which token program to use for the CPI.
    __phantom: PhantomData<P>,
}

impl<'view, P: TokenProgram> SyncNative<'view, P> {
    pub fn new(account: AccountView<'view>) -> Self {
        Self {
            account,
            __phantom: PhantomData,
        }
    }
}

/// The number of accounts required by the [`SyncNative`] instruction.
pub const SYNC_NATIVE_ACCOUNTS_BUFFER_LEN: usize = 1;
/// The length of the [`SyncNative`] instruction data.
pub const SYNC_NATIVE_DATA_LEN: usize = 1;
/// The [`SyncNative`] instruction discriminator.
pub const SYNC_NATIVE_DISCRIMINATOR: [u8; 1] = [17];

type AccountMetaBuffer<'meta> = [AccountMeta<'meta>; SYNC_NATIVE_ACCOUNTS_BUFFER_LEN];
type CpiAccountBuffer<'view> = [CpiAccount<'view>; SYNC_NATIVE_ACCOUNTS_BUFFER_LEN];
type DataBuffer = [u8; SYNC_NATIVE_DATA_LEN];

impl<'view, P: TokenProgram>
    CpiSerialize<'view, AccountMetaBuffer<'view>, CpiAccountBuffer<'view>, DataBuffer>
    for SyncNative<'view, P>
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
        let data = SYNC_NATIVE_DISCRIMINATOR;

        let metas = [AccountMeta::writable(self.account.address())];

        let views = init_cpi_accounts!(self.account,);

        Ok(CpiCtx::new(&P::ID, metas, views, data))
    }
}
