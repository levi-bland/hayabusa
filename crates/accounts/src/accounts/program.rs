// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use crate::{FromAccountView, NoMeta, ToAccountView};
use core::ops::Deref;
use hayabusa_common::{address_eq, AccountView, Address, ProgramId};
use hayabusa_errors::{ErrorCode, ProgramError, Result};
use hayabusa_utility::{error_msg, hint::unlikely};

pub struct Program<'ix, T>
where
    T: ProgramId,
{
    pub account_view: &'ix AccountView,
    _phantom: core::marker::PhantomData<T>,
}

unsafe impl<'ix, T> FromAccountView<'ix> for Program<'ix, T>
where
    T: ProgramId,
{
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
        if unlikely(!account_view.executable()) {
            error_msg!(
                "Program::try_from_account_view: program account is not executable.",
                ErrorCode::ProgramAccountNotExecutable,
            );
        }

        if unlikely(!address_eq(account_view.address(), &T::ID)) {
            error_msg!(
                "Program::try_from_account_view: program ID mismatch",
                ProgramError::IncorrectProgramId,
            );
        }

        Ok(Program {
            account_view,
            _phantom: core::marker::PhantomData,
        })
    }
}

impl<'ix, T> ToAccountView<'ix> for Program<'ix, T>
where
    T: ProgramId,
{
    #[inline(always)]
    fn to_account_view(&self) -> &'ix AccountView {
        self.account_view
    }
}

impl<'ix, T: ProgramId> Deref for Program<'ix, T> {
    type Target = AccountView;

    #[inline(always)]
    fn deref(&self) -> &'ix Self::Target {
        self.account_view
    }
}

pub struct System;

impl ProgramId for System {
    const ID: Address = hayabusa_system_program::ID;
}
