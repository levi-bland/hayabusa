// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

use crate::prelude::*;
use core::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    ptr::{copy, copy_nonoverlapping, read, write},
    slice::{from_raw_parts, from_raw_parts_mut},
};

#[derive(Clone)]
pub struct StackVec<T: Copy, const MAX_LEN: usize> {
    data: MaybeUninit<[T; MAX_LEN]>,
    len: usize,
}

impl<T: Copy, const MAX_LEN: usize> Default for StackVec<T, MAX_LEN> {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Copy, const MAX_LEN: usize> StackVec<T, MAX_LEN> {
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            data: MaybeUninit::uninit(),
            len: 0,
        }
    }

    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline(always)]
    pub const fn capacity(&self) -> usize {
        MAX_LEN
    }

    #[inline(always)]
    pub const fn remaining(&self) -> usize {
        MAX_LEN - self.len
    }

    /// Push a slice represented as a raw ptr to a [`StackVec`].
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self.len + len <= MAX_LEN`.
    #[inline(always)]
    pub unsafe fn push_ptr_unchecked(&mut self, src: *const T, len: usize) {
        copy_nonoverlapping(src, self.as_mut_ptr().add(self.len), len);
        self.len += len;
    }

    #[inline(always)]
    pub fn try_push_slice(&mut self, src: &[T]) -> Result<()> {
        if hint::unlikely(self.len + src.len() > MAX_LEN) {
            return Err(ErrorCode::StackVecMaxLengthExceeded.into());
        }

        // SAFETY: the `MAX_LEN` bound is respected.
        unsafe { self.push_ptr_unchecked(src.as_ptr(), src.len()) };

        Ok(())
    }

    /// Push a `T` to the [`StackVec`] without checking the `MAX_LEN` bound.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self.len <= MAX_LEN`
    /// or it will write into unallocated memory, which is UB.
    #[inline(always)]
    pub unsafe fn push_unchecked(&mut self, val: T) {
        write(self.as_mut_ptr().add(self.len), val);
        self.len += 1;
    }

    #[inline(always)]
    pub fn try_push(&mut self, val: T) -> Result<()> {
        if hint::unlikely(self.len > MAX_LEN) {
            return Err(ErrorCode::StackVecMaxLengthExceeded.into());
        }

        // SAFETY: The `MAX_LEN` bound is respected so the write is guaranteed to be valid.
        unsafe {
            self.push_unchecked(val);
        }

        Ok(())
    }

    #[inline(always)]
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        Some(unsafe { read(self.as_ptr().add(self.len)) })
    }

    /// Insert a `T` into a [`StackVec`] without checking index bounds, and `MAX_LEN` bound.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `index <= self.len`, and that `self.len < MAX_LEN`.
    #[inline(always)]
    pub unsafe fn insert_unchecked(&mut self, index: usize, val: T) {
        let ptr = self.as_mut_ptr().add(index);
        copy(ptr, ptr.add(1), self.len - index);
        write(ptr, val);
        self.len += 1;
    }

    #[inline(always)]
    pub fn try_insert(&mut self, index: usize, val: T) -> Result<()> {
        if hint::unlikely(index >= self.len) {
            return Err(ErrorCode::StackVecIndexOutOfBounds.into());
        }

        if hint::unlikely(self.len < MAX_LEN) {
            return Err(ErrorCode::StackVecMaxLengthExceeded.into());
        }

        unsafe {
            self.insert_unchecked(index, val);
        }

        Ok(())
    }

    #[inline(always)]
    pub unsafe fn remove_unchecked(&mut self, index: usize) -> T {
        let val = unsafe { read(self.as_ptr().add(index)) };
        unsafe {
            let p = self.as_mut_ptr().add(index);
            copy(p.add(1), p, self.len - index - 1);
        }
        self.len -= 1;
        val
    }

    #[inline(always)]
    pub fn remove(&mut self, index: usize) -> Result<T> {
        if hint::unlikely(index >= self.len) {
            return Err(ErrorCode::StackVecIndexOutOfBounds.into());
        }

        Ok(unsafe { self.remove_unchecked(index) })
    }

    #[inline(always)]
    pub unsafe fn swap_remove_unchecked(&mut self, index: usize) -> T {
        let val = unsafe { read(self.as_ptr().add(index)) };
        self.len -= 1;
        if index != self.len {
            unsafe {
                write(
                    self.as_mut_ptr().add(index),
                    read(self.as_ptr().add(self.len)),
                );
            }
        }
        val
    }

    #[inline(always)]
    pub fn swap_remove(&mut self, index: usize) -> Result<T> {
        if hint::unlikely(index >= self.len) {
            return Err(ErrorCode::StackVecIndexOutOfBounds.into());
        }

        Ok(unsafe { self.swap_remove_unchecked(index) })
    }

    #[inline(always)]
    pub fn truncate(&mut self, len: usize) {
        if len < self.len {
            self.len = len;
        }
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    #[inline(always)]
    pub fn first(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            Some(&self[0])
        }
    }

    #[inline(always)]
    pub fn last(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            Some(&self[self.len - 1])
        }
    }

    #[inline(always)]
    pub fn first_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            None
        } else {
            Some(&mut self[0])
        }
    }

    #[inline(always)]
    pub fn last_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            None
        } else {
            let len = self.len;
            Some(&mut self[len - 1])
        }
    }

    #[inline(always)]
    pub fn as_ptr(&self) -> *const T {
        self.data.as_ptr() as *const T
    }

    #[inline(always)]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.data.as_mut_ptr() as *mut T
    }
}

impl<T: Copy, const MAX_LEN: usize> AsRef<[T]> for StackVec<T, MAX_LEN> {
    #[inline(always)]
    fn as_ref(&self) -> &[T] {
        unsafe { from_raw_parts(self.data.as_ptr() as *const T, self.len) }
    }
}

impl<T: Copy, const MAX_LEN: usize> AsMut<[T]> for StackVec<T, MAX_LEN> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut [T] {
        unsafe { from_raw_parts_mut(self.data.as_mut_ptr() as *mut T, self.len) }
    }
}

impl<T: Copy, const MAX_LEN: usize> Deref for StackVec<T, MAX_LEN> {
    type Target = [T];

    #[inline(always)]
    fn deref(&self) -> &[T] {
        self.as_ref()
    }
}

impl<T: Copy, const MAX_LEN: usize> DerefMut for StackVec<T, MAX_LEN> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut()
    }
}
