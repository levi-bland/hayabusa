// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

#![no_std]

pub mod instructions;

use hayabusa_common::{address, address::Address, traits::Ids};

pub struct TokenInterface;

impl Ids for TokenInterface {
    const IDS: &[Address] = &[address!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")];
}
