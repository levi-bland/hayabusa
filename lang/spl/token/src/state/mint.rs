// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use crate::{Token, instructions::InitializeMint2};
use hayabusa_common::{
    Result,
    account_view::AccountView,
    accounts::{account::Account, mutable::Mut, signer::Signer},
    address::{Address, address_eq},
    cpi::{Seed, SignerBumpness},
    error::ErrorCode,
    hint,
    syscalls::{try_create_program_address, try_find_program_address},
    system_program::{create_or_allocate_account, create_or_allocate_pda},
    traits::{
        __AccountDiscriminatorMode, __AccountMarker, __InitAllowed, __Owner,
        __ParseAccountDispatch, __SplAccount, __SplAccountMarker, AccountInit, Cast, CpiSerialize,
        Id, NoMeta, Owner, Ownership, ParseAccount, Space, ToAccountView,
    },
};
use hayabusa_err_macro::err;
use hayabusa_invoke_cpi_macros::invoke;
use solana_program_error::ProgramError;

/// Mint account layout.
#[derive(Clone)]
#[repr(C)]
pub struct Mint {
    /// Indicates whether the mint authority is present or not.
    mint_authority_flag: [u8; 4],

    /// Optional authority used to mint new tokens. The mint authority may only
    /// be provided during mint creation. If no mint authority is present
    /// then the mint has a fixed supply and no further tokens may be
    /// minted.
    mint_authority: Address,

    /// Total supply of tokens.
    supply: [u8; 8],

    /// Number of base 10 digits to the right of the decimal place.
    decimals: u8,

    /// Is `true` if this structure has been initialized.
    is_initialized: u8,

    /// Indicates whether the freeze authority is present or not.
    freeze_authority_flag: [u8; 4],

    /// Optional authority to freeze token accounts.
    freeze_authority: Address,
}

impl Mint {
    #[inline(always)]
    pub fn has_mint_authority(&self) -> bool {
        self.mint_authority_flag[0] == 1
    }

    #[inline(always)]
    pub fn mint_authority(&self) -> Option<&Address> {
        if self.has_mint_authority() {
            Some(self.mint_authority_unchecked())
        } else {
            None
        }
    }

    /// Return the mint authority.
    ///
    /// This method should be used when the caller knows that the mint will have
    /// a mint authority set since it skips the `Option` check.
    #[inline(always)]
    pub fn mint_authority_unchecked(&self) -> &Address {
        &self.mint_authority
    }

    #[inline(always)]
    pub fn supply(&self) -> u64 {
        u64::from_le_bytes(self.supply)
    }

    #[inline(always)]
    pub fn decimals(&self) -> u8 {
        self.decimals
    }

    #[inline(always)]
    pub fn is_initialized(&self) -> bool {
        self.is_initialized == 1
    }

    #[inline(always)]
    pub fn has_freeze_authority(&self) -> bool {
        self.freeze_authority_flag[0] == 1
    }

    #[inline(always)]
    pub fn freeze_authority(&self) -> Option<&Address> {
        if self.has_freeze_authority() {
            Some(self.freeze_authority_unchecked())
        } else {
            None
        }
    }

    /// Return the freeze authority.
    ///
    /// This method should be used when the caller knows that the mint will have
    /// a freeze authority set since it skips the `Option` check.
    #[inline(always)]
    pub fn freeze_authority_unchecked(&self) -> &Address {
        &self.freeze_authority
    }
}

unsafe impl __AccountMarker for Mint {}

unsafe impl __SplAccountMarker for Mint {}

unsafe impl __InitAllowed for Mint {}

#[cfg(feature = "bpf")]
unsafe impl Cast for Mint {}

unsafe impl Ownership for Mint {
    type OwnerType = __Owner;

    #[inline(always)]
    fn check_ownership(view: AccountView<'_>) -> Result<()> {
        Self::check_owner(view)
    }
}

impl Owner for Mint {
    const OWNER: &Address = Token::ID;
}

impl Space for Mint {}

unsafe impl __AccountDiscriminatorMode for Mint {
    type Mode = __SplAccount;
}

impl<'view> __ParseAccountDispatch<'view, __SplAccount, Account<'view, Mint>> for Mint {
    #[inline(always)]
    fn dispatch(view: AccountView<'view>) -> Result<Account<'view, Mint>> {
        Mint::check_owner(view)?;
        Mint::check_space(view)?;

        // SAFETY: required invariants are checked above.
        if hint::unlikely(!unsafe { Mint::cast_unchecked(view)? }.is_initialized()) {
            err!(
                "Mint::dispatch: account is not initialized",
                ErrorCode::InvalidAccount,
            );
        }

        // SAFETY: required invariants are checked above.
        Ok(unsafe { Account::new(view) })
    }
}

impl<'view> AccountInit<'view> for Mint {
    type Meta<'a>
        = __TokenMintInitMeta<'view, 'a>
    where
        'view: 'a;

    type PdaMeta<'a, 'b, 'c>
        = __TokenMintPdaInitMeta<'view, 'a, 'b, 'c>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a;

    #[inline(never)]
    fn init<'a>(view: AccountView<'view>, meta: &Self::Meta<'a>) -> Result<()>
    where
        'view: 'a,
    {
        // non-PDA's must be signers
        let _: Signer<'view> = Signer::parse(view, &mut NoMeta)?;

        create_or_allocate_account(meta.payer.to_account_view(), view, Token::ID, Mint::SPACE)?;

        invoke!(InitializeMint2 {
            mint: view,
            decimals: meta.decimals,
            mint_authority: meta.mint_authority,
            freeze_authority: meta.freeze_authority,
        });

        Ok(())
    }

    #[inline(never)]
    fn init_pda<'a, 'b, 'c>(
        view: AccountView<'view>,
        meta: &mut Self::PdaMeta<'a, 'b, 'c>,
    ) -> Result<()>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a,
    {
        // &mut is pretty disgusting here but it's necessary
        match &mut meta.signer {
            SignerBumpness::With(signer) => {
                let addr =
                    try_create_program_address(signer.as_slice_of_slices(), meta.program_id)?;

                if hint::unlikely(!address_eq(&addr, view.address())) {
                    err!(
                        "Mint::init_pda: PDA address mismatch: expected {}, got {}",
                        ErrorCode::InvalidProgramAccount,
                        addr,
                        view.address(),
                    );
                }

                create_or_allocate_pda(
                    meta.payer.to_account_view(),
                    view,
                    Token::ID,
                    &[*signer],
                    Mint::SPACE,
                )?;
            }
            SignerBumpness::Without(signer, bump_slot) => {
                let (addr, bump) =
                    try_find_program_address(signer.as_slice_of_slices(), meta.program_id)?;

                if hint::unlikely(!address_eq(&addr, view.address())) {
                    err!(
                        "Mint::init_pda: PDA address mismatch: expected {}, got {}",
                        ErrorCode::InvalidProgramAccount,
                        addr,
                        view.address(),
                    );
                }

                let slot: &'c mut u8 = bump_slot.take().ok_or(ErrorCode::MetaAlreadyConsumed)?;
                *slot = bump;
                let bump_ref: &'c u8 = slot; // move the &mut, downgrade to shared for all of 'c
                signer.push(Seed::from(core::slice::from_ref(bump_ref))); // 'c: 'b, so it coerces

                create_or_allocate_pda(
                    meta.payer.to_account_view(),
                    view,
                    Token::ID,
                    &[signer.as_signer()],
                    Mint::SPACE,
                )?;
            }
        }

        invoke!(InitializeMint2 {
            mint: view,
            decimals: meta.decimals,
            mint_authority: meta.mint_authority,
            freeze_authority: meta.freeze_authority,
        });

        Ok(())
    }
}

pub struct __TokenMintInitMeta<'view, 'a> {
    pub payer: &'a Mut<Signer<'view>>,
    pub mint_authority: &'view Address,
    pub freeze_authority: Option<&'view Address>,
    pub decimals: u8,
}

impl<'view: 'a, 'a> __TokenMintInitMeta<'view, 'a> {
    #[inline(always)]
    pub fn new(
        payer: &'a Mut<Signer<'view>>,
        mint_authority: &'view Address,
        freeze_authority: Option<&'view Address>,
        decimals: u8,
    ) -> Self {
        Self {
            payer,
            mint_authority,
            freeze_authority,
            decimals,
        }
    }
}

pub struct __TokenMintPdaInitMeta<'view, 'a, 'b, 'c> {
    pub payer: &'c Mut<Signer<'view>>,
    pub mint_authority: &'view Address,
    pub freeze_authority: Option<&'view Address>,
    pub decimals: u8,
    pub signer: SignerBumpness<'b, 'a, 'c>,
    pub program_id: &'view Address,
}

impl<'view, 'a, 'b, 'c> __TokenMintPdaInitMeta<'view, 'a, 'b, 'c> {
    #[inline(always)]
    pub fn new(
        payer: &'c Mut<Signer<'view>>,
        mint_authority: &'view Address,
        freeze_authority: Option<&'view Address>,
        decimals: u8,
        signer: SignerBumpness<'b, 'a, 'c>,
        program_id: &'view Address,
    ) -> Self {
        Self {
            payer,
            mint_authority,
            freeze_authority,
            decimals,
            signer,
            program_id,
        }
    }
}
