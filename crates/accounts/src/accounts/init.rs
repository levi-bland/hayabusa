// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use crate::{
    Account, FromAccountView, Mut, NoDiscriminator, NoMeta, Program, System, ToAccountView,
    UncheckedAccount, WithDiscriminator, WritableAllowed,
};
use core::{marker::PhantomData, ops::Deref};
use hayabusa_cast::Cast;
use hayabusa_common::{AccountView, Address, ProgramId};
use hayabusa_cpi::CpiCtx;
use hayabusa_discriminator::Discriminator;
use hayabusa_errors::Result;
use hayabusa_system_program::instructions::{create_account, CreateAccount};
use hayabusa_token::{
    instructions::{initialize_account3, initialize_mint2, InitializeAccount3, InitializeMint2},
    state::{Mint, TokenAccount},
    Token,
};
use solana_instruction_view::cpi::Signer;

pub struct Init<T>(T);

impl<T> Deref for Init<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

unsafe impl<'ix, T: Cast + Discriminator> FromAccountView<'ix>
    for Init<Account<'ix, T, WithDiscriminator>>
{
    type Meta<'a, 'b, 'c, 'd>
        = InitMeta<'ix, 'a, 'b, 'c, 'd>
    where
        'ix: 'a + 'd,
        'd: 'c,
        'c: 'b;

    fn try_from_account_view<'a, 'b, 'c, 'd>(
        view: &'ix AccountView,
        meta: Self::Meta<'a, 'b, 'c, 'd>,
    ) -> Result<Self>
    where
        'ix: 'a + 'd,
        'd: 'c,
        'c: 'b,
    {
        Mut::<UncheckedAccount>::try_from_account_view(view, NoMeta)?;

        let cpi_ctx = CpiCtx::try_new(
            meta.system_program_view,
            CreateAccount {
                from: meta.payer_view,
                to: view,
            },
            meta.signers,
        )?;

        create_account(cpi_ctx, meta.program_id, T::DISCRIMINATED_LEN as u64)?;

        // SAFETY: the account creation ensures correct program id and data length.
        //         guaranteed to be no aliasing issues because at construction there cannot be any
        //         other refs to the same view.
        let data = unsafe { view.borrow_unchecked_mut() };

        data[..8].copy_from_slice(T::DISCRIMINATOR);

        Ok(Init(Account {
            view,
            _phantom: PhantomData,
        }))
    }
}

pub struct InitMeta<'ix, 'a, 'b, 'c, 'd>
where
    'ix: 'a + 'd,
    'd: 'c,
    'c: 'b,
{
    pub program_id: &'a Address,
    pub payer_view: &'ix AccountView,
    pub system_program_view: &'ix AccountView,
    pub signers: Option<&'b [Signer<'d, 'c>]>,
}

#[allow(unused)]
impl<'ix, 'a, 'b, 'c, 'd> InitMeta<'ix, 'a, 'b, 'c, 'd>
where
    'ix: 'a + 'd,
    'd: 'c,
    'c: 'b,
{
    pub fn new(
        program_id: &'a Address,
        payer: &Mut<impl ToAccountView<'ix> + WritableAllowed>,
        system_program: &Program<'ix, System>,
        signers: Option<&'b [Signer<'d, 'c>]>,
    ) -> Self {
        Self {
            program_id,
            payer_view: payer.to_account_view(),
            system_program_view: system_program.to_account_view(),
            signers,
        }
    }
}

unsafe impl<'ix> FromAccountView<'ix> for Init<Account<'ix, Mint, NoDiscriminator>> {
    type Meta<'a, 'b, 'c, 'd>
        = TokenMintInitMeta<'ix, 'a, 'b, 'c, 'd>
    where
        'ix: 'a + 'd,
        'd: 'c,
        'c: 'b;

    fn try_from_account_view<'a, 'b, 'c, 'd>(
        view: &'ix AccountView,
        meta: Self::Meta<'a, 'b, 'c, 'd>,
    ) -> Result<Self>
    where
        'ix: 'a + 'd,
        'd: 'c,
        'c: 'b,
    {
        let _: Mut<UncheckedAccount> = Mut::try_from_account_view(view, NoMeta)?;

        let cpi_ctx = CpiCtx::try_new(
            meta.system_program_view,
            CreateAccount {
                from: meta.payer_view,
                to: view,
            },
            meta.signers,
        )?;

        create_account(cpi_ctx, &Token::ID, Mint::LEN as u64)?;

        let cpi_ctx = CpiCtx::try_new(
            meta.token_program_view,
            InitializeMint2 { mint: view },
            meta.signers,
        )?;

        initialize_mint2(
            cpi_ctx,
            meta.decimals,
            meta.mint_authority,
            meta.freeze_authority,
        )?;

        Ok(Init(Account {
            view,
            _phantom: PhantomData,
        }))
    }
}

pub struct TokenMintInitMeta<'ix, 'a, 'b, 'c, 'd>
where
    'ix: 'a + 'd,
    'd: 'c,
    'c: 'b,
{
    pub decimals: u8,
    pub mint_authority: &'a Address,
    pub freeze_authority: Option<&'a Address>,
    pub payer_view: &'ix AccountView,
    pub system_program_view: &'ix AccountView,
    pub token_program_view: &'ix AccountView,
    pub signers: Option<&'b [Signer<'d, 'c>]>,
}

impl<'ix, 'a, 'b, 'c, 'd> TokenMintInitMeta<'ix, 'a, 'b, 'c, 'd>
where
    'ix: 'a + 'd,
    'd: 'c,
    'c: 'b,
{
    pub fn new(
        decimals: u8,
        mint_authority: &'a Address,
        freeze_authority: Option<&'a Address>,
        payer: &Mut<impl ToAccountView<'ix> + WritableAllowed>,
        system_program: &Program<'ix, System>,
        token_program: &Program<'ix, Token>,
        signers: Option<&'b [Signer<'d, 'c>]>,
    ) -> Self {
        Self {
            decimals,
            mint_authority,
            freeze_authority,
            payer_view: payer.to_account_view(),
            system_program_view: system_program.to_account_view(),
            token_program_view: token_program.to_account_view(),
            signers,
        }
    }
}

unsafe impl<'ix> FromAccountView<'ix> for Init<Account<'ix, TokenAccount, NoDiscriminator>> {
    type Meta<'a, 'b, 'c, 'd>
        = TokenAccountInitMeta<'ix, 'a, 'b, 'c, 'd>
    where
        'ix: 'a + 'd,
        'd: 'c,
        'c: 'b;

    fn try_from_account_view<'a, 'b, 'c, 'd>(
        view: &'ix AccountView,
        meta: Self::Meta<'a, 'b, 'c, 'd>,
    ) -> Result<Self>
    where
        'ix: 'a + 'd,
        'd: 'c,
        'c: 'b,
    {
        let _: Mut<UncheckedAccount> = Mut::try_from_account_view(view, NoMeta)?;

        let cpi_ctx = CpiCtx::try_new(
            meta.system_program_view,
            CreateAccount {
                from: meta.payer_view,
                to: view,
            },
            meta.signers,
        )?;

        create_account(cpi_ctx, &Token::ID, TokenAccount::LEN as u64)?;

        let cpi_ctx = CpiCtx::try_new(
            meta.token_program_view,
            InitializeAccount3 {
                account: view,
                mint: meta.mint_view,
            },
            meta.signers,
        )?;

        initialize_account3(cpi_ctx, meta.owner_pk)?;

        Ok(Init(Account {
            view,
            _phantom: PhantomData,
        }))
    }
}

pub struct TokenAccountInitMeta<'ix, 'a, 'b, 'c, 'd>
where
    'ix: 'a + 'd,
    'd: 'c,
    'c: 'b,
{
    pub owner_pk: &'a Address,
    pub payer_view: &'ix AccountView,
    pub mint_view: &'ix AccountView,
    pub system_program_view: &'ix AccountView,
    pub token_program_view: &'ix AccountView,
    pub signers: Option<&'b [Signer<'d, 'c>]>,
}

impl<'ix, 'a, 'b, 'c, 'd> TokenAccountInitMeta<'ix, 'a, 'b, 'c, 'd>
where
    'ix: 'a + 'd,
    'd: 'c,
    'c: 'b,
{
    pub fn new(
        owner_pk: &'a Address,
        payer: &Mut<impl ToAccountView<'ix> + WritableAllowed>,
        mint: &impl ToAccountView<'ix>,
        system_program: &Program<'ix, System>,
        token_program: &Program<'ix, Token>,
        signers: Option<&'b [Signer<'d, 'c>]>,
    ) -> Self {
        Self {
            owner_pk,
            payer_view: payer.to_account_view(),
            mint_view: mint.to_account_view(),
            system_program_view: system_program.to_account_view(),
            token_program_view: token_program.to_account_view(),
            signers,
        }
    }
}
