// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_view::AccountView,
    address::Address,
    traits::{AddressTrait, NoMeta, ParseAccount, ToAccountView, __PdaAllowed, __WritableAllowed},
    Result,
};

#[derive(Clone, Copy)]
pub struct UncheckedAccount<'view>(AccountView<'view>);

impl<'view> ParseAccount<'view> for UncheckedAccount<'view> {
    type Meta<'a, 'b, 'c>
        = NoMeta
    where
        'view: 'c,
        'c: 'b,
        'b: 'a;

    #[inline(always)]
    fn parse<'a, 'b, 'c>(view: AccountView<'view>, _: &mut Self::Meta<'a, 'b, 'c>) -> Result<Self>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a,
    {
        Ok(UncheckedAccount(view))
    }
}

impl<'view> ToAccountView<'view> for UncheckedAccount<'view> {
    #[inline(always)]
    fn to_account_view(&self) -> AccountView<'view> {
        self.0
    }
}

unsafe impl __PdaAllowed for UncheckedAccount<'_> {}

unsafe impl __WritableAllowed for UncheckedAccount<'_> {}

impl<'view> AddressTrait<'view> for UncheckedAccount<'view> {
    #[inline(always)]
    fn address(&self) -> &'view Address {
        self.0.address()
    }

    #[inline(always)]
    fn address_owned(&self) -> Address {
        *self.0.address()
    }
}
