// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

pub mod clock;
pub mod fees;
pub mod instructions;
pub mod rent;

use crate::prelude::*;

/// The program ID of the sysvar program.
pub const SYSVAR_PROGRAM_ID: Address = address!("Sysvar1111111111111111111111111111111111111");

/// Implements the [`Sysvar::get`] method for both SBF and host targets.
#[macro_export]
macro_rules! impl_sysvar_get {
    ($syscall_name:ident) => {
        fn get() -> Result<Self> {
            let mut var = core::mem::MaybeUninit::<Self>::uninit();
            let var_addr = var.as_mut_ptr() as *mut _ as *mut u8;

            #[cfg(any(target_os = "solana", target_arch = "bpf"))]
            let result = unsafe { $crate::prelude::syscalls::definitions::$syscall_name(var_addr) };

            #[cfg(not(any(target_os = "solana", target_arch = "bpf")))]
            let result = core::hint::black_box(var_addr as *const _ as u64);
            const SUCCESS: u64 = 0;
            match result {
                SUCCESS => {
                    // SAFETY: The syscall initialized the memory.
                    Ok(unsafe { var.assume_init() })
                }
                // Unexpected errors are folded into `UnsupportedSysvar`.
                _ => Err($crate::prelude::ProgramError::UnsupportedSysvar),
            }
        }
    };
    // This variant uses the generic `sol_get_sysvar` syscall. Note that it only
    // supports sysvars without padding or with padding at the end of their byte
    // layout since the syscall data follows bincode serialization.
    ($syscall_id:expr, $padding:literal) => {
        #[inline(always)]
        fn get() -> Result<Self> {
            let mut var = core::mem::MaybeUninit::<Self>::uninit();
            let var_addr = var.as_mut_ptr() as *mut _ as *mut u8;

            #[cfg(any(target_os = "solana", target_arch = "bpf"))]
            // SAFETY: The allocation is valid for the size of `Self`. It fixes
            // the size to `size_of::<Self>() - $padding` for the syscall since
            // the byte layout follows bincode serialization; the remaining bytes
            // are considered padding and initialized to zero.
            let result = unsafe {
                let length = core::mem::size_of::<Self>() - $padding;
                // Make sure all bytes are initialized.
                var_addr.add(length).write_bytes(0, $padding);

                $crate::prelude::syscalls::definitions::sol_get_sysvar(
                    &$syscall_id as *const _ as *const u8,
                    var_addr,
                    0,
                    length as u64,
                )
            };

            #[cfg(not(any(target_os = "solana", target_arch = "bpf")))]
            let result = {
                // SAFETY: The allocation is valid for the size of `Self`.
                unsafe { var_addr.write_bytes(0, size_of::<Self>()) };
                core::hint::black_box(var_addr as *const _ as u64)
            };

            const SUCCESS: u64 = 0;
            const OFFSET_LENGTH_EXCEEDS_SYSVAR: u64 = 1;
            const SYSVAR_NOT_FOUND: u64 = 2;
            match result {
                SUCCESS => {
                    // SAFETY: The syscall initialized the memory and
                    // padding bytes are set to zero.
                    Ok(unsafe { var.assume_init() })
                }
                OFFSET_LENGTH_EXCEEDS_SYSVAR => Err($crate::prelude::ProgramError::InvalidArgument),
                SYSVAR_NOT_FOUND => Err($crate::prelude::ProgramError::UnsupportedSysvar),
                // Unexpected errors are folded into `UnsupportedSysvar`.
                _ => Err($crate::prelude::ProgramError::UnsupportedSysvar),
            }
        }
    };
}
