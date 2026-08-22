// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use crate::prelude::*;
use core::ops::Deref;

#[derive(Clone, Copy)]
pub struct UncheckedAccount<'view> {
    view: AccountView<'view>,
}

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
        Ok(UncheckedAccount { view })
    }
}

impl<'view> ToAccountView<'view> for UncheckedAccount<'view> {
    #[inline(always)]
    fn to_account_view(&self) -> AccountView<'view> {
        self.view
    }
}

impl<'view> Deref for UncheckedAccount<'view> {
    type Target = AccountView<'view>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.view
    }
}

unsafe impl PdaAllowed for UncheckedAccount<'_> {}

unsafe impl WritableAllowed for UncheckedAccount<'_> {}

impl<'view> AccountAddress<'view> for UncheckedAccount<'view> {
    #[inline(always)]
    fn address(&self) -> &'view Address {
        self.view.address()
    }

    #[inline(always)]
    fn address_owned(&self) -> Address {
        *self.view.address()
    }
}
