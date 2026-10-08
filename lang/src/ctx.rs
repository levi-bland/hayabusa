// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use crate::prelude::*;

pub struct Ctx<'view, T: Bumps> {
    pub program_id: &'view Address,
    pub accounts: &'view mut T,
    pub remaining_accounts: &'view [AccountView<'view>],
    pub bumps: T::Bumps,
}

impl<'view, T> Ctx<'view, T>
where
    T: Bumps + ParseAccounts<'view, T::Bumps>,
{
    #[inline(always)]
    pub fn new(
        program_id: &'view Address,
        accounts: &'view mut T,
        remaining_accounts: &'view [AccountView<'view>],
        bumps: T::Bumps,
    ) -> Self {
        Self {
            program_id,
            accounts,
            remaining_accounts,
            bumps,
        }
    }

    #[inline(always)]
    pub fn remaining_iter(&self) -> AccountIter<'view> {
        AccountIter::new(self.remaining_accounts)
    }
}
