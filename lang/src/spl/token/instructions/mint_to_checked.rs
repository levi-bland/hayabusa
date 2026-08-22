// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

use core::mem::MaybeUninit;

use crate::{init_cpi_accounts, prelude::*, spl::token::Token};

/// Mints new tokens to an account, checking token decimals.
pub struct MintToChecked<'view> {
    /// Mint account.
    pub mint: AccountView<'view>,
    /// Token account.
    pub account: AccountView<'view>,
    /// Mint authority.
    pub mint_authority: AccountView<'view>,
    /// Amount of tokens to mint.
    pub amount: u64,
    /// Token decimals.
    pub decimals: u8,
}

/// The number of accounts required by the [`MintToChecked`] instruction.
pub const MINT_TO_CHECKED_ACCOUNTS_BUFFER_LEN: usize = 3;
/// The length of the [`MintToChecked`] instruction data.
pub const MINT_TO_CHECKED_DATA_LEN: usize = 10;
/// The [`MintToChecked`] instruction discriminator.
pub const MINT_TO_CHECKED_DISCRIMINATOR: [u8; 1] = [14];

type AccountMetaBuffer<'meta> = [AccountMeta<'meta>; MINT_TO_CHECKED_ACCOUNTS_BUFFER_LEN];
type CpiAccountBuffer<'view> = [CpiAccount<'view>; MINT_TO_CHECKED_ACCOUNTS_BUFFER_LEN];
type DataBuffer = [u8; MINT_TO_CHECKED_DATA_LEN];

impl<'view>
    SerializeCpiCtx<
        'view,
        AccountMetaBuffer<'view>,
        CpiAccountBuffer<'view>,
        DataBuffer,
    > for MintToChecked<'view>
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
        // SAFETY: buffer fully initialized before `assume_init`.
        let data = unsafe {
            let mut buf = MaybeUninit::<DataBuffer>::uninit();
            let ptr = buf.as_mut_ptr() as *mut u8;
            ptr.write(MINT_TO_CHECKED_DISCRIMINATOR[0]);
            (ptr.add(1) as *mut [u8; 8]).write(self.amount.to_le_bytes());
            ptr.add(9).write(self.decimals);

            buf.assume_init()
        };

        let metas = [
            AccountMeta::writable(self.mint.address()),
            AccountMeta::writable(self.account.address()),
            AccountMeta::readonly_signer(self.mint_authority.address()),
        ];

        let views = init_cpi_accounts!(self.mint, self.account, self.mint_authority,);

        Ok(CpiCtx::new(&Token::ID, metas, views, data))
    }
}
