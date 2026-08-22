// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

#![no_std]

use core::mem::MaybeUninit;

use hayabusa_errors::{ErrorCode, Result};
use solana_address::Address;
pub use solana_define_syscall::definitions::*;

pub const MAX_SEEDS: usize = 16;
pub const MAX_SEED_LEN: usize = 32;
pub const MAX_TOTAL_LEN: usize = MAX_SEEDS * MAX_SEED_LEN; // 512

pub fn try_find_program_address(seeds: &[&[u8]], program_id: &Address) -> Result<(Address, u8)> {
    let mut seed_buf = MaybeUninit::<[u8; MAX_TOTAL_LEN]>::uninit();
    let seed_len = flatten_seeds_raw(seeds, &mut seed_buf)?;

    let mut pda = MaybeUninit::<[u8; 32]>::uninit();
    let mut bump: u8 = 0;

    // SAFETY: bounds are checked on `seed_buf`, memory layouts of [u8; N] and MaybeUninit<[u8; N]> are identical
    let rc = unsafe {
        sol_try_find_program_address(
            seed_buf.as_ptr() as *const u8,
            seed_len as u64,
            program_id.as_ref().as_ptr(),
            pda.as_mut_ptr() as *mut u8,
            &mut bump as *mut u8,
        )
    };

    if rc == 0 {
        let pda = unsafe { pda.assume_init() };
        Ok((Address::new_from_array(pda), bump))
    } else {
        Err(ErrorCode::SyscallFailed.into())
    }
}

pub fn try_create_program_address(seeds: &[&[u8]], program_id: &Address) -> Result<Address> {
    let mut seed_buf = MaybeUninit::<[u8; MAX_TOTAL_LEN]>::uninit();
    let seed_len = flatten_seeds_raw(seeds, &mut seed_buf)?;

    let mut pda = MaybeUninit::<[u8; 32]>::uninit();

    let rc = unsafe {
        sol_create_program_address(
            seed_buf.as_ptr() as *const u8,
            seed_len as u64,
            program_id.as_ref().as_ptr(),
            pda.as_mut_ptr() as *mut u8,
        )
    };

    if rc == 0 {
        let pda = unsafe { pda.assume_init() };
        Ok(Address::new_from_array(pda))
    } else {
        Err(ErrorCode::SyscallFailed.into())
    }
}

/// Flattens `seeds` into `out`.
///
/// Returns total number of bytes written.
///
/// Safety guarantees enforced:
/// - bounds checked
/// - no overlapping writes
/// - no out-of-bounds reads
pub fn flatten_seeds_raw(
    seeds: &[&[u8]],
    out: &mut MaybeUninit<[u8; MAX_TOTAL_LEN]>,
) -> Result<usize> {
    if seeds.len() > MAX_SEEDS {
        return Err(ErrorCode::TooManySeeds.into());
    }

    let mut offset: usize = 0;
    let out_ptr = out.as_mut_ptr() as *mut u8;

    for seed in seeds {
        let len = seed.len();

        if len > MAX_SEED_LEN {
            return Err(ErrorCode::SeedsTooLong.into());
        }

        if offset + len > MAX_TOTAL_LEN {
            return Err(ErrorCode::SeedsTooLong.into());
        }

        unsafe {
            core::ptr::copy_nonoverlapping(seed.as_ptr(), out_ptr.add(offset), len);
        }

        offset += len;
    }

    Ok(offset)
}
