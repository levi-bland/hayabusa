// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use core::{marker::PhantomData, mem::MaybeUninit, ptr::copy_nonoverlapping};

use crate::TokenProgram;
use hayabusa_common::{
    Result,
    account_meta::AccountMeta,
    account_view::AccountView,
    cpi::{CpiAccount, CpiCtx},
    init_cpi_accounts,
    traits::CpiSerialize,
};
use hayabusa_invoke_cpi_macros::CpiInstruction;

/// Like [`super::Burn`], but the mint's decimals are checked by the
/// caller. Does not support accounts associated with the native mint;
/// use [`super::CloseAccount`] instead.
#[derive(CpiInstruction)]
pub struct BurnChecked<'view, P: TokenProgram> {
    /// The account to burn from.
    pub account: AccountView<'view>,
    /// The token mint.
    pub mint: AccountView<'view>,
    /// The account's owner or delegate.
    pub authority: AccountView<'view>,
    /// The amount of tokens to burn.
    pub amount: u64,
    /// Expected number of base 10 digits to the right of the decimal place.
    pub decimals: u8,
    /// Signifies which token program to use for the CPI.
    __phantom: PhantomData<P>,
}

impl<'view, P: TokenProgram> BurnChecked<'view, P> {
    pub fn new(
        account: AccountView<'view>,
        mint: AccountView<'view>,
        authority: AccountView<'view>,
        amount: u64,
        decimals: u8,
    ) -> Self {
        Self {
            account,
            mint,
            authority,
            amount,
            decimals,
            __phantom: PhantomData,
        }
    }
}

/// The number of accounts required by the [`BurnChecked`] instruction.
pub const BURN_CHECKED_ACCOUNTS_BUFFER_LEN: usize = 3;
/// The length of the [`BurnChecked`] instruction data.
pub const BURN_CHECKED_DATA_LEN: usize = 10;
/// The [`BurnChecked`] instruction discriminator.
pub const BURN_CHECKED_DISCRIMINATOR: [u8; 1] = [15];

type AccountMetaBuffer<'meta> = [AccountMeta<'meta>; BURN_CHECKED_ACCOUNTS_BUFFER_LEN];
type CpiAccountBuffer<'view> = [CpiAccount<'view>; BURN_CHECKED_ACCOUNTS_BUFFER_LEN];
type DataBuffer = [u8; BURN_CHECKED_DATA_LEN];

impl<'view, P: TokenProgram>
    CpiSerialize<'view, AccountMetaBuffer<'view>, CpiAccountBuffer<'view>, DataBuffer>
    for BurnChecked<'view, P>
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
            (ptr as *mut [u8; 1]).write(BURN_CHECKED_DISCRIMINATOR);
            copy_nonoverlapping(
                self.amount.to_le_bytes().as_ptr(),
                ptr.add(1),
                size_of::<u64>(),
            );
            ptr.add(9).write(self.decimals);

            buf.assume_init()
        };

        let metas = [
            AccountMeta::writable(self.account.address()),
            AccountMeta::writable(self.mint.address()),
            AccountMeta::readonly_signer(self.authority.address()),
        ];

        let views = init_cpi_accounts!(self.account, self.mint, self.authority,);

        Ok(CpiCtx::new(&P::ID, metas, views, data))
    }
}
