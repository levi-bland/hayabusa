// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_meta::AccountMeta,
    account_view::AccountView,
    address::Address,
    cpi::{CpiAccount, CpiCtx},
    init_cpi_accounts,
    system_program::{System, MAX_SEED_LEN},
    traits::{CpiSerialize, Id},
    vec::StackVec,
    Result,
};
use core::{mem::size_of, ptr::addr_of};
use hayabusa_invoke_cpi_macros::CpiInstruction;
/// Assign an account to a program based on a seed.
#[derive(CpiInstruction)]
pub struct AssignWithSeed<'view, 'id, 'seed, const SEED_LEN: usize> {
    /// Account being assigned to a program.
    pub account: AccountView<'view>,
    /// Base account.
    ///
    /// The account matching the base Pubkey below must be provided as
    /// a signer, but may be the same as the funding account and provided
    /// as account 0.
    pub base: AccountView<'view>,
    /// Array of ASCII bytes; no longer than `MAX_SEED_LEN`.
    pub seed: &'seed [u8; SEED_LEN],
    /// Address of the program that will own the new account.
    pub owner: &'id Address,
}

/// The number of accounts expected by the `AssignWithSeed` system program instruction.
pub const ASSIGN_WITH_SEED_ACCOUNTS_BUFFER_LEN: usize = 2;
/// The maximum instruction data length accepted by the `AssignWithSeed` system program instruction.
pub const ASSIGN_WITH_SEED_MAX_DATA_LEN: usize = ASSIGN_WITH_SEED_DATA_BASE_LEN + MAX_SEED_LEN;
/// The length of the instruction data prepending/postpending the variable length `seed` parameter
/// for the `AssignWithSeed` system program instruction.
pub const ASSIGN_WITH_SEED_DATA_BASE_LEN: usize = 76;
/// The discriminator of the `AssignWithSeed` system program instruction.
pub const ASSIGN_WITH_SEED_DISCRIMINATOR: [u8; 4] = [10, 0, 0, 0];

type AccountMetaBuffer<'meta> = [AccountMeta<'meta>; ASSIGN_WITH_SEED_ACCOUNTS_BUFFER_LEN];
type CpiAccountBuffer<'view> = [CpiAccount<'view>; ASSIGN_WITH_SEED_ACCOUNTS_BUFFER_LEN];
type DataBuffer = StackVec<u8, ASSIGN_WITH_SEED_MAX_DATA_LEN>;

impl<'view, const SEED_LEN: usize>
    CpiSerialize<'view, AccountMetaBuffer<'view>, CpiAccountBuffer<'view>, DataBuffer>
    for AssignWithSeed<'view, '_, '_, SEED_LEN>
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
        // - [0..4  ]: instruction discriminator
        // - [4..36 ]: base pubkey
        // - [36..44]: seed length
        // - [44..  ]: seed (max 32)
        // - [.. +32]: owner pubkey
        //
        // SAFETY: `CpiCtx` will only ever touch up to `ASSIGN_WITH_SEED_DATA_PRE_POST_SEED_LEN + SEED_LEN`
        // bytes via `data_length`, so the uninitialized tail is never read.
        // The upper bound of `ASSIGN_WITH_SEED_MAX_DATA_LEN` is respected by the unsafe StackVec pushes.
        let data = unsafe {
            let mut buf = DataBuffer::new();
            buf.push_ptr_unchecked(
                ASSIGN_WITH_SEED_DISCRIMINATOR.as_ptr(),
                size_of::<[u8; 4]>(),
            );
            buf.push_ptr_unchecked(self.base.address().as_ref().as_ptr(), size_of::<Address>());
            let seed_len = SEED_LEN as u64;
            buf.push_ptr_unchecked(addr_of!(seed_len) as *const u8, size_of::<u64>());
            buf.push_ptr_unchecked(self.seed.as_ptr(), SEED_LEN);
            buf.push_ptr_unchecked(self.owner as *const _ as *const u8, size_of::<Address>());

            buf
        };

        let metas = [
            AccountMeta::writable(self.account.address()),
            AccountMeta::readonly_signer(self.base.address()),
        ];

        let views = init_cpi_accounts!(self.account, self.base,);

        Ok(CpiCtx::new(&System::ID, metas, views, data))
    }
}
