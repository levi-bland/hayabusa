// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use super::Token;
use hayabusa_spl_token_common as token_common;

pub type Approve<'view> = token_common::instructions::approve::Approve<'view, Token>;
pub type ApproveChecked<'view> =
    token_common::instructions::approve_checked::ApproveChecked<'view, Token>;
pub type Burn<'view> = token_common::instructions::burn::Burn<'view, Token>;
pub type BurnChecked<'view> = token_common::instructions::burn_checked::BurnChecked<'view, Token>;
pub type CloseAccount<'view> =
    token_common::instructions::close_account::CloseAccount<'view, Token>;
pub type FreezeAccount<'view> =
    token_common::instructions::freeze_account::FreezeAccount<'view, Token>;
pub type InitializeAccount<'view> =
    token_common::instructions::initialize_account::InitializeAccount<'view, Token>;
pub type InitializeAccount2<'view, 'addr> =
    token_common::instructions::initialize_account2::InitializeAccount2<'view, 'addr, Token>;
pub type InitializeAccount3<'view, 'addr> =
    token_common::instructions::initialize_account3::InitializeAccount3<'view, 'addr, Token>;
pub type InitializeMint<'view, 'addr> =
    token_common::instructions::initialize_mint::InitializeMint<'view, 'addr, Token>;
pub type InitializeMint2<'view, 'addr> =
    token_common::instructions::initialize_mint2::InitializeMint2<'view, 'addr, Token>;
pub type InitializeMultisig<'view, 'signers, const ACCOUNTS: usize> =
    token_common::instructions::initialize_multisig::InitializeMultisig<
        'view,
        'signers,
        ACCOUNTS,
        Token,
    >;
pub type InitializeMultisig2<'view, 'signers, const ACCOUNTS: usize> =
    token_common::instructions::initialize_multisig2::InitializeMultisig2<
        'view,
        'signers,
        ACCOUNTS,
        Token,
    >;
pub type MintTo<'view> = token_common::instructions::mint_to::MintTo<'view, Token>;
pub type MintToChecked<'view> =
    token_common::instructions::mint_to_checked::MintToChecked<'view, Token>;
pub type Revoke<'view> = token_common::instructions::revoke::Revoke<'view, Token>;
pub type SetAuthority<'view, 'addr> =
    token_common::instructions::set_authority::SetAuthority<'view, 'addr, Token>;
pub type SyncNative<'view> = token_common::instructions::sync_native::SyncNative<'view, Token>;
pub type ThawAccount<'view> = token_common::instructions::thaw_account::ThawAccount<'view, Token>;
pub type Transfer<'view> = token_common::instructions::transfer::Transfer<'view, Token>;
pub type TransferChecked<'view> =
    token_common::instructions::transfer_checked::TransferChecked<'view, Token>;
