// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use core::{marker::PhantomData, mem::MaybeUninit, ptr::copy_nonoverlapping};

use crate::TokenProgram;
use hayabusa_common::{
    Result,
    account_meta::AccountMeta,
    account_view::AccountView,
    cpi::{CpiAccount, CpiCtx},
    init_cpi_accounts,
    traits::CpiSerialize,
};
use hayabusa_invoke_cpi_macros::CpiInstruction;

/// Approves a delegate. A delegate is given the authority over tokens
/// on behalf of the source account's owner.
#[derive(CpiInstruction)]
pub struct Approve<'view, P: TokenProgram> {
    /// The source account.
    pub source: AccountView<'view>,
    /// The delegate.
    pub delegate: AccountView<'view>,
    /// The source account's owner.
    pub owner: AccountView<'view>,
    /// The amount of tokens the delegate is approved for.
    pub amount: u64,
    /// Signifies which token program to use for the CPI.
    __phantom: PhantomData<P>,
}

impl<'view, P: TokenProgram> Approve<'view, P> {
    pub fn new(
        source: AccountView<'view>,
        delegate: AccountView<'view>,
        owner: AccountView<'view>,
        amount: u64,
    ) -> Self {
        Self {
            source,
            delegate,
            owner,
            amount,
            __phantom: PhantomData,
        }
    }
}
/// The number of accounts required by the [`Approve`] instruction.
pub const APPROVE_ACCOUNTS_BUFFER_LEN: usize = 3;
/// The length of the [`Approve`] instruction data.
pub const APPROVE_DATA_LEN: usize = 9;
/// The [`Approve`] instruction discriminator.
pub const APPROVE_DISCRIMINATOR: [u8; 1] = [4];

type AccountMetaBuffer<'meta> = [AccountMeta<'meta>; APPROVE_ACCOUNTS_BUFFER_LEN];
type CpiAccountBuffer<'view> = [CpiAccount<'view>; APPROVE_ACCOUNTS_BUFFER_LEN];
type DataBuffer = [u8; APPROVE_DATA_LEN];

impl<'view, P: TokenProgram>
    CpiSerialize<'view, AccountMetaBuffer<'view>, CpiAccountBuffer<'view>, DataBuffer>
    for Approve<'view, P>
{
    #[inline(always)]
    fn serialize(
        &self,
    ) -> Result<
        CpiCtx<
            'view,
            'view,
            'static,
            AccountMetaBuffer<'view>,
            CpiAccountBuffer<'view>,
            DataBuffer,
        >,
    > {
        // SAFETY: buffer is fully initialized before `assume_init`.
        let data = unsafe {
            let mut buf = MaybeUninit::<DataBuffer>::uninit();
            let ptr = buf.as_mut_ptr() as *mut u8;
            (ptr as *mut [u8; 1]).write(APPROVE_DISCRIMINATOR);
            copy_nonoverlapping(
                self.amount.to_le_bytes().as_ptr(),
                ptr.add(1),
                size_of::<u64>(),
            );

            buf.assume_init()
        };

        let metas = [
            AccountMeta::writable(self.source.address()),
            AccountMeta::readonly(self.delegate.address()),
            AccountMeta::readonly_signer(self.owner.address()),
        ];

        let views = init_cpi_accounts!(self.source, self.delegate, self.owner,);

        Ok(CpiCtx::new(&P::ID, metas, views, data))
    }
}
