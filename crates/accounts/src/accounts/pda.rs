// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

#![allow(unused)]

use crate::{accounts::account::Account, FromAccountView, Mut, WritableAllowed};
use core::{marker::PhantomData, ops::Deref};
use hayabusa_cast::Cast;
use hayabusa_common::{AccountView, Ref, RefMut};
use hayabusa_errors::Result;
use hayabusa_pda::CheckSeeds;

pub struct Pda<T>(T);

unsafe impl<'ix, T: CheckSeeds + Cast> FromAccountView<'ix> for Pda<Account<'ix, T>> {
    type Meta<'a, 'b, 'c, 'd>
        = <T as CheckSeeds>::Meta<'a>
    where
        'ix: 'a + 'd,
        'd: 'c,
        'c: 'b;

    fn try_from_account_view<'a, 'b, 'c, 'd>(
        view: &'ix AccountView,
        meta: Self::Meta<'a, 'b, 'c, 'd>,
    ) -> Result<Self>
    where
        'ix: 'a + 'd,
        'd: 'c,
        'c: 'b,
    {
        T::check_len(view.data_len())?;
        T::check_owner(view.address())?;
        T::check_discriminator(view)?;

        // SAFETY: the above ensures the cast is sound, and there are guaranteed to be no other
        // valid references to the underlying data at the point this is called in account construction
        let account = unsafe { T::cast_unchecked(view) };
        account.check_pda_seeds(view.address(), meta)?;

        Ok(Pda(Account {
            view,
            _phantom: PhantomData,
        }))
    }
}

impl<T> Deref for Pda<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
