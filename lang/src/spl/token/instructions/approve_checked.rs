// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use core::{mem::MaybeUninit, ptr::copy_nonoverlapping};

use crate::{init_cpi_accounts, prelude::*, spl::token::Token};

/// Like [`super::Approve`], but the mint and its decimals are checked
/// by the caller. This may be useful when creating transactions
/// offline or within a hardware wallet.
pub struct ApproveChecked<'view> {
    /// The source account.
    pub source: AccountView<'view>,
    /// The token mint.
    pub mint: AccountView<'view>,
    /// The delegate.
    pub delegate: AccountView<'view>,
    /// The source account's owner.
    pub owner: AccountView<'view>,
    /// The amount of tokens the delegate is approved for.
    pub amount: u64,
    /// Expected number of base 10 digits to the right of the decimal place.
    pub decimals: u8,
}

/// The number of accounts required by the [`ApproveChecked`] instruction.
pub const APPROVE_CHECKED_ACCOUNTS_BUFFER_LEN: usize = 4;
/// The length of the [`ApproveChecked`] instruction data.
pub const APPROVE_CHECKED_DATA_LEN: usize = 10;
/// The [`ApproveChecked`] instruction discriminator.
pub const APPROVE_CHECKED_DISCRIMINATOR: [u8; 1] = [13];

type AccountMetaBuffer<'meta> = [AccountMeta<'meta>; APPROVE_CHECKED_ACCOUNTS_BUFFER_LEN];
type CpiAccountBuffer<'view> = [CpiAccount<'view>; APPROVE_CHECKED_ACCOUNTS_BUFFER_LEN];
type DataBuffer = [u8; APPROVE_CHECKED_DATA_LEN];

impl<'view>
    SerializeCpiCtx<
        'view,
        AccountMetaBuffer<'view>,
        CpiAccountBuffer<'view>,
        DataBuffer,
    > for ApproveChecked<'view>
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
            (ptr as *mut [u8; 1]).write(APPROVE_CHECKED_DISCRIMINATOR);
            copy_nonoverlapping(
                self.amount.to_le_bytes().as_ptr(),
                ptr.add(1),
                size_of::<u64>(),
            );
            ptr.add(9).write(self.decimals);

            buf.assume_init()
        };

        let metas = [
            AccountMeta::writable(self.source.address()),
            AccountMeta::readonly(self.mint.address()),
            AccountMeta::readonly(self.delegate.address()),
            AccountMeta::readonly_signer(self.owner.address()),
        ];

        let views = init_cpi_accounts!(self.source, self.mint, self.delegate, self.owner,);

        Ok(CpiCtx::new(&Token::ID, metas, views, data))
    }
}