// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use core::{marker::PhantomData, ops::Deref};

use crate::{
    prelude::*,
    syscalls::{try_create_program_address, try_find_program_address},
    traits::internal::__AccountDiscriminatorMode,
};

pub struct Pda<T>(pub(crate) T);

unsafe impl<T> WritableAllowed for Pda<T> {}

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
    Account<'view, T>: __ParseAccountDispatch<'view, T::Mode>,
{
    type Meta<'a, 'b, 'c>
        = __PdaByteSliceMeta<'a, 'b, 'c>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a;

    #[inline(always)]
    fn parse<'a, 'b, 'c>(view: AccountView<'view>, meta: &mut Self::Meta<'a, 'b, 'c>) -> Result<Self>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a,
    {
        let pda = if let Some(ptr) = meta.bump_ptr {
            // in the case of an unknown bump, validate the account invariants. the case of a known
            //bump in the seeds means the account invariants are already implicitly validated.
            let _: Account<'_, T> = Account::parse(view, &mut NoMeta)?;

            let (pda, bump) = try_find_program_address(meta.seeds, &T::OWNER)?;

            // SAFETY: `meta.bump_ptr` is guaranteed to be a valid ptr by construction.
            unsafe {
                ptr.write(bump);
            }

            pda
        } else {
            try_create_program_address(meta.seeds, &T::OWNER)?
        };

        if hint::unlikely(!address_eq(view.address(), &pda)) {
            err!(
                "Pda::parse: account is not the expected PDA",
                ErrorCode::InvalidProgramAccount,
            );
        }

        Ok(Pda(Account {
            view,
            __phantom: PhantomData::<T>,
        }))
    }

    #[inline(always)]
    unsafe fn parse_unchecked<'a, 'b, 'c>(view: AccountView<'view>, meta: &mut Self::Meta<'a, 'b, 'c>) -> Result<Self>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a,
    {
        let pda = if let Some(ptr) = meta.bump_ptr {
            let (pda, bump) = try_find_program_address(meta.seeds, &T::OWNER)?;

            // SAFETY: `meta.bump_ptr` is guaranteed to be a valid ptr by construction.
            unsafe {
                ptr.write(bump);
            }

            pda
        } else {
            try_create_program_address(meta.seeds, &T::OWNER)?
        };

        if hint::unlikely(!address_eq(view.address(), &pda)) {
            err!(
                "Pda::parse_unchecked: account is not the expected PDA",
                ErrorCode::InvalidProgramAccount,
            );
        }

        Ok(Pda(Account {
            view,
            __phantom: PhantomData::<T>,
        }))
    }
}

pub struct __PdaByteSliceMeta<'a, 'b, 'c>
where
    'b: 'a,
{
    pub seeds: &'a [&'b [u8]],
    pub bump_ptr: Option<*mut u8>,
    __phantom: PhantomData<&'c mut u8>,
}

impl<'a, 'b, 'c> __PdaByteSliceMeta<'a, 'b, 'c>
where
    'b: 'a,
{
    #[inline(always)]
    pub fn new(seeds: &'a [&'b [u8]], bump_ref: Option<&'c mut u8>) -> Self {
        Self {
            seeds,
            bump_ptr: bump_ref.map(|r| r as *mut u8),
            __phantom: PhantomData,
        }
    }
}