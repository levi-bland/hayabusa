// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

use crate::{init_cpi_accounts, prelude::*, system_program::MAX_SEED_LEN};
use core::ptr::addr_of;

/// Create a new account at an address derived from a base pubkey and a seed.
pub struct CreateAccountWithSeed<'view, 'id, 'seed, const SEED_LEN: usize> {
    /// Funding account.
    pub from: AccountView<'view>,
    /// New account.
    pub to: AccountView<'view>,
    /// Base account.
    ///
    /// The account matching the base Pubkey below must be provided as
    /// a signer, but may be the same as the funding account and provided
    /// as account 0.
    pub base: Option<AccountView<'view>>,
    /// ASCII byte array, no longer than `MAX_SEED_LEN`.
    pub seed: &'seed [u8; SEED_LEN],
    /// Number of bytes of memory to allocate.
    pub space: u64,
    /// Address of the program that will own the account.
    pub owner: &'id Address,
}

impl<const SEED_LEN: usize> CreateAccountWithSeed<'_, '_, '_, SEED_LEN> {
    const _ASSERT: () = assert!(SEED_LEN <= MAX_SEED_LEN, "SEED_LEN must be <= MAX_SEED_LEN");
}

/// The number of accounts expected by the `CreateAccountWithSeed` system program instruction.
pub const CREATE_ACCOUNT_WITH_SEED_ACCOUNTS_BUFFER_LEN: usize = 3;
/// The `CreateAccountWithSeed` system program instruction data length prepending/postpending the variable length `seed` field.
pub const CREATE_ACCOUNT_WITH_SEED_DATA_BASE_LEN: usize = 92;
/// The `CreateAccountWithSeed` system program maximum instruction data length.
pub const CREATE_ACCOUNT_WITH_SEED_MAX_DATA_LEN: usize =
    CREATE_ACCOUNT_WITH_SEED_DATA_BASE_LEN + MAX_SEED_LEN;
/// The `CreateAccountWithSeed` system program instruction discriminator;
pub const CREATE_ACCOUNT_WITH_SEED_DISCRIMINATOR: [u8; 4] = [3, 0, 0, 0];

type AccountMetaBuffer<'meta> = [AccountMeta<'meta>; CREATE_ACCOUNT_WITH_SEED_ACCOUNTS_BUFFER_LEN];
type CpiAccountBuffer<'view> = [CpiAccount<'view>; CREATE_ACCOUNT_WITH_SEED_ACCOUNTS_BUFFER_LEN];
type DataBuffer = StackVec<u8, CREATE_ACCOUNT_WITH_SEED_MAX_DATA_LEN>;

impl<'view, const SEED_LEN: usize>
    SerializeCpiCtx<
        'view,
        AccountMetaBuffer<'view>,
        CpiAccountBuffer<'view>,
        DataBuffer,
    > for CreateAccountWithSeed<'view, '_, '_, SEED_LEN>
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
        let lamports = system_program::minimum_balance(self.space as usize)?;
        let base = self.base.unwrap_or(self.from);
        let seed_len = SEED_LEN as u64;

        // instruction data
        // - [0..4  ]: instruction discriminator
        // - [4..36 ]: base pubkey
        // - [36..44]: seed length
        // - [44..  ]: seed (max 32)
        // - [..  +8]: lamports
        // - [..  +8]: account space
        // - [.. +32]: owner pubkey
        //
        // SAFETY: `CpiCtx` will only ever touch up to `CREATE_ACCOUNT_WITH_SEED_DATA_BASE_LEN + SEED_LEN`
        // bytes via `data_length`, so the uninitialized tail is never read.
        // The upper bound of `CREATE_ACCOUNT_WITH_SEED_MAX_DATA_LEN` is respected.
        let data = unsafe {
            let mut buf = DataBuffer::new();
            buf.push_ptr_unchecked(
                CREATE_ACCOUNT_WITH_SEED_DISCRIMINATOR.as_ptr(),
                size_of::<[u8; 4]>(),
            );
            buf.push_ptr_unchecked(base.address().as_ref().as_ptr(), size_of::<Address>());
            buf.push_ptr_unchecked(addr_of!(seed_len) as *const u8, size_of::<u64>());
            buf.push_ptr_unchecked(self.seed.as_ptr(), SEED_LEN);
            buf.push_ptr_unchecked(addr_of!(lamports) as *const u8, size_of::<u64>());
            buf.push_ptr_unchecked(addr_of!(self.space) as *const u8, size_of::<u64>());
            buf.push_ptr_unchecked(self.owner as *const _ as *const u8, size_of::<Address>());

            buf
        };

        let metas = [
            AccountMeta::writable_signer(self.from.address()),
            AccountMeta::writable(self.to.address()),
            AccountMeta::readonly_signer(base.address()),
        ];

        let views = init_cpi_accounts!(self.from, self.to, base,);

        Ok(CpiCtx::new(&System::ID, metas, views, data))
    }
}
