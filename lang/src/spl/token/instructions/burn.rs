// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use core::mem::MaybeUninit;

use crate::{init_cpi_accounts, prelude::*, spl::token::Token};

/// Mints new tokens to an account.
pub struct Burn<'view> {
    /// Token account being burned from.
    pub account: AccountView<'view>,
    /// Mint account.
    pub mint: AccountView<'view>,
    /// Authority of the token account.
    pub authority: AccountView<'view>,
    /// Amount of tokens to mint.
    pub amount: u64,
}

/// The number of accounts required by the [`Burn`] instruction.
pub const BURN_ACCOUNTS_BUFFER_LEN: usize = 3;
/// The length of the [`Burn`] instruction data.
pub const BURN_DATA_LEN: usize = 9;
/// The [`Burn`] instruction discriminator.
pub const BURN_DISCRIMINATOR: [u8; 1] = [7];

type AccountMetaBuffer<'meta> = [AccountMeta<'meta>; BURN_ACCOUNTS_BUFFER_LEN];
type CpiAccountBuffer<'view> = [CpiAccount<'view>; BURN_ACCOUNTS_BUFFER_LEN];
type DataBuffer = [u8; BURN_DATA_LEN];

impl<'view>
    SerializeCpiCtx<
        'view,
        AccountMetaBuffer<'view>,
        CpiAccountBuffer<'view>,
        DataBuffer,
    > for Burn<'view>
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
            (ptr as *mut [u8; 1]).write(BURN_DISCRIMINATOR);
            (ptr.add(1) as *mut [u8; 8]).write(self.amount.to_le_bytes());

            buf.assume_init()
        };

        let metas = [
            AccountMeta::writable(self.account.address()),
            AccountMeta::writable(self.mint.address()),
            AccountMeta::readonly_signer(self.authority.address()),
        ];

        let views = init_cpi_accounts!(self.account, self.mint, self.authority,);

        Ok(CpiCtx::new(&Token::ID, metas, views, data))
    }
}
