// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use core::mem::MaybeUninit;

use hayabusa_common::{
    Result,
    account_meta::AccountMeta,
    account_view::AccountView,
    cpi::{CpiAccount, CpiCtx},
    init_cpi_accounts,
    traits::CpiSerialize,
};
use hayabusa_invoke_cpi_macros::CpiInstruction;

/// Mints new tokens to an account.
#[derive(CpiInstruction)]
pub struct MintTo<'view> {
    /// Mint account.
    pub mint: AccountView<'view>,
    /// Token account.
    pub account: AccountView<'view>,
    /// Mint authority.
    pub mint_authority: AccountView<'view>,
    /// The token program to use for the CPI.
    pub token_program: AccountView<'view>,
    /// Amount of tokens to mint.
    pub amount: u64,
}

/// The number of accounts required by the [`MintTo`] instruction.
pub const MINT_TO_ACCOUNTS_BUFFER_LEN: usize = 3;
/// The length of the [`MintTo`] instruction data.
pub const MINT_TO_DATA_LEN: usize = 9;
/// The [`MintTo`] instruction discriminator.
pub const MINT_TO_DISCRIMINATOR: [u8; 1] = [7];

type AccountMetaBuffer<'meta> = [AccountMeta<'meta>; MINT_TO_ACCOUNTS_BUFFER_LEN];
type CpiAccountBuffer<'view> = [CpiAccount<'view>; MINT_TO_ACCOUNTS_BUFFER_LEN];
type DataBuffer = [u8; MINT_TO_DATA_LEN];

impl<'view> CpiSerialize<'view, AccountMetaBuffer<'view>, CpiAccountBuffer<'view>, DataBuffer>
    for MintTo<'view>
{
    #[inline(always)]
    fn serialize(
        &self,
    ) -> Result<
        CpiCtx<'view, 'view, 'view, AccountMetaBuffer<'view>, CpiAccountBuffer<'view>, DataBuffer>,
    > {
        // SAFETY: buffer is fully initialized before `assume_init`.
        let data = unsafe {
            let mut buf = MaybeUninit::<DataBuffer>::uninit();
            let ptr = buf.as_mut_ptr() as *mut u8;
            (ptr as *mut [u8; 1]).write(MINT_TO_DISCRIMINATOR);
            (ptr.add(1) as *mut [u8; 8]).write(self.amount.to_le_bytes());

            buf.assume_init()
        };

        let metas = [
            AccountMeta::writable(self.mint.address()),
            AccountMeta::writable(self.account.address()),
            AccountMeta::readonly_signer(self.mint_authority.address()),
        ];

        let views = init_cpi_accounts!(self.mint, self.account, self.mint_authority,);

        Ok(CpiCtx::new(
            self.token_program.address(),
            metas,
            views,
            data,
        ))
    }
}
