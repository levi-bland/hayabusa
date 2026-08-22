#![no_std]

use hayabusa::prelude::*;

declare_id!("HPoDm7Kf63B6TpFKV7S8YSd7sGde6sVdztiDBEVkfuxz");

#[program]
pub mod counter_program {
    pub fn initialize_counter(ctx: Ctx<InitializeCounter>) -> Result<()> {
        let mut counter = ctx.accounts.counter.cast_mut()?;

        counter.authority = *ctx.accounts.signer.address();

        Ok(())
    }

    pub fn increment_counter(ctx: Ctx<IncrementCounter>) -> Result<()> {
        let mut counter = ctx.accounts.counter.cast_mut()?;

        counter.count += 1;

        Ok(())
    }
}

#[derive(ParseAccounts)]
pub struct InitializeCounter<'view> {
    pub signer: Mut<Signer<'view>>,
    #[account(payer = signer)]
    pub counter: Init<Account<'view, Counter>>,
    pub system_program: Program<'view, System>,
}

#[derive(ParseAccounts)]
pub struct IncrementCounter<'view> {
    pub authority: Signer<'view>,
    #[account(has_one = authority)]
    pub counter: Mut<Account<'view, Counter>>,
}

#[account]
pub struct Counter {
    pub count: u64,
    pub authority: Address,
}
