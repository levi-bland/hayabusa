// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

use crate::{FromAccountView, NoMeta, ToAccountView, WritableAllowed};
use hayabusa_common::AccountView;
use hayabusa_errors::Result;

pub struct UncheckedAccount<'ix> {
    account_view: &'ix AccountView,
}

unsafe impl<'ix> FromAccountView<'ix> for UncheckedAccount<'ix> {
    type Meta<'a, 'b, 'c, 'd>
        = NoMeta
    where
        'ix: 'a + 'd,
        'd: 'c,
        'c: 'b;

    #[inline(always)]
    fn try_from_account_view<'a, 'b, 'c, 'd>(
        account_view: &'ix AccountView,
        _: Self::Meta<'a, 'b, 'c, 'd>,
    ) -> Result<Self>
    where
        'ix: 'a + 'd,
        'd: 'c,
        'c: 'b,
    {
        Ok(UncheckedAccount { account_view })
    }
}

impl<'ix> ToAccountView<'ix> for UncheckedAccount<'ix> {
    #[inline(always)]
    fn to_account_view(&self) -> &'ix AccountView {
        self.account_view
    }
}

impl core::ops::Deref for UncheckedAccount<'_> {
    type Target = AccountView;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.account_view
    }
}

impl WritableAllowed for UncheckedAccount<'_> {}
