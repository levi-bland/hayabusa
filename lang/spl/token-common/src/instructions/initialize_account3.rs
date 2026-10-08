// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use core::{marker::PhantomData, mem::MaybeUninit, ptr::copy_nonoverlapping};

use crate::TokenProgram;
use hayabusa_common::{
    Result,
    account_meta::AccountMeta,
    account_view::AccountView,
    address::Address,
    cpi::{CpiAccount, CpiCtx},
    init_cpi_accounts,
    traits::CpiSerialize,
};
use hayabusa_invoke_cpi_macros::CpiInstruction;

/// Like [`super::InitializeAccount`], but the owner pubkey is
/// passed via instruction data rather than the accounts list. This
/// variant may be preferable when using Cross Program Invocation from
/// an instruction that does not need the owner's `AccountView`
/// otherwise. Also does not require the Rent sysvar.
#[derive(CpiInstruction)]
pub struct InitializeAccount3<'view, 'addr, P: TokenProgram> {
    /// The account to initialize.
    pub account: AccountView<'view>,
    /// The mint the account will be associated with.
    pub mint: AccountView<'view>,
    /// The new account's owner or multisig.
    pub owner: &'addr Address,
    /// Signifies which token program to use for the CPI.
    __phantom: PhantomData<P>,
}

impl<'view, 'addr, P: TokenProgram> InitializeAccount3<'view, 'addr, P> {
    pub fn new(
        account: AccountView<'view>,
        mint: AccountView<'view>,
        owner: &'addr Address,
    ) -> Self {
        Self {
            account,
            mint,
            owner,
            __phantom: PhantomData,
        }
    }
}

/// The number of accounts required by the [`InitializeAccount3`] instruction.
pub const INITIALIZE_ACCOUNT3_ACCOUNTS_BUFFER_LEN: usize = 2;
/// The length of the [`InitializeAccount3`] instruction data.
pub const INITIALIZE_ACCOUNT3_DATA_LEN: usize = 33;
/// The [`InitializeAccount3`] instruction discriminator.
pub const INITIALIZE_ACCOUNT3_DISCRIMINATOR: [u8; 1] = [18];

type AccountMetaBuffer<'meta> = [AccountMeta<'meta>; INITIALIZE_ACCOUNT3_ACCOUNTS_BUFFER_LEN];
type CpiAccountBuffer<'view> = [CpiAccount<'view>; INITIALIZE_ACCOUNT3_ACCOUNTS_BUFFER_LEN];
type DataBuffer = [u8; INITIALIZE_ACCOUNT3_DATA_LEN];

impl<'view, P: TokenProgram>
    CpiSerialize<'view, AccountMetaBuffer<'view>, CpiAccountBuffer<'view>, DataBuffer>
    for InitializeAccount3<'view, '_, P>
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
            (ptr as *mut [u8; 1]).write(INITIALIZE_ACCOUNT3_DISCRIMINATOR);
            copy_nonoverlapping(
                self.owner.as_ref().as_ptr(),
                ptr.add(1),
                size_of::<Address>(),
            );

            buf.assume_init()
        };

        let metas = [
            AccountMeta::writable(self.account.address()),
            AccountMeta::readonly(self.mint.address()),
        ];

        let views = init_cpi_accounts!(self.account, self.mint,);

        Ok(CpiCtx::new(&P::ID, metas, views, data))
    }
}
