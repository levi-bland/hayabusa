// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use core::marker::PhantomData;

use crate::{
    account_view::AccountView,
    traits::{
        Cast, Discriminator, NoMeta, Owners, Ownership, ParseAccount, ToAccountView,
        __AccountDiscriminatorMode, __InterfaceAccountType, __NoDiscriminator, __Owners,
        __ParseAccountDispatch, __WithDiscriminator,
    },
    Result,
};

pub struct InterfaceAccount<'view, T: __InterfaceAccountType> {
    view: AccountView<'view>,
    __phantom: PhantomData<T>,
}

impl<'view, T: __InterfaceAccountType + __AccountDiscriminatorMode> ParseAccount<'view>
    for InterfaceAccount<'view, T>
where
    T: __ParseAccountDispatch<'view, T::Mode, InterfaceAccount<'view, T>>,
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
    ) -> Result<InterfaceAccount<'view, T>>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a,
    {
        <T as __ParseAccountDispatch<'view, T::Mode, InterfaceAccount<'view, T>>>::dispatch(view)
    }
}

impl<'view, T> __ParseAccountDispatch<'view, __WithDiscriminator, InterfaceAccount<'view, T>> for T
where
    T: __InterfaceAccountType
        + __AccountDiscriminatorMode<Mode = __WithDiscriminator>
        + Cast
        + Owners
        + Ownership<OwnerType = __Owners>
        + Discriminator,
{
    #[inline(always)]
    fn dispatch(view: AccountView<'view>) -> Result<InterfaceAccount<'view, T>> {
        T::check_space(view)?;
        T::check_owners(view)?;

        // SAFETY: `T::check_space` ensures the account data is large enough
        // for a discriminator.
        unsafe {
            T::check_discriminator_skip_length_check(view)?;
        }

        Ok(InterfaceAccount {
            view,
            __phantom: PhantomData::<T>,
        })
    }
}

impl<'view, T> __ParseAccountDispatch<'view, __NoDiscriminator, InterfaceAccount<'view, T>> for T
where
    T: __InterfaceAccountType
        + __AccountDiscriminatorMode<Mode = __NoDiscriminator>
        + Cast
        + Owners
        + Ownership<OwnerType = __Owners>,
{
    #[inline(always)]
    fn dispatch(view: AccountView<'view>) -> Result<InterfaceAccount<'view, T>> {
        T::check_space(view)?;
        T::check_owners(view)?;

        Ok(InterfaceAccount {
            view,
            __phantom: PhantomData::<T>,
        })
    }
}

impl<'view, T: __InterfaceAccountType> ToAccountView<'view> for InterfaceAccount<'view, T> {
    #[inline(always)]
    fn to_account_view(&self) -> AccountView<'view> {
        self.view
    }
}
