// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use core::ops::Deref;

use crate::{
    account_view::AccountView,
    address::Address,
    traits::{AddressTrait, NoMeta, ParseAccount, ParseSysvar, ToAccountView},
    Result,
};

#[derive(Clone, Copy)]
pub struct Sysvar<T> {
    sysvar: T,
}

impl<'view, T> ParseAccount<'view> for Sysvar<T>
where
    T: ParseSysvar<'view>,
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
    ) -> Result<Sysvar<T>>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a,
    {
        Ok(Sysvar {
            sysvar: <T as ParseSysvar<'view>>::parse(view)?,
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

impl<'view, T> AddressTrait<'view> for Sysvar<T>
where
    T: AddressTrait<'view>,
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
