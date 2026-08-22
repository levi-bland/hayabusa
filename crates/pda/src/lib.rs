// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

#![no_std]

mod check_seeds;

pub use check_seeds::*;

use solana_instruction_view::cpi::Signer;

pub trait Seeds {
    fn to_signer(&self) -> Signer<'_, '_>;
}
