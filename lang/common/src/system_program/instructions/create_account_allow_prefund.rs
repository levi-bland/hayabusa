// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use crate::prelude::*;

/// Create a new account allowing the account to be prefunded.
///
/// This instruction is identical to `CreateAccount` except
/// that it allows the account being created to already have
/// lamports in it.
pub struct CreateAccountAllowPrefund<'view, 'id> {
    /// The account being created.
    pub to: AccountView<'view>,
    /// Number of bytes of memory to allocate.
    pub space: u64,
    /// Address of the program that will own the account.
    pub owner: &'id Address,
    /// Funding for the new account.
    ///
    /// If `None`, the instruction will not transfer any
    /// lamports to the new account.
    pub funding: Option<Funding<'view>>,
}

/// Funding lamports to transfer into a newly created account.
pub struct Funding<'view> {
    /// Funding account.
    pub from: AccountView<'view>,
    /// Number of lamports to trasnfer to the new account.
    pub lamports: u64,
}
