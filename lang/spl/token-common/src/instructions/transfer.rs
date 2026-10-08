// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use core::mem::MaybeUninit;

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

/// Transfer tokens from one token account to another.
#[derive(CpiInstruction)]
pub struct Transfer<'view, P: TokenProgram> {
    /// Sender account.
    pub from: AccountView<'view>,
    /// Recipient account.
    pub to: AccountView<'view>,
    /// Authority account.
    pub authority: AccountView<'view>,
    /// Amount of micro-tokens to transfer.
    pub amount: u64,
    /// Signifies which token program to use for the CPI.
    __phantom: PhantomData<P>,
}

impl<'view, P: TokenProgram> Transfer<'view, P> {
    pub fn new(
        from: AccountView<'view>,
        to: AccountView<'view>,
        authority: AccountView<'view>,
        amount: u64,
    ) -> Self {
        Self {
            from,
            to,
            authority,
            amount,
            __phantom: PhantomData,
        }
    }
}

/// The number of accounts required by the [`Transfer`] instruction.
pub const TRANSFER_ACCOUNTS_BUFFER_LEN: usize = 3;
/// The length of the [`Transfer`] instruction data.
pub const TRANSFER_DATA_LEN: usize = 9;
/// The [`Transfer`] instruction discriminator.
pub const TRANSFER_DISCRIMINATOR: [u8; 1] = [3];

type AccountMetaBuffer<'meta> = [AccountMeta<'meta>; TRANSFER_ACCOUNTS_BUFFER_LEN];
type CpiAccountBuffer<'view> = [CpiAccount<'view>; TRANSFER_ACCOUNTS_BUFFER_LEN];
type DataBuffer = [u8; TRANSFER_DATA_LEN];

impl<'view, P: TokenProgram>
    CpiSerialize<'view, AccountMetaBuffer<'view>, CpiAccountBuffer<'view>, DataBuffer>
    for Transfer<'view, P>
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
        // SAFETY: all bytes written before `assume_init`.
        let data = unsafe {
            let mut buf = MaybeUninit::<DataBuffer>::uninit();
            let ptr = buf.as_mut_ptr() as *mut u8;
            ptr.write(TRANSFER_DISCRIMINATOR[0]);
            (ptr.add(1) as *mut [u8; 8]).write(self.amount.to_le_bytes());

            buf.assume_init()
        };

        let metas = [
            AccountMeta::writable(self.from.address()),
            AccountMeta::writable(self.to.address()),
            AccountMeta::readonly_signer(self.authority.address()),
        ];

        let views = init_cpi_accounts!(self.from, self.to, self.authority);

        Ok(CpiCtx::new(&P::ID, metas, views, data))
    }
}
