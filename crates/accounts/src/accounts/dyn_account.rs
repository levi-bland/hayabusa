// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

use crate::{FromAccountView, Mut, NoMeta, ToAccountView, WritableAllowed};
use core::marker::PhantomData;
use hayabusa_common::{AccountView, Ref, RefMut};
use hayabusa_errors::Result;

pub struct DynAccount<'ix, T>
where
    T: DynAccountBuild<'ix>,
{
    view: &'ix AccountView,
    _phantom: PhantomData<T>,
}

impl<'ix, T> DynAccount<'ix, T>
where
    T: DynAccountBuild<'ix>,
{
    /// # Safety
    /// Caller must ensure no aliasing issues; data len, owner, discriminator
    /// are all automatically validated
    #[inline(always)]
    pub unsafe fn build_unchecked(&self) -> Result<&'ix T::Object> {
        T::build_unchecked(self.view)
    }

    #[inline(always)]
    pub fn build(&self) -> Result<Ref<'ix, T::Object>> {
        T::build(self.view)
    }
}

impl<'ix, T> Mut<DynAccount<'ix, T>>
where
    T: DynAccountBuild<'ix>,
{
    /// # Safety
    /// Caller must ensure no aliasing issues; data len, owner, discriminator
    /// are all automatically validated
    #[inline(always)]
    pub unsafe fn build_mut_unchecked(&self) -> Result<&'ix mut T::Object> {
        T::build_mut_unchecked(self.view)
    }

    #[inline(always)]
    pub fn build_mut(&self) -> Result<RefMut<'ix, T::Object>> {
        T::build_mut(self.view)
    }
}

unsafe impl<'ix, T> FromAccountView<'ix> for DynAccount<'ix, T>
where
    T: DynAccountBuild<'ix>,
{
    type Meta<'a, 'b, 'c, 'd>
        = NoMeta
    where
        'ix: 'a + 'd,
        'd: 'c,
        'c: 'b;

    #[inline(always)]
    fn try_from_account_view<'a, 'b, 'c, 'd>(
        view: &'ix AccountView,
        _meta: Self::Meta<'a, 'b, 'c, 'd>,
    ) -> Result<Self>
    where
        'ix: 'a + 'd,
        'd: 'c,
        'c: 'b,
    {
        Ok(Self {
            view,
            _phantom: PhantomData,
        })
    }
}

impl<'ix, T: DynAccountBuild<'ix>> WritableAllowed for DynAccount<'ix, T> {}

impl<'ix, T: DynAccountBuild<'ix>> ToAccountView<'ix> for DynAccount<'ix, T> {
    #[inline(always)]
    fn to_account_view(&self) -> &'ix AccountView {
        self.view
    }
}

/// Trait for building dynamically dispatched account types
/// Useful for when you have multiple different account layouts for the same concept
/// Implement this trait for a ZST/Dispatcher type using the `impl_dyn_account_build!` macro
pub trait DynAccountBuild<'ix> {
    /// The trait object type; i.e. type Object = dyn Trait
    type Object: ?Sized;

    unsafe fn build_unchecked(view: &'ix AccountView) -> Result<&'ix Self::Object>;
    unsafe fn build_mut_unchecked(view: &'ix AccountView) -> Result<&'ix mut Self::Object>;
    fn build(view: &'ix AccountView) -> Result<Ref<'ix, Self::Object>>;
    fn build_mut(view: &'ix AccountView) -> Result<RefMut<'ix, Self::Object>>;
}

#[macro_export]
macro_rules! impl_dyn_account_build {
    ($trait:ident, $ty:ident, $($accty:ident),+ $(,)?) => {
        impl<'ix> DynAccountBuild<'ix> for $ty {
            type Object = dyn $trait;

            #[inline(always)]
            unsafe fn build_unchecked(view: &'ix AccountView) -> Result<&'ix Self::Object> {
                let disc = get_discriminator(view)?;

                match &disc[..] {
                    $(<$accty as Discriminator>::DISCRIMINATOR => {
                        <$accty as Cast>::check_len(view.data_len())?;
                        <$accty as Cast>::check_owner(view.owner())?;

                        Ok(<$accty as Cast>::cast_unchecked(view) as &dyn $trait)
                    })+
                    _ => {
                        error_msg!(
                            "DynAccountBuild: invalid discriminator",
                            ErrorCode::InvalidAccountDiscriminator,
                        );
                    }
                }
            }

            #[inline(always)]
            unsafe fn build_mut_unchecked(view: &'ix AccountView) -> Result<&'ix mut Self::Object> {
                let disc = get_discriminator(view)?;

                match &disc[..] {
                    $(<$accty as Discriminator>::DISCRIMINATOR => {
                        <$accty as Cast>::check_len(view.data_len())?;
                        <$accty as Cast>::check_owner(view.owner())?;

                        Ok(<$accty as Cast>::cast_mut_unchecked(view) as &mut dyn $trait)
                    })+
                    _ => {
                        error_msg!(
                            "DynAccountBuild: invalid discriminator",
                            ErrorCode::InvalidAccountDiscriminator,
                        );
                    }
                }
            }

            #[inline(always)]
            fn build(view: &'ix AccountView) -> Result<Ref<'ix, Self::Object>> {
                let disc = get_discriminator(view)?;

                match &disc[..] {
                    $(<$accty as Discriminator>::DISCRIMINATOR => {
                        <$accty as Cast>::check_owner(view.owner())?;

                        Ok(Ref::map(<$accty as Cast>::cast(view)?, |acc| acc as &dyn $trait))
                    })+
                    _ => {
                        error_msg!(
                            "DynAccountBuild: invalid discriminator",
                            ErrorCode::InvalidAccountDiscriminator,
                        );
                    }
                }
            }

            #[inline(always)]
            fn build_mut(view: &'ix AccountView) -> Result<RefMut<'ix, Self::Object>> {
                let disc = get_discriminator(view)?;

                match &disc[..] {
                    $(<$accty as Discriminator>::DISCRIMINATOR => {
                        <$accty as Cast>::check_owner(view.owner())?;

                        Ok(RefMut::map(<$accty as Cast>::cast_mut(view)?, |acc| acc as &mut dyn $trait))
                    })+
                    _ => {
                        error_msg!(
                            "DynAccountBuild: invalid discriminator",
                            ErrorCode::InvalidAccountDiscriminator,
                        );
                    }
                }
            }
        }
    };
}
