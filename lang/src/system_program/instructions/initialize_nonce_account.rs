// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use core::{
    mem::MaybeUninit,
    ptr::{addr_of, copy_nonoverlapping},
};

use crate::{init_cpi_accounts, prelude::*};

/// Drive state of Uninitialized nonce account to Initialized, setting the nonce
/// value.
///
/// The [`Address`] parameter specifies the entity authorized to execute nonce
/// instruction on the account
///
/// No signatures are required to execute this instruction, enabling derived
/// nonce account addresses.
pub struct InitializeNonceAccount<'view, 'id> {
    /// Nonce account.
    pub account: AccountView<'view>,
    /// Recent blockhash sysvar.
    pub recent_blockhash_sysvar: AccountView<'view>,
    /// Rent sysvar.
    pub rent_sysvar: AccountView<'view>,
    /// Indicates the entity authorized to execute nonce instruction on the account.
    pub authority: &'id Address,
}

/// The number of accounts expected by the `InitializeNonceAccount` system program instruction.
pub const INITIALIZE_NONCE_ACCOUNT_ACCOUNTS_BUFFER_LEN: usize = 3;
/// The length of the instruction data expected by the `InitializeNonceAccount` system program instruction.
pub const INITIALIZE_NONCE_ACCOUNT_DATA_LEN: usize = 36;
/// The discriminator of the `InitializeNonceAccount` system program instruction.
pub const INITIALIZE_NONCE_ACCOUNT_DISCRIMINATOR: [u8; 4] = [6, 0, 0, 0];

type AccountMetaBuffer<'meta> = [AccountMeta<'meta>; INITIALIZE_NONCE_ACCOUNT_ACCOUNTS_BUFFER_LEN];
type CpiAccountBuffer<'view> = [CpiAccount<'view>; INITIALIZE_NONCE_ACCOUNT_ACCOUNTS_BUFFER_LEN];
type DataBuffer = [u8; INITIALIZE_NONCE_ACCOUNT_DATA_LEN];

impl<'view>
    SerializeCpiCtx<
        'view,
        AccountMetaBuffer<'view>,
        CpiAccountBuffer<'view>,
        DataBuffer,
    > for InitializeNonceAccount<'view, '_>
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
        // instruction data
        // - [0..4 ]: instruction discriminator
        // - [4..36]: authority address
        //
        // SAFETY: all bytes written before `assume_init`.
        let data = unsafe {
            let mut buf = MaybeUninit::<DataBuffer>::uninit();
            let ptr = buf.as_mut_ptr() as *mut u8;
            (ptr as *mut [u8; 4]).write_unaligned(INITIALIZE_NONCE_ACCOUNT_DISCRIMINATOR);
            copy_nonoverlapping(
                addr_of!(self.authority) as *const u8,
                ptr.add(4),
                size_of::<Address>(),
            );

            buf.assume_init()
        };

        let metas = [
            AccountMeta::writable(self.account.address()),
            AccountMeta::readonly(self.recent_blockhash_sysvar.address()),
            AccountMeta::readonly(self.rent_sysvar.address()),
        ];

        let views =
            init_cpi_accounts!(self.account, self.recent_blockhash_sysvar, self.rent_sysvar,);

        Ok(CpiCtx::new(&System::ID, metas, views, data))
    }
}
