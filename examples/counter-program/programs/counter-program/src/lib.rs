// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

#![no_std]
#![allow(dead_code, unexpected_cfgs)]

use hayabusa::prelude::*;

declare_id!("HPoDm7Kf63B6TpFKV7S8YSd7sGde6sVdztiDBEVkfuxz");

#[program]
mod counter_program {
    use super::{InitializeCounter, UpdateCounter};

    fn update_counter(ctx: Ctx<UpdateCounter>, amount: u64) -> Result<()> {
        let mut counter = ctx.accounts.counter.cast_mut()?;

        counter.count += amount;

        Ok(())
    }

    fn initialize_counter(ctx: Ctx<InitializeCounter>) -> Result<()> {
        let mut counter = ctx.accounts.counter.cast_mut()?;

        counter.authority = *ctx.accounts.user.address();

        Ok(())
    }
}

#[derive(ParseAccounts)]
pub struct UpdateCounter<'view> {
    pub counter: Mut<Account<'view, CounterAccount>>,
    #[account(
        address = &counter.cast()?.authority,
    )]
    pub user: Signer<'view>,
}

#[derive(ParseAccounts)]
pub struct InitializeCounter<'view> {
    pub user: Mut<Signer<'view>>,
    #[account(
        payer = user,
    )]
    pub counter: Init<Account<'view, CounterAccount>>,
    pub system_program: Program<'view, System>,
    pub rent: Sysvar<Rent<'view>>,
}

#[account]
pub struct CounterAccount {
    pub count: u64,
    pub authority: Address,
}
