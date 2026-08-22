// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use crate::prelude::*;
use core::ops::Deref;

/// Used to denote where an account must be mutable, i.e. `Mut<Account<'_, T>>`
#[derive(Clone, Copy)]
pub struct Mut<T>(T);

impl<'view, T: ParseAccount<'view> + WritableAllowed> ParseAccount<'view> for Mut<T> {
    type Meta<'a, 'b, 'c>
        = <T as ParseAccount<'view>>::Meta<'a, 'b, 'c>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a;

    #[inline(always)]
    fn parse<'a, 'b, 'c>(view: AccountView<'view>, meta: &mut Self::Meta<'a, 'b, 'c>) -> Result<Mut<T>>
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

    #[inline(always)]
    unsafe fn parse_unchecked<'a, 'b, 'c>(view: AccountView<'view>, meta: &mut Self::Meta<'a, 'b, 'c>) -> Result<Self>
    where 
        'view: 'c,
        'c: 'b,
        'b: 'a,
    {
        if hint::unlikely(!view.is_writable()) {
            err!(
                "Mut::parse_unchecked: account not writable",
                ErrorCode::AccountNotWritable,
            );
        }

        Ok(Mut(T::parse_unchecked(view, meta)?))
    }
}

impl<T> Deref for Mut<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
