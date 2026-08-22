// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

#![no_std]

use hayabusa_accounts::ProgramIds;
use hayabusa_common::Address;

pub struct TokenInterface;

impl ProgramIds for TokenInterface {
    const IDS: &'static [Address] = &[hayabusa_token::ID, hayabusa_token2022::ID];
}
