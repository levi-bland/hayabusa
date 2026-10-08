// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use core::ops::Deref;

use crate::{
    account_view::AccountView,
    error::ErrorCode,
    hint,
    traits::{ParseAccount, ToAccountView},
    Result,
};
use hayabusa_err_macro::err;
use solana_program_error::ProgramError;

pub struct Executable<T>(T);

impl<'view, T: ParseAccount<'view>> ParseAccount<'view> for Executable<T> {
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
    ) -> Result<Executable<T>>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a,
    {
        if hint::unlikely(!view.executable()) {
            err!(
                "Executable::parse: account is not executable",
                ErrorCode::AccountNotExecutable,
            );
        }

        Ok(Executable(T::parse(view, meta)?))
    }
}

impl<'view, T: ToAccountView<'view>> ToAccountView<'view> for Executable<T> {
    #[inline(always)]
    fn to_account_view(&self) -> AccountView<'view> {
        self.0.to_account_view()
    }
}

impl<T> Deref for Executable<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
