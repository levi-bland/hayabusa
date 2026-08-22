// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

use core::{mem::MaybeUninit, ptr::copy_nonoverlapping};

use crate::{init_cpi_accounts, prelude::*};

pub struct Assign<'view, 'id> {
    /// Account to be assigned to a program
    pub account: AccountView<'view>,
    /// The address/id of the program that will own the account
    pub owner: &'id Address,
}

/// The number of accounts required by the `Assign` system program instruction
pub const ASSIGN_ACCOUNT_ACCOUNTS_BUFFER_LEN: usize = 1;
/// The length of the `Assign` instruction data
pub const ASSIGN_DATA_LEN: usize = 36;
/// The `Assign` instruction discriminator
pub const ASSIGN_DISCRIMINATOR: [u8; 4] = [1, 0, 0, 0];

type AccountMetaBuffer<'meta> = [AccountMeta<'meta>; ASSIGN_ACCOUNT_ACCOUNTS_BUFFER_LEN];
type CpiAccountBuffer<'view> = [CpiAccount<'view>; ASSIGN_ACCOUNT_ACCOUNTS_BUFFER_LEN];
type DataBuffer = [u8; ASSIGN_DATA_LEN];

impl<'view>
    SerializeCpiCtx<
        'view,
        AccountMetaBuffer<'view>,
        CpiAccountBuffer<'view>,
        DataBuffer,
    > for Assign<'view, '_>
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
        // - [4..36]: owner pubkey
        //
        // SAFETY: all 36 bytes are initialized before `assume_init` is called.
        let data = unsafe {
            let mut buf = MaybeUninit::<DataBuffer>::uninit();
            let ptr = buf.as_mut_ptr() as *mut u8;
            (ptr as *mut [u8; 4]).write_unaligned(ASSIGN_DISCRIMINATOR);
            copy_nonoverlapping(
                self.owner.as_ref().as_ptr(),
                ptr.add(4),
                size_of::<Address>(),
            );

            buf.assume_init()
        };

        let metas = [AccountMeta::writable_signer(self.account.address())];
        let views = init_cpi_accounts!(self.account);

        Ok(CpiCtx::new(&System::ID, metas, views, data))
    }
}
