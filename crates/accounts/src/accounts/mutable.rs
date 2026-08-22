// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

use crate::{FromAccountView, WritableAllowed};
use core::ops::Deref;
use hayabusa_common::AccountView;
use hayabusa_errors::{ErrorCode, ProgramError, Result};
use hayabusa_utility::{error_msg, hint::unlikely};

pub struct Mut<T>(T);

unsafe impl<'ix, T> FromAccountView<'ix> for Mut<T>
where
    T: FromAccountView<'ix> + WritableAllowed,
{
    type Meta<'a, 'b, 'c, 'd>
        = T::Meta<'a, 'b, 'c, 'd>
    where
        'ix: 'a + 'd,
        'd: 'c,
        'c: 'b;

    #[inline(always)]
    fn try_from_account_view<'a, 'b, 'c, 'd>(
        account_view: &'ix AccountView,
        meta: Self::Meta<'a, 'b, 'c, 'd>,
    ) -> Result<Self>
    where
        'ix: 'a + 'd,
        'd: 'c,
        'c: 'b,
    {
        if unlikely(!account_view.is_writable()) {
            error_msg!(
                "Mut::try_from_account_view: account not writable",
                ErrorCode::AccountNotWritable,
            );
        }

        Ok(Mut(T::try_from_account_view(account_view, meta)?))
    }
}

impl<T> Deref for Mut<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
