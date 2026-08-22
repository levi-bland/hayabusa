// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

use core::mem::MaybeUninit;

use crate::{init_cpi_accounts, prelude::*, spl::token::Token};

/// Transfer tokens from one token account to another.
pub struct Transfer<'view> {
    /// Sender account.
    pub from: AccountView<'view>,
    /// Recipient account.
    pub to: AccountView<'view>,
    /// Authority account.
    pub authority: AccountView<'view>,
    /// Amount of micro-tokens to transfer.
    pub amount: u64,
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

impl<'view>
    SerializeCpiCtx<
        'view,
        AccountMetaBuffer<'view>,
        CpiAccountBuffer<'view>,
        DataBuffer,
    > for Transfer<'view>
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

        Ok(CpiCtx::new(&Token::ID, metas, views, data))
    }
}
