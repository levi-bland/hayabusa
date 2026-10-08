// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use core::marker::PhantomData;

use crate::TokenProgram;
use hayabusa_common::{
    Result,
    account_meta::AccountMeta,
    account_view::AccountView,
    cpi::{CpiAccount, CpiCtx},
    init_cpi_accounts,
    traits::CpiSerialize,
};
use hayabusa_invoke_cpi_macros::CpiInstruction;

#[derive(CpiInstruction)]
pub struct InitializeAccount<'view, P: TokenProgram> {
    /// The token account being initialized.
    pub account: AccountView<'view>,
    /// The mint this token account will be associated with.
    pub mint: AccountView<'view>,
    /// The new account's owner.
    pub owner: AccountView<'view>,
    /// Rent sysvar.
    pub rent: AccountView<'view>,
    /// Signifies which token program to use for the CPI.
    __phantom: PhantomData<P>,
}

impl<'view, P: TokenProgram> InitializeAccount<'view, P> {
    pub fn new(
        account: AccountView<'view>,
        mint: AccountView<'view>,
        owner: AccountView<'view>,
        rent: AccountView<'view>,
    ) -> Self {
        Self {
            account,
            mint,
            owner,
            rent,
            __phantom: PhantomData,
        }
    }
}

/// The number of accounts required by the [`InitializeAccount`] instruction.
pub const INITIALIZE_ACCOUNT_ACCOUNTS_BUFFER_LEN: usize = 4;
/// The length of the [`InitializeAccount`] instruction data.
pub const INITIALIZE_ACCOUNT_DATA_LEN: usize = 1;
/// The [`InitializeAccount`] instruction discriminator.
pub const INITIALIZE_ACCOUNT_DISCRIMINATOR: [u8; 1] = [1];

type AccountMetaBuffer<'meta> = [AccountMeta<'meta>; INITIALIZE_ACCOUNT_ACCOUNTS_BUFFER_LEN];
type CpiAccountBuffer<'view> = [CpiAccount<'view>; INITIALIZE_ACCOUNT_ACCOUNTS_BUFFER_LEN];
type DataBuffer = [u8; INITIALIZE_ACCOUNT_DATA_LEN];

impl<'view, P: TokenProgram>
    CpiSerialize<'view, AccountMetaBuffer<'view>, CpiAccountBuffer<'view>, DataBuffer>
    for InitializeAccount<'view, P>
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
            AccountMeta::readonly(self.mint.address()),
            AccountMeta::readonly(self.owner.address()),
            AccountMeta::readonly(self.rent.address()),
        ];

        let views = init_cpi_accounts!(self.account, self.mint, self.owner, self.rent,);

        Ok(CpiCtx::new(
            &P::ID,
            metas,
            views,
            INITIALIZE_ACCOUNT_DISCRIMINATOR,
        ))
    }
}
