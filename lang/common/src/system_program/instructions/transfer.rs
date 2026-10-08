// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use core::mem::MaybeUninit;

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

#[derive(CpiInstruction)]
pub struct Transfer<'view> {
    /// Funding account
    pub from: AccountView<'view>,
    /// Recipient account
    pub to: AccountView<'view>,
    /// The amount of lamports to transfer
    pub lamports: u64,
}

/// The number of accounts required by the `Transfer` system program instruction
pub const TRANSFER_ACCOUNTS_BUFFER_LEN: usize = 2;
/// The length of the `Transfer` instruction data
pub const TRANSFER_DATA_LEN: usize = 12;
/// The `Transfer` instruction discriminator
pub const TRANSFER_DISCRIMINATOR: [u8; 4] = [2, 0, 0, 0];

type AccountMetaBuffer<'meta> = [AccountMeta<'meta>; TRANSFER_ACCOUNTS_BUFFER_LEN];
type CpiAccountBuffer<'view> = [CpiAccount<'view>; TRANSFER_ACCOUNTS_BUFFER_LEN];
type DataBuffer = [u8; TRANSFER_DATA_LEN];

impl<'view> CpiSerialize<'view, AccountMetaBuffer<'view>, CpiAccountBuffer<'view>, DataBuffer>
    for Transfer<'view>
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
        // ix data
        // - [0..4]: discriminator
        // - [4..12]: lamports amount
        //
        // SAFETY: all 12 bytes are initialized before calling `assume_init`
        let data = unsafe {
            let mut buf = MaybeUninit::<DataBuffer>::uninit();
            let ptr = buf.as_mut_ptr() as *mut u8;
            (ptr as *mut [u8; 4]).write_unaligned(TRANSFER_DISCRIMINATOR);
            (ptr.add(4) as *mut u64).write_unaligned(self.lamports);

            buf.assume_init()
        };

        let metas = [
            AccountMeta::writable_signer(self.from.address()),
            AccountMeta::writable(self.to.address()),
        ];

        let views = init_cpi_accounts!(self.from, self.to,);

        Ok(CpiCtx::new(&System::ID, metas, views, data))
    }
}
