// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

pub mod instructions;
pub mod state;

use hayabusa_common::{address, address::Address, traits::Id};
use hayabusa_spl_token_common::TokenProgram;

pub struct Token;

impl Id for Token {
    const ID: &Address = &address!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
}

unsafe impl TokenProgram for Token {}
