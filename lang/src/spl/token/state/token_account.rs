// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

use core::marker::PhantomData;

#[cfg(feature = "bpf")]
use crate::traits::Cast;
use crate::{
    prelude::*,
    spl::token::{
        instructions::InitializeAccount3,
        state::{account_state::AccountState, mint::Mint},
        Token,
    },
    syscalls::try_find_program_address,
    system_program::instructions::create_account::CreateAccount,
    traits::internal::{__AccountDiscriminatorMode, __NoDiscriminator},
};

/// Token account data.
#[derive(Clone)]
#[repr(C)]
pub struct TokenAccount {
    /// The mint associated with this account
    mint: Address,

    /// The owner of this account.
    owner: Address,

    /// The amount of tokens this account holds.
    amount: [u8; 8],

    /// Indicates whether the delegate is present or not.
    delegate_flag: [u8; 4],

    /// If `delegate` is `Some` then `delegated_amount` represents
    /// the amount authorized by the delegate.
    delegate: Address,

    /// The account's state.
    state: u8,

    /// Indicates whether this account represents a native token or not.
    is_native: [u8; 4],

    /// When `is_native.is_some()` is `true`, this is a native token, and the
    /// value logs the rent-exempt reserve. An Account is required to be
    /// rent-exempt, so the value is used by the Processor to ensure that
    /// wrapped SOL accounts do not drop below this threshold.
    native_amount: [u8; 8],

    /// The amount delegated.
    delegated_amount: [u8; 8],

    /// Indicates whether the close authority is present or not.
    close_authority_flag: [u8; 4],

    /// Optional authority to close the account.
    close_authority: Address,
}

impl TokenAccount {
    #[inline(always)]
    pub fn mint(&self) -> &Address {
        &self.mint
    }

    #[inline(always)]
    pub fn owner(&self) -> &Address {
        &self.owner
    }

    #[inline(always)]
    pub fn amount(&self) -> u64 {
        u64::from_le_bytes(self.amount)
    }

    #[inline(always)]
    pub fn has_delegate(&self) -> bool {
        self.delegate_flag[0] == 1
    }

    #[inline(always)]
    pub fn delegate(&self) -> Option<&Address> {
        if self.has_delegate() {
            Some(self.delegate_unchecked())
        } else {
            None
        }
    }

    /// Use this when you know the account will have a delegate and want to skip
    /// the `Option` check.
    #[inline(always)]
    pub fn delegate_unchecked(&self) -> &Address {
        &self.delegate
    }

    #[inline(always)]
    pub fn state(&self) -> AccountState {
        self.state.into()
    }

    #[inline(always)]
    pub fn is_native(&self) -> bool {
        self.is_native[0] == 1
    }

    #[inline(always)]
    pub fn native_amount(&self) -> Option<u64> {
        if self.is_native() {
            Some(self.native_amount_unchecked())
        } else {
            None
        }
    }

    /// Return the native amount.
    ///
    /// This method should be used when the caller knows that the token is
    /// native since it skips the `Option` check.
    #[inline(always)]
    pub fn native_amount_unchecked(&self) -> u64 {
        u64::from_le_bytes(self.native_amount)
    }

    #[inline(always)]
    pub fn delegated_amount(&self) -> u64 {
        u64::from_le_bytes(self.delegated_amount)
    }

    #[inline(always)]
    pub fn has_close_authority(&self) -> bool {
        self.close_authority_flag[0] == 1
    }

    #[inline(always)]
    pub fn close_authority(&self) -> Option<&Address> {
        if self.has_close_authority() {
            Some(self.close_authority_unchecked())
        } else {
            None
        }
    }

    /// Use this when you know the account will have a close authority and want to skip
    /// the `Option` check.
    #[inline(always)]
    pub fn close_authority_unchecked(&self) -> &Address {
        &self.close_authority
    }

    #[inline(always)]
    pub fn is_initialized(&self) -> bool {
        self.state != AccountState::Uninitialized as u8
    }

    #[inline(always)]
    pub fn is_frozen(&self) -> bool {
        self.state == AccountState::Frozen as u8
    }
}

// SAFETY: TokenAccount is a valid account layout.
unsafe impl __AccountMarker for TokenAccount {}

// SAFETY: TokenAccount is a valid SPL account layout.
unsafe impl __SplAccountMarker for TokenAccount {}

impl crate::traits::sealed::Sealed for TokenAccount {}

#[cfg(feature = "bpf")]
unsafe impl Cast for TokenAccount {}

impl Owner for TokenAccount {
    const OWNER: Address = Token::ID;
}

impl Space for TokenAccount {}

impl __AccountDiscriminatorMode for TokenAccount {
    type Mode = __NoDiscriminator;
}

impl<'view> AccountInit<'view, Account<'view, TokenAccount>> for TokenAccount {
    type Meta = __TokenAccountInitMeta<'view>;

    type PdaMeta<'a, 'b, 'c>
        = __TokenAccountPdaInitMeta<'view, 'a, 'b, 'c>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a;

    #[inline]
    fn init_account(
        view: AccountView<'view>,
        meta: &Self::Meta,
    ) -> Result<Account<'view, TokenAccount>> {
        let _: Mut<Signer<'_>> = Mut::parse(view, &mut NoMeta)?;

        invoke!(CreateAccount {
            from: meta.payer.to_account_view(),
            to: view,
            owner_program: &Token::ID,
            space: TokenAccount::SPACE as u64,
        });

        invoke!(InitializeAccount3 {
            account: view,
            mint: meta.mint.to_account_view(),
            owner: meta.owner,
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
    ) -> Result<Pda<Account<'view, TokenAccount>>>
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
                space: TokenAccount::SPACE as u64,
            },
            core::slice::from_ref(&meta.signer.as_signer()),
        );

        invoke!(InitializeAccount3 {
            account: view,
            mint: meta.mint.to_account_view(),
            owner: meta.owner,
        });

        Ok(Pda(Account {
            view,
            __phantom: PhantomData,
        }))
    }
}

pub struct __TokenAccountInitMeta<'view> {
    pub payer: Mut<Signer<'view>>,
    pub mint: Account<'view, Mint>,
    pub owner: &'view Address,
}

impl<'view> __TokenAccountInitMeta<'view> {
    #[inline(always)]
    pub fn new(
        payer: Mut<Signer<'view>>,
        mint: Account<'view, Mint>,
        owner: &'view Address,
    ) -> Self {
        Self { payer, mint, owner }
    }
}

pub struct __TokenAccountPdaInitMeta<'view, 'a, 'b, 'c>
where
    'view: 'c,
    'c: 'b,
    'b: 'a,
{
    pub payer: Mut<Signer<'view>>,
    pub mint: Account<'view, Mint>,
    pub owner: &'view Address,
    pub signer: GrowableSigner<'b, 'a>,
    pub bump_ptr: *mut u8,
    __phantom: PhantomData<&'c mut u8>,
}

impl<'view, 'a, 'b, 'c> __TokenAccountPdaInitMeta<'view, 'a, 'b, 'c>
where
    'view: 'c,
    'c: 'b,
    'b: 'a,
{
    #[inline(always)]
    pub fn new(
        payer: Mut<Signer<'view>>,
        mint: Account<'view, Mint>,
        owner: &'view Address,
        signer: GrowableSigner<'b, 'a>,
        bump_ref: &'c mut u8,
    ) -> Self {
        Self {
            payer,
            mint,
            owner,
            signer,
            bump_ptr: bump_ref as *mut _,
            __phantom: PhantomData,
        }
    }
}
