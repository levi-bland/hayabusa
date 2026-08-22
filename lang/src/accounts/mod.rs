// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

pub mod account;
pub mod dyn_account;
pub mod init;
pub mod interface;
pub mod mutable;
pub mod pda;
pub mod program;
pub mod signer;
pub mod system_account;
pub mod sysvar;
pub mod unchecked_account;

use crate::prelude::*;

impl<'view, T: ParseAccount<'view>> ParseAccount<'view> for Option<T> {
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
    ) -> Result<Option<T>>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a,
    {
        if hint::unlikely(address_eq(view.address(), &System::ID)) {
            return Ok(None);
        }

        Ok(Some(T::parse(view, meta)?))
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
