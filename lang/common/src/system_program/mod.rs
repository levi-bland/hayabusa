// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

pub mod instructions;

use crate::{
    account_view::AccountView,
    address,
    address::Address,
    cpi::Signer,
    sysvars::rent::layout::Rent,
    traits::{CpiSerialize, Id, Sysvar},
    Result,
};
use hayabusa_invoke_cpi_macros::{invoke, invoke_with_signers};
use instructions::{
    allocate::Allocate, assign::Assign, create_account::CreateAccount, transfer::Transfer,
};

pub struct System;

impl Id for System {
    const ID: &Address = &address!("11111111111111111111111111111111");
}

/// The maximum `seed` length for instructions requiring a seed.
pub const MAX_SEED_LEN: usize = 32;

pub fn minimum_balance(space: usize) -> Result<u64> {
    let rent = Rent::get()?;

    rent.try_minimum_balance(space)
}

pub fn create_or_allocate_account<'view>(
    from: AccountView<'view>,
    to: AccountView<'view>,
    owner_program: &'view Address,
    space: usize,
) -> Result<()> {
    let lamports = to.lamports();
    let min_lamports = Rent::get()?.minimum_balance_unchecked(space);

    if lamports != 0 {
        if lamports < min_lamports {
            let diff = min_lamports - lamports;

            invoke!(Transfer {
                from: from,
                to: to,
                lamports: diff,
            });
        }

        invoke!(Allocate {
            account: to,
            space: space as u64,
        });

        invoke!(Assign {
            account: to,
            owner: owner_program,
        });
    } else {
        invoke!(CreateAccount {
            from: from,
            to: to,
            owner_program: owner_program,
            space: space as u64,
        });
    }

    Ok(())
}

pub fn create_or_allocate_pda<'view>(
    from: AccountView<'view>,
    to: AccountView<'view>,
    owner_program: &'view Address,
    signers: &[Signer<'_, '_>],
    space: usize,
) -> Result<()> {
    let lamports = to.lamports();
    let min_lamports = Rent::get()?.minimum_balance_unchecked(space);

    if lamports != 0 {
        if lamports < min_lamports {
            let diff = min_lamports - lamports;

            invoke!(Transfer {
                from: from,
                to: to,
                lamports: diff,
            });
        }

        invoke_with_signers!(
            Allocate {
                account: to,
                space: space as u64,
            },
            signers,
        );

        invoke_with_signers!(
            Assign {
                account: to,
                owner: owner_program,
            },
            signers,
        );
    } else {
        invoke_with_signers!(
            CreateAccount {
                from: from,
                to: to,
                owner_program: owner_program,
                space: space as u64,
            },
            signers,
        );
    }

    Ok(())
}
