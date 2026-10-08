// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

pub mod instructions;

use hayabusa_common::traits::Id;

pub unsafe trait TokenProgram: Id {}
