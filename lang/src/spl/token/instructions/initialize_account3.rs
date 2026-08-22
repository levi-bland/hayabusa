// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use core::{mem::MaybeUninit, ptr::copy_nonoverlapping};

use crate::{init_cpi_accounts, prelude::*, spl::token::Token};

/// Like [`super::InitializeAccount`], but the owner pubkey is
/// passed via instruction data rather than the accounts list. This
/// variant may be preferable when using Cross Program Invocation from
/// an instruction that does not need the owner's `AccountInfo`
/// otherwise. Also does not require the Rent sysvar.
pub struct InitializeAccount3<'view, 'addr> {
    /// The account to initialize.
    pub account: AccountView<'view>,
    /// The mint the account will be associated with.
    pub mint: AccountView<'view>,
    /// The new account's owner or multisig.
    pub owner: &'addr Address,
}

/// The number of accounts required by the [`InitializeAccount3`] instruction.
pub const INITIALIZE_ACCOUNT3_ACCOUNTS_BUFFER_LEN: usize = 2;
/// The length of the [`InitializeAccount3`] instruction data.
pub const INITIALIZE_ACCOUNT3_DATA_LEN: usize = 33;
/// The [`InitializeAccount3`] instruction discriminator.
pub const INITIALIZE_ACCOUNT3_DISCRIMINATOR: [u8; 1] = [18];

type AccountMetaBuffer<'meta> = [AccountMeta<'meta>; INITIALIZE_ACCOUNT3_ACCOUNTS_BUFFER_LEN];
type CpiAccountBuffer<'view> = [CpiAccount<'view>; INITIALIZE_ACCOUNT3_ACCOUNTS_BUFFER_LEN];
type DataBuffer = [u8; INITIALIZE_ACCOUNT3_DATA_LEN];

impl<'view>
    SerializeCpiCtx<
        'view,
        AccountMetaBuffer<'view>,
        CpiAccountBuffer<'view>,
        DataBuffer,
    > for InitializeAccount3<'view, '_>
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
        // SAFETY: buffer is fully initialized before `assume_init`.
        let data = unsafe {
            let mut buf = MaybeUninit::<DataBuffer>::uninit();
            let ptr = buf.as_mut_ptr() as *mut u8;
            (ptr as *mut [u8; 1]).write(INITIALIZE_ACCOUNT3_DISCRIMINATOR);
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
        ];

        let views = init_cpi_accounts!(self.account, self.mint,);

        Ok(CpiCtx::new(&Token::ID, metas, views, data))
    }
}
