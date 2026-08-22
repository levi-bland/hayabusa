// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

use crate::prelude::*;

#[repr(C)]
#[derive(Clone)]
pub struct AccountMeta<'a> {
    /// Address of the account.
    pub address: &'a Address,
    /// Indicates whether the account is mutable/writable.
    pub is_writable: bool,
    /// Indicates whether the account signed the instruction or not.
    pub is_signer: bool,
}

impl<'a> AccountMeta<'a> {
    #[inline(always)]
    pub const fn new(address: &'a Address, is_writable: bool, is_signer: bool) -> Self {
        Self {
            address,
            is_writable,
            is_signer,
        }
    }

    #[inline(always)]
    pub const fn readonly(address: &'a Address) -> Self {
        Self::new(address, false, false)
    }

    #[inline(always)]
    pub const fn writable(address: &'a Address) -> Self {
        Self::new(address, true, false)
    }

    #[inline(always)]
    pub const fn readonly_signer(address: &'a Address) -> Self {
        Self::new(address, false, true)
    }

    #[inline(always)]
    pub const fn writable_signer(address: &'a Address) -> Self {
        Self::new(address, true, true)
    }
}

impl<'view> From<AccountView<'view>> for AccountMeta<'view> {
    #[inline(always)]
    fn from(view: AccountView<'view>) -> Self {
        AccountMeta::new(view.address(), view.is_writable(), view.is_signer())
    }
}
