// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_view::AccountView,
    error::ErrorCode,
    hint,
    traits::{ParseAccount, __WritableAllowed},
    Result,
};
use core::ops::Deref;
use hayabusa_err_macro::err;
use solana_program_error::ProgramError;

/// Used to denote where an account must be mutable, i.e. `Mut<Account<'_, T>>`
pub struct Mut<T: __WritableAllowed>(T);

impl<'view, T: ParseAccount<'view> + __WritableAllowed> ParseAccount<'view> for Mut<T> {
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
    ) -> Result<Mut<T>>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a,
    {
        if hint::unlikely(!view.is_writable()) {
            err!(
                "Mut::parse: account not writable",
                ErrorCode::AccountNotWritable,
            );
        }

        Ok(Mut(T::parse(view, meta)?))
    }
}

impl<T: __WritableAllowed> Deref for Mut<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
