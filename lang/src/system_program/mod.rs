// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

pub mod instructions;

use crate::prelude::*;

pub struct System;

impl ProgramId for System {
    const ID: Address = address!("11111111111111111111111111111111");
}

/// The maximum `seed` length for instructions requiring a seed.
pub const MAX_SEED_LEN: usize = 32;

pub fn minimum_balance(space: usize) -> Result<u64> {
    let rent = Rent::get()?;

    rent.try_minimum_balance(space)
}
