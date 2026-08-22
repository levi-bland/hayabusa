// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use crate::{prelude::Address, Result};
use core::{
    marker::PhantomData,
    mem::{size_of, ManuallyDrop},
    ops::{Deref, DerefMut},
    ptr::{addr_of_mut, write, NonNull},
    slice::{from_raw_parts, from_raw_parts_mut},
};
use solana_program_error::ProgramError;

/// Maximum number of bytes a program may add to an account during a
/// single top-level instruction.
pub const MAX_PERMITTED_DATA_INCREASE: usize = 1_024 * 10;

/// Value to indicate that an account is not borrowed.
///
/// This value is the same as `solana_program_entrypoint::NON_DUP_MARKER`.
pub const NOT_BORROWED: u8 = u8::MAX;

/// Raw account data.
///
/// This struct is wrapped by [`AccountView`], which provides safe access
/// to account information. At runtime, the account's data is serialized
/// directly after the `Account` struct in memory, with its size specified
/// by [`RuntimeAccount::data_len`].
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct RuntimeAccount {
    /// Borrow state for account data.
    ///
    /// This reuses the memory reserved for the duplicate flag in the
    /// account to track data borrows. It represents the numbers of
    /// borrows available. The value `0` indicates that the account
    /// data is mutably borrowed, while values between `2` and `255`
    /// indicate the number of immutable borrows that can still be
    /// allocated. An account's data can only be mutably borrowed
    /// when there are no other active borrows, i.e., when this value
    /// is equal to [`NOT_BORROWED`]
    pub borrow_state: u8,

    /// Indicates whether the transaction was signed by this account.
    pub is_signer: u8,

    /// Indicates whether the account is writable.
    pub is_writable: u8,

    /// Indicates whether this account represents a program.
    pub executable: u8,

    /// Padding for alignment.
    ///
    /// The value of this field is not directly used and always set to `0`.
    /// Entrypoint implementations may use this space for their own purposes,
    /// e.g., to track account resizing.
    pub padding: [u8; 4],

    /// Address of the account.
    pub address: Address,

    /// Program that owns this account. Modifiable by programs.
    pub owner: Address,

    /// The lamports in the account. Modifiable by programs.
    pub lamports: u64,

    /// Length of the data. Modifiable by programs.
    pub data_len: u64,
}

/// Wrapper struct for a `RuntimeAccount`.
///
/// This struct provides safe access to the data in a `RuntimeAccount`.
/// It is also used to track borrows of the account data, given that
/// an account can be "shared" across multiple `AccountView` instances.
///
/// `'view` ensures proper lifetime variance, otherwise returned references have the lifetime
/// of the `AccountView` wrapper itself, this is still sound but less correct and more
/// constraining.
///
/// # Invariants
///
/// - The `raw` pointer must be valid and point to memory containing a
///   `RuntimeAccount` struct, immediately followed by the account's data
///   region.
/// - The length of the account data must exactly match the value stored in
///   `RuntimeAccount::data_len`.
///
/// These conditions must always hold for any `AccountView` created from
/// a raw pointer.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct AccountView<'view> {
    /// Raw (pointer to) account data.
    ///
    /// Note that this is a pointer can be shared across multiple `AccountView`.
    raw: *mut RuntimeAccount,
    __phantom: PhantomData<&'view mut RuntimeAccount>,
}

impl<'view> AccountView<'view> {
    /// Creates a new [`AccountView`] for a given raw account pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the `raw` pointer is valid and points
    /// to memory containing a `RuntimeAccount` struct, immediately followed by
    /// the account's data region.
    #[inline(always)]
    pub unsafe fn new_unchecked(raw: *mut RuntimeAccount) -> Self {
        Self {
            raw,
            __phantom: PhantomData::<&'view mut RuntimeAccount>,
        }
    }

    /// Address of the account.
    #[inline(always)]
    pub fn address(&self) -> &'view Address {
        // SAFETY: The `raw` pointer is guaranteed to be valid.
        unsafe { &(*self.raw).address }
    }

    /// Return a reference to the address of the program that owns this account.
    ///
    /// For ownership checks, it is recommended to use the [`Self::owned_by`]
    /// method instead.
    ///
    /// # Important
    ///
    /// This method returns a reference to the owner field of the account, which
    /// can be modified by programs using [`Self::assign`]. It is the caller's
    /// responsibility to ensure that this reference is not used after the
    /// account owner has been changed.
    #[inline(always)]
    pub fn owner(&self) -> &'view Address {
        // SAFETY: The `raw` pointer is guaranteed to be valid.
        unsafe { &(*self.raw).owner }
    }

    /// Indicate whether the transaction was signed by this account.
    #[inline(always)]
    pub fn is_signer(&self) -> bool {
        // SAFETY: The `raw` pointer is guaranteed to be valid.
        unsafe { (*self.raw).is_signer != 0 }
    }

    /// Indicate whether the account is writable or not.
    #[inline(always)]
    pub fn is_writable(&self) -> bool {
        // SAFETY: The `raw` pointer is guaranteed to be valid.
        unsafe { (*self.raw).is_writable != 0 }
    }

    /// Indicate whether this account represents an executable program
    /// or not.
    #[inline(always)]
    pub fn executable(&self) -> bool {
        // SAFETY: The `raw` pointer is guaranteed to be valid.
        unsafe { (*self.raw).executable != 0 }
    }

    /// Return the size of the account data.
    #[inline(always)]
    pub fn data_len(&self) -> usize {
        // SAFETY: The `raw` pointer is guaranteed to be valid.
        unsafe { (*self.raw).data_len as usize }
    }

    /// Return the lamports in the account.
    #[inline(always)]
    pub fn lamports(&self) -> u64 {
        // SAFETY: The `raw` pointer is guaranteed to be valid.
        unsafe { (*self.raw).lamports }
    }

    /// Set the lamports in the account.
    #[inline(always)]
    pub fn set_lamports(&mut self, lamports: u64) {
        // SAFETY: The `raw` pointer is guaranteed to be valid.
        unsafe {
            (*self.raw).lamports = lamports;
        }
    }

    /// Indicates whether the account data is empty or not.
    ///
    /// An account is considered empty if the data length is zero.
    #[inline(always)]
    pub fn is_data_empty(&self) -> bool {
        // SAFETY: The `raw` pointer is guaranteed to be valid.
        self.data_len() == 0
    }

    /// Checks if the account is owned by the given program.
    #[inline(always)]
    pub fn owned_by(&self, program: &Address) -> bool {
        // SAFETY: The `raw` pointer is guaranteed to be valid.
        unsafe { (*self.raw).owner == *program }
    }

    /// Changes the owner of the account.
    ///
    /// # Safety
    ///
    /// It is undefined behavior to use this method while there is an active reference
    /// to the `owner` returned by [`Self::owner`].
    #[allow(clippy::clone_on_copy)]
    #[inline(always)]
    pub unsafe fn assign(&mut self, new_owner: &Address) {
        write(addr_of_mut!((*self.raw).owner), new_owner.clone());
    }

    /// Return `true` if the account data is borrowed in any form.
    #[inline(always)]
    pub fn is_borrowed(&self) -> bool {
        unsafe { (*self.raw).borrow_state != NOT_BORROWED }
    }

    /// Return `true` if the account data is mutably borrowed.
    #[inline(always)]
    pub fn is_borrowed_mut(&self) -> bool {
        unsafe { (*self.raw).borrow_state == 0 }
    }

    /// Returns an immutable reference to the data in the account.
    ///
    /// # Safety
    ///
    /// This method is unsafe because it does not return a `Ref`, thus leaving the borrow
    /// flag untouched. Useful when an instruction has verified non-duplicate accounts. This
    /// method can also alias memory and thus cause UB.
    #[inline(always)]
    pub unsafe fn borrow_unchecked(&self) -> &'view [u8] {
        from_raw_parts(self.data_ptr(), self.data_len())
    }

    /// Returns a mutable reference to the data in the account.
    ///
    /// # Safety
    ///
    /// This method is unsafe because it does not return a `RefMut`, thus leaving the borrow
    /// flag untouched. Useful when an instruction has verified non-duplicate accounts.
    /// This method can also alias memory and thus cause UB.
    #[allow(clippy::mut_from_ref)]
    #[inline(always)]
    pub unsafe fn borrow_mut_unchecked(&self) -> &'view mut [u8] {
        from_raw_parts_mut(self.data_mut_ptr(), self.data_len())
    }

    /// Tries to get an immutable reference to the account data, failing if the account
    /// is already mutably borrowed. It is recommended to not use this on SBF target
    /// in favour of `borrow`.
    pub fn try_borrow(&self) -> Result<Ref<'view, [u8]>> {
        // check if the account data can be borrowed
        self.check_borrow()?;

        let borrow_state = self.raw as *mut u8;
        // Use one immutable borrow for data by subtracting `1` from the data
        // borrow counter bits; we are guaranteed that there is at least one
        // immutable borrow available.
        //
        // SAFETY: The `borrow_state` is a mutable pointer to the borrow state
        // of the account, which is guaranteed to be valid.
        unsafe { *borrow_state -= 1 };

        // return the reference to data
        Ok(Ref {
            value: unsafe { NonNull::from(from_raw_parts(self.data_ptr(), self.data_len())) },
            state: unsafe { NonNull::new_unchecked(borrow_state) },
            __phantom: PhantomData,
        })
    }

    /// Tries to get a mutable reference to the account data, failing if the account
    /// is already borrowed in any form. It is recommended to not use this on SBF
    /// target in favour of `borrow_mut`.
    pub fn try_borrow_mut(&self) -> Result<RefMut<'view, [u8]>> {
        // check if the account data can be mutably borrowed
        self.check_borrow_mut()?;

        let borrow_state = self.raw as *mut u8;
        // Set the mutable data borrow bit to `0`; we are guaranteed that account
        // data is not already borrowed in any form.
        //
        // SAFETY: The `borrow_state` is a mutable pointer to the borrow state
        // of the account, which is guaranteed to be valid.
        unsafe { *borrow_state = 0 };

        // return the mutable reference to data
        Ok(RefMut {
            value: unsafe {
                NonNull::from(from_raw_parts_mut(self.data_mut_ptr(), self.data_len()))
            },
            state: unsafe { NonNull::new_unchecked(borrow_state) },
            __phantom: PhantomData,
        })
    }

    /// Check if it is possible to get an immutable reference to the account data,
    /// failing if the account is already mutably borrowed or there are not enough
    /// immutable borrows available.
    #[inline(always)]
    pub fn check_borrow(&self) -> Result<()> {
        // There must be at least one immutable borrow available.
        //
        // SAFETY: The `raw` pointer is guaranteed to be valid.
        if unsafe { (*self.raw).borrow_state } < 2 {
            return Err(ProgramError::AccountBorrowFailed);
        }

        Ok(())
    }

    /// Checks if it is possible to get a mutable reference to the account data,
    /// failing if the account is already borrowed in any form.
    #[inline(always)]
    pub fn check_borrow_mut(&self) -> Result<()> {
        // SAFETY: The `raw` pointer is guaranteed to be valid.
        if unsafe { (*self.raw).borrow_state } != NOT_BORROWED {
            return Err(ProgramError::AccountBorrowFailed);
        }

        Ok(())
    }

    /// Zero out the the account's data length, lamports and owner fields, effectively
    /// closing the account.
    ///
    /// Note: This does not zero the account data. The account data will be zeroed by
    /// the runtime at the end of the instruction where the account was closed or at the
    /// next CPI call.
    ///
    /// # Safety
    ///
    /// The lamports must be moved from the account prior to closing it to prevent
    /// an unbalanced instruction error.
    ///
    /// It also makes assumptions about the layout and location of memory
    /// referenced by `RuntimeAccount` fields. It should only be called for
    /// instances of `AccountView` that were created by the runtime and received
    /// in the `process_instruction` entrypoint of a program.
    ///
    /// It also will alias the `RuntimeAccount` struct if other references into that region
    /// are extant.
    #[cfg(feature = "bpf")]
    #[inline(always)]
    pub unsafe fn close(&self) {
        // We take advantage that the 48 bytes before the account data are:
        // - 32 bytes for the owner
        // - 8 bytes for the lamports
        // - 8 bytes for the data_len
        //
        // So we can zero out them directly.
        core::ptr::write_bytes(self.data_mut_ptr().sub(48), 0, 48);
    }

    /// Returns a raw pointer to the `RuntimeAccount` struct.
    pub fn account_ptr(&self) -> *const RuntimeAccount {
        self.raw as *const _
    }

    /// Returns a mutable raw pointer to the `RuntimeAccount` struct.
    pub fn account_mut_ptr(&mut self) -> *mut RuntimeAccount {
        self.raw
    }

    /// Returns the memory address of the account data.
    #[inline(always)]
    pub fn data_ptr(&self) -> *const u8 {
        // SAFETY: The `raw` pointer is guaranteed to be valid.
        unsafe { (self.raw as *const u8).add(size_of::<RuntimeAccount>()) }
    }

    /// Returns the memory address of the account data.
    #[inline(always)]
    pub fn data_mut_ptr(&self) -> *mut u8 {
        // SAFETY: The `raw` pointer is guaranteed to be valid.
        unsafe { (self.raw as *mut u8).add(size_of::<RuntimeAccount>()) }
    }
}

impl<'view> AsRef<AccountView<'view>> for AccountView<'view> {
    #[inline(always)]
    fn as_ref(&self) -> &AccountView<'view> {
        self
    }
}

/// Reference to account data with checked borrow rules.
#[derive(Debug)]
pub struct Ref<'a, T: ?Sized> {
    value: NonNull<T>,
    state: NonNull<u8>,
    __phantom: PhantomData<&'a T>,
}

impl<'a, T: ?Sized> Ref<'a, T> {
    /// Maps a reference to a new type.
    #[inline(always)]
    pub fn map<U: ?Sized, F>(orig: Ref<'a, T>, f: F) -> Ref<'a, U>
    where
        F: FnOnce(&T) -> &U,
    {
        // Avoid decrementing the borrow flag on drop.
        let orig = ManuallyDrop::new(orig);
        Ref {
            value: NonNull::from(f(&*orig)),
            state: orig.state,
            __phantom: PhantomData,
        }
    }

    /// Tries to makes a new `Ref` for a component of the borrowed data.
    #[inline(always)]
    pub fn try_map<U: ?Sized, E>(
        orig: Ref<'a, T>,
        f: impl FnOnce(&T) -> core::result::Result<&U, E>,
    ) -> core::result::Result<Ref<'a, U>, (Self, E)> {
        // Avoid decrementing the borrow flag on Drop.
        let orig = ManuallyDrop::new(orig);
        match f(&*orig) {
            Ok(value) => Ok(Ref {
                value: NonNull::from(value),
                state: orig.state,
                __phantom: PhantomData,
            }),
            Err(e) => Err((ManuallyDrop::into_inner(orig), e)),
        }
    }
}

impl<T: ?Sized> Deref for Ref<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.value.as_ref() }
    }
}

impl<T: ?Sized> Drop for Ref<'_, T> {
    fn drop(&mut self) {
        // Increment the available borrow count.
        unsafe { *self.state.as_mut() += 1 };
    }
}

#[derive(Debug)]
pub struct RefMut<'a, T: ?Sized> {
    value: NonNull<T>,
    state: NonNull<u8>,
    __phantom: PhantomData<&'a mut T>,
}

impl<'a, T: ?Sized> RefMut<'a, T> {
    /// Maps a mutable reference to a new type.
    #[inline(always)]
    pub fn map<U: ?Sized, F>(orig: RefMut<'a, T>, f: F) -> RefMut<'a, U>
    where
        F: FnOnce(&mut T) -> &mut U,
    {
        // Avoid decrementing the borrow flag on Drop.
        let mut orig = ManuallyDrop::new(orig);
        RefMut {
            value: NonNull::from(f(&mut *orig)),
            state: orig.state,
            __phantom: PhantomData,
        }
    }

    /// Tries to makes a new `RefMut` for a component of the borrowed data.
    ///
    /// On failure, the original guard is returned alongside with the error
    /// returned by the closure.
    #[inline(always)]
    pub fn try_map<U: ?Sized, E>(
        orig: RefMut<'a, T>,
        f: impl FnOnce(&mut T) -> core::result::Result<&mut U, E>,
    ) -> core::result::Result<RefMut<'a, U>, (Self, E)> {
        // Avoid decrementing the borrow flag on Drop.
        let mut orig = ManuallyDrop::new(orig);
        match f(&mut *orig) {
            Ok(value) => Ok(RefMut {
                value: NonNull::from(value),
                state: orig.state,
                __phantom: PhantomData,
            }),
            Err(e) => Err((ManuallyDrop::into_inner(orig), e)),
        }
    }
}

impl<T: ?Sized> Deref for RefMut<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.value.as_ref() }
    }
}
impl<T: ?Sized> DerefMut for RefMut<'_, T> {
    fn deref_mut(&mut self) -> &mut <Self as core::ops::Deref>::Target {
        unsafe { self.value.as_mut() }
    }
}

impl<T: ?Sized> Drop for RefMut<'_, T> {
    fn drop(&mut self) {
        // Reset the borrow state.
        unsafe { *self.state.as_mut() = NOT_BORROWED };
    }
}
