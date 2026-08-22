// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

use core::{mem::MaybeUninit, ptr::copy_nonoverlapping};

use crate::{init_cpi_accounts, prelude::*};

/// Change the entity authorized to execute nonce instructions on the account.
pub struct AuthorizeNonceAccount<'view, 'auth> {
    /// Nonce account.
    pub account: AccountView<'view>,
    /// Nonce authority.
    pub authority: AccountView<'view>,
    /// New address authorized to execute nonce instructions on the account.
    pub new_authority: &'auth Address,
}

/// The number of accounts expected by the `AuthorizeNonceAccount` system program instruction.
pub const AUTHORIZE_NONCE_ACCOUNT_ACCOUNTS_BUFFER_LEN: usize = 2;
/// The `AuthorizeNonceAccount` system program instruction data length.
pub const AUTHORIZE_NONCE_ACCOUNT_DATA_LEN: usize = 36;
/// The `AuthorizeNonceAccount` system program instruction discriminator.
pub const AUTHORIZE_NONCE_ACCOUNT_DISCRIMINATOR: [u8; 4] = [7, 0, 0, 0];

type AccountMetaBuffer<'meta> = [AccountMeta<'meta>; AUTHORIZE_NONCE_ACCOUNT_ACCOUNTS_BUFFER_LEN];
type CpiAccountBuffer<'view> = [CpiAccount<'view>; AUTHORIZE_NONCE_ACCOUNT_ACCOUNTS_BUFFER_LEN];
type DataBuffer = [u8; AUTHORIZE_NONCE_ACCOUNT_DATA_LEN];

impl<'view>
    SerializeCpiCtx<
        'view,
        AccountMetaBuffer<'view>,
        CpiAccountBuffer<'view>,
        DataBuffer,
    > for AuthorizeNonceAccount<'view, '_>
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
        // -  [0..4 ]: instruction discriminator
        // -  [4..36]: new authority
        //
        // SAFETY: all 36 bytes are initialized before `assume_init`.
        let data = unsafe {
            let mut buf = MaybeUninit::<DataBuffer>::uninit();
            let ptr = buf.as_mut_ptr() as *mut u8;
            (ptr as *mut [u8; 4]).write_unaligned(AUTHORIZE_NONCE_ACCOUNT_DISCRIMINATOR);
            copy_nonoverlapping(self.new_authority.as_ref().as_ptr(), ptr.add(4), 32);

            buf.assume_init()
        };

        let metas = [
            AccountMeta::writable(self.account.address()),
            AccountMeta::readonly_signer(self.authority.address()),
        ];

        let views = init_cpi_accounts!(self.account, self.authority,);

        Ok(CpiCtx::new(&System::ID, metas, views, data))
    }
}
