// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

use core::marker::PhantomData;

use crate::{
    hint,
    syscalls::{try_create_program_address, try_find_program_address},
    ErrorCode, Result, ValidateMeta,
};
use hayabusa_err_macro::err;
use solana_account_view::AccountView;
use solana_address::Address;
use solana_program_error::ProgramError;

pub struct CheckAddress<'a, Next: ValidateMeta> {
    address: &'a Address,
    next: Next,
}

unsafe impl<'a, Next: ValidateMeta> ValidateMeta for CheckAddress<'a, Next> {
    #[inline(always)]
    fn validate(&self, view: &AccountView) -> Result<()> {
        if hint::unlikely(self.address != view.address()) {
            err!(
                "CheckAddress::validate: address different than expected",
                ErrorCode::InvalidAddress,
            );
        }

        self.next.validate(view)
    }
}

pub struct Pda<'a, 'b, 'c, 'd, Next: ValidateMeta> {
    seeds: &'a [&'b [u8]],
    program_id: &'c Address,
    bump: Option<*mut u8>,
    next: Next,
    _phantom: PhantomData<&'d mut u8>,
}

unsafe impl<'a, 'b, 'c, 'd, Next: ValidateMeta> ValidateMeta for Pda<'a, 'b, 'c, 'd, Next> {
    #[inline(always)]
    fn validate(&self, view: &AccountView) -> Result<()> {
        let address = if let Some(bump) = self.bump {
            let (pda_address, b) = try_find_program_address(self.seeds, self.program_id)?;

            // SAFETY: This is admittedly pretty disgusting, but should be sound
            unsafe { *bump = b };

            pda_address
        } else {
            try_create_program_address(self.seeds, self.program_id)?
        };

        if hint::unlikely(&address != view.address()) {
            err!(
                "Pda::validate: account is not the required PDA",
                ErrorCode::InvalidSeeds,
            );
        }

        self.next.validate(view)
    }
}

pub struct Constraint<F, Next: ValidateMeta>
where
    F: for<'a> Fn(&'a AccountView) -> Result<()>,
{
    f: F,
    next: Next,
}

unsafe impl<F, Next: ValidateMeta> ValidateMeta for Constraint<F, Next>
where
    F: for<'a> Fn(&'a AccountView) -> Result<()>,
{
    #[inline(always)]
    fn validate(&self, view: &AccountView) -> Result<()> {
        (self.f)(view)?;

        self.next.validate(view)
    }
}

#[cfg(test)]
mod test {
    use core::panic;

    use solana_account_view::{AccountView, RuntimeAccount};
    use solana_address::Address;
    use solana_program_error::ProgramError;

    use crate::{
        metas::{CheckAddress, Constraint},
        NoMeta, ValidateMeta,
    };

    #[test]
    fn test_check_address_validator() {
        let addr = Address::default();
        let mut runtime_account = RuntimeAccount::default();
        let view =
            unsafe { AccountView::new_unchecked(&mut runtime_account as *mut RuntimeAccount) };
        let meta = CheckAddress {
            address: &addr,
            next: NoMeta,
        };

        match meta.validate(&view) {
            Ok(_) => (),
            Err(e) => panic!("failed with errorcode: {}", e),
        };
    }

    #[test]
    fn test_constraint_validator() {
        let addr = Address::new_from_array([1u8; 32]);
        let mut runtime_account = RuntimeAccount::default();
        let view =
            unsafe { AccountView::new_unchecked(&mut runtime_account as *mut RuntimeAccount) };
        let meta = Constraint {
            f: |v| {
                if &addr != v.address() {
                    return Err(ProgramError::InvalidAccountData);
                }

                Ok(())
            },
            next: NoMeta,
        };

        match meta.validate(&view) {
            Ok(_) => (),
            Err(e) => panic!("failed with errorcode: {}", e),
        }
    }
}
