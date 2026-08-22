// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use crate::{init_cpi_accounts, prelude::*};

pub struct Allocate<'view> {
    /// Account to be allocated
    pub account: AccountView<'view>,
    /// The space to allocate
    pub space: u64,
}

/// The number of accounts expected by the [`Allocate`] system program instruction.
pub const ALLOCATE_ACCOUNT_ACCOUNTS_BUFFER_LEN: usize = 1;
/// The length of the [`Allocate`] system program instruction data.
pub const ALLOCATE_DATA_LEN: usize = 12;
/// The [`Allocate`] system program instruction discriminator.
pub const ALLOCATE_DISCRIMINATOR: [u8; 4] = [8, 0, 0, 0];

type AccountMetaBuffer<'meta> = [AccountMeta<'meta>; ALLOCATE_ACCOUNT_ACCOUNTS_BUFFER_LEN];
type CpiAccountBuffer<'view> = [CpiAccount<'view>; ALLOCATE_ACCOUNT_ACCOUNTS_BUFFER_LEN];
type DataBuffer = [u8; ALLOCATE_DATA_LEN];

impl<'view>
    SerializeCpiCtx<
        'view,
        AccountMetaBuffer<'view>,
        CpiAccountBuffer<'view>,
        DataBuffer,
    > for Allocate<'view>
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
        // - [0..4] discriminator
        // - [4..12] space
        //
        // SAFETY: all 12 bytes are written before `assume_init`.
        let data = unsafe {
            let mut buf = core::mem::MaybeUninit::<DataBuffer>::uninit();
            let ptr = buf.as_mut_ptr() as *mut u8;
            (ptr as *mut [u8; 4]).write_unaligned(ALLOCATE_DISCRIMINATOR);
            (ptr.add(4) as *mut u64).write_unaligned(self.space);
            buf.assume_init()
        };

        let metas = [AccountMeta::writable_signer(self.account.address())];
        let views = init_cpi_accounts!(self.account);

        Ok(CpiCtx::new(&System::ID, metas, views, data))
    }
}
