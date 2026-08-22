// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

use solana_program_error::ProgramError;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ErrorCode {
    UnknownInstruction = 100,
    BufferFull,
    InvalidAccountDiscriminator,
    AccountNotSigner,
    InvalidAccount,
    AccountNotWritable,
    InvalidProgram,
    InvalidSeeds,
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
}

impl From<ErrorCode> for ProgramError {
    #[inline(always)]
    fn from(e: ErrorCode) -> ProgramError {
        ProgramError::Custom(e as u32)
    }
}
