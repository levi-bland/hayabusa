// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use crate::{FromAccountView, Mut, NoMeta, ToAccountView, WritableAllowed};
use core::marker::PhantomData;
use hayabusa_cast::Cast;
use hayabusa_common::{AccountView, Ref, RefMut};
use hayabusa_discriminator::Discriminator;
use hayabusa_errors::Result;

/// Marker: skip discriminator check
pub struct NoDiscriminator;
/// Marker: require discriminator check
pub struct WithDiscriminator;

pub struct Account<'ix, T: Cast, D = WithDiscriminator> {
    pub(crate) view: &'ix AccountView,
    pub(crate) _phantom: PhantomData<(T, D)>,
}

unsafe impl<'ix, T: Cast> FromAccountView<'ix> for Account<'ix, T, NoDiscriminator> {
    type Meta<'a, 'b, 'c, 'd>
        = NoMeta
    where
        'ix: 'a + 'd,
        'd: 'c,
        'c: 'b;

    fn try_from_account_view<'a, 'b, 'c, 'd>(
        view: &'ix AccountView,
        _meta: Self::Meta<'a, 'b, 'c, 'd>,
    ) -> Result<Self>
    where
        'ix: 'a + 'd,
        'd: 'c,
        'c: 'b,
    {
        T::check_len(view.data_len())?;

        // SAFETY: no aliasing issues here
        unsafe {
            T::check_owner(view.owner())?;
        }

        Ok(Self {
            view,
            _phantom: PhantomData,
        })
    }
}

unsafe impl<'ix, T: Cast + Discriminator> FromAccountView<'ix>
    for Account<'ix, T, WithDiscriminator>
{
    type Meta<'a, 'b, 'c, 'd>
        = NoMeta
    where
        'ix: 'a + 'd,
        'd: 'c,
        'c: 'b;

    fn try_from_account_view<'a, 'b, 'c, 'd>(
        view: &'ix AccountView,
        _: Self::Meta<'a, 'b, 'c, 'd>,
    ) -> Result<Self>
    where
        'ix: 'a + 'd,
        'd: 'c,
        'c: 'b,
    {
        // SAFETY: no aliasing issues
        unsafe {
            T::check_len(view.data_len())?;
            T::check_discriminator(view)?;
            T::check_owner(view.owner())?;
        }

        Ok(Self {
            view,
            _phantom: PhantomData,
        })
    }
}

impl<'ix, T: Cast> Account<'ix, T> {
    #[inline(always)]
    pub fn cast(&self) -> Result<Ref<'ix, T>> {
        T::cast(self.view)
    }

    /// # Safety
    /// Caller must ensure soundness at call site;
    /// i.e. account data len, discriminator, owner, etc...
    #[inline(always)]
    pub unsafe fn cast_unchecked(&self) -> &'ix T {
        T::cast_unchecked(self.view)
    }
}

impl<'ix, T: Cast> Mut<Account<'ix, T>> {
    #[inline(always)]
    pub fn cast_mut(&self) -> Result<RefMut<'ix, T>> {
        T::cast_mut(self.view)
    }

    /// # Safety
    /// Caller must ensure soundness at call site;
    /// i.e. account data len, discriminator, owner, etc...
    #[allow(clippy::mut_from_ref)]
    #[inline(always)]
    pub unsafe fn cast_mut_unchecked(&self) -> &'ix mut T {
        T::cast_mut_unchecked(self.view)
    }
}

impl<'ix, T: Cast> ToAccountView<'ix> for Account<'ix, T> {
    fn to_account_view(&self) -> &'ix AccountView {
        self.view
    }
}

impl<T: Cast> WritableAllowed for Account<'_, T> {}
