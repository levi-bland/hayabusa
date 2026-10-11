// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use core::mem::MaybeUninit;

use hayabusa_common::{
    Result,
    account_meta::AccountMeta,
    account_view::AccountView,
    cpi::{CpiAccount, CpiCtx},
    init_cpi_accounts,
    traits::CpiSerialize,
};
use hayabusa_invoke_cpi_macros::CpiInstruction;

/// Transfer Tokens from one token account to another, checking the mint.
#[derive(CpiInstruction)]
pub struct TransferChecked<'view> {
    /// Sender account.
    pub from: AccountView<'view>,
    /// Mint account.
    pub mint: AccountView<'view>,
    /// Recipient account.
    pub to: AccountView<'view>,
    /// Authority account.
    pub authority: AccountView<'view>,
    /// The token program to use for the CPI.
    pub token_program: AccountView<'view>,
    /// Amount of tokens to transfer.
    pub amount: u64,
    /// Token decimals.
    pub decimals: u8,
}

/// The number of accounts required by the [`TransferChecked`] instruction.
pub const TRANSFER_CHECKED_ACCOUNTS_BUFFER_LEN: usize = 4;
/// The length of the [`TransferChecked`] instruction data.
pub const TRANSFER_CHECKED_DATA_LEN: usize = 10;
/// The [`TransferChecked`] instruction discriminator.
pub const TRANSFER_CHECKED_DISCRIMINATOR: [u8; 1] = [12];

type AccountMetaBuffer<'meta> = [AccountMeta<'meta>; TRANSFER_CHECKED_ACCOUNTS_BUFFER_LEN];
type CpiAccountBuffer<'view> = [CpiAccount<'view>; TRANSFER_CHECKED_ACCOUNTS_BUFFER_LEN];
type DataBuffer = [u8; TRANSFER_CHECKED_DATA_LEN];

impl<'view> CpiSerialize<'view, AccountMetaBuffer<'view>, CpiAccountBuffer<'view>, DataBuffer>
    for TransferChecked<'view>
{
    #[inline(always)]
    fn serialize(
        &self,
    ) -> Result<
        CpiCtx<'view, 'view, 'view, AccountMetaBuffer<'view>, CpiAccountBuffer<'view>, DataBuffer>,
    > {
        // SAFETY: buffer fully initialized before `assume_init`.
        let data = unsafe {
            let mut buf = MaybeUninit::<DataBuffer>::uninit();
            let ptr = buf.as_mut_ptr() as *mut u8;
            (ptr as *mut [u8; 1]).write(TRANSFER_CHECKED_DISCRIMINATOR);
            (ptr.add(1) as *mut [u8; 8]).write(self.amount.to_le_bytes());
            ptr.add(9).write(self.decimals);

            buf.assume_init()
        };

        let metas = [
            AccountMeta::writable(self.from.address()),
            AccountMeta::readonly(self.mint.address()),
            AccountMeta::writable(self.to.address()),
            AccountMeta::readonly_signer(self.authority.address()),
        ];

        let views = init_cpi_accounts!(self.from, self.mint, self.to, self.authority,);

        Ok(CpiCtx::new(
            self.token_program.address(),
            metas,
            views,
            data,
        ))
    }
}
