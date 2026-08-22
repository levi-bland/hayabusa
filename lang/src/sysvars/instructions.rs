// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use core::{marker::PhantomData, slice::from_raw_parts};

use crate::{prelude::*, sysvars::SYSVAR_PROGRAM_ID};

#[derive(Clone, Copy, Debug)]
pub struct Instructions<'view> {
    view: AccountView<'view>,
}

impl Instructions<'_> {
    /// The address of the [`Instructions`] sysvar.
    pub const ID: Address = address!("Sysvar1nstructions1111111111111111111111111");
}

impl<'view> TrySysvarFromAccountView<'view> for Instructions<'view> {
    #[inline(always)]
    fn try_sysvar_from_account_view(view: AccountView<'view>) -> Result<Self> {
        if hint::unlikely(!address_eq(view.address(), &Instructions::ID)) {
            err!(
                "Instructions::try_sysvar_from_account_view: invalid account",
                ErrorCode::InvalidAccount,
            );
        }

        Ok(Self { view })
    }
}

impl<'view> ToAccountView<'view> for Instructions<'view> {
    #[inline(always)]
    fn to_account_view(&self) -> AccountView<'view> {
        self.view
    }
}

impl<'view> AccountAddress<'view> for Instructions<'view> {
    #[inline(always)]
    fn address(&self) -> &'view Address {
        &Self::ID
    }

    #[inline(always)]
    fn address_owned(&self) -> Address {
        Self::ID
    }
}

impl<'view> Instructions<'view> {
    /// Load the number of instructions in the currently executing transaction.
    #[inline(always)]
    pub fn num_instructions(&self) -> u16 {
        // SAFETY: The first 2 bytes of the Instructions sysvar data represents the
        // number of instructions. The `AccountView` stored is guaranteed to be the
        // sysvar account by construction.
        unsafe { *(self.view.data_ptr() as *const u16) }
    }

    /// Load the index of the currently executing instruction in the transaction.
    #[inline(always)]
    pub fn load_current_index(&self) -> u16 {
        let len = self.view.data_len();
        // SAFETY: The last 2 bytes of the Instructions sysvar data represents the current
        // instruction index. The `AccountView` stored is guaranteed to be the
        // sysvar account by construction.
        unsafe { *(self.view.data_ptr().add(len - 2) as *const u16) }
    }

    /// Creates and returns an `IntrospectedInstruction` for the instruction at the specified index.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check if the provided index is out of bounds. It is
    /// typically used internally with the `load_instruction_at` or `load_instruction_relative` functions,
    /// which perform the necessary index verification.
    #[inline(always)]
    pub unsafe fn deserialize_instruction_unchecked(
        &self,
        index: usize,
    ) -> IntrospectedInstruction<'view> {
        let offset = *(self
            .view
            .data_ptr()
            .add(size_of::<u16>() + index * size_of::<u16>()) as *const u16);

        IntrospectedInstruction {
            raw: self.view.data_ptr().add(offset as usize),
            __phantom: PhantomData,
        }
    }

    /// Creates and returns an `IntrospectedInstruction` for the instruction at the specified index.
    #[inline(always)]
    pub fn load_instruction_at(&self, index: usize) -> Result<IntrospectedInstruction<'view>> {
        if index >= self.num_instructions() as usize {
            return Err(ErrorCode::InstructionsSysvarIxIndexTooLarge.into());
        }

        // SAFETY: the index has been checked to be in bounds.
        Ok(unsafe { self.deserialize_instruction_unchecked(index) })
    }

    /// Creates and returns an `InstrospectedInstruction` relative to the current instruction in
    /// the currently executing transaction.
    #[inline(always)]
    pub fn get_instruction_relative(
        &self,
        index_relative_to_current: i64,
    ) -> Result<IntrospectedInstruction<'view>> {
        let current_index = self.load_current_index() as i64;
        let index = current_index.saturating_add(index_relative_to_current);

        if index < 0 {
            return Err(ErrorCode::InstructionsSysvarIxIndexLessThanZero.into());
        }

        self.load_instruction_at(index as usize)
    }
}

impl Owner for Instructions<'_> {
    const OWNER: Address = SYSVAR_PROGRAM_ID;
}

#[repr(C)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntrospectedInstruction<'view> {
    raw: *const u8,
    __phantom: PhantomData<&'view [u8]>,
}

impl<'view> IntrospectedInstruction<'view> {
    /// Get the account meta at the specified index.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not verify if the index is out of bounds.
    ///
    /// It is typically used internally within the `get_account_meta_at` function, which
    /// performs the necessary index verification. However, to optimize performance for users
    /// who are sure that the index is in bounds, we have exposed it as an unsafe function.
    #[inline(always)]
    pub unsafe fn get_account_meta_at_unchecked(
        &self,
        index: usize,
    ) -> IntrospectedAccountMeta<'view> {
        let offset = size_of::<u16>() + (index * IntrospectedAccountMeta::LEN);
        *(self.raw.add(offset) as *const IntrospectedAccountMeta<'view>)
    }

    /// Get the account meta at the specified index.
    ///
    /// # Errors
    ///
    /// Returns [`ProgramError::InvalidArgument`] if the index is out of bounds.
    #[inline(always)]
    pub fn get_account_meta_at(&self, index: usize) -> Result<AccountMeta<'view>> {
        // SAFETY: The first 2 bytes represent the number of accounts in the instruction.
        let num_accounts = unsafe { *(self.raw as *const u16) };

        if index >= num_accounts as usize {
            return Err(ProgramError::InvalidArgument);
        }

        // SAFETY: The index was checked to be in bounds.
        Ok(unsafe { self.get_account_meta_at_unchecked(index).to_account_meta() })
    }

    /// Get the program ID of the `Instruction`.
    #[inline(always)]
    pub fn get_program_id(&self) -> &'view Address {
        // SAFETY: The first 2 bytes represent the number of accounts in the instruction.
        let num_accounts = unsafe { *(self.raw as *const u16) };

        // SAFETY: The program ID is located after the account metas.
        unsafe {
            &*(self.raw.add(
                size_of::<u16>() + num_accounts as usize * size_of::<IntrospectedAccountMeta>(),
            ) as *const Address)
        }
    }

    /// Get the instruction data of the `Instruction`.
    #[inline(always)]
    pub fn get_instruction_data(&self) -> &'view [u8] {
        // SAFETY: The first 2 bytes represent the number of accounts in the instruction.
        let offset = unsafe { *(self.raw as *const u16) } as usize
            * size_of::<IntrospectedAccountMeta>()
            + size_of::<Address>();

        // SAFETY: The instruction data length is located after the program ID.
        let data_len = unsafe { *(self.raw.add(size_of::<u16>() + offset) as *const u16) };

        // SAFETY: The instruction data is located after the data length.
        unsafe {
            from_raw_parts(
                self.raw.add(size_of::<u16>() + offset + size_of::<u16>()),
                data_len as usize,
            )
        }
    }
}

/// The bit positions for the signer flags in the `AccountMeta`.
const IS_SIGNER: u8 = 0b00000001;

/// The bit positions for the writable flags in the `AccountMeta`.
const IS_WRITABLE: u8 = 0b00000010;

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IntrospectedAccountMeta<'view> {
    /// Account flags:
    ///   * bit `0`: signer
    ///   * bit `1`: writable
    flags: u8,

    /// The account address.
    addr: *const Address,

    /// Tie addr to the `'view` lifetime.
    __phantom: PhantomData<&'view Address>,
}

impl<'view> IntrospectedAccountMeta<'view> {
    const LEN: usize = core::mem::size_of::<Self>();

    /// Indicate whether the account is writable or not.
    #[inline(always)]
    pub fn is_writable(&self) -> bool {
        (self.flags & IS_WRITABLE) != 0
    }

    /// Indicate whether the account is a signer or not.
    #[inline(always)]
    pub fn is_signer(&self) -> bool {
        (self.flags & IS_SIGNER) != 0
    }

    /// Convert the `IntrospectedAccountMeta` to an `AccountMeta`.
    #[inline(always)]
    pub fn to_account_meta(&self) -> AccountMeta<'view> {
        // SAFETY: `self.addr` is guaranteed to be a valid raw ptr into the `Instructions` sysvar
        // account view.
        AccountMeta::new(unsafe { &*self.addr }, self.is_writable(), self.is_signer())
    }
}
