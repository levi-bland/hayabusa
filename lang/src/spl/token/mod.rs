// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

pub mod instructions;
pub mod state;

use crate::prelude::*;

pub struct Token;

impl ProgramId for Token {
    const ID: Address = address!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
}
