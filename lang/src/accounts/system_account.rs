// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use crate::{prelude::*, system_program::System};
use core::ops::Deref;

#[derive(Clone, Copy)]
pub struct SystemAccount<'view> {
    view: AccountView<'view>,
}

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
        if hint::unlikely(!view.owned_by(&System::ID)) {
            err!(
                "SystemAccount::parse: account is not owned by the system program",
                ErrorCode::InvalidAccountOwner,
            );
        }

        Ok(SystemAccount { view })
    }

    #[inline(always)]
    unsafe fn parse_unchecked<'a, 'b, 'c>(view: AccountView<'view>, _: &mut Self::Meta<'a, 'b, 'c>) -> Result<Self>
    where 
        'view: 'c,
        'c: 'b,
        'b: 'a,
    {
        Ok(SystemAccount { view })
    }
}

unsafe impl PdaAllowed for SystemAccount<'_> {}

unsafe impl WritableAllowed for SystemAccount<'_> {}

impl<'view> ToAccountView<'view> for SystemAccount<'view> {
    #[inline(always)]
    fn to_account_view(&self) -> AccountView<'view> {
        self.view
    }
}

impl<'view> Deref for SystemAccount<'view> {
    type Target = AccountView<'view>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.view
    }
}

impl<'view> AccountAddress<'view> for SystemAccount<'view> {
    #[inline(always)]
    fn address(&self) -> &'view Address {
        self.view.address()
    }

    #[inline(always)]
    fn address_owned(&self) -> Address {
        *self.view.address()
    }
}
