// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use core::{marker::PhantomData, ops::Deref};

use crate::{
    account_view::{AccountView, Ref, RefMut},
    accounts::{account::Account, mutable::Mut, pda::Pda, unchecked_account::UncheckedAccount},
    error::ErrorCode,
    hint,
    system_program::System,
    traits::{AccountInit, Cast, Id, NoMeta, ParseAccount, __InitAllowed},
    Result,
};
use hayabusa_err_macro::err;
use solana_program_error::ProgramError;

pub struct Init<T: __InitAllowed>(T);

impl<T: __InitAllowed> Deref for Init<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'view, T: Cast> Init<Account<'view, T>> {
    /// Safely get a [`Ref<T>`] from [`Init<Account<'_, T>>`].
    #[inline(always)]
    pub fn cast(&self) -> Result<Ref<'view, T>> {
        // SAFETY: `Account` can only be constructed via `ParseAccount`,
        // which ensures the account-level invariants like owner, data len,
        // discriminator are all checked, making this unsafe call safe.
        unsafe { T::cast_unchecked(self.view) }
    }

    /// Get a `&T` from [`Init<Account<'_, T>>`].
    ///
    /// # Safety
    ///
    /// While [`Account`] can only be constructed via [`ParseAccount`], which ensures
    /// all account-level invariants are upheld, this method is still unsafe to call
    /// because it creates a reference to the account data without tracking the borrow,
    /// if there are other active (mutable) references in certain situations this can
    /// cause aliasing UB.
    #[inline(always)]
    pub unsafe fn cast_untracked(&self) -> &T {
        T::cast_untracked(self.view)
    }

    /// Safely get a [`RefMut<T>`] from [`Init<Account<'_, T>>`].
    #[inline(always)]
    pub fn cast_mut(&self) -> Result<RefMut<'view, T>> {
        // SAFETY: `Account` can only be constructed via `ParseAccount`,
        // which ensures the account-level invariants like owner, data len,
        // discriminator are all checked, making this unsafe call safe.
        unsafe { T::cast_mut_unchecked(self.view) }
    }

    /// Get a `&mut T` from [`Init<Account<'_, T>>`].
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

impl<'view, T: Cast> Init<Pda<Account<'view, T>>> {
    /// Safely get a [`Ref<T>`] from [`Init<Pda<Account<'_, T>>>`].
    #[inline(always)]
    pub fn cast(&self) -> Result<Ref<'view, T>> {
        // SAFETY: `Account` can only be constructed via `ParseAccount`,
        // which ensures the account-level invariants like owner, data len,
        // discriminator are all checked, making this unsafe call safe.
        unsafe { T::cast_unchecked(self.view) }
    }

    /// Get a `&T` from [`Init<Pda<Account<'_, T>>>`].
    ///
    /// # Safety
    ///
    /// While [`Account`] can only be constructed via [`ParseAccount`], which ensures
    /// all account-level invariants are upheld, this method is still unsafe to call
    /// because it creates a reference to the account data without tracking the borrow,
    /// if there are other active (mutable) references in certain situations this can
    /// cause aliasing UB.
    #[inline(always)]
    pub unsafe fn cast_untracked(&self) -> &T {
        T::cast_untracked(self.view)
    }

    /// Safely get a [`RefMut<T>`] from [`Init<Pda<Account<'_, T>>>`].
    #[inline(always)]
    pub fn cast_mut(&self) -> Result<RefMut<'view, T>> {
        // SAFETY: `Account` can only be constructed via `ParseAccount`,
        // which ensures the account-level invariants like owner, data len,
        // discriminator are all checked, making this unsafe call safe.
        unsafe { T::cast_mut_unchecked(self.view) }
    }

    /// Get a `&mut T` from [`Init<Pda<Account<'_, T>>>`].
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

impl<'view, T> ParseAccount<'view> for Init<Account<'view, T>>
where
    T: AccountInit<'view> + Cast,
{
    type Meta<'a, 'b, 'c>
        = <T as AccountInit<'view>>::Meta<'c>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a;

    #[inline]
    fn parse<'a, 'b, 'c>(
        view: AccountView<'view>,
        meta: &mut Self::Meta<'a, 'b, 'c>,
    ) -> Result<Init<Account<'view, T>>>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a,
    {
        let _: Mut<UncheckedAccount<'view>> = Mut::parse(view, &mut NoMeta)?;

        if hint::unlikely(!view.owned_by(System::ID)) {
            err!(
                "Init::parse: account is not owned by the system program",
                ErrorCode::InvalidAccountOwner,
            );
        }

        if hint::unlikely(view.data_len() != 0) {
            err!(
                "Init::parse: account is already allocated",
                ErrorCode::InitAccountAlreadyAllocated,
            );
        }

        <T as AccountInit<'view>>::init(view, meta)?;

        Ok(Init(Account {
            view,
            __phantom: PhantomData,
        }))
    }
}

impl<'view, T> ParseAccount<'view> for Init<Pda<Account<'view, T>>>
where
    T: AccountInit<'view> + Cast,
{
    type Meta<'a, 'b, 'c>
        = <T as AccountInit<'view>>::PdaMeta<'a, 'b, 'c>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a;

    #[inline]
    fn parse<'a, 'b, 'c>(
        view: AccountView<'view>,
        meta: &mut Self::Meta<'a, 'b, 'c>,
    ) -> Result<Init<Pda<Account<'view, T>>>>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a,
    {
        let _: Mut<UncheckedAccount<'view>> = Mut::parse(view, &mut NoMeta)?;

        if hint::unlikely(!view.owned_by(System::ID)) {
            err!(
                "Init::parse: account is not owned by the system program",
                ErrorCode::InvalidAccountOwner,
            );
        }

        if hint::unlikely(view.data_len() != 0) {
            err!(
                "Init::parse: account is already allocated",
                ErrorCode::InitAccountAlreadyAllocated,
            );
        }

        <T as AccountInit<'view>>::init_pda(view, meta)?;

        Ok(Init(Pda(Account {
            view,
            __phantom: PhantomData,
        })))
    }
}
