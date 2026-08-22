// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

use crate::{init_cpi_accounts, prelude::*};
use core::{mem::MaybeUninit, ptr::copy_nonoverlapping};

/// Parameters required by the `CreateAccount` system program instruction
pub struct CreateAccount<'view, 'id> {
    /// Funding account
    pub from: AccountView<'view>,
    /// New account
    pub to: AccountView<'view>,
    /// The address of the program that will own the account
    pub owner_program: &'id Address,
    /// The space to allocate for the account in bytes
    pub space: u64,
}

/// The length of the instruction data for the `CreateAccount` system program instruction
pub const CREATE_ACCOUNT_DATA_LEN: usize = 52;
/// The number of accounts the `CreateAccount` system program instruction requires
pub const CREATE_ACCOUNT_ACCOUNTS_BUFFER_LEN: usize = 2;
/// The discriminator of the `CreateAccount` system program instruction
pub const CREATE_ACCOUNT_DISCRIMINATOR: [u8; 4] = [0; 4];

type AccountMetaBuffer<'meta> = [AccountMeta<'meta>; CREATE_ACCOUNT_ACCOUNTS_BUFFER_LEN];
type CpiAccountBuffer<'view> = [CpiAccount<'view>; CREATE_ACCOUNT_ACCOUNTS_BUFFER_LEN];
type DataBuffer = [u8; CREATE_ACCOUNT_DATA_LEN];

impl<'view>
    SerializeCpiCtx<
        'view,
        AccountMetaBuffer<'view>,
        CpiAccountBuffer<'view>,
        DataBuffer,
    > for CreateAccount<'view, '_>
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
        let lamports = system_program::minimum_balance(self.space as usize)?;

        // ix data layout
        // - [0..4] discriminator
        // - [4..12] lamports
        // - [12..20] account space
        // - [20..52] owning program's address
        //
        // SAFETY: all 52 bytes are initialized before `assume_init`,
        //         `Address` is guaranteed to be 32 bytes.
        let data = unsafe {
            let mut buf = MaybeUninit::<DataBuffer>::uninit();
            let ptr = buf.as_mut_ptr() as *mut u8;
            (ptr as *mut [u8; 4]).write_unaligned(CREATE_ACCOUNT_DISCRIMINATOR);
            (ptr.add(4) as *mut u64).write_unaligned(lamports);
            (ptr.add(12) as *mut u64).write_unaligned(self.space);
            copy_nonoverlapping(self.owner_program.as_ref().as_ptr(), ptr.add(20), 32);

            buf.assume_init()
        };

        let views = init_cpi_accounts!(self.from, self.to,);

        // SAFETY: buffer is fully initialized before `assume_init`.
        let metas = unsafe {
            let mut buf = MaybeUninit::<AccountMetaBuffer>::uninit();
            let ptr = buf.as_mut_ptr() as *mut AccountMeta;
            ptr.write(AccountMeta::writable_signer(self.from.address()));
            ptr.add(1)
                .write(AccountMeta::writable_signer(self.to.address()));

            buf.assume_init()
        };

        Ok(CpiCtx::new(&System::ID, metas, views, data))
    }
}
