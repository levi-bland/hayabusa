// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

use core::marker::PhantomData;

use crate::{
    prelude::*, syscalls::try_find_program_address,
    system_program::instructions::create_account::CreateAccount,
};

/// Used by `Program<'view, T>` to denote the program id of `T`
/// where `T` is a unit struct
pub trait ProgramId {
    const ID: Address;
}

/// Used by `Interface<'view, T>` to denote the possible
/// program ids of `T`
pub trait Ids {
    const IDS: &[Address];
}

/// Used to denote the program that owns an account
pub trait Owner {
    const OWNER: Address;

    #[inline(always)]
    fn check_owner(addr: &Address) -> Result<()> {
        if hint::unlikely(addr != &Self::OWNER) {
            err!("Owner::check_owner: invalid owner", ErrorCode::InvalidAccountOwner);
        }

        Ok(())
    }
}

/// 8-byte SHA-256 derived discriminator
pub trait Discriminator {
    /// The 8-byte discriminator prepending Hayabusa account data
    const DISCRIMINATOR: &[u8; 8];

    #[inline(always)]
    fn check_discriminator(view: AccountView<'_>) -> Result<()> {
        if hint::unlikely(view.data_len() < Self::DISCRIMINATOR.len()) {
            err!(
                "Discriminator::check_discriminator: account too short for a discriminator",
                ErrorCode::InvalidAccountLength,
            );
        }

        // SAFETY: We have ensured the account data is at least 8 bytes.
        unsafe { Self::check_discriminator_unchecked(view) }
    }

    /// Check account discriminator without checking account data length before casting.
    ///
    /// # Safety
    ///
    /// If the ptr is not long enough to cast into a `&[u8; 8]` it will cause UB.
    #[inline(always)]
    unsafe fn check_discriminator_unchecked(view: AccountView<'_>) -> Result<()> {
        let discriminator = &*(view.data_ptr() as *const [u8; 8]);

        if hint::unlikely(discriminator != Self::DISCRIMINATOR) {
            err!(
                "Discriminator::check_discriminator_unchecked: invalid discriminator",
                ErrorCode::InvalidAccountDiscriminator,
            );
        }

        Ok(())
    }
}

/// The space an account requires; its size in bytes
pub trait Space: Sized + __AccountMarker {
    /// The space/length of `Self` in bytes.
    const SPACE: usize = core::mem::size_of::<Self>();

    /// Validate that received `space` value is identical
    /// to `Self::SPACE`.
    #[inline(always)]
    fn check_space(space: usize) -> Result<()> {
        if hint::unlikely(space != Self::SPACE) {
            err!(
                "check_space: invalid account length",
                ErrorCode::InvalidAccountLength,
            );
        }

        Ok(())
    }
}

impl<T: Discriminator + __AccountMarker> Space for T {
    const SPACE: usize = 8 + core::mem::size_of::<Self>();
}

pub trait Bumps {
    type Bumps: Sized + Default;
}

/// Denote the byte in which a pda bump is stored in a particular account type
pub trait BumpOffset: Sized {
    /// The offset of the account's `bump` field
    const BUMP_OFFSET: usize;

    /// Load the bump directly by its field offset
    ///
    /// # Safety
    ///
    /// This associated function assumes the view's data_len is at least `BUMP_OFFSET`
    /// and that the bump byte is not currently aliased by a mutable borrow (although
    /// in practice it doesn't matter).
    #[inline(always)]
    unsafe fn read_bump(view: AccountView<'_>) -> u8 {
        let ptr = view.data_ptr();
        *(ptr.add(Self::BUMP_OFFSET))
    }
}

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

pub(crate) mod sealed {
    pub trait Sealed {}
}

/// Sealed marker trait to denote that a type is a SPL account.
pub unsafe trait __SplAccountMarker: sealed::Sealed {}

/// Trait for casting raw account data pointers into typed references.
///
/// # Safety
///
/// The [`Cast`] trait is unsafe to implement because it uses unsafe pointer casts internally.
/// These operations are not guaranteed to be sound unless the alignment of `Self` and
/// `view.data_mut_ptr()` are identical, `view.data_len()` and `size_of::<Self>()` are identical.
/// (or `size_of::<Self>() - 8` for accounts with a discriminator). `view.data_mut_ptr()` must
/// be a valid value of `Self`, no invalid bit patterns. The Solana runtime handles proper
/// alignment of account data. (entrypoint)
#[cfg(feature = "bpf")]
pub unsafe trait Cast: Owner + Space + __AccountMarker {
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
        Self::check_space(view.data_len())?;
        Self::check_owner(view.owner())?;

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
        Self::check_space(view.data_len())?;
        Self::check_owner(view.owner())?;

        // SAFETY: The required invariants are checked above.
        unsafe { Self::cast_mut_unchecked(view) }
    }
}

#[cfg(feature = "bpf")]
unsafe impl<T: Discriminator + Owner + Space + __AccountMarker> Cast for T {
    #[inline(always)]
    unsafe fn cast_untracked(view: AccountView<'_>) -> &T {
        // skip discriminator bytes
        let undiscriminated = view.data_ptr().add(8);
        &*(undiscriminated as *const Self)
    }

    #[inline(always)]
    unsafe fn cast_mut_untracked(view: AccountView<'_>) -> &mut T {
        let undiscriminated = view.data_mut_ptr().add(8);
        &mut *(undiscriminated as *mut Self)
    }

    #[inline(always)]
    unsafe fn cast_unchecked(view: AccountView<'_>) -> Result<Ref<'_, T>> {
        Ok(Ref::map(view.try_borrow()?, |data| {
            let undiscriminated = data.as_ptr().add(8);
            &*(undiscriminated as *const Self)
        }))
    }

    #[inline(always)]
    unsafe fn cast_mut_unchecked(view: AccountView<'_>) -> Result<RefMut<'_, T>> {
        Ok(RefMut::map(view.try_borrow_mut()?, |data| {
            let undiscriminated = data.as_mut_ptr().add(8);
            &mut *(undiscriminated as *mut Self)
        }))
    }

    #[inline(always)]
    fn cast(view: AccountView<'_>) -> Result<Ref<'_, T>> {
        Self::check_space(view.data_len())?;
        Self::check_owner(view.owner())?;

        // SAFETY: `Self::check_space` on types implementing `Discriminator`
        // implicitly verifies that `view.data_len()  >= 8`, so `Self::check_discriminator_unchecked`
        // is a safe operation. On SBF target, the account data is guaranteed to be properly
        // aligned, required account level invariants have been checked above.
        unsafe {
            Self::check_discriminator_unchecked(view)?;

            Self::cast_unchecked(view)
        }
    }

    #[inline(always)]
    fn cast_mut(view: AccountView<'_>) -> Result<RefMut<'_, T>> {
        Self::check_space(view.data_len())?;
        Self::check_owner(view.owner())?;

        // SAFETY: `Self::check_space` on types implementing `Discriminator`
        // implicitly verifies that `view.data_len()  >= 8`, so `Self::check_discriminator_unchecked`
        // is a safe operation. On SBF target, the account data is guaranteed to be properly
        // aligned, required account level invariants have been checked above.
        unsafe {
            Self::check_discriminator_unchecked(view)?;

            Self::cast_mut_unchecked(view)
        }
    }
}

pub trait ParseAccounts<'view, Bumps>: Sized {
    const NUM_ACCOUNTS: usize;

    fn parse_accounts(
        views: &mut AccountIter<'view>,
        data: &[u8],
        bumps: &mut Bumps,
    ) -> Result<Self>;
}

/// Trait to define the validation logic for some account type's meta
pub trait AccountValidator {
    fn validate(&self) -> Result<()>;
}

/// Parse an account wrapper type from an [`AccountView`] and some metadata.
pub trait ParseAccount<'view>: Sized {
    type Meta<'a, 'b, 'c>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a;

    fn parse<'a, 'b, 'c>(view: AccountView<'view>, meta: &mut Self::Meta<'a, 'b, 'c>) -> Result<Self>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a;

    unsafe fn parse_unchecked<'a, 'b, 'c>(view: AccountView<'view>, meta: &mut Self::Meta<'a, 'b, 'c>) -> Result<Self>
    where 
        'view: 'c,
        'c: 'b,
        'b: 'a,
    {
        Self::parse(view, meta)
    }
}

impl<'view> ParseAccount<'view> for AccountView<'view> {
    type Meta<'a, 'b, 'c>
        = NoMeta
    where
        'view: 'c,
        'c: 'b,
        'b: 'a;

    #[inline(always)]
    fn parse<'a, 'b, 'c>(view: AccountView<'view>, _: &mut Self::Meta<'a, 'b, 'c>) -> Result<Self>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a,
    {
        Ok(view)
    }
}

pub trait __ParseAccountDispatch<'view, D: internal::__DiscriminatorMarker>: Sized {
    fn dispatch(view: AccountView<'view>) -> Result<Self>;
}

/// ZST representing no metadata for an account wrapper,
/// more semantic than `()`.
pub struct NoMeta;

/// Get a sysvar from an [`AccountView`].
pub trait TrySysvarFromAccountView<'view>: Sized {
    fn try_sysvar_from_account_view(view: AccountView<'view>) -> Result<Self>;
}

/// Get an inner [`AccountView`] from an account wrapper like [`Account<'_, T>`].
pub trait ToAccountView<'view> {
    /// Returns an [`AccountView`] pointer.
    fn to_account_view(&self) -> AccountView<'view>;
}

/// Denotes whether an account wrapper type can be writable or not.
///
/// # Safety
///
/// Some account types like [`Program<'_, T>`] are executable and therefore cannot
/// be writable, ever. This trait must be unsafe because manual proof of an
/// account type being capable of being writable is necessary.
pub unsafe trait WritableAllowed {}

/// Denotes whether an account wrapper type can be a PDA or not.
///
/// # Safety
///
/// Some account types can never be a PDA, i.e. [`Sysvar<T>`], [`Program<'_, T>`].
pub unsafe trait PdaAllowed {}

pub trait AccountAddress<'view> {
    fn address(&self) -> &'view Address;

    fn address_owned(&self) -> Address;
}

/// Serialize a CPI call into the expected wire format.
pub trait SerializeCpiCtx<'view, M, V, D>
where
    M: AsRef<[AccountMeta<'view>]>,
    V: AsRef<[CpiAccount<'view>]>,
    D: AsRef<[u8]>,
{
    fn serialize(&self) -> Result<CpiCtx<'view, 'view, 'static, M, V, D>>;
}

/// A type that holds sysvar data.
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

pub trait PdaSeeds: BumpOffset {
    const PREFIX: &[u8];

    type Seeds<'seeds>;

    fn check_seeds<'seeds>(&self, seeds: &Self::Seeds<'seeds>) -> Result<()>;
}

#[cfg(feature = "bpf")]
pub trait AccountInit<'view, AccountType> {
    type Meta;

    type PdaMeta<'a, 'b, 'c>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a;

    fn init_account(view: AccountView<'view>, meta: &Self::Meta) -> Result<AccountType>;

    fn init_pda_account<'a, 'b, 'c>(
        view: AccountView<'view>,
        meta: &mut Self::PdaMeta<'a, 'b, 'c>,
    ) -> Result<Pda<AccountType>>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a;
}

#[cfg(feature = "bpf")]
impl<'view, T> AccountInit<'view, Account<'view, T>> for T
where
    T: Cast + Discriminator,
{
    type Meta = __AccountInitMeta<'view>;

    type PdaMeta<'a, 'b, 'c>
        = __PdaAccountInitMeta<'view, 'a, 'b, 'c>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a;

    #[inline]
    fn init_account(view: AccountView<'view>, meta: &Self::Meta) -> Result<Account<'view, T>> {
        let _: Mut<Signer<'_>> = Mut::parse(view, &mut NoMeta)?;

        invoke!(CreateAccount {
            from: meta.payer.to_account_view(),
            to: view,
            owner_program: &T::OWNER,
            space: T::SPACE as u64,
        });

        let ptr = view.data_mut_ptr();

        // SAFETY: ptr cast is guaranteed to be sound because T::SPACE ensures proper
        // account data length. Account data is 0x8 align so it is safe to do a direct write,
        // although since we are writing a 0x1 type, alignment is irrelevant anyway.
        unsafe {
            ptr.cast::<[u8; 8]>().write(*T::DISCRIMINATOR);
        }

        Ok(Account {
            view,
            __phantom: PhantomData,
        })
    }

    #[inline]
    fn init_pda_account<'a, 'b, 'c>(
        view: AccountView<'view>,
        meta: &mut Self::PdaMeta<'a, 'b, 'c>,
    ) -> Result<Pda<Account<'view, T>>>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a,
    {
        let _: Mut<UncheckedAccount<'_>> = Mut::parse(view, &mut NoMeta)?;

        invoke_with_signers!(
            CreateAccount {
                from: meta.payer.to_account_view(),
                to: view,
                owner_program: &T::OWNER,
                space: T::SPACE as u64,
            },
            &[meta.signer],
        );

        let ptr = view.data_mut_ptr();

        // SAFETY: `T::DISCRIMINATOR` is guaranteed to be within the bounds of `T::SPACE`
        // and the account is guaranteed to be of length `T::SPACE` by the CPI above.
        unsafe {
            ptr.cast::<[u8; 8]>().write(*T::DISCRIMINATOR);
        }

        let seeds = meta.signer.as_slice_of_slices();
        let (_, bump) = try_find_program_address(seeds, &T::OWNER)?;

        // SAFETY: `meta.bump_ptr` is guaranteed to be a valid ptr by calling code.
        unsafe {
            meta.bump_ptr.write(bump);
        }

        Ok(Pda(Account {
            view,
            __phantom: PhantomData,
        }))
    }
}

#[cfg(feature = "bpf")]
pub struct __AccountInitMeta<'view> {
    pub payer: Mut<Signer<'view>>,
}

#[cfg(feature = "bpf")]
pub struct __PdaAccountInitMeta<'view, 'a, 'b, 'c>
where
    'view: 'c,
    'c: 'b,
    'b: 'a,
{
    pub payer: Mut<Signer<'view>>,
    pub signer: CpiSigner<'b, 'a>,
    pub bump_ptr: *mut u8,
    __phantom: PhantomData<&'c mut u8>,
}

#[cfg(feature = "bpf")]
impl<'view, 'a, 'b, 'c> __PdaAccountInitMeta<'view, 'a, 'b, 'c>
where
    'view: 'c,
    'c: 'b,
    'b: 'a,
{
    #[inline(always)]
    pub fn new(
        payer: Mut<Signer<'view>>,
        signer: CpiSigner<'b, 'a>,
        bump_ref: &'c mut u8,
    ) -> __PdaAccountInitMeta<'view, 'a, 'b, 'c> {
        Self {
            payer,
            signer,
            bump_ptr: bump_ref as *mut u8,
            __phantom: PhantomData::<&'c mut u8>,
        }
    }
}

pub mod internal {
    mod sealed {
        pub trait Sealed {}
    }

    #[doc(hidden)]
    pub unsafe trait __DiscriminatorMarker: sealed::Sealed {}

    #[doc(hidden)]
    pub struct __WithDiscriminator;
    #[doc(hidden)]
    pub struct __NoDiscriminator;

    impl sealed::Sealed for __WithDiscriminator {}
    impl sealed::Sealed for __NoDiscriminator {}

    unsafe impl __DiscriminatorMarker for __WithDiscriminator {}
    unsafe impl __DiscriminatorMarker for __NoDiscriminator {}

    #[doc(hidden)]
    pub trait __AccountDiscriminatorMode {
        type Mode: __DiscriminatorMarker;
    }
}
