// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use core::marker::PhantomData;

use crate::{
    account_view::AccountView,
    accounts::executable::Executable,
    error::ErrorCode,
    hint,
    traits::{Ids, NoMeta, ParseAccount, ToAccountView},
    Result,
};
use hayabusa_err_macro::err;
use solana_program_error::ProgramError;

pub struct Interface<'view, T: Ids> {
    view: AccountView<'view>,
    __phantom: PhantomData<T>,
}

impl<'view, T: Ids> ParseAccount<'view> for Interface<'view, T> {
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
    ) -> Result<Interface<'view, T>>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a,
    {
        let _: Executable<AccountView<'_>> = Executable::parse(view, &mut NoMeta)?;

        if hint::unlikely(!T::IDS.contains(view.address())) {
            err!(
                "Interface::parse: account is not a valid program in the interface",
                ErrorCode::InvalidInterfaceProgram,
            );
        }

        Ok(Interface {
            view,
            __phantom: PhantomData,
        })
    }
}

impl<'view, T: Ids> ToAccountView<'view> for Interface<'view, T> {
    #[inline(always)]
    fn to_account_view(&self) -> AccountView<'view> {
        self.view
    }
}
