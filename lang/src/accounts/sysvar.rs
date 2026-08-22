// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

use core::ops::Deref;

use crate::prelude::*;

#[derive(Clone, Copy)]
pub struct Sysvar<T> {
    sysvar: T,
}

impl<'view, T> ParseAccount<'view> for Sysvar<T>
where
    T: TrySysvarFromAccountView<'view>,
{
    type Meta<'a, 'b, 'c>
        = NoMeta
    where
        'view: 'c,
        'c: 'b,
        'b: 'a;

    #[inline(always)]
    fn parse<'a, 'b, 'c>(view: AccountView<'view>, _: &mut Self::Meta<'a, 'b, 'c>) -> Result<Sysvar<T>>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a,
    {
        Ok(Sysvar {
            sysvar: T::try_sysvar_from_account_view(view)?,
        })
    }
}

impl<'view, T> ToAccountView<'view> for Sysvar<T>
where
    T: ToAccountView<'view>,
{
    #[inline(always)]
    fn to_account_view(&self) -> AccountView<'view> {
        self.sysvar.to_account_view()
    }
}

impl<T> Deref for Sysvar<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.sysvar
    }
}

impl<'view, T> AccountAddress<'view> for Sysvar<T>
where
    T: AccountAddress<'view>,
{
    #[inline(always)]
    fn address(&self) -> &'view Address {
        self.sysvar.address()
    }

    #[inline(always)]
    fn address_owned(&self) -> Address {
        *self.sysvar.address()
    }
}
