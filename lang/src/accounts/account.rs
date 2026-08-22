// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

use crate::{
    prelude::*,
    traits::internal::{__AccountDiscriminatorMode, __NoDiscriminator, __WithDiscriminator},
};
use core::marker::PhantomData;

#[derive(Clone, Copy)]
pub struct Account<'view, T: Cast> {
    pub(crate) view: AccountView<'view>,
    pub(crate) __phantom: PhantomData<T>,
}

impl<'view, T: Cast + __AccountDiscriminatorMode> ParseAccount<'view> for Account<'view, T>
where
    Account<'view, T>: __ParseAccountDispatch<'view, T::Mode>,
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
        <Self as __ParseAccountDispatch<'view, T::Mode>>::dispatch(view)
    }

    #[inline(always)]
    unsafe fn parse_unchecked<'a, 'b, 'c>(
        view: AccountView<'view>,
        _: &mut Self::Meta<'a, 'b, 'c>,
    ) -> Result<Account<'view, T>>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a,
    {
        Ok(Account {
            view,
            __phantom: PhantomData::<T>,
        })
    }
}

impl<'view, T> __ParseAccountDispatch<'view, __WithDiscriminator> for Account<'view, T>
where
    T: Cast + Discriminator + __AccountDiscriminatorMode<Mode = __WithDiscriminator>,
{
    #[inline(always)]
    fn dispatch(view: AccountView<'view>) -> Result<Account<'view, T>> {
        T::check_space(view.data_len())?;
        T::check_owner(view.owner())?;

        // SAFETY: `T::check_space` ensures the account data is large enough
        // for a discriminator.
        unsafe {
            T::check_discriminator_unchecked(view)?;
        }

        Ok(Account {
            view,
            __phantom: PhantomData::<T>,
        })
    }
}

impl<'view, T> __ParseAccountDispatch<'view, __NoDiscriminator> for Account<'view, T>
where
    T: Cast + __AccountDiscriminatorMode<Mode = __NoDiscriminator>,
{
    #[inline(always)]
    fn dispatch(view: AccountView<'view>) -> Result<Account<'view, T>> {
        T::check_space(view.data_len())?;
        T::check_owner(view.owner())?;

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

impl<'view, T: Cast + __AccountDiscriminatorMode> AccountAddress<'view> for Account<'view, T> {
    #[inline(always)]
    fn address(&self) -> &'view Address {
        self.view.address()
    }

    #[inline(always)]
    fn address_owned(&self) -> Address {
        *self.view.address()
    }
}

unsafe impl<T: Cast> WritableAllowed for Account<'_, T> {}

unsafe impl<T: Cast> PdaAllowed for Account<'_, T> {}

impl<'view, T: Cast + __AccountDiscriminatorMode> Account<'view, T> {
    /// Safely get a [`Ref<T>`] from [`Account<'_, T>`].
    #[inline(always)]
    pub fn cast(&self) -> Result<Ref<'view, T>> {
        // SAFETY: `Account` can only be constructed via `ParseAccount`,
        // which ensures the account-level invariants like owner, data len,
        // discriminator are all checked, making this unsafe call safe.
        unsafe { T::cast_unchecked(self.view) }
    }

    /// Get a `&T` from [`Account<'_, T>`].
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
