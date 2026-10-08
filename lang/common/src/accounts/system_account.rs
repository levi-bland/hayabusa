// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_view::AccountView,
    address::Address,
    error::ErrorCode,
    hint,
    system_program::System,
    traits::{
        AddressTrait, Id, NoMeta, ParseAccount, ToAccountView, __PdaAllowed, __WritableAllowed,
    },
    Result,
};
use core::ops::Deref;
use hayabusa_err_macro::err;
use solana_program_error::ProgramError;

#[derive(Clone, Copy)]
pub struct SystemAccount<'view>(AccountView<'view>);

impl<'view> ParseAccount<'view> for SystemAccount<'view> {
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
    ) -> Result<SystemAccount<'view>>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a,
    {
        if hint::unlikely(!view.owned_by(System::ID)) {
            err!(
                "SystemAccount::parse: account is not owned by the system program",
                ErrorCode::InvalidAccountOwner,
            );
        }

        Ok(SystemAccount(view))
    }
}

unsafe impl __PdaAllowed for SystemAccount<'_> {}

unsafe impl __WritableAllowed for SystemAccount<'_> {}

impl<'view> ToAccountView<'view> for SystemAccount<'view> {
    #[inline(always)]
    fn to_account_view(&self) -> AccountView<'view> {
        self.0
    }
}

impl<'view> Deref for SystemAccount<'view> {
    type Target = AccountView<'view>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'view> AddressTrait<'view> for SystemAccount<'view> {
    #[inline(always)]
    fn address(&self) -> &'view Address {
        self.0.address()
    }

    #[inline(always)]
    fn address_owned(&self) -> Address {
        *self.0.address()
    }
}
