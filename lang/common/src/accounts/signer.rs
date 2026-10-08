// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use hayabusa_err_macro::err;
use solana_program_error::ProgramError;

use crate::{
    account_view::AccountView,
    address::Address,
    error::ErrorCode,
    hint,
    traits::{AddressTrait, NoMeta, ParseAccount, ToAccountView, __PdaAllowed, __WritableAllowed},
    Result,
};

pub struct Signer<'view>(AccountView<'view>);

impl<'view> ParseAccount<'view> for Signer<'view> {
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
        if hint::unlikely(!view.is_signer()) {
            err!(
                "Signer::parse: account is not a signer.",
                ErrorCode::AccountNotSigner,
            );
        }

        Ok(Signer(view))
    }
}

impl<'view> ToAccountView<'view> for Signer<'view> {
    #[inline(always)]
    fn to_account_view(&self) -> AccountView<'view> {
        self.0
    }
}

impl<'view> AddressTrait<'view> for Signer<'view> {
    #[inline(always)]
    fn address(&self) -> &'view Address {
        self.0.address()
    }

    #[inline(always)]
    fn address_owned(&self) -> Address {
        *self.0.address()
    }
}

unsafe impl __WritableAllowed for Signer<'_> {}

unsafe impl __PdaAllowed for Signer<'_> {}

pub mod wrapper {
    use core::ops::Deref;

    use hayabusa_err_macro::err;
    use solana_program_error::ProgramError;

    use crate::{
        account_view::AccountView,
        error::ErrorCode,
        hint,
        traits::{ParseAccount, ToAccountView, __PdaAllowed, __WritableAllowed},
        Result,
    };

    pub struct Signer<T>(T);

    impl<'view, T: ParseAccount<'view>> ParseAccount<'view> for Signer<T> {
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
        ) -> Result<Self>
        where
            'view: 'c,
            'c: 'b,
            'b: 'a,
        {
            if hint::unlikely(!view.is_signer()) {
                err!(
                    "Signer::parse: account is not a signer.",
                    ErrorCode::AccountNotSigner,
                );
            }

            Ok(Signer(T::parse(view, meta)?))
        }
    }

    impl<'view, T: ToAccountView<'view>> ToAccountView<'view> for Signer<T> {
        #[inline(always)]
        fn to_account_view(&self) -> AccountView<'view> {
            self.0.to_account_view()
        }
    }

    impl<T> Deref for Signer<T> {
        type Target = T;

        #[inline(always)]
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    unsafe impl<T> __WritableAllowed for Signer<T> {}

    unsafe impl<T> __PdaAllowed for Signer<T> {}
}
