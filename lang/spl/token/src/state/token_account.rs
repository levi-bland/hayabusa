// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "bpf")]
use hayabusa_common::traits::Cast;
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
        __AccountDiscriminatorMode, __AccountMarker, __Owner, __ParseAccountDispatch, __SplAccount,
        __SplAccountMarker, AccountInit, CpiSerialize, Id, NoMeta, Owner, Ownership, ParseAccount,
        Space, ToAccountView,
    },
};
use hayabusa_err_macro::err;
use hayabusa_invoke_cpi_macros::invoke;
use solana_program_error::ProgramError;

use crate::{
    Token,
    instructions::InitializeAccount3,
    state::{account_state::AccountState, mint::Mint},
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

unsafe impl __AccountMarker for TokenAccount {}

unsafe impl __SplAccountMarker for TokenAccount {}

#[cfg(feature = "bpf")]
unsafe impl Cast for TokenAccount {}

impl Owner for TokenAccount {
    const OWNER: &Address = Token::ID;
}

unsafe impl Ownership for TokenAccount {
    type OwnerType = __Owner;

    #[inline(always)]
    fn check_ownership(view: AccountView<'_>) -> Result<()> {
        Self::check_owner(view)
    }
}

impl Space for TokenAccount {}

unsafe impl __AccountDiscriminatorMode for TokenAccount {
    type Mode = __SplAccount;
}

impl<'view>
    __ParseAccountDispatch<
        'view,
        <TokenAccount as __AccountDiscriminatorMode>::Mode,
        Account<'view, TokenAccount>,
    > for TokenAccount
{
    #[inline(always)]
    fn dispatch(view: AccountView<'view>) -> Result<Account<'view, TokenAccount>> {
        TokenAccount::check_owner(view)?;
        TokenAccount::check_space(view)?;

        // SAFETY: `TokenAccount::cast_unchecked` is safe because we have already checked the owner and space.
        if hint::unlikely(!unsafe { TokenAccount::cast_unchecked(view)? }.is_initialized()) {
            err!(
                "TokenAccount::dispatch: account is not initialized",
                ErrorCode::InvalidAccount,
            );
        }

        Ok(unsafe { Account::new(view) })
    }
}

#[cfg(feature = "bpf")]
impl<'view> AccountInit<'view> for TokenAccount {
    type Meta<'a>
        = __TokenAccountInitMeta<'view, 'a>
    where
        'view: 'a;

    type PdaMeta<'a, 'b, 'c>
        = __TokenAccountPdaInitMeta<'view, 'a, 'b, 'c>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a;

    #[inline(never)]
    fn init<'a>(view: AccountView<'view>, meta: &Self::Meta<'a>) -> Result<()>
    where
        'view: 'a,
    {
        // non-PDA's must have signed the transaction
        let _: Signer<'view> = Signer::parse(view, &mut NoMeta)?;

        create_or_allocate_account(
            meta.payer.to_account_view(),
            view,
            Token::ID,
            TokenAccount::SPACE,
        )?;

        invoke!(InitializeAccount3 {
            account: view,
            mint: meta.mint.to_account_view(),
            owner: meta.owner,
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
        match &mut meta.signer {
            SignerBumpness::With(signer) => {
                let addr =
                    try_create_program_address(signer.as_slice_of_slices(), meta.program_id)?;

                if hint::unlikely(!address_eq(&addr, view.address())) {
                    err!(
                        "TokenAccount::init_pda: PDA address mismatch: expected {}, got {}",
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
                    TokenAccount::SPACE,
                )?;
            }
            SignerBumpness::Without(signer, bump_slot) => {
                let (addr, bump) =
                    try_find_program_address(signer.as_slice_of_slices(), meta.program_id)?;

                if hint::unlikely(!address_eq(&addr, view.address())) {
                    err!(
                        "TokenAccount::init_pda: PDA address mismatch: expected {}, got {}",
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
                    TokenAccount::SPACE,
                )?;
            }
        }

        invoke!(InitializeAccount3 {
            account: view,
            mint: meta.mint.to_account_view(),
            owner: meta.owner,
        });

        Ok(())
    }
}

#[cfg(feature = "bpf")]
pub struct __TokenAccountInitMeta<'view, 'a> {
    pub payer: &'a Mut<Signer<'view>>,
    pub mint: &'a Account<'view, Mint>,
    pub owner: &'view Address,
}

#[cfg(feature = "bpf")]
impl<'view: 'a, 'a> __TokenAccountInitMeta<'view, 'a> {
    #[inline(always)]
    pub fn new(
        payer: &'a Mut<Signer<'view>>,
        mint: &'a Account<'view, Mint>,
        owner: &'view Address,
    ) -> Self {
        Self { payer, mint, owner }
    }
}

#[cfg(feature = "bpf")]
pub struct __TokenAccountPdaInitMeta<'view, 'a, 'b, 'c> {
    pub payer: &'c Mut<Signer<'view>>,
    pub mint: &'c Account<'view, Mint>,
    pub owner: &'view Address,
    pub program_id: &'view Address,
    pub signer: SignerBumpness<'b, 'a, 'c>,
}

#[cfg(feature = "bpf")]
impl<'view, 'a, 'b, 'c> __TokenAccountPdaInitMeta<'view, 'a, 'b, 'c> {
    #[inline(always)]
    pub fn new(
        payer: &'c Mut<Signer<'view>>,
        mint: &'c Account<'view, Mint>,
        owner: &'view Address,
        program_id: &'view Address,
        signer: SignerBumpness<'b, 'a, 'c>,
    ) -> Self {
        Self {
            payer,
            mint,
            owner,
            program_id,
            signer,
        }
    }
}
