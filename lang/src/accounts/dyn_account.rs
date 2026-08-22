// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

use core::ops::Deref;

use crate::prelude::*;

pub struct DynAccount<'view, D: DynAccountDispatch<'view>>(ValidatedView<'view, D::AccountType>);

impl<'view, D: DynAccountDispatch<'view>> DynAccount<'view, D> {
    #[inline(always)]
    pub fn ty(&self) -> D::AccountType {
        self.0 .1
    }

    #[inline(always)]
    pub fn build(&self) -> Result<Ref<'view, D::Object>> {
        D::build(self.0)
    }

    /// # Safety
    /// This method is unsafe because it is possible to alias the
    /// underlying account memory.
    #[inline(always)]
    pub unsafe fn build_untracked(&self) -> &'view D::Object {
        D::build_untracked(self.0)
    }
}

impl<'view, D: DynAccountDispatch<'view>> Mut<DynAccount<'view, D>> {
    #[inline(always)]
    pub fn build_mut(&self) -> Result<RefMut<'view, D::Object>> {
        D::build_mut(self.0)
    }

    /// # Safety
    /// This method is unsafe because it is possible to alias the
    /// underlying account memory.
    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn build_mut_untracked(&self) -> &'view mut D::Object {
        D::build_mut_untracked(self.0)
    }
}

impl<'view, D: DynAccountDispatch<'view>> ParseAccount<'view> for DynAccount<'view, D> {
    type Meta<'a, 'b, 'c>
        = NoMeta
    where
        'view: 'a + 'c,
        'c: 'b,
        'b: 'a;

    #[inline(always)]
    fn parse<'a, 'b, 'c>(
        view: AccountView<'view>,
        _: &mut Self::Meta<'a, 'b, 'c>,
    ) -> Result<DynAccount<'view, D>>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a,
    {
        let view = D::validate_account(view)?;

        Ok(DynAccount(view))
    }
}

pub trait DynAccountDispatch<'view> {
    type Object: ?Sized;
    type AccountType: Copy;

    fn build(view: ValidatedView<'view, Self::AccountType>) -> Result<Ref<'view, Self::Object>>;

    /// # Safety
    /// This method is unsafe because it is possible to alias the
    /// underlying account memory.
    unsafe fn build_untracked(view: ValidatedView<'view, Self::AccountType>)
        -> &'view Self::Object;

    fn build_mut(
        view: ValidatedView<'view, Self::AccountType>,
    ) -> Result<RefMut<'view, Self::Object>>;

    /// # Safety
    /// This method is unsafe because it is possible to alias the
    /// underlying account memory.
    unsafe fn build_mut_untracked(
        view: ValidatedView<'view, Self::AccountType>,
    ) -> &'view mut Self::Object;

    fn validate_account(
        view: AccountView<'view>,
    ) -> Result<ValidatedView<'view, Self::AccountType>>;
}

#[derive(Clone, Copy)]
pub struct ValidatedView<'view, A: Copy>(AccountView<'view>, A);

impl<'view, A: Copy> ValidatedView<'view, A> {
    pub fn marker(&self) -> A {
        self.1
    }

    /// # Safety
    /// You must manually ensure the invariants of this type are upheld.
    /// Invariants are dependent on the specific use case.
    pub unsafe fn new(view: AccountView<'view>, marker: A) -> ValidatedView<'view, A> {
        ValidatedView(view, marker)
    }
}

impl<'view, A: Copy> ToAccountView<'view> for ValidatedView<'view, A> {
    #[inline(always)]
    fn to_account_view(&self) -> AccountView<'view> {
        self.0
    }
}

impl<'view, A: Copy> Deref for ValidatedView<'view, A> {
    type Target = AccountView<'view>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

unsafe impl<'view, D: DynAccountDispatch<'view>> WritableAllowed for DynAccount<'view, D> {}

