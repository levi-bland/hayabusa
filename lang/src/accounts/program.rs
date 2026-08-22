// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use crate::prelude::*;
use core::{marker::PhantomData, ops::Deref};

#[derive(Clone, Copy)]
pub struct Program<'view, T: ProgramId> {
    view: AccountView<'view>,
    __phantom: PhantomData<T>,
}

impl<'view, T: ProgramId> ParseAccount<'view> for Program<'view, T> {
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
        if hint::unlikely(!view.executable()) {
            err!(
                "Program::parse: account is not executable",
                ErrorCode::ProgramAccountNotExecutable,
            );
        }

        if hint::unlikely(!address_eq(view.address(), &T::ID)) {
            err!(
                "Program::parse: incorrect program ID",
                ErrorCode::InvalidProgram,
            );
        }

        Ok(Program {
            view,
            __phantom: PhantomData::<T>,
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

impl<'view, T: ProgramId> ToAccountView<'view> for Program<'view, T> {
    #[inline(always)]
    fn to_account_view(&self) -> AccountView<'view> {
        self.view
    }
}

impl<'view, T: ProgramId> Deref for Program<'view, T> {
    type Target = AccountView<'view>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.view
    }
}
