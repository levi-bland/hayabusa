// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use core::ops::Deref;

#[cfg(feature = "bpf")]
use crate::account_view::{Ref, RefMut};
use crate::{
    account_iter::AccountIter,
    account_meta::AccountMeta,
    account_view::AccountView,
    accounts::{mutable::Mut, signer::Signer},
    address::Address,
    cpi::{CpiAccount, CpiCtx, SignerBumpness},
    error::ErrorCode,
    hint, Result,
};
use hayabusa_err_macro::err;
use solana_program_error::ProgramError;

// ============= MARKERS =============

/// Denotes whether an account wrapper type can be writable or not.
///
/// # Safety
///
/// Some account types like [`Program<'_, T>`] are executable and therefore cannot
/// be writable, ever. This trait must be unsafe because manual proof of an
/// account type being capable of being writable is necessary.
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be wrapped in `Mut<...>`",
    label = "account type not writable",
    note = "only non-executable accounts can be wrapped in `Mut<...>`, `Program<'_, T>` is not writable"
)]
pub unsafe trait __WritableAllowed {}

/// Denotes whether an account wrapper type can be a PDA or not.
///
/// # Safety
///
/// Some account types can never be a PDA, i.e. [`Sysvar<T>`], [`Program<'_, T>`].
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be wrapped in `Pda<...>`",
    label = "invalid wrapper order",
    note = "put `Mut` on the outside: `Mut<Pda<...>>`"
)]
pub unsafe trait __PdaAllowed {}

pub unsafe trait __InitAllowed {}

/// Marker trait to denote that a type is an account.
///
/// # Safety
///
/// This trait is unsafe to implement for types that are not Solana
/// accounts, because it is the marker trait required for other unsafe
/// traits like [`Cast`], for which the default impls are not guarnateed
/// to be sound for any architecture other than sBPF
/// (assuming account type level invariants upheld).
pub unsafe trait __AccountMarker {}

pub unsafe trait __SplAccountMarker {}

mod sealed {
    pub trait Sealed {}
}

#[doc(hidden)]
pub trait __InterfaceAccountType {}

#[doc(hidden)]
pub unsafe trait __DiscriminatorMarker: sealed::Sealed {}

#[doc(hidden)]
pub unsafe trait __OwnerTypeMarker: sealed::Sealed {}

#[doc(hidden)]
pub struct __WithDiscriminator;
impl sealed::Sealed for __WithDiscriminator {}
unsafe impl __DiscriminatorMarker for __WithDiscriminator {}

#[doc(hidden)]
pub struct __NoDiscriminator;
impl sealed::Sealed for __NoDiscriminator {}
unsafe impl __DiscriminatorMarker for __NoDiscriminator {}

#[doc(hidden)]
pub struct __SplAccount;
impl sealed::Sealed for __SplAccount {}
unsafe impl __DiscriminatorMarker for __SplAccount {}

#[doc(hidden)]
pub struct __Owner;
impl sealed::Sealed for __Owner {}
unsafe impl __OwnerTypeMarker for __Owner {}

#[doc(hidden)]
pub struct __Owners;
impl sealed::Sealed for __Owners {}
unsafe impl __OwnerTypeMarker for __Owners {}

#[doc(hidden)]
pub unsafe trait __AccountDiscriminatorMode {
    type Mode: __DiscriminatorMarker;
}

// ============= IDS & OWNERSHIP =============

/// Used by a type like `Program<'view, T>` to denote the program id of `T`.
pub trait Id {
    const ID: &Address;
}

/// Used by a type like `Interface<'view, T>` to denote the possible program ids of `T`.
pub trait Ids {
    const IDS: &[Address];
}

/// Trait for checking ownership of an account by an owner program(s).
///
/// ## Safety
///
/// This trait is unsafe to implement because `Self::OwnerType` must
/// be set to the correct marker trait, i.e. [`__Owner`] or [`__Owners`].
/// The compiler can't verify correctness, so if your account type implements
/// [`Owner`] but supplies [`__Owners`] instead of [`__Owner`] as the associated type,
/// downstream trait implementations won't exist.
pub unsafe trait Ownership {
    type OwnerType: __OwnerTypeMarker;

    fn check_ownership(view: AccountView<'_>) -> Result<()>;
}

/// Trait denoting the owner program of an account, and providing a method to check ownership.
pub trait Owner {
    const OWNER: &Address;

    #[inline(always)]
    fn check_owner(view: AccountView<'_>) -> Result<()> {
        if hint::unlikely(!view.owned_by(Self::OWNER)) {
            err!(
                "Owner::check_owner: invalid owner, expected {} but got {}",
                ErrorCode::InvalidAccountOwner,
                Self::OWNER,
                view.owner_owned(),
            );
        }

        Ok(())
    }
}

/// Trait denoting the possible owner programs of an account, and providing a method to check ownership.
pub trait Owners {
    const OWNERS: &[Address];

    #[inline(always)]
    fn check_owners(view: AccountView<'_>) -> Result<()> {
        if hint::unlikely(!view.owned_by_any(Self::OWNERS)) {
            err!(
                "Owners::check_owners: invalid owner",
                ErrorCode::InvalidAccountOwner,
            );
        }

        Ok(())
    }

    #[inline(always)]
    fn check_specified_owner(view: AccountView<'_>, owner: &Address) -> Result<()> {
        if hint::unlikely(!view.owned_by(owner)) {
            err!(
                "Owners::check_specified_owner: invalid owner",
                ErrorCode::InvalidAccountOwner,
            );
        }

        Ok(())
    }
}

// ============= DISCRIMINATORS =============

/// Trait denoting a variable length discriminator for an account (capped, and by default at 8 bytes).
pub trait Discriminator {
    /// The discriminator bytes.
    const DISCRIMINATOR: &[u8];

    /// Checks the discriminator of an account, rejects the transaction if the discriminator is invalid.
    #[inline(always)]
    fn check_discriminator(view: AccountView<'_>) -> Result<()> {
        if hint::unlikely(view.data_len() < Self::DISCRIMINATOR.len()) {
            err!(
                "Discriminator::check_discriminator: account too short for a discriminator",
                ErrorCode::InvalidAccountLength,
            );
        }

        let discriminator = &view.try_borrow()?[..Self::DISCRIMINATOR.len()];

        if hint::unlikely(discriminator != Self::DISCRIMINATOR) {
            err!(
                "Discriminator::check_discriminator: invalid discriminator, expected {} but got {}",
                ErrorCode::InvalidAccountDiscriminator,
                Self::DISCRIMINATOR,
                discriminator,
            );
        }

        Ok(())
    }

    /// Checks the discriminator of an account, rejects the transaction if the discriminator is invalid.
    ///
    /// ## Safety
    ///
    /// This associated function is unsafe because it does not check the length of the account.
    /// Ensure `view.data_len() >= Self::DISCRIMINATOR.len()` before calling this function.
    #[inline(always)]
    unsafe fn check_discriminator_skip_length_check(view: AccountView<'_>) -> Result<()> {
        let discriminator = &view.try_borrow()?[..Self::DISCRIMINATOR.len()];

        if hint::unlikely(discriminator != Self::DISCRIMINATOR) {
            err!(
                "Discriminator::check_discriminator_skip_length_check: invalid discriminator, expected {} but got {}",
                ErrorCode::InvalidAccountDiscriminator,
                Self::DISCRIMINATOR,
                discriminator,
            );
        }

        Ok(())
    }
}

/// Trait denoting the length/space of an account, size in bytes.
pub trait Space: Sized + __AccountMarker {
    /// The space/length of `Self` in bytes.
    const SPACE: usize = core::mem::size_of::<Self>();

    /// Validate that the account is the correct size.
    #[inline(always)]
    fn check_space(view: AccountView<'_>) -> Result<()> {
        if hint::unlikely(view.data_len() != Self::SPACE) {
            err!(
                "Space::check_space: invalid account length",
                ErrorCode::InvalidAccountLength,
            );
        }

        Ok(())
    }
}

impl<T: Discriminator + __AccountMarker> Space for T {
    const SPACE: usize = Self::DISCRIMINATOR.len() + core::mem::size_of::<T>();
}

/// Trait denoting the type storing the bumps of the account in a `Ctx<T>`.
pub trait Bumps {
    type Bumps: Sized + Default;
}

/// ============= CASTING =============

/// Trait for casting raw account data pointers into typed references.
///
/// # Safety
///
/// The [`Cast`] trait is unsafe to implement because it uses unsafe pointer casts internally.
/// These operations are not guaranteed to be sound unless the alignment of `Self` and
/// `view.data_mut_ptr()` are identical, `view.data_len()` and `size_of::<Self>()` are identical.
/// (or `size_of::<Self>() + Discriminator::LENGTH` for accounts with a discriminator). `view.data_mut_ptr()` must
/// be a valid value of `Self`, no invalid bit patterns. The Solana runtime handles proper alignment of account data. (entrypoint)
#[cfg(feature = "bpf")]
pub unsafe trait Cast: Ownership + Space + __AccountMarker {
    /// Get a `&T` from an [`AccountView`] with a raw ptr cast.
    ///
    /// # Safety
    ///
    /// Caller must ensure the account isn't aliased by a mutable pointer,
    /// and that account level invariants such as: discriminator, data length,
    /// owner, etc... are all validated.
    #[inline(always)]
    unsafe fn cast_untracked(view: AccountView<'_>) -> &Self {
        &*(view.data_ptr() as *const Self)
    }

    /// Get a `&mut T` from an [`AccountView`] with a raw ptr cast.
    ///
    /// # Safety
    ///
    /// Caller must ensure the account isn't aliased by any type of pointer,
    /// and that account level invariants such as: discriminator, data length,
    /// owner, etc... are all validated.
    #[allow(clippy::mut_from_ref)]
    #[inline(always)]
    unsafe fn cast_mut_untracked(view: AccountView<'_>) -> &mut Self {
        &mut *(view.data_mut_ptr() as *mut Self)
    }

    /// Get a [`Ref<T>`] without checking account-level invariants.
    ///
    /// # Safety
    ///
    /// Caller must ensure the account isn't aliased by any mutable pointer,
    /// and that account level invariants such as: discriminator, data length,
    /// owner, etc... are all validated.
    unsafe fn cast_unchecked(view: AccountView<'_>) -> Result<Ref<'_, Self>> {
        Ok(Ref::map(view.try_borrow()?, |data| {
            &*(data.as_ptr() as *const Self)
        }))
    }

    /// Get a [`RefMut<T>`] without checking account-level invariants.
    ///
    /// # Safety
    ///
    /// Caller must ensure the account isn't aliased by any type of pointer,
    /// and that account level invariants such as: discriminator, data length,
    /// owner, etc... are all validated.
    unsafe fn cast_mut_unchecked(view: AccountView<'_>) -> Result<RefMut<'_, Self>> {
        Ok(RefMut::map(view.try_borrow_mut()?, |data| {
            &mut *(data.as_mut_ptr() as *mut Self)
        }))
    }

    /// Safely get a [`Ref<T>`] from an [`AccountView`].
    ///
    /// # Errors
    ///
    /// Returns an error if the account data length or owner is invalid.
    /// Will also return an error if the implementing type implements
    /// [`Discriminator`] and the discriminator does not match.
    #[inline(always)]
    fn cast(view: AccountView<'_>) -> Result<Ref<'_, Self>> {
        Self::check_space(view)?;
        Self::check_ownership(view)?;

        // SAFETY: The required invariants are checked above.
        unsafe { Self::cast_unchecked(view) }
    }

    /// Safely get a [`RefMut<T>`] from an [`AccountView`].
    ///
    /// # Errors
    ///
    /// Returns an error if the account data length or owner is invalid.
    /// Will also return an error if the implementing type implements
    /// [`Discriminator`] and the discriminator does not match.
    #[inline(always)]
    fn cast_mut(view: AccountView<'_>) -> Result<RefMut<'_, Self>> {
        Self::check_space(view)?;
        Self::check_ownership(view)?;

        // SAFETY: The required invariants are checked above.
        unsafe { Self::cast_mut_unchecked(view) }
    }
}

#[cfg(feature = "bpf")]
unsafe impl<T: Discriminator + Ownership + Space + __AccountMarker> Cast for T {
    #[inline(always)]
    unsafe fn cast_untracked(view: AccountView<'_>) -> &T {
        let undiscriminated = view
            .data_ptr()
            .add(<T as Discriminator>::DISCRIMINATOR.len());
        &*(undiscriminated as *const T)
    }

    #[inline(always)]
    unsafe fn cast_mut_untracked(view: AccountView<'_>) -> &mut T {
        let undiscriminated = view
            .data_mut_ptr()
            .add(<T as Discriminator>::DISCRIMINATOR.len());
        &mut *(undiscriminated as *mut T)
    }

    #[inline(always)]
    unsafe fn cast_unchecked(view: AccountView<'_>) -> Result<Ref<'_, T>> {
        Ok(Ref::map(view.try_borrow()?, |data| {
            let undiscriminated = data.as_ptr().add(<T as Discriminator>::DISCRIMINATOR.len());
            &*(undiscriminated as *const T)
        }))
    }

    #[inline(always)]
    unsafe fn cast_mut_unchecked(view: AccountView<'_>) -> Result<RefMut<'_, T>> {
        Ok(RefMut::map(view.try_borrow_mut()?, |data| {
            let undiscriminated = data
                .as_mut_ptr()
                .add(<T as Discriminator>::DISCRIMINATOR.len());
            &mut *(undiscriminated as *mut T)
        }))
    }

    #[inline(always)]
    fn cast(view: AccountView<'_>) -> Result<Ref<'_, T>> {
        T::check_space(view)?;
        T::check_ownership(view)?;
        T::check_discriminator(view)?;

        // SAFETY: The required invariants are checked above.
        unsafe { T::cast_unchecked(view) }
    }

    #[inline(always)]
    fn cast_mut(view: AccountView<'_>) -> Result<RefMut<'_, T>> {
        T::check_space(view)?;
        T::check_ownership(view)?;
        T::check_discriminator(view)?;

        // SAFETY: The required invariants are checked above.
        unsafe { T::cast_mut_unchecked(view) }
    }
}

// ============= PARSING =============

/// Trait for parsing a list of [`AccountView`] into a `T`.
pub trait ParseAccounts<'view, Bumps>: Sized {
    /// The number of named accounts to parse.
    const NUM_ACCOUNTS: usize;

    /// Parse the accounts into a `T`.
    fn parse_accounts(
        views: &mut AccountIter<'view>,
        data: &[u8],
        bumps: &mut Bumps,
    ) -> Result<Self>;
}

/// Parse an account wrapper type from an [`AccountView`], plus optional metadata.
pub trait ParseAccount<'view>: Sized {
    type Meta<'a, 'b, 'c>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a;

    fn parse<'a, 'b, 'c>(
        view: AccountView<'view>,
        meta: &mut Self::Meta<'a, 'b, 'c>,
    ) -> Result<Self>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a;
}

impl<'view, T: ParseAccount<'view>> ParseAccount<'view> for Option<T> {
    type Meta<'a, 'b, 'c>
        = __OptionMeta<<T as ParseAccount<'view>>::Meta<'a, 'b, 'c>>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a;

    #[inline(always)]
    fn parse<'a, 'b, 'c>(
        view: AccountView<'view>,
        meta: &mut Self::Meta<'a, 'b, 'c>,
    ) -> Result<Option<T>>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a,
    {
        if hint::unlikely(crate::address::address_eq(
            view.address(),
            meta.null_account,
        )) {
            return Ok(None);
        }

        Ok(Some(T::parse(view, &mut meta.meta)?))
    }
}

pub struct __OptionMeta<M> {
    null_account: &'static Address,
    meta: M,
}

/// ZST representing no metadata.
pub struct NoMeta;

impl<'view> ParseAccount<'view> for AccountView<'view> {
    type Meta<'a, 'b, 'c>
        = NoMeta
    where
        'view: 'c,
        'c: 'b,
        'b: 'a;

    #[inline(always)]
    fn parse<'a, 'b, 'c>(
        view: AccountView<'view>,
        _: &mut Self::Meta<'a, 'b, 'c>,
    ) -> Result<AccountView<'view>>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a,
    {
        Ok(view)
    }
}

/// Internal framework trait to dispatch at compile time to respective account discriminator modes.
pub trait __ParseAccountDispatch<'view, D: __DiscriminatorMarker, A: ParseAccount<'view>> {
    fn dispatch(view: AccountView<'view>) -> Result<A>;
}

/// Get a sysvar from an [`AccountView`].
pub trait ParseSysvar<'view>: Sized {
    fn parse(view: AccountView<'view>) -> Result<Self>;
}

/// Get a copy of the inner [`AccountView`] from an account wrapper type like `Account<'view, T>`.
pub trait ToAccountView<'view> {
    /// Returns a copy of the [`AccountView`] stored inside `Self`.
    fn to_account_view(&self) -> AccountView<'view>;
}

/// Get the address of an account from a wrapper like `Account<'view, T>`.
pub trait AddressTrait<'view> {
    /// Returns a reference to the [`Address`] of the account.
    fn address(&self) -> &'view Address;

    /// Returns the [`Address`] of the account.
    fn address_owned(&self) -> Address;
}

/// Serialize a CPI call into the expected wire format.
pub trait CpiSerialize<'view, M, V, D>
where
    M: AsRef<[AccountMeta<'view>]>,
    V: AsRef<[CpiAccount<'view>]>,
    D: AsRef<[u8]>,
{
    fn serialize(&self) -> Result<CpiCtx<'view, 'view, '_, M, V, D>>;
}

/// A trait for types that hold sysvar data.
pub trait Sysvar: Sized {
    /// Load the sysvar directly from the runtime.
    ///
    /// This is the preferred way to load a sysvar. Calling this method does not
    /// incur any deserialization overhead, and does not require the sysvar
    /// account to be passed to the program.
    ///
    /// Not all sysvars support this method. If not, it returns
    /// [`ProgramError::UnsupportedSysvar`].
    fn get() -> Result<Self> {
        Err(ProgramError::UnsupportedSysvar)
    }
}

#[cfg(feature = "bpf")]
pub trait AccountInit<'view> {
    type Meta<'a>
    where
        'view: 'a;

    type PdaMeta<'a, 'b, 'c>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a;

    fn init<'a>(view: AccountView<'view>, meta: &Self::Meta<'a>) -> Result<()>
    where
        'view: 'a;

    fn init_pda<'a, 'b, 'c>(
        view: AccountView<'view>,
        meta: &mut Self::PdaMeta<'a, 'b, 'c>,
    ) -> Result<()>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a;
}

#[allow(unused)]
pub struct __AccountInitMeta<'view, 'a> {
    pub payer: &'a Mut<Signer<'view>>,
}

#[allow(unused)]
impl<'view, 'a> __AccountInitMeta<'view, 'a> {
    pub fn new(payer: &'a Mut<Signer<'view>>) -> Self {
        Self { payer }
    }
}

#[allow(unused)]
pub struct __AccountPdaInitMeta<'view, 'a, 'b, 'c> {
    pub payer: &'c Mut<Signer<'view>>,
    pub signer: SignerBumpness<'b, 'a, 'c>,
}

#[allow(unused)]
impl<'view, 'a, 'b, 'c> __AccountPdaInitMeta<'view, 'a, 'b, 'c> {
    pub fn new(payer: &'c Mut<Signer<'view>>, signer: SignerBumpness<'b, 'a, 'c>) -> Self {
        Self { payer, signer }
    }
}

/// A trait for dispatching enum accounts.
///
/// ```ignore
/// #[derive(EnumAccountDispatch)]
/// pub enum AccountEnum<'view> {
///     Account1(Account<'view, T>),
///     Account2(Account<'view, U>),
/// }
/// ```
pub unsafe trait EnumAccountDispatch<'view>: Sized + __EnumAccountMarker {
    type RefEnum: Sized;
    type RefMutEnum: Sized;
    type UntrackedRefEnum: Sized;
    type UntrackedMutEnum: Sized;

    fn validate_account(view: AccountView<'view>) -> Result<Self>;

    fn load(&self) -> Result<Self::RefEnum>;
    unsafe fn load_untracked(&self) -> Self::UntrackedRefEnum;

    fn load_mut(&self) -> Result<Self::RefMutEnum>;
    unsafe fn load_mut_untracked(&self) -> Self::UntrackedMutEnum;
}

pub trait __EnumAccountMarker {}

/// ## Safety
///
/// This trait is unsafe to implement manually because it is possible to alias
/// or return a corrupted misrepresentation of the account memory, i.e. by returning
/// an incorrect account type marker in `validate_account`. Use the [`impl_dyn_account_dispatch!`]
/// macro to implement this trait safely.
///
/// ```ignore
/// impl_dyn_account_dispatch!(MyAccountDispatch, MyAccountType1, MyAccountType2, MyAccountType3, ...);
/// ```
pub unsafe trait DynAccountDispatch<'view> {
    type Object: ?Sized;
    type AccountType: Copy;

    fn build(view: ValidatedView<'view, Self::AccountType>) -> Result<Ref<'view, Self::Object>>;

    /// # Safety
    /// This method is unsafe because it is possible to alias the
    /// underlying account memory.
    unsafe fn build_untracked(view: ValidatedView<'view, Self::AccountType>)
        -> &'view Self::Object;

    /// # Safety
    /// This method does NOT check that the underlying [`AccountView`] is mutable,
    /// if you attempt to access the returned mutable pointer and the account is immutable,
    /// you will cause a runtime panic.
    unsafe fn build_mut(
        view: ValidatedView<'view, Self::AccountType>,
    ) -> Result<RefMut<'view, Self::Object>>;

    /// # Safety
    /// This method is unsafe because it is possible to alias the
    /// underlying account memory.
    unsafe fn build_mut_untracked(
        view: ValidatedView<'view, Self::AccountType>,
    ) -> &'view mut Self::Object;

    fn validate_account(
        view: AccountView<'view>,
    ) -> Result<ValidatedView<'view, Self::AccountType>>;
}

#[derive(Clone, Copy)]
pub struct ValidatedView<'view, A: Copy>(AccountView<'view>, A);

impl<'view, A: Copy> ValidatedView<'view, A> {
    pub fn marker(&self) -> A {
        self.1
    }

    /// # Safety
    /// You must manually ensure the invariants of this type are upheld.
    /// Invariants are dependent on the specific use case.
    pub unsafe fn new(view: AccountView<'view>, marker: A) -> ValidatedView<'view, A> {
        ValidatedView(view, marker)
    }
}

impl<'view, A: Copy> ToAccountView<'view> for ValidatedView<'view, A> {
    #[inline(always)]
    fn to_account_view(&self) -> AccountView<'view> {
        self.0
    }
}

impl<'view, A: Copy> Deref for ValidatedView<'view, A> {
    type Target = AccountView<'view>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait DynamicPdaValidate<'view> {
    type Meta;

    /// ## Safety
    /// This associated function can use [`Cast::cast_unchecked`], owner, length
    /// and discriminator must be validated before calling this function.
    unsafe fn validate_pda(view: AccountView<'view>, meta: &Self::Meta) -> Result<()>;
}
