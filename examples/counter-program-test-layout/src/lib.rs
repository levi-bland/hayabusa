#![no_std]

use hayabusa::prelude::*;

declare_id!("HPoDm7Kf63B6TpFKV7S8YSd7sGde6sVdztiDBEVkfuxz");

#[program]
mod counter_program {
    pub fn initialize_counter(ctx: Ctx<InitializeCounter>, num: u64) -> Result<()> {
        let mut counter = ctx.accounts.counter.cast_mut()?;

        counter.authority = ctx.accounts.signer.address_owned();
        counter.count = num;
        counter.bump = ctx.bumps.counter;

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
    #[account(
        payer = signer,
        seeds = [
            Counter::SEED,
            signer.address().as_ref(),
        ],
        bump,
    )]
    pub counter: Init<Pda<Account<'view, Counter>>>,
    pub system_program: Program<'view, System>,
}

#[derive(ParseAccounts)]
pub struct IncrementCounter<'view> {
    pub authority: Signer<'view>,
    #[account(
        has_one = authority,
        seeds = [
            Counter::SEED,
            authority.address().as_ref(),
        ],
        bump = counter.bump,
    )]
    pub counter: Mut<Pda<Account<'view, Counter>>>,
}

#[account]
#[seed("counter")]
pub struct Counter {
    pub count: u64,
    pub authority: Address,
    pub bump: u8,
    _padding: [u8; 7],
}
