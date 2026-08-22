// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use crate::{FromAccountView, NoMeta, WritableAllowed};
use core::ops::Deref;
use hayabusa_common::{address_eq, AccountView, Address};
use hayabusa_errors::{ErrorCode, ProgramError, Result};
use hayabusa_meta_attribute_macro::meta;
use hayabusa_utility::{error_msg, hint::unlikely};

pub struct CheckedAddress<T> {
    pub account: T,
}

unsafe impl<'ix, 'a, 'b, 'c, 'd, T> FromAccountView<'ix> for CheckedAddress<T>
where
    T: FromAccountView<'ix, Meta<'a, 'b, 'c, 'd> = NoMeta>,
    'ix: 'a + 'd,
    'd: 'c,
    'c: 'b,
{
    type Meta<'e, 'f, 'g, 'h>
        = CheckedAddressMeta<'e>
    where
        'ix: 'e + 'h,
        'h: 'g,
        'g: 'f;

    #[inline(always)]
    fn try_from_account_view<'e, 'f, 'g, 'h>(
        account_view: &'ix AccountView,
        meta: Self::Meta<'e, 'f, 'g, 'h>,
    ) -> Result<Self>
    where
        'ix: 'e + 'h,
        'h: 'g,
        'g: 'f,
    {
        if unlikely(!address_eq(account_view.address(), meta.address)) {
            error_msg!(
                "CheckedAddress::try_from_account_view: invalid account address.",
                ErrorCode::InvalidAccount,
            );
        }

        Ok(Self {
            account: T::try_from_account_view(account_view, NoMeta)?,
        })
    }
}

impl<'ix, T> Deref for CheckedAddress<T>
where
    T: FromAccountView<'ix>,
{
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.account
    }
}

impl<'ix, T: FromAccountView<'ix>> WritableAllowed for CheckedAddress<T> {}

#[meta]
pub struct CheckedAddressMeta<'a> {
    pub address: &'a Address,
}
