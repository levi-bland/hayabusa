// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_view::AccountView,
    accounts::{mutable::Mut, pda::Pda},
    traits::{
        DynamicPdaValidate, EnumAccountDispatch, NoMeta, ParseAccount, __EnumAccountMarker,
        __WritableAllowed,
    },
    Result,
};

pub struct EnumAccount<E: __EnumAccountMarker>(E);

impl<'view, E: EnumAccountDispatch<'view>> ParseAccount<'view> for EnumAccount<E> {
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
    ) -> Result<EnumAccount<E>>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a,
    {
        E::validate_account(view).map(|e| EnumAccount(e))
    }
}

impl<'view, E> ParseAccount<'view> for Pda<EnumAccount<E>>
where
    E: EnumAccountDispatch<'view> + DynamicPdaValidate<'view>,
{
    type Meta<'a, 'b, 'c>
        = <E as DynamicPdaValidate<'view>>::Meta
    where
        'view: 'c,
        'c: 'b,
        'b: 'a;

    #[inline(always)]
    fn parse<'a, 'b, 'c>(
        view: AccountView<'view>,
        meta: &mut Self::Meta<'a, 'b, 'c>,
    ) -> Result<Pda<EnumAccount<E>>>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a,
    {
        let e = E::validate_account(view)?;

        // SAFETY: The invariantes required by `DynamicPdaValidate::validate_pda` are checked by `E::validate_account`.
        unsafe { E::validate_pda(view, meta)? };

        Ok(Pda(EnumAccount(e)))
    }
}

impl<'view, E: EnumAccountDispatch<'view>> EnumAccount<E> {
    #[inline(always)]
    pub fn load(&self) -> Result<E::RefEnum> {
        self.0.load()
    }

    #[inline(always)]
    pub unsafe fn load_untracked(&self) -> E::UntrackedRefEnum {
        self.0.load_untracked()
    }
}

impl<'view, E: EnumAccountDispatch<'view>> Mut<EnumAccount<E>> {
    #[inline(always)]
    pub fn load_mut(&self) -> Result<E::RefMutEnum> {
        self.0.load_mut()
    }

    #[inline(always)]
    pub unsafe fn load_mut_untracked(&self) -> E::UntrackedMutEnum {
        self.0.load_mut_untracked()
    }
}

unsafe impl<'view, E: EnumAccountDispatch<'view>> __WritableAllowed for EnumAccount<E> {}
