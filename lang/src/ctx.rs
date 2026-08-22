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

pub struct AccountIter<'view> {
    /// Current position in the account slice. Points to the next account to return.
    ptr: *const AccountView<'view>,
    /// One-past-the-end pointer. This is a valid (but not dereferenceable) pointer
    /// marking the exclusive end of the iteration range. When `ptr == end`, iteration is complete.
    end: *const AccountView<'view>,
}

impl<'view> AccountIter<'view> {
    /// Create a new `AccountIter` from an `AccountView` slice
    #[inline(always)]
    pub fn new(slice: &'view [AccountView<'view>]) -> AccountIter<'view> {
        let ptr = slice.as_ptr();
        // SAFETY: Adding slice.len() to the base pointer produces a pointer one past
        // the last element, which is a valid (but not dereferenceable) pointer.
        // This is the standard half-open range [begin, end) pattern.
        let end = unsafe { ptr.add(slice.len()) };

        Self { ptr, end }
    }

    /// Advance the iterator and get the next view
    #[allow(clippy::should_implement_trait)]
    #[inline(always)]
    pub fn next(&mut self) -> Result<AccountView<'view>> {
        if hint::unlikely(self.ptr == self.end) {
            err!(
                "AccountIter::next: no accounts remaining",
                ErrorCode::InvalidAccount,
            );
        }

        // SAFETY: We just checked that ptr < end, so ptr points to a valid element
        // within the original slice.
        unsafe { Ok(self.next_unchecked()) }
    }

    #[inline(always)]
    pub unsafe fn next_unchecked(&mut self) -> AccountView<'view> {
        let current = *self.ptr;
        self.ptr = self.ptr.add(1);
        current
    }

    /// Collect the remaining accounts into a slice
    #[inline(always)]
    pub fn remaining(&self) -> &'view [AccountView<'view>] {
        let len = self.remaining_len();
        // SAFETY: ptr points to valid elements within the original slice,
        // and len is the correct number of remaining elements (ptr..end).
        unsafe { core::slice::from_raw_parts(self.ptr, len) }
    }

    /// Remaining number of elements
    #[inline(always)]
    pub fn remaining_len(&self) -> usize {
        // SAFETY: end >= ptr by construction (both derived from the same slice)
        unsafe { self.end.offset_from(self.ptr) as usize }
    }

    /// Check if a certain number of accounts remain.
    #[inline(always)]
    pub fn has_remaining(&self, num_accounts: usize) -> Result<()> {
        if hint::unlikely(self.remaining_len() < num_accounts) {
            err!(
                "AccountIter::has_remaining: not enough accounts remaining",
                ErrorCode::InvalidAccount,
            );
        }

        Ok(())
    }
}
