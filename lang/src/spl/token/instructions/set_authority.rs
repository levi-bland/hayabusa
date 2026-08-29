// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use core::{
    mem::MaybeUninit,
    ptr::{copy_nonoverlapping, write_bytes},
};

use crate::{init_cpi_accounts, prelude::*, spl::token::Token};

/// The type of authority being updated by [`SetAuthority`].
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthorityType {
    /// Authority to mint new tokens.
    MintTokens = 0,
    /// Authority to freeze any account associated with the mint.
    FreezeAccount = 1,
    /// Owner of a given token account.
    AccountOwner = 2,
    /// Authority to close a token account.
    CloseAccount = 3,
}

/// Sets a new authority of a mint or account.
///
/// The new authority is encoded as a `COption<Address>`: a one byte tag
/// followed by the address. The tag is zero when `new_authority` is
/// [`None`], in which case the trailing 32 bytes are zeroed and ignored
/// by the program (it stops reading after the tag).
pub struct SetAuthority<'view, 'addr> {
    /// The mint or account to change the authority of.
    pub account: AccountView<'view>,
    /// The current authority of the mint or account.
    pub authority: AccountView<'view>,
    /// The type of authority to update.
    pub authority_type: AuthorityType,
    /// The new authority, or [`None`] to clear it.
    pub new_authority: Option<&'addr Address>,
}

/// The number of accounts required by the [`SetAuthority`] instruction.
pub const SET_AUTHORITY_ACCOUNTS_BUFFER_LEN: usize = 2;
/// The length of the [`SetAuthority`] instruction data.
pub const SET_AUTHORITY_DATA_LEN: usize = 35;
/// The [`SetAuthority`] instruction discriminator.
pub const SET_AUTHORITY_DISCRIMINATOR: [u8; 1] = [6];

type AccountMetaBuffer<'meta> = [AccountMeta<'meta>; SET_AUTHORITY_ACCOUNTS_BUFFER_LEN];
type CpiAccountBuffer<'view> = [CpiAccount<'view>; SET_AUTHORITY_ACCOUNTS_BUFFER_LEN];
type DataBuffer = [u8; SET_AUTHORITY_DATA_LEN];

impl<'view>
    SerializeCpiCtx<
        'view,
        AccountMetaBuffer<'view>,
        CpiAccountBuffer<'view>,
        DataBuffer,
    > for SetAuthority<'view, '_>
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
        // SAFETY: buffer is fully initialized before `assume_init`; both
        // branches write the option tag and all 32 trailing bytes.
        let data = unsafe {
            let mut buf = MaybeUninit::<DataBuffer>::uninit();
            let ptr = buf.as_mut_ptr() as *mut u8;
            (ptr as *mut [u8; 1]).write(SET_AUTHORITY_DISCRIMINATOR);
            ptr.add(1).write(self.authority_type as u8);

            match self.new_authority {
                Some(new_authority) => {
                    ptr.add(2).write(1);
                    copy_nonoverlapping(
                        new_authority.as_ref().as_ptr(),
                        ptr.add(3),
                        size_of::<Address>(),
                    );
                }
                None => {
                    ptr.add(2).write(0);
                    write_bytes(ptr.add(3), 0, size_of::<Address>());
                }
            }

            buf.assume_init()
        };

        let metas = [
            AccountMeta::writable(self.account.address()),
            AccountMeta::readonly_signer(self.authority.address()),
        ];

        let views = init_cpi_accounts!(self.account, self.authority,);

        Ok(CpiCtx::new(&Token::ID, metas, views, data))
    }
}