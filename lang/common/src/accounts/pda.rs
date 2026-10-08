// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_view::{AccountView, RefMut},
    accounts::{account::Account, mutable::Mut},
    address::address_eq,
    error::ErrorCode,
    hint,
    syscalls::{try_create_program_address, try_find_program_address},
    traits::{
        Cast, NoMeta, ParseAccount, __AccountDiscriminatorMode, __InitAllowed,
        __ParseAccountDispatch, __WritableAllowed,
    },
    Result,
};
use core::ops::Deref;
use hayabusa_err_macro::err;
use solana_program_error::ProgramError;

pub struct Pda<T>(pub(crate) T);

unsafe impl<T> __WritableAllowed for Pda<T> {}
unsafe impl<T> __InitAllowed for Pda<T> {}

impl<T> Deref for Pda<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'view, T: Cast + __AccountDiscriminatorMode> Mut<Pda<Account<'view, T>>> {
    /// Safely get a [`RefMut<T>`] from [`Account<'_, T>`].
    #[inline(always)]
    pub fn cast_mut(&self) -> Result<RefMut<'view, T>> {
        // SAFETY: `Account` can only be constructed via `ParseAccount`,
        // which ensures the account-level invariants like owner, data len,
        // discriminator are all checked, making this unsafe call safe.
        unsafe { T::cast_mut_unchecked(self.view) }
    }

    /// Get a `&mut T` from [`Account<'_, T>`].
    ///
    /// # Safety
    ///
    /// While [`Account`] can only be constructed via [`ParseAccount`], which ensures
    /// all account-level invariants are upheld, this method is still unsafe to call
    /// because it creates a reference to the account data without tracking the borrow,
    /// if there are other active (mutable) references in certain situations this can
    /// cause aliasing UB.
    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn cast_mut_untracked(&self) -> &mut T {
        T::cast_mut_untracked(self.view)
    }
}

impl<'view, T: Cast + __AccountDiscriminatorMode> ParseAccount<'view> for Pda<Account<'view, T>>
where
    T: __ParseAccountDispatch<'view, T::Mode, Account<'view, T>>,
{
    type Meta<'a, 'b, 'c>
        = __PdaByteSliceMeta<'a, 'b, 'c>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a;

    #[inline(always)]
    fn parse<'a, 'b, 'c>(
        view: AccountView<'view>,
        meta: &mut Self::Meta<'a, 'b, 'c>,
    ) -> Result<Self>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a,
    {
        let account: Account<'_, T> = Account::parse(view, &mut NoMeta)?;

        let pda = match &mut meta.bump_slot {
            Some(slot) => {
                let (pda, bump) = try_find_program_address(meta.seeds, unsafe { account.owner() })?;

                **slot = bump;

                pda
            }
            None => try_create_program_address(meta.seeds, unsafe { account.owner() })?,
        };

        if hint::unlikely(!address_eq(view.address(), &pda)) {
            err!(
                "Pda::parse: account is not the expected PDA",
                ErrorCode::InvalidProgramAccount,
            );
        }

        Ok(Pda(Account::parse(view, &mut NoMeta)?))
    }
}

pub struct __PdaByteSliceMeta<'a, 'b, 'c>
where
    'b: 'a,
{
    pub seeds: &'a [&'b [u8]],
    pub bump_slot: Option<&'c mut u8>,
}

impl<'a, 'b, 'c> __PdaByteSliceMeta<'a, 'b, 'c>
where
    'b: 'a,
{
    #[inline(always)]
    pub fn new(seeds: &'a [&'b [u8]], bump_slot: Option<&'c mut u8>) -> Self {
        Self { seeds, bump_slot }
    }
}
