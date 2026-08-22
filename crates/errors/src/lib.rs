// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

#![no_std]

extern crate alloc;
use alloc::boxed::Box;

mod error_code;
pub use error_code::*;
pub use solana_program_error::ProgramError;

pub type Result<T> = core::result::Result<T, ProgramError>;

pub enum Error {
    HayabusaError(Box<HayabusaError>),
    ProgramError(Box<ProgramErrorWithOrigin>),
}

pub struct HayabusaError {
    pub code: u32,
    pub msg: &'static str,
    pub origin: Option<ErrorOrigin>,
}

pub struct ProgramErrorWithOrigin {
    pub program_error: ProgramError,
    pub origin: Option<ErrorOrigin>,
}

#[derive(Debug)]
pub struct ErrorOrigin {
    pub file: &'static str,
    pub line: u32,
}
