// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use core::marker::PhantomData;

use crate::{
    prelude::*,
    spl::token::{instructions::InitializeMint2, Token},
    syscalls::try_find_program_address,
    system_program::instructions::create_account::CreateAccount,
    traits::internal::{__AccountDiscriminatorMode, __NoDiscriminator},
};

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

impl crate::traits::sealed::Sealed for Mint {}

#[cfg(feature = "bpf")]
unsafe impl Cast for Mint {}

impl Owner for Mint {
    const OWNER: Address = Token::ID;
}

impl Space for Mint {}

impl __AccountDiscriminatorMode for Mint {
    type Mode = __NoDiscriminator;
}

impl<'view> AccountInit<'view, Account<'view, Mint>> for Mint {
    type Meta = __TokenMintInitMeta<'view>;

    type PdaMeta<'a, 'b, 'c>
        = __TokenMintPdaInitMeta<'view, 'a, 'b, 'c>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a;

    #[inline]
    fn init_account(view: AccountView<'view>, meta: &Self::Meta) -> Result<Account<'view, Mint>> {
        let _: Mut<Signer<'_>> = Mut::parse(view, &mut NoMeta)?;

        invoke!(CreateAccount {
            from: meta.payer.to_account_view(),
            to: view,
            owner_program: &Token::ID,
            space: Mint::SPACE as u64,
        });

        invoke!(InitializeMint2 {
            mint: view,
            decimals: meta.decimals,
            mint_authority: meta.mint_authority,
            freeze_authority: meta.freeze_authority,
        });

        Ok(Account {
            view,
            __phantom: PhantomData,
        })
    }

    #[inline]
    fn init_pda_account<'a, 'b, 'c>(
        view: AccountView<'view>,
        meta: &mut Self::PdaMeta<'a, 'b, 'c>,
    ) -> Result<Pda<Account<'view, Mint>>>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a,
    {
        let _: Mut<UncheckedAccount<'_>> = Mut::parse(view, &mut NoMeta)?;

        let (_, bump) = try_find_program_address(meta.signer.as_slice_of_slices(), &Token::ID)?;
       
        // SAFETY: `meta.bump_ptr` is a valid `'c`-rooted pointer (guaranteed by
        // calling code), and `'c: 'b`, so a reborrow of it lives at least `'b`
        // satisfying `CpiSigner<'b, 'a>`'s seed lifetime. Write the bump, then
        // hand a `Seed` over the same byte to the signer.
        unsafe {
            meta.bump_ptr.write(bump);
            let bump_byte: &'b [u8] = core::slice::from_raw_parts(meta.bump_ptr as *const u8, 1);
            meta.signer.push(Seed::from(bump_byte));
        }

        invoke_with_signers!(
            CreateAccount {
                from: meta.payer.to_account_view(),
                to: view,
                owner_program: &Token::ID,
                space: Mint::SPACE as u64,
            },
            &[meta.signer.as_signer()],
        );

        invoke!(InitializeMint2 {
            mint: view,
            decimals: meta.decimals,
            mint_authority: meta.mint_authority,
            freeze_authority: meta.freeze_authority,
        });

        Ok(Pda(Account {
            view,
            __phantom: PhantomData,
        }))
    }
}

pub struct __TokenMintInitMeta<'view> {
    pub payer: Mut<Signer<'view>>,
    pub mint_authority: &'view Address,
    pub freeze_authority: Option<&'view Address>,
    pub decimals: u8,
}

impl<'view> __TokenMintInitMeta<'view> {
    #[inline(always)]
    pub fn new(
        payer: Mut<Signer<'view>>,
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

pub struct __TokenMintPdaInitMeta<'view, 'a, 'b, 'c>
where
    'view: 'c,
    'c: 'b,
    'b: 'a,
{
    pub payer: Mut<Signer<'view>>,
    pub mint_authority: &'view Address,
    pub freeze_authority: Option<&'view Address>,
    pub decimals: u8,
    pub signer: GrowableSigner<'b, 'a>,
    pub bump_ptr: *mut u8,
    __phantom: PhantomData<&'c mut u8>,
}

impl<'view, 'a, 'b, 'c> __TokenMintPdaInitMeta<'view, 'a, 'b, 'c>
where
    'view: 'c,
    'c: 'b,
    'b: 'a,
{
    #[inline(always)]
    pub fn new(
        payer: Mut<Signer<'view>>,
        mint_authority: &'view Address,
        freeze_authority: Option<&'view Address>,
        decimals: u8,
        signer: GrowableSigner<'b, 'a>,
        bump_ref: &'c mut u8,
    ) -> Self {
        Self {
            payer,
            mint_authority,
            freeze_authority,
            decimals,
            signer,
            bump_ptr: bump_ref as *mut u8,
            __phantom: PhantomData,
        }
    }
}
