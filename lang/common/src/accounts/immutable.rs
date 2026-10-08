// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use core::ops::Deref;

use crate::{account_view::AccountView, error::ErrorCode, hint, traits::ParseAccount, Result};
use hayabusa_err_macro::err;
use solana_program_error::ProgramError;

pub struct Immut<T>(T);

impl<'view, T: ParseAccount<'view>> ParseAccount<'view> for Immut<T> {
    type Meta<'a, 'b, 'c>
        = <T as ParseAccount<'view>>::Meta<'a, 'b, 'c>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a;

    #[inline(always)]
    fn parse<'a, 'b, 'c>(
        view: AccountView<'view>,
        meta: &mut Self::Meta<'a, 'b, 'c>,
    ) -> Result<Immut<T>>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a,
    {
        if hint::unlikely(view.is_writable()) {
            err!(
                "Immut::parse: account not immutable",
                ErrorCode::AccountNotImmutable,
            );
        }

        Ok(Immut(T::parse(view, meta)?))
    }
}

impl<T> Deref for Immut<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
