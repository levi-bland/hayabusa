// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use core::ptr::addr_of;

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
use hayabusa_invoke_cpi_macros::CpiInstruction;

/// Allocate space for and assign an account at an address derived
/// from a base address and seed.
#[derive(CpiInstruction)]
pub struct AllocateWithSeed<'view, 'id, 'seed, const SEED_LEN: usize> {
    /// Account being allocated
    pub account: AccountView<'view>,
    /// Base account.
    ///
    /// The account matching the base Address below must be provided as
    /// a signer, but may be the same as the funding account and provided
    /// as account 0.
    pub base: AccountView<'view>,
    /// ASCII bytes, no longer than `Address::MAX_SEED_LEN`.
    pub seed: &'seed [u8; SEED_LEN],
    /// Number of bytes to allocate.
    pub space: u64,
    /// Address/ID of the program that will own the account.
    pub owner: &'id Address,
}

impl<const SEED_LEN: usize> AllocateWithSeed<'_, '_, '_, SEED_LEN> {
    const _ASSERT: () = assert!(SEED_LEN <= MAX_SEED_LEN, "SEED_LEN must be <= MAX_SEED_LEN");
}

/// The number of accounts required by the `AllocateWithSeed` system program instruction
pub const ALLOCATE_WITH_SEED_ACCOUNTS_BUFFER_LEN: usize = 2;
/// The `AllocateWithSeed` system program instruction data length prepending/postpending the variable length `seed` field.
pub const ALLOCATE_WITH_SEED_DATA_BASE_LEN: usize = 84;
/// The `AllocateWithSeed` system program instruction maximum data length
pub const ALLOCATE_WITH_SEED_DATA_MAX_LEN: usize = ALLOCATE_WITH_SEED_DATA_BASE_LEN + MAX_SEED_LEN;
/// The `AllocateWithSeed` system program instruction discriminator
pub const ALLOCATE_WITH_SEED_DISCRIMINATOR: [u8; 4] = [9, 0, 0, 0];

type AccountMetaBuffer<'meta> = [AccountMeta<'meta>; ALLOCATE_WITH_SEED_ACCOUNTS_BUFFER_LEN];
type CpiAccountBuffer<'view> = [CpiAccount<'view>; ALLOCATE_WITH_SEED_ACCOUNTS_BUFFER_LEN];
type DataBuffer = StackVec<u8, ALLOCATE_WITH_SEED_DATA_MAX_LEN>;

impl<'view, const SEED_LEN: usize>
    CpiSerialize<'view, AccountMetaBuffer<'view>, CpiAccountBuffer<'view>, DataBuffer>
    for AllocateWithSeed<'view, '_, '_, SEED_LEN>
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
        // ix data
        // - [0..4  ]: discriminator
        // - [4..36 ]: base pubkey
        // - [36..44]: seed length
        // - [44..44+SEED_LEN]: seed bytes
        // - [44+SEED_LEN..52+SEED_LEN]: space (u64 le)
        // - [52+SEED_LEN..84+SEED_LEN]: owner pubkey
        //
        // SAFETY: `CpiCtx` will only ever touch up to `ALLOCATE_WITH_SEED_DATA_BASE_LEN + SEED_LEN`
        // bytes via `data_length`, so the uninitialized tail is never read.
        // The `ALLOCATE_WITH_SEED_DATA_MAX_LEN` upper bound is respected.
        let data = unsafe {
            let mut buf = DataBuffer::new();
            buf.push_ptr_unchecked(
                ALLOCATE_WITH_SEED_DISCRIMINATOR.as_ptr(),
                size_of::<[u8; 4]>(),
            );
            buf.push_ptr_unchecked(self.base.address().as_ref().as_ptr(), size_of::<Address>());
            let seed_len = SEED_LEN as u64;
            buf.push_ptr_unchecked(addr_of!(seed_len) as *const u8, size_of::<u64>());
            buf.push_ptr_unchecked(self.seed.as_ptr(), SEED_LEN);
            buf.push_ptr_unchecked(addr_of!(self.space) as *const u8, size_of::<u64>());
            buf.push_ptr_unchecked(self.owner.as_ref().as_ptr(), size_of::<Address>());

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
