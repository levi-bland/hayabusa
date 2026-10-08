// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_view::{AccountView, Ref, RefMut},
    accounts::mutable::Mut,
    traits::{NoMeta, ParseAccount, ToAccountView, ValidatedView, __WritableAllowed},
    Result,
};

pub struct DynAccount<'view, D: DynAccountDispatch<'view>>(ValidatedView<'view, D::AccountType>);

impl<'view, D: DynAccountDispatch<'view>> DynAccount<'view, D> {
    #[inline(always)]
    pub fn ty(&self) -> D::AccountType {
        self.0.marker()
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
        // SAFETY: [`Mut`] guarantees the underlying [`AccountView`] is mutable.
        unsafe { D::build_mut(self.0) }
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
        'view: 'c,
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

impl<'view, D: DynAccountDispatch<'view>> ToAccountView<'view> for DynAccount<'view, D> {
    #[inline(always)]
    fn to_account_view(&self) -> AccountView<'view> {
        self.0.to_account_view()
    }
}

/// ## Safety
///
/// This trait is unsafe to implement manually because it is possible to alias
/// or return a corrupted misrepresentation of the account memory, i.e. by returning
/// an incorrect account type marker in `validate_account`. Use the [`impl_dyn_account_dispatch!`]
/// macro to implement this trait safely.
///
/// ```ignore
/// impl_dyn_account_dispatch!(MyAccountDispatch, MyAccountType1, MyAccountType2, MyAccountType3, ...);
/// ```
pub unsafe trait DynAccountDispatch<'view> {
    type Object: ?Sized;
    type AccountType: Copy;

    fn build(view: ValidatedView<'view, Self::AccountType>) -> Result<Ref<'view, Self::Object>>;

    /// # Safety
    /// This method is unsafe because it is possible to alias the
    /// underlying account memory.
    unsafe fn build_untracked(view: ValidatedView<'view, Self::AccountType>)
        -> &'view Self::Object;

    /// # Safety
    /// This method does NOT check that the underlying [`AccountView`] is mutable,
    /// if you attempt to access the returned mutable pointer and the account is immutable,
    /// you will cause a runtime panic.
    unsafe fn build_mut(
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

unsafe impl<'view, D: DynAccountDispatch<'view>> __WritableAllowed for DynAccount<'view, D> {}
