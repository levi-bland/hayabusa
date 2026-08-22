// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use crate::{FromAccountView, NoMeta, ToAccountView, WritableAllowed};
use core::ops::Deref;
use hayabusa_common::AccountView;
use hayabusa_errors::{ErrorCode, ProgramError, Result};
use hayabusa_utility::{error_msg, hint::unlikely};

pub struct SystemAccount<'ix> {
    pub account_view: &'ix AccountView,
}

unsafe impl<'ix> FromAccountView<'ix> for SystemAccount<'ix> {
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
        if unlikely(!account_view.owned_by(&hayabusa_system_program::ID)) {
            error_msg!(
                "SystemAccount::try_from_account_view: invalid account owner, must be system program",
                ErrorCode::InvalidAccount,
            );
        }

        Ok(SystemAccount { account_view })
    }
}

impl<'ix> ToAccountView<'ix> for SystemAccount<'ix> {
    #[inline(always)]
    fn to_account_view(&self) -> &'ix AccountView {
        self.account_view
    }
}

impl WritableAllowed for SystemAccount<'_> {}

impl<'ix> Deref for SystemAccount<'ix> {
    type Target = AccountView;

    #[inline(always)]
    fn deref(&self) -> &'ix Self::Target {
        self.account_view
    }
}
