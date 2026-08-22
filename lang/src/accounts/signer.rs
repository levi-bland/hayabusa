// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use crate::prelude::*;
use core::{marker::PhantomData, ops::Deref};

#[derive(Clone)]
pub struct Signer<'view, T = AccountView<'view>> {
    inner: T,
    __phantom: PhantomData<AccountView<'view>>,
}

impl<'view> ParseAccount<'view> for Signer<'view, AccountView<'view>> {
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
    ) -> Result<Signer<'view, AccountView<'view>>>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a,
    {
        if hint::unlikely(!view.is_signer()) {
            err!(
                "Signer::parse: account is not a signer.",
                ErrorCode::AccountNotSigner,
            );
        }

        Ok(Signer {
            inner: view,
            __phantom: PhantomData,
        })
    }

    #[inline(always)]
    unsafe fn parse_unchecked<'a, 'b, 'c>(view: AccountView<'view>, _: &mut Self::Meta<'a, 'b, 'c>) -> Result<Self>
    where 
        'view: 'c,
        'c: 'b,
        'b: 'a,
    {
        Ok(Signer {
            inner: view,
            __phantom: PhantomData,
        })
    }
}

impl<'view, T: ParseAccount<'view> + ToAccountView<'view>> ParseAccount<'view>
    for Signer<'view, T>
{
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
    ) -> Result<Signer<'view, T>>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a,
    {
        if hint::unlikely(!view.is_signer()) {
            err!(
                "Signer::parse: account is not a signer.",
                ErrorCode::AccountNotSigner,
            );
        }

        Ok(Signer {
            inner: T::parse(view, meta)?,
            __phantom: PhantomData,
        })
    }

    #[inline(always)]
    unsafe fn parse_unchecked<'a, 'b, 'c>(view: AccountView<'view>, meta: &mut Self::Meta<'a, 'b, 'c>) -> Result<Self>
    where 
        'view: 'c,
        'c: 'b,
        'b: 'a,
    {
        Self::parse(view, meta)
    }
}

impl<'view, T: ToAccountView<'view>> ToAccountView<'view> for Signer<'view, T> {
    #[inline(always)]
    fn to_account_view(&self) -> AccountView<'view> {
        self.inner.to_account_view()
    }
}

impl<'view> ToAccountView<'view> for Signer<'view, AccountView<'view>> {
    #[inline(always)]
    fn to_account_view(&self) -> AccountView<'view> {
        self.inner
    }
}

unsafe impl<T> WritableAllowed for Signer<'_, T> {}

unsafe impl<T> PdaAllowed for Signer<'_, T> {}

impl<'view, T> Deref for Signer<'view, T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'view> AccountAddress<'view> for Signer<'view, AccountView<'view>> {
    #[inline(always)]
    fn address(&self) -> &'view Address {
        self.inner.address()
    }

    #[inline(always)]
    fn address_owned(&self) -> Address {
        *self.inner.address()
    }
}
