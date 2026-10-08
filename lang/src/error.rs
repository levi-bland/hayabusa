// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use solana_program_error::ProgramError;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ErrorCode {
    UnknownInstruction = 100,
    BufferFull,
    InvalidAccountDiscriminator,
    AccountNotSigner,
    AccountNotExecutable,
    InvalidAccount,
    AccountNotWritable,
    AccountNotImmutable,
    InvalidProgram,
    InvalidInterfaceProgram,
    InvalidSeeds,
    MetaAlreadyConsumed,
    SyscallFailed,
    SeedsTooLong,
    TooManySeeds,
    InvalidIndex,
    ProgramAccountNotExecutable,
    InvalidAddress,
    ProgramAddressDoesNotExist,
    InvalidAccountOwner,
    InvalidAccountLength,
    MissingReturnData,
    DataReturnedFromIncorrectProgram,
    CpiExceedsMaxAccounts,
    InvalidSysvarAccount,
    InstructionsSysvarIxIndexTooLarge,
    InstructionsSysvarIxIndexLessThanZero,
    StackVecMaxLengthExceeded,
    StackVecIndexOutOfBounds,
    InvalidProgramAccount,
    UnexpectedEndOfBuffer,
    InvalidUtf8,
    Token2022ExtensionBadAccountType,
    Token2022ExtensionLayoutMismatch,
    Token2022ExtensionDirtyPadding,
    Token2022ExtensionUninitializedAccount,
}

impl From<ErrorCode> for ProgramError {
    #[inline(always)]
    fn from(e: ErrorCode) -> ProgramError {
        ProgramError::Custom(e as u32)
    }
}
