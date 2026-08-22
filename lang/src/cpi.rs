// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use crate::{account_view::RuntimeAccount, prelude::*};
use core::{
    marker::PhantomData,
    mem::{offset_of, MaybeUninit},
    ops::Deref,
    ptr::{addr_of, addr_of_mut, copy_nonoverlapping},
    slice::from_raw_parts,
};
#[cfg(any(target_os = "solana", target_arch = "bpf"))]
use solana_define_syscall::{
    define_syscall,
    definitions::{sol_get_return_data, sol_invoke_signed_c, sol_set_return_data},
};

/// Invoke a CPI without any program signers.
///
/// ## Example
///
/// ```ignore
/// invoke!(CreateAccount {
///     from: ...,
///     to: ...,
///     owner_program: ...,
///     space: ...,
/// });
/// ```
///
#[macro_export]
macro_rules! invoke {
    ($cpi_struct:ident { $($fields:tt)* }) => {
        $cpi_struct { $($fields)* }.serialize()?.invoke()?
    };
    ($cpi_struct:expr) => {
        $cpi_struct.serialize()?.invoke()?
    };
}

/// Invoke a CPI with a single program signer.
///
/// ## Example
///
/// ```ignore
/// let seeds = seeds!(...);
/// invoke_signed!(
///     CreateAccount {
///         from: ...,
///         to: ...,
///         owner_program: ...,
///         space: ...,
///     },
///     seeds,
/// );
/// ```
///
#[macro_export]
macro_rules! invoke_signed {
    ($cpi_struct:ident { $($fields:tt)* }, $seeds:expr $(,)?) => {
        $cpi_struct { $($fields)* }.serialize()?.invoke_signed($seeds)?
    };
    ($cpi_struct:expr, $seeds:expr $(,)?) => {
        $cpi_struct.serialize()?.invoke_signed($seeds)?
    };
}

/// Invoke a CPI with a slice of program signers.
///
/// ## Example
///
/// ```ignore
/// let seeds = seeds!(...);
/// let signer = Signer::from(seeds);
///
/// invoke_with_signers!(
///     CreateAccount {
///         from: ...,
///         to: ...,
///         owner_program: ...,
///         space: ...,
///     },
///     &[signer],
/// );
/// ```
#[macro_export]
macro_rules! invoke_with_signers {
    ($cpi_struct:ident { $($fields:tt)* }, $signers:expr $(,)?) => {
        $cpi_struct { $($fields)* }.serialize()?.invoke_with_signers($signers)?
    };
    ($cpi_struct:expr, $signers:expr $(,)?) => {
        $cpi_struct.serialize()?.invoke_with_signers($signers)?
    };
}

#[macro_export]
macro_rules! invoke_with_return_data {
    ($cpi_struct:ident { $($fields:tt)* }) => {
        $cpi_struct { $($fields)* }.serialize()?.invoke_with_return_data()?
    };
    ($cpi_struct:expr) => {
        $cpi_struct.serialize()?.invoke_with_return_data()?
    };
}

#[macro_export]
macro_rules! invoke_signed_with_return_data {
    ($cpi_struct:ident { $($fields:tt)* }, $seeds:expr) => {
        $cpi_struct { $($fields)* }
            .serialize()?
            .invoke_signed_with_return_data($seeds)?
    };
    ($cpi_struct:expr, $seeds:expr $(,)?) => {
        $cpi_struct
            .serialize()?
            .invoke_signed_with_return_data($seeds)?
    };
}

#[macro_export]
macro_rules! invoke_with_signers_with_return_data {
    ($cpi_struct:ident { $($fields:tt)* }, $signers:expr) => {
        $cpi_struct { $($fields)* }
            .serialize()?
            .invoke_with_signers_with_return_data($signers)?
    };
    ($cpi_struct:expr, $signers:expr $(,)?) => {
        $cpi_struct
            .serialize()?
            .invoke_with_signers_with_return_data($signers)?
    };
}

// due to cfgs this is needed to silence warnings
#[allow(dead_code)]
/// Holds metadata required to make CPI calls.
pub struct CpiCtx<'view, 'meta, 'id, M, V, D>
where
    M: AsRef<[AccountMeta<'meta>]>,
    V: AsRef<[CpiAccount<'view>]>,
    D: AsRef<[u8]>,
{
    /// The program being invoked.
    program_id: *const Address,
    /// Account metas
    metas: M,
    /// Account views (CpiAccount)
    views: V,
    /// Serialized instruction data
    data: D,
    /// PhantomData to tie variance
    __phantom: PhantomData<(
        &'id Address,
        &'meta [AccountMeta<'meta>],
        &'view [AccountView<'view>],
    )>,
}

#[allow(dead_code)]
impl<'view, 'meta, 'id, M, V, D> CpiCtx<'view, 'meta, 'id, M, V, D>
where
    M: AsRef<[AccountMeta<'meta>]>,
    V: AsRef<[CpiAccount<'view>]>,
    D: AsRef<[u8]>,
{
    /// Creates a new [`CpiCtx`]
    #[inline(always)]
    pub fn new(program_id: &'id Address, metas: M, views: V, data: D) -> Self {
        Self {
            program_id: program_id as *const Address,
            metas,
            views,
            data,
            __phantom: PhantomData,
        }
    }

    #[inline(always)]
    #[must_use = "CPI Result must be handled with `?`"]
    pub fn invoke(&self) -> Result<()> {
        self.invoke_with_signers_inner(&[])
    }

    #[inline(always)]
    #[must_use = "CPI Result must be handled with `?`"]
    pub fn invoke_signed(&self, seeds: &[Seed]) -> Result<()> {
        self.invoke_with_signers_inner(&[Signer::from(seeds)])
    }

    #[inline(always)]
    #[must_use = "CPI Result must be handled with `?`"]
    pub fn invoke_with_signers(&self, signers: &[Signer]) -> Result<()> {
        self.invoke_with_signers_inner(signers)
    }

    #[inline(always)]
    #[must_use = "CPI Result must be handled with `?`"]
    pub fn invoke_with_return_data(&self) -> Result<ReturnData> {
        self.invoke_with_signers_with_return_data_inner(&[])
    }

    #[inline(always)]
    #[must_use = "CPI Result must be handled with `?`"]
    pub fn invoke_signed_with_return_data(&self, seeds: &[Seed]) -> Result<ReturnData> {
        self.invoke_with_signers_with_return_data_inner(&[Signer::from(seeds)])
    }

    #[inline(always)]
    #[must_use = "CPI Result must be handled with `?`"]
    pub fn invoke_with_signers_with_return_data(&self, signers: &[Signer]) -> Result<ReturnData> {
        self.invoke_with_signers_with_return_data_inner(signers)
    }

    #[inline(always)]
    fn invoke_with_signers_with_return_data_inner(&self, signers: &[Signer]) -> Result<ReturnData> {
        set_return_data(&[]);

        self.invoke_with_signers_inner(signers)?;

        let ret = get_return_data()?;

        // SAFETY: `self.program_id` is guarnateed to be a valid pointer derived from an `AccountView`
        if hint::unlikely(!address_eq(ret.program_id(), unsafe { &*self.program_id })) {
            err!(
                "CpiCtx::invoke_with_signers_with_return_data_inner: data returned from wrong program",
                ErrorCode::DataReturnedFromIncorrectProgram,
            );
        }

        Ok(ret)
    }

    #[allow(unused_variables)]
    #[inline(always)]
    fn invoke_with_signers_inner(&self, signers: &[Signer]) -> Result<()> {
        #[cfg(any(target_os = "solana", target_arch = "bpf"))]
        {
            let num_accounts = self.metas.as_ref().len() as u64;

            let instruction = CInstruction {
                program_id: self.program_id,
                accounts: self.metas.as_ref().as_ptr(),
                accounts_len: num_accounts,
                data: self.data.as_ref().as_ptr(),
                data_len: self.data.as_ref().len() as u64,
            };

            // SAFETY: `CInstruction` is `#[repr(C)]` and layout compatible with
            // the C struct expected by `sol_invoke_signed_c`. all pointers are valid.
            let ret = unsafe {
                sol_invoke_signed_c(
                    &instruction as *const _ as *const u8,
                    self.views.as_ref().as_ptr() as *const u8,
                    num_accounts,
                    signers as *const _ as *const u8,
                    signers.len() as u64,
                )
            };

            Self::result_from_raw(ret)
        }
        #[cfg(not(any(target_os = "solana", target_arch = "bpf")))]
        {
            Err(ProgramError::InvalidArgument)
        }
    }

    #[inline(always)]
    fn result_from_raw(result: u64) -> Result<()> {
        if result == 0 {
            Ok(())
        } else {
            #[cold]
            fn error(result: u64) -> ProgramError {
                ProgramError::from(result)
            }
            Err(error(result))
        }
    }
}

/// An `Instruction` as expected by `sol_invoke_signed_c`.
///
/// DO NOT EXPOSE THIS STRUCT:
///
/// To ensure pointers are valid upon use, the scope of this struct should
/// only be limited to the stack where `sol_invoke_signed_c` happens and then
/// discarded immediately after.
#[allow(dead_code)]
#[repr(C)]
struct CInstruction<'view> {
    /// Public key of the program.
    program_id: *const Address,

    /// Accounts expected by the program instruction.
    accounts: *const AccountMeta<'view>,

    /// Number of accounts expected by the program instruction.
    accounts_len: u64,

    /// Data expected by the program instruction.
    data: *const u8,

    /// Length of the data expected by the program instruction.
    data_len: u64,
}

/// Maximum number of accounts allowed in `invoke` and `invoke_with_bounds`
/// functions.
pub const MAX_STATIC_CPI_ACCOUNTS: usize = 64;

/// Maximum number of accounts allowed in a cross-program invocation.
//
// Note: This value will increase to 255 when SIMD-0339 is activated.
pub const MAX_CPI_ACCOUNTS: usize = 128;

/// An account for CPI invocations.
///
/// This struct contains the same information as an [`AccountView`], but has
/// the memory layout as expected by `sol_invoke_signed_c` syscall.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CpiAccount<'view> {
    /// Address of the account.
    address: *const Address,

    /// Number of lamports owned by this account.
    lamports: *const u64,

    /// Length of data in bytes.
    data_len: u64,

    /// On-chain data within this account.
    data: *const u8,

    /// Program that owns this account.
    owner: *const Address,

    /// The epoch at which this account will next owe rent.
    rent_epoch: u64,

    /// Transaction was signed by this account's key?
    is_signer: u8,

    /// Is the account writable?
    is_writable: u8,

    /// This account's data contains a loaded program (and is now read-only).
    executable: u8,

    /// Padding so the last 4 bytes can be seen as a `u32`.
    _padding: u8,

    /// The pointers to the `AccountView` data are only valid for as long as the
    /// underlying raw ptr lives, lifetime denoted by 'view. Instead of holding a
    /// reference to the actual `AccountView`, which would increase the size of
    /// the type, we claim to hold a reference without actually holding one using a
    /// `PhantomData<AccountView<'view>>`.
    _account_view: PhantomData<AccountView<'view>>,
}

// Make sure the layout of `CpiAccount` and `RuntimeAccount` are compatible for the fields
// that are copied over as a single value in `CpiAccount::init_from_account_view`.
#[allow(clippy::arithmetic_side_effects)]
const _: () = {
    const RUNTIME_SIGNER_OFFSET: usize = offset_of!(RuntimeAccount, is_signer);
    const CPI_SIGNER_OFFSET: usize = offset_of!(CpiAccount<'static>, is_signer);

    assert!(
        offset_of!(RuntimeAccount, is_writable) - RUNTIME_SIGNER_OFFSET
            == offset_of!(CpiAccount<'static>, is_writable) - CPI_SIGNER_OFFSET
    );

    assert!(
        offset_of!(RuntimeAccount, executable) - RUNTIME_SIGNER_OFFSET
            == offset_of!(CpiAccount<'static>, executable) - CPI_SIGNER_OFFSET
    );

    assert!(
        offset_of!(RuntimeAccount, padding) - RUNTIME_SIGNER_OFFSET
            == offset_of!(CpiAccount<'static>, _padding) - CPI_SIGNER_OFFSET
    );
};

/// Initializes a fixed-size `MaybeUninit` buffer by writing each account sequentially.
///
/// Usage:
/// ```
/// let views = init_cpi_accounts!(self.account, self.mint, self.token_program);
/// ```
#[macro_export]
macro_rules! init_cpi_accounts {
    ($($account:expr),+ $(,)?) => {{
        // Count the number of accounts at compile time.
        const N: usize = { let mut n = 0usize; $( let _ = stringify!($account); n += 1; )* n };

        // SAFETY: every element is written via `ptr.add(i).write(...)` before
        // `assume_init` is called, so the buffer is fully initialised.
        unsafe {
            let mut buf = ::core::mem::MaybeUninit::<[CpiAccount; N]>::uninit();
            let ptr = buf.as_mut_ptr() as *mut CpiAccount;
            let mut _i = 0usize;
            $(
                ptr.add(_i).write(CpiAccount::from($account));
                _i += 1;
            )*
            buf.assume_init()
        }
    }};
}

impl<'view> From<AccountView<'view>> for CpiAccount<'view> {
    #[inline(always)]
    fn from(view: AccountView<'view>) -> Self {
        let mut uninit = MaybeUninit::<Self>::uninit();
        Self::init_from_account_view(view, &mut uninit);
        // SAFETY: `init_from_account_view` initializes all fields of the struct
        unsafe { uninit.assume_init() }
    }
}

impl<'view> CpiAccount<'view> {
    /// Initialize a `CpiAccount` struct with information from an `AccountView`.
    ///
    /// After this function is called, the `uninit` parameter will be initialized
    /// with the information from the `view`.
    #[inline(always)]
    pub fn init_from_account_view(view: AccountView<'view>, uninit: &mut MaybeUninit<Self>) {
        let uninit_ptr = uninit.as_mut_ptr();
        let view_ptr = view.account_ptr();

        // SAFETY: `uninit_ptr` is valid for writes and `view_ptr` is valid for reads
        // since they have been obtained from references.
        unsafe {
            (*uninit_ptr).address = addr_of!((*view_ptr).address);
            (*uninit_ptr).lamports = addr_of!((*view_ptr).lamports);
            (*uninit_ptr).data_len = (*view_ptr).data_len;
            (*uninit_ptr).data = view_ptr.add(1) as *const u8;
            (*uninit_ptr).owner = addr_of!((*view_ptr).owner);
            // The `rent_epoch` field is not present in the `AccountView` struct,
            // since the value occurs after the variable data of the account in
            // the runtime input data.
            (*uninit_ptr).rent_epoch = 0;
            // The `is_signer`, `is_writable`, and `executable` fields are contiguous in memory,
            // so we can write them with a single operation. We copy an extra byte on purpose so
            // it translates to 32-bit load/store operations.
            let src = addr_of!((*view_ptr).is_signer);
            let dst = addr_of_mut!((*uninit_ptr).is_signer);
            copy_nonoverlapping(src, dst, size_of::<u32>());
        }
    }
}

/// Represents a signer seed.
///
/// This struct contains the same information as a `[u8]`, but
/// has the memory layout as expected by `sol_invoke_signed_c`
/// syscall.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Seed<'bytes> {
    /// Seed bytes.
    pub(crate) seed: *const u8,

    /// Length of the seed bytes.
    pub(crate) len: u64,

    /// The pointer to the seed bytes is only valid while the `&'bytes [u8]` lives. Instead
    /// of holding a reference to the actual `[u8]`, which would increase the size of the
    /// type, we claim to hold a reference without actually holding one using a
    /// `PhantomData<&'bytes [u8]>`.
    _bytes: PhantomData<&'bytes [u8]>,
}

impl<'bytes> From<&'bytes [u8]> for Seed<'bytes> {
    #[inline(always)]
    fn from(value: &'bytes [u8]) -> Self {
        Self {
            seed: value.as_ptr(),
            len: value.len() as u64,
            _bytes: PhantomData::<&[u8]>,
        }
    }
}

impl<'bytes, const SIZE: usize> From<&'bytes [u8; SIZE]> for Seed<'bytes> {
    #[inline(always)]
    fn from(value: &'bytes [u8; SIZE]) -> Self {
        Self {
            seed: value.as_ptr(),
            len: value.len() as u64,
            _bytes: PhantomData::<&[u8]>,
        }
    }
}

impl<'bytes> Deref for Seed<'bytes> {
    type Target = [u8];

    #[inline(always)]
    fn deref(&self) -> &'bytes Self::Target {
        unsafe { from_raw_parts(self.seed, self.len as usize) }
    }
}

/// Represents a [program derived address][pda] (PDA) signer controlled by the
/// calling program.
///
/// [pda]: https://solana.com/docs/core/cpi#program-derived-addresses
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Signer<'bytes, 'seeds> {
    /// Signer seeds.
    pub(crate) seeds: *const Seed<'bytes>,

    /// Number of seeds.
    pub(crate) len: u64,

    /// The pointer to the seeds is only valid while the `&'seeds [Seed<'bytes>]` lives. Instead
    /// of holding a reference to the actual `[Seed<'bytes>]`, which would increase the size
    /// of the type, we claim to hold a reference without actually holding one using a
    /// `PhantomData<&'seeds [Seed<'bytes>]>`.
    _seeds: PhantomData<&'seeds [Seed<'bytes>]>,
}

impl<'bytes, 'seeds> From<&'seeds [Seed<'bytes>]> for Signer<'bytes, 'seeds> {
    fn from(value: &'seeds [Seed<'bytes>]) -> Self {
        Self {
            seeds: value.as_ptr(),
            len: value.len() as u64,
            _seeds: PhantomData::<&'seeds [Seed<'bytes>]>,
        }
    }
}

impl<'bytes, 'seeds, const SIZE: usize> From<&'seeds [Seed<'bytes>; SIZE]>
    for Signer<'bytes, 'seeds>
{
    fn from(value: &'seeds [Seed<'bytes>; SIZE]) -> Self {
        Self {
            seeds: value.as_ptr(),
            len: value.len() as u64,
            _seeds: PhantomData::<&'seeds [Seed<'bytes>]>,
        }
    }
}

impl<'bytes, 'seeds> Signer<'bytes, 'seeds> {
    #[inline(always)]
    pub fn as_slice_of_slices(&self) -> &'seeds [&'bytes [u8]] {
        unsafe { from_raw_parts(self.seeds as *const &'bytes [u8], self.len as usize) }
    }
}

/// A growable view over a pre-allocated `[Seed; N]` buffer (typically built
/// with `seeds!(...; reserve N)`), allowing seeds to be appended up to the
/// buffer's capacity before being converted into a `Signer` for CPI.
pub struct GrowableSigner<'bytes, 'seeds> {
    seeds: &'seeds mut [Seed<'bytes>],
    len: usize,
}

impl<'bytes, 'seeds> GrowableSigner<'bytes, 'seeds> {
    /// `buf` is the full reserved buffer (e.g. from `seeds!(...; reserve N)`);
    /// `initial_len` is how many of its leading entries are already populated
    /// with real seeds.
    #[inline(always)]
    pub fn new(buf: &'seeds mut [Seed<'bytes>], initial_len: usize) -> Self {
        assert!(initial_len <= buf.len(), "CpiSigner: initial_len exceeds capacity");
        Self { seeds: buf, len: initial_len }
    }

    #[inline(always)]
    pub fn push(&mut self, seed: Seed<'bytes>) {
        assert!(self.len < self.seeds.len(), "CpiSigner: capacity exceeded");
        self.seeds[self.len] = seed;
        self.len += 1;
    }

    #[inline(always)]
    pub fn as_seeds(&self) -> &[Seed<'bytes>] {
        &self.seeds[..self.len]
    }

    #[inline(always)]
    pub fn as_slice_of_slices(&self) -> &[&'bytes [u8]] {
        unsafe {
            core::slice::from_raw_parts(self.seeds.as_ptr() as *const &'bytes [u8], self.len)
        }
    }

    #[inline(always)]
    pub fn as_signer(&self) -> Signer<'bytes, '_> {
        Signer::from(self.as_seeds())
    }
}


/// Convenience macro for constructing a `[Seed; N]` array from a list of seeds
/// to create a [`Signer`].
///
/// # Example
///
/// Creating seeds array and signer for a PDA with a single seed and bump value:
/// ```
/// use solana_address::Address;
/// use solana_instruction_view::{cpi::Signer, seeds};
///
/// let pda_bump = 0xffu8;
/// let pda_ref = &[pda_bump];
/// let example_key = Address::default();
/// let seeds = seeds!(b"seed", example_key.as_ref(), pda_ref);
/// let signer = Signer::from(&seeds);
/// ```
#[macro_export]
macro_rules! seeds {
    ( $($seed:expr),* ) => {
        [$(
            $crate::cpi::Seed::from($seed),
        )*]
    };
    ( $($seed:expr),* ; reserve $n:expr $(,)? ) => {{
        // Build the fixed seeds first, then pad the rest with zero-length
        // placeholder seeds (`&[]`), which are valid, harmless `Seed`s that
        // get overwritten or never read until `push`/`set_len` makes them live.
        const N: usize = $n;
        let mut __seeds: [$crate::cpi::Seed; N] = core::array::from_fn(|_| $crate::cpi::Seed::from(&[][..]));
        let mut __i = 0usize;
        $(
            __seeds[__i] = $crate::cpi::Seed::from($seed);
            __i += 1;
        )*
        __seeds
    }};
}

/// Maximum size that can be set using [`set_return_data`].
pub const MAX_RETURN_DATA: usize = 1024;

/// Set the running program's return data.
///
/// Return data is a dedicated per-transaction buffer for data passed
/// from cross-program invoked programs back to their caller.
///
/// The maximum size of return data is [`MAX_RETURN_DATA`]. Return data is
/// retrieved by the caller with [`get_return_data`].
#[inline(always)]
pub fn set_return_data(data: &[u8]) {
    #[cfg(any(target_os = "solana", target_arch = "bpf"))]
    unsafe {
        sol_set_return_data(data.as_ptr(), data.len() as u64)
    };

    #[cfg(not(any(target_os = "solana", target_arch = "bpf")))]
    core::hint::black_box(data);
}

/// Get the return data from an invoked program.
///
/// For every transaction there is a single buffer with maximum length
/// [`MAX_RETURN_DATA`], paired with an [`Address`] representing the program ID of
/// the program that most recently set the return data. Thus the return data is
/// a global resource and care must be taken to ensure that it represents what
/// is expected: called programs are free to set or not set the return data; and
/// the return data may represent values set by programs multiple calls down the
/// call stack, depending on the circumstances of transaction execution.
///
/// Return data is set by the callee with [`set_return_data`].
///
/// Return data is cleared before every CPI invocation - a program that
/// has invoked no other programs can expect the return data to be `Err`; if no
/// return data was set by the previous CPI invocation, then this function
/// returns `Err`.
///
/// Return data is not cleared after returning from CPI invocations. A
/// program that has called another program may retrieve return data that was
/// not set by the called program, but instead set by a program further down the
/// call stack; or, if a program calls itself recursively, it is possible that
/// the return data was not set by the immediate call to that program, but by a
/// subsequent recursive call to that program. Likewise, an external RPC caller
/// may see return data that was not set by the program it is directly calling,
/// but by a program that program called.
///
/// For more about return data see the [documentation for the return data proposal][rdp].
///
/// [rdp]: https://docs.solanalabs.com/proposals/return-data
#[inline]
pub fn get_return_data() -> Result<ReturnData> {
    #[cfg(any(target_os = "solana", target_arch = "bpf"))]
    {
        let mut data = MaybeUninit::<[u8; MAX_RETURN_DATA]>::uninit();
        let mut program_id = MaybeUninit::<Address>::uninit();

        let size = unsafe {
            sol_get_return_data(
                data.as_mut_ptr() as *mut u8,
                MAX_RETURN_DATA as u64,
                program_id.as_mut_ptr() as *mut _ as *mut u8,
            )
        };

        if size == 0 {
            err!(
                "get_return_data: missing return data",
                ErrorCode::MissingReturnData,
            );
        } else {
            Ok(ReturnData {
                program_id: unsafe { program_id.assume_init() },
                data,
                size: core::cmp::min(size as usize, MAX_RETURN_DATA),
            })
        }
    }

    #[cfg(not(any(target_os = "solana", target_arch = "bpf")))]
    core::hint::black_box(Err(ErrorCode::MissingReturnData.into()))
}

/// Struct to hold the return data from an invoked program.
pub struct ReturnData {
    /// Program that most recently set the return data.
    program_id: Address,

    /// Return data set by the program.
    data: MaybeUninit<[u8; MAX_RETURN_DATA]>,

    /// Length of the return data.
    size: usize,
}

impl ReturnData {
    /// Returns the program that most recently set the return data.
    pub fn program_id(&self) -> &Address {
        &self.program_id
    }

    /// Return the data set by the program.
    pub fn as_slice(&self) -> &[u8] {
        unsafe { from_raw_parts(self.data.as_ptr() as _, self.size) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account_view::{RuntimeAccount, NOT_BORROWED};

    #[test]
    #[allow(clippy::used_underscore_binding)]
    fn test_write_from_account_view() {
        // 8-byte aligned `RuntimeAccount` header plus 8 bytes of account data.
        let mut raw = [0u64; size_of::<RuntimeAccount>() / size_of::<u64>() + 1];
        let account = raw.as_mut_ptr() as *mut RuntimeAccount;

        // SAFETY: `account` is a pointer to an array of 96 bytes.
        unsafe {
            (*account).borrow_state = NOT_BORROWED;
            (*account).is_signer = 1;
            (*account).is_writable = 0;
            (*account).executable = 0;
            (*account).padding = [2; 4];
            (*account).address = Address::from([1u8; 32]);
            (*account).owner = Address::from([2u8; 32]);
            (*account).lamports = 42;
            (*account).data_len = 8;
            // Add some data to the account.
            let data = (account as *mut u8).add(size_of::<RuntimeAccount>());
            data.copy_from_nonoverlapping([9u8; 8].as_ptr(), 8);
        }

        // SAFETY: `account` was initialized as a `RuntimeAccount`.
        let account_view = unsafe { AccountView::new_unchecked(account) };
        let mut cpi_account = MaybeUninit::<CpiAccount>::uninit();

        CpiAccount::init_from_account_view(account_view, &mut cpi_account);
        // SAFETY: `cpi_account` was initialized by `CpiAccount::init_from_account_view`.
        let cpi_account = unsafe { cpi_account.assume_init() };

        assert_eq!(cpi_account.address, unsafe { addr_of!((*account).address) });
        assert_eq!(cpi_account.lamports, unsafe {
            addr_of!((*account).lamports)
        });
        assert_eq!(cpi_account.data_len, 8);
        assert_eq!(cpi_account.data, account_view.data_ptr());
        assert_eq!(cpi_account.owner, unsafe { addr_of!((*account).owner) });
        assert_eq!(cpi_account.rent_epoch, 0);
        assert_eq!(cpi_account.is_signer, 1);
        assert_eq!(cpi_account.is_writable, 0);
        assert_eq!(cpi_account.executable, 0);
        // The padding field should have the first byte from the `RuntimeAccount` padding.
        assert_eq!(cpi_account._padding, 2);
    }
}
