// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use crate::{address::Address, error::ErrorCode, Result};
use hayabusa_err_macro::err;
use solana_program_error::ProgramError;

#[inline(always)]
pub fn try_find_program_address(seeds: &[&[u8]], program_id: &Address) -> Result<(Address, u8)> {
    match solana_address::Address::try_find_program_address(seeds, &program_id.to_solana_address())
    {
        Some((addr, bump)) => Ok((Address::from(addr), bump)),
        None => err!(
            "try_find_program_address: program address does not exist",
            ErrorCode::ProgramAddressDoesNotExist,
        ),
    }
}

#[inline(always)]
pub fn try_create_program_address(seeds: &[&[u8]], program_id: &Address) -> Result<Address> {
    // Open a PR to standardise assoc. func. naming, this should be try_create_program_address
    match solana_address::Address::create_program_address(seeds, &program_id.to_solana_address()) {
        Ok(a) => Ok(Address::from(a)),
        Err(_) => err!(
            "try_create_program_address: program address does not exist",
            ErrorCode::ProgramAddressDoesNotExist,
        ),
    }
}

#[inline(always)]
pub fn sol_log_data(data: &[&[u8]]) {
    #[cfg(any(target_os = "solana", target_arch = "bpf"))]
    unsafe {
        solana_define_syscall::definitions::sol_log_data(
            data as *const _ as *const u8,
            data.len() as u64,
        )
    };

    #[cfg(not(any(target_os = "solana", target_arch = "bpf")))]
    core::hint::black_box(data);
}
