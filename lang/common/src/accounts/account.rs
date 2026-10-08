// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_view::{AccountView, Ref, RefMut},
    accounts::mutable::Mut,
    address::Address,
    traits::{
        AddressTrait, Cast, Discriminator, NoMeta, Owner, Ownership, ParseAccount, ToAccountView,
        __AccountDiscriminatorMode, __InitAllowed, __NoDiscriminator, __Owner,
        __ParseAccountDispatch, __PdaAllowed, __WithDiscriminator, __WritableAllowed,
    },
    Result,
};
use core::marker::PhantomData;

pub struct Account<'view, T: Cast> {
    pub(crate) view: AccountView<'view>,
    pub(crate) __phantom: PhantomData<T>,
}

impl<'view, T: Cast> Account<'view, T> {
    /// Create a new [`Account<'view, T>`] from a [`AccountView<'view>`], bypassing [`ParseAccount::parse`] checks.
    ///
    /// ## Safety
    ///
    /// You must manually ensure the invariants defined in [`__ParseAccountDispatch::dispatch`] for `T` are upheld,
    /// otherwise the [`Account::cast`] method family could cause UB.
    #[inline(always)]
    pub unsafe fn new(view: AccountView<'view>) -> Account<'view, T> {
        Account {
            view,
            __phantom: PhantomData::<T>,
        }
    }

    pub unsafe fn owner(&self) -> &'view Address {
        self.view.owner()
    }
}

impl<'view, T: Cast + __AccountDiscriminatorMode> ParseAccount<'view> for Account<'view, T>
where
    T: __ParseAccountDispatch<'view, T::Mode, Account<'view, T>>,
{
    type Meta<'a, 'b, 'c>
        = NoMeta
    where
        'view: 'c,
        'c: 'b,
        'b: 'a;

    #[inline(always)]
    fn parse<'a, 'b, 'c>(
        view: AccountView<'view>,
        _: &mut Self::Meta<'a, 'b, 'c>,
    ) -> Result<Account<'view, T>>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a,
    {
        <T as __ParseAccountDispatch<'view, T::Mode, Account<'view, T>>>::dispatch(view)
    }
}

impl<'view, T> __ParseAccountDispatch<'view, __WithDiscriminator, Account<'view, T>> for T
where
    T: Cast
        + Discriminator
        + __AccountDiscriminatorMode<Mode = __WithDiscriminator>
        + Owner
        + Ownership<OwnerType = __Owner>,
{
    #[inline(always)]
    fn dispatch(view: AccountView<'view>) -> Result<Account<'view, T>> {
        T::check_space(view)?;
        T::check_owner(view)?;

        // SAFETY: `T::check_space` ensures the account data is large enough
        // for a discriminator.
        unsafe {
            T::check_discriminator_skip_length_check(view)?;
        }

        Ok(Account {
            view,
            __phantom: PhantomData::<T>,
        })
    }
}

impl<'view, T> __ParseAccountDispatch<'view, __NoDiscriminator, Account<'view, T>> for T
where
    T: Cast
        + __AccountDiscriminatorMode<Mode = __NoDiscriminator>
        + Owner
        + Ownership<OwnerType = __Owner>,
{
    #[inline(always)]
    fn dispatch(view: AccountView<'view>) -> Result<Account<'view, T>> {
        T::check_space(view)?;
        T::check_owner(view)?;

        Ok(Account {
            view,
            __phantom: PhantomData::<T>,
        })
    }
}

impl<'view, T: Cast + __AccountDiscriminatorMode> ToAccountView<'view> for Account<'view, T> {
    #[inline(always)]
    fn to_account_view(&self) -> AccountView<'view> {
        self.view
    }
}

impl<'view, T: Cast + __AccountDiscriminatorMode> AddressTrait<'view> for Account<'view, T> {
    #[inline(always)]
    fn address(&self) -> &'view Address {
        self.view.address()
    }

    #[inline(always)]
    fn address_owned(&self) -> Address {
        *self.view.address()
    }
}

unsafe impl<T: Cast> __WritableAllowed for Account<'_, T> {}

unsafe impl<T: Cast> __PdaAllowed for Account<'_, T> {}

unsafe impl<T: Cast> __InitAllowed for Account<'_, T> {}

impl<'view, T: Cast + __AccountDiscriminatorMode> Account<'view, T> {
    /// Safely get a [`Ref<T>`] from [`Account<'_, T>`].
    #[inline(always)]
    pub fn cast(&self) -> Result<Ref<'view, T>> {
        // SAFETY: `Account` can only be constructed via `ParseAccount` or unsafely via `Account::new`,
        // the latter assumes the invariants have been checked, the former
        // ensures the account-level invariants like owner, data len,
        // discriminator are all checked, making this unsafe call safe.
        unsafe { T::cast_unchecked(self.view) }
    }

    /// Get a `&'view T` from [`Account<'view, T>`].
    ///
    /// # Safety
    ///
    /// While [`Account`] can only be constructed via [`ParseAccount`], [`ParseAccount::parse`] ensures
    /// all account-level invariants are upheld, this method is still unsafe to call
    /// because it creates a reference to the account data without tracking the borrow,
    /// if there are other active (mutable) references in certain situations this can
    /// cause aliasing UB.
    #[inline(always)]
    pub unsafe fn cast_untracked(&self) -> &'view T {
        T::cast_untracked(self.view)
    }
}

impl<'view, T: Cast + __AccountDiscriminatorMode> Mut<Account<'view, T>> {
    /// Safely get a [`RefMut<T>`] from [`Account<'_, T>`].
    #[inline(always)]
    pub fn cast_mut(&self) -> Result<RefMut<'view, T>> {
        // SAFETY: `Account` can only be constructed via `ParseAccount`,
        // which ensures the account-level invariants like owner, data len,
        // discriminator are all checked, making this unsafe call safe.
        unsafe { T::cast_mut_unchecked(self.view) }
    }

    /// Get a `&'view mut T` from [`Account<'view, T>`].
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
    pub unsafe fn cast_mut_untracked(&self) -> &'view mut T {
        T::cast_mut_untracked(self.view)
    }
}
