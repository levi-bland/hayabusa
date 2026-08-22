// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

#![no_std]

use bytemuck::Pod;
use hayabusa_common::{AccountView, Address, Ref, RefMut};
use hayabusa_discriminator::Discriminator;
use hayabusa_errors::{ProgramError, Result};
use hayabusa_utility::{error_msg, hint::unlikely, Len, OwnerProgram};

/// # Safety
/// Trait is unsafe to implement unless you can guarantee correct
/// struct and ptr alignment on the implementing type.
pub unsafe trait Cast
where
    Self: OwnerProgram + Len + FromBytesUnchecked,
{
    #[inline(always)]
    fn check_owner(owner: &Address) -> Result<()> {
        if unlikely(owner != &Self::OWNER) {
            error_msg!("Cast: invalid owner", ProgramError::InvalidAccountOwner,);
        }

        Ok(())
    }

    #[inline(always)]
    fn check_len(len: usize) -> Result<()> {
        if unlikely(len != Self::LEN) {
            error_msg!(
                "Cast: invalid length",
                ProgramError::InvalidAccountData, // TODO: add a dedicated error code for this
            );
        }

        Ok(())
    }

    /// # Safety
    /// The caller must ensure the borrow is sound at the call site;
    /// i.e. check data_len, owner, discriminator, etc...
    #[inline(always)]
    unsafe fn cast_unchecked(view: &AccountView) -> &Self {
        let data = view.borrow_unchecked();
        &*(data.as_ptr() as *const Self)
    }

    /// # Safety
    /// The caller must ensure the mutable borrow is sound at the call site;
    /// i.e. check data_len, owner, discriminator, etc...
    #[allow(clippy::mut_from_ref)]
    #[inline(always)]
    unsafe fn cast_mut_unchecked(view: &AccountView) -> &mut Self {
        let data = view.borrow_unchecked_mut();
        &mut *(data.as_mut_ptr() as *mut Self)
    }

    #[inline(always)]
    fn cast(view: &AccountView) -> Result<Ref<Self>> {
        Self::check_len(view.data_len())?;

        Ok(Ref::map(view.try_borrow()?, |data| unsafe {
            &*(data.as_ptr() as *const Self)
        }))
    }

    #[inline(always)]
    fn cast_mut(view: &AccountView) -> Result<RefMut<Self>> {
        Self::check_len(view.data_len())?;

        Ok(RefMut::map(view.try_borrow_mut()?, |data| unsafe {
            &mut *(data.as_mut_ptr() as *mut Self)
        }))
    }
}

unsafe impl<T> Cast for T
where
    T: Pod + OwnerProgram + Len + FromBytesUnchecked + Discriminator,
{
    #[inline(always)]
    unsafe fn cast_unchecked(view: &AccountView) -> &T {
        let undiscriminated = &view.borrow_unchecked()[8..];
        &*(undiscriminated.as_ptr() as *const T)
    }

    #[inline(always)]
    unsafe fn cast_mut_unchecked(view: &AccountView) -> &mut T {
        let undiscriminated = &mut view.borrow_unchecked_mut()[8..];
        &mut *(undiscriminated.as_mut_ptr() as *mut T)
    }

    #[inline(always)]
    fn cast(view: &AccountView) -> Result<Ref<T>> {
        Self::check_len(view.data_len())?;

        Ok(Ref::map(view.try_borrow()?, |data| unsafe {
            let undiscriminated = &data[8..];
            &*(undiscriminated.as_ptr() as *const T)
        }))
    }

    #[inline(always)]
    fn cast_mut(view: &AccountView) -> Result<RefMut<T>> {
        Self::check_len(view.data_len())?;

        Ok(RefMut::map(view.try_borrow_mut()?, |data| unsafe {
            let undiscriminated = &mut data[8..];
            &mut *(undiscriminated.as_mut_ptr() as *mut T)
        }))
    }
}

/// Unsafe to call either trait method
///
/// You must ensure proper alignment of Self
pub trait FromBytesUnchecked: Sized {
    /// # Safety
    /// You must ensure proper alignment of Self, and bytes.len() == size_of::<Self>()
    unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        &*(bytes.as_ptr() as *const Self)
    }
    /// # Safety
    /// You must ensure proper alignment of Self, and bytes.len() == size_of::<Self>()
    unsafe fn from_bytes_unchecked_mut(bytes: &mut [u8]) -> &mut Self {
        &mut *(bytes.as_mut_ptr() as *mut Self)
    }
}
