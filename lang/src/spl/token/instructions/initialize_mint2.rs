// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

use core::ptr::addr_of;

use crate::{init_cpi_accounts, prelude::*, spl::token::Token};

/// Like [`super::InitializeMint`], but does not require the Rent
/// sysvar to be provided
pub struct InitializeMint2<'view, 'addr> {
    /// The mint being initialized.
    pub mint: AccountView<'view>,
    /// The number of base 10 digits to the right of the decimal place.
    pub decimals: u8,
    /// The authority to have token minting privileges.
    pub mint_authority: &'addr Address,
    /// The optional freeze authority of the new mint.
    pub freeze_authority: Option<&'addr Address>,
}

/// The number of accounts required by the [`InitializeMint2`] instruction.
pub const INITIALIZE_MINT2_ACCOUNTS_BUFFER_LEN: usize = 1;
/// The base data length of the [`InitializeMint2`] instruction.
pub const INITIALIZE_MINT2_BASE_DATA_LEN: usize = 35;
/// The maximum length of the [`InitializeMint2`] instruction data.
pub const INITIALIZE_MINT2_MAX_DATA_LEN: usize =
    INITIALIZE_MINT2_BASE_DATA_LEN + size_of::<Address>();
/// The [`InitializeMint2`] instruction discriminator.
pub const INITIALIZE_MINT2_DISCRIMINATOR: [u8; 1] = [20];

type AccountMetaBuffer<'meta> = [AccountMeta<'meta>; INITIALIZE_MINT2_ACCOUNTS_BUFFER_LEN];
type CpiAccountBuffer<'view> = [CpiAccount<'view>; INITIALIZE_MINT2_ACCOUNTS_BUFFER_LEN];
type DataBuffer = StackVec<u8, INITIALIZE_MINT2_MAX_DATA_LEN>;

impl<'view>
    SerializeCpiCtx<
        'view,
        AccountMetaBuffer<'view>,
        CpiAccountBuffer<'view>,
        DataBuffer,
    > for InitializeMint2<'view, '_>
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
        // SAFETY: `INITIALIZE_MINT_MAX_DATA_LEN` upper bound is respected.
        let data = unsafe {
            let mut buf = DataBuffer::new();
            buf.push_ptr_unchecked(
                INITIALIZE_MINT2_DISCRIMINATOR.as_ptr(),
                size_of::<[u8; 1]>(),
            );
            buf.push_ptr_unchecked(addr_of!(self.decimals), size_of::<u8>());
            buf.push_ptr_unchecked(self.mint_authority.as_ref().as_ptr(), size_of::<Address>());

            if let Some(freeze_authority) = self.freeze_authority {
                let discriminant = 1u8;
                buf.push_ptr_unchecked(addr_of!(discriminant), size_of::<u8>());
                buf.push_ptr_unchecked(freeze_authority.as_ref().as_ptr(), size_of::<Address>());
            } else {
                let discriminant = 0u8;
                buf.push_ptr_unchecked(addr_of!(discriminant), size_of::<u8>());
            }

            buf
        };

        let metas = [AccountMeta::writable(self.mint.address())];

        let views = init_cpi_accounts!(self.mint);

        Ok(CpiCtx::new(&Token::ID, metas, views, data))
    }
}
