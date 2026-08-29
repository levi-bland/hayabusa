// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use crate::{init_cpi_accounts, prelude::*, spl::token::Token};

/// Closes an account by transferring all of its lamports to the
/// destination account. Non-native accounts may only be closed if
/// their token amount is zero.
pub struct CloseAccount<'view> {
    /// The account to close.
    pub account: AccountView<'view>,
    /// The destination for the closed account's lamports.
    pub destination: AccountView<'view>,
    /// The account's owner.
    pub owner: AccountView<'view>,
}

/// The number of accounts required by the [`CloseAccount`] instruction.
pub const CLOSE_ACCOUNT_ACCOUNTS_BUFFER_LEN: usize = 3;
/// The length of the [`CloseAccount`] instruction data.
pub const CLOSE_ACCOUNT_DATA_LEN: usize = 1;
/// The [`CloseAccount`] instruction discriminator.
pub const CLOSE_ACCOUNT_DISCRIMINATOR: [u8; 1] = [9];

type AccountMetaBuffer<'meta> = [AccountMeta<'meta>; CLOSE_ACCOUNT_ACCOUNTS_BUFFER_LEN];
type CpiAccountBuffer<'view> = [CpiAccount<'view>; CLOSE_ACCOUNT_ACCOUNTS_BUFFER_LEN];
type DataBuffer = [u8; CLOSE_ACCOUNT_DATA_LEN];

impl<'view>
    SerializeCpiCtx<
        'view,
        AccountMetaBuffer<'view>,
        CpiAccountBuffer<'view>,
        DataBuffer,
    > for CloseAccount<'view>
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
        let data = CLOSE_ACCOUNT_DISCRIMINATOR;

        let metas = [
            AccountMeta::writable(self.account.address()),
            AccountMeta::writable(self.destination.address()),
            AccountMeta::readonly_signer(self.owner.address()),
        ];

        let views = init_cpi_accounts!(self.account, self.destination, self.owner,);

        Ok(CpiCtx::new(&Token::ID, metas, views, data))
    }
}