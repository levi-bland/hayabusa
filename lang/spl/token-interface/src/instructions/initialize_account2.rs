// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use core::{mem::MaybeUninit, ptr::copy_nonoverlapping};

use hayabusa_common::{
    Result,
    account_meta::AccountMeta,
    account_view::AccountView,
    address::Address,
    cpi::{CpiAccount, CpiCtx},
    init_cpi_accounts,
    traits::CpiSerialize,
};
use hayabusa_invoke_cpi_macros::CpiInstruction;

/// Like [`super::InitializeAccount`], but the owner pubkey is passed
/// via instruction data rather than the accounts list. Unlike
/// [`super::InitializeAccount3`], this variant still requires the Rent
/// sysvar to be provided.
#[derive(CpiInstruction)]
pub struct InitializeAccount2<'view, 'addr> {
    /// The account to initialize.
    pub account: AccountView<'view>,
    /// The mint the account will be associated with.
    pub mint: AccountView<'view>,
    /// The Rent sysvar.
    pub rent_sysvar: AccountView<'view>,
    /// The token program to use for the CPI.
    pub token_program: AccountView<'view>,
    /// The new account's owner or multisig.
    pub owner: &'addr Address,
}

/// The number of accounts required by the [`InitializeAccount2`] instruction.
pub const INITIALIZE_ACCOUNT2_ACCOUNTS_BUFFER_LEN: usize = 3;
/// The length of the [`InitializeAccount2`] instruction data.
pub const INITIALIZE_ACCOUNT2_DATA_LEN: usize = 33;
/// The [`InitializeAccount2`] instruction discriminator.
pub const INITIALIZE_ACCOUNT2_DISCRIMINATOR: [u8; 1] = [16];

impl<'view> CpiSerialize<'view> for InitializeAccount2<'view, '_> {
    type Metas = [AccountMeta<'view>; INITIALIZE_ACCOUNT2_ACCOUNTS_BUFFER_LEN];
    type CpiAccounts = [CpiAccount<'view>; INITIALIZE_ACCOUNT2_ACCOUNTS_BUFFER_LEN];
    type Data = [u8; INITIALIZE_ACCOUNT2_DATA_LEN];

    #[inline(always)]
    fn serialize(
        &self,
    ) -> Result<CpiCtx<'view, 'view, 'view, Self::Metas, Self::CpiAccounts, Self::Data>> {
        // SAFETY: buffer is fully initialized before `assume_init`.
        let data = unsafe {
            let mut buf = MaybeUninit::<Self::Data>::uninit();
            let ptr = buf.as_mut_ptr() as *mut u8;
            (ptr as *mut [u8; 1]).write(INITIALIZE_ACCOUNT2_DISCRIMINATOR);
            copy_nonoverlapping(
                self.owner.as_ref().as_ptr(),
                ptr.add(1),
                size_of::<Address>(),
            );

            buf.assume_init()
        };

        let metas = [
            AccountMeta::writable(self.account.address()),
            AccountMeta::readonly(self.mint.address()),
            AccountMeta::readonly(self.rent_sysvar.address()),
        ];

        let views = init_cpi_accounts!(self.account, self.mint, self.rent_sysvar,);

        Ok(CpiCtx::new(
            self.token_program.address(),
            metas,
            views,
            data,
        ))
    }
}
