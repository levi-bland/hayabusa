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
use core::ptr::addr_of;
use hayabusa_invoke_cpi_macros::CpiInstruction;

/// Transfer lamports from a derived address.
#[derive(CpiInstruction)]
pub struct TransferWithSeed<'view, 'id, 'seed, const SEED_LEN: usize> {
    /// Funding account.
    pub from: AccountView<'view>,
    /// Base account.
    ///
    /// The account matching the base [`Address`] below must be provided as
    /// a signer, but may be the same as the funding account and provided
    /// as account 0.
    pub base: AccountView<'view>,
    /// Recipient account.
    pub to: AccountView<'view>,
    /// Amount of lamports to transfer.
    pub lamports: u64,
    /// ACSII byte slice, no longer than [`MAX_SEED_LEN`]
    pub seed: &'seed [u8; SEED_LEN],
    /// Address of the program that will own the new account.
    pub owner: &'id Address,
}

impl<const SEED_LEN: usize> TransferWithSeed<'_, '_, '_, SEED_LEN> {
    pub const _ASSERT: () = assert!(SEED_LEN <= MAX_SEED_LEN, "SEED_LEN must be <= MAX_SEED_LEN");
}

/// The number of accounts expected by the [`TransferWithSeed`] system program instruction.
pub const TRANSFER_WITH_SEED_ACCOUNTS_BUFFER_LEN: usize = 3;
/// The maximum instruction data length of the [`TransferWithSeed`] system program instruction.
pub const TRANSFER_WITH_SEED_MAX_DATA_LEN: usize = TRANSFER_WITH_SEED_DATA_BASE_LEN + MAX_SEED_LEN;
/// The length of the instruction data of the [`TransferWithSeed`] system program
/// instruction minus the variable `seed` parameter.
pub const TRANSFER_WITH_SEED_DATA_BASE_LEN: usize = 52;
/// The discriminator of the [`TransferWithSeed`] system program instruction.
pub const TRANSFER_WITH_SEED_DISCRIMINATOR: [u8; 4] = [11, 0, 0, 0];

type AccountMetaBuffer<'meta> = [AccountMeta<'meta>; TRANSFER_WITH_SEED_ACCOUNTS_BUFFER_LEN];
type CpiAccountBuffer<'view> = [CpiAccount<'view>; TRANSFER_WITH_SEED_ACCOUNTS_BUFFER_LEN];
type DataBuffer = StackVec<u8, TRANSFER_WITH_SEED_MAX_DATA_LEN>;

impl<'view, const SEED_LEN: usize>
    CpiSerialize<'view, AccountMetaBuffer<'view>, CpiAccountBuffer<'view>, DataBuffer>
    for TransferWithSeed<'view, '_, '_, SEED_LEN>
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
        let _: () = Self::_ASSERT;

        // instruction data
        // - [0..4  ]: instruction discriminator
        // - [4..12 ]: lamports amount
        // - [12..20]: seed length
        // - [20..  ]: seed (max 32)
        // - [.. +32]: owner address
        //
        // SAFETY: we respect the `TRANSFER_WITH_SEED_MAX_DATA_LEN` bound.
        let data = unsafe {
            let mut buf = DataBuffer::new();
            buf.push_ptr_unchecked(
                TRANSFER_WITH_SEED_DISCRIMINATOR.as_ptr(),
                size_of::<[u8; 4]>(),
            );
            buf.push_ptr_unchecked(addr_of!(self.lamports) as *const u8, size_of::<u64>());
            let seed_len = SEED_LEN as u64;
            buf.push_ptr_unchecked(addr_of!(seed_len) as *const u8, size_of::<u64>());
            buf.push_ptr_unchecked(self.seed.as_ptr(), SEED_LEN);
            buf.push_ptr_unchecked(self.owner.as_ref().as_ptr(), size_of::<Address>());

            buf
        };

        let metas = [
            AccountMeta::writable(self.from.address()),
            AccountMeta::readonly_signer(self.base.address()),
            AccountMeta::writable(self.to.address()),
        ];

        let views = init_cpi_accounts!(self.from, self.base, self.to,);

        Ok(CpiCtx::new(&System::ID, metas, views, data))
    }
}
