// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

use core::ops::Deref;

use crate::prelude::*;

pub mod layout {
    use crate::{impl_sysvar_get, prelude::*, sysvars::SYSVAR_PROGRAM_ID};

    /// A representation of network time.
    ///
    /// All members of `Clock` start from 0 upon network boot.
    #[repr(C)]
    #[derive(Clone, Debug)]
    pub struct Clock {
        /// The current slot.
        pub slot: u64,

        /// The timestamp of the first slot in this epoch.
        pub epoch_start_timestamp: i64,

        /// The current epoch.
        pub epoch: u64,

        /// The future epoch for which the leader schedule has
        /// most recently been calculated.
        pub leader_schedule_epoch: u64,

        /// The approximate real world time of the current slot.
        ///
        /// This value was originally computed from genesis creation time and
        /// network time in slots, incurring a lot of drift. Following activation of
        /// the [`timestamp_correction` and `timestamp_bounding`][tsc] features it
        /// is calculated using a [validator timestamp oracle][oracle].
        ///
        /// [tsc]: https://docs.solanalabs.com/implemented-proposals/bank-timestamp-correction
        /// [oracle]: https://docs.solanalabs.com/implemented-proposals/validator-timestamp-oracle
        pub unix_timestamp: i64,
    }

    impl SysvarTrait for Clock {
        impl_sysvar_get!(Clock::ID, 0);
    }

    impl Owner for Clock {
        const OWNER: Address = SYSVAR_PROGRAM_ID;
    }

    impl Clock {
        /// The ID of the [`Clock`] sysvar
        pub const ID: Address = address!("SysvarC1ock11111111111111111111111111111111");
    }
}

/// Clock sysvar account representation, carries its own [`AccountView`]
pub struct Clock<'view> {
    /// The `Clock` sysvar account view.
    view: AccountView<'view>,
    /// Deserialized clock sysvar data.
    clock: layout::Clock,
}

impl Clock<'_> {
    /// The ID of the [`Clock`] sysvar.
    pub const ID: Address = layout::Clock::ID;
}

// TODO: probably come back and check these constants because of alpenglow network upgrade

/// At 160 ticks/s, 64 ticks per slot implies that leader rotation and voting will happen
/// every 400 ms. A fast voting cadence ensures faster finality and convergence
pub const DEFAULT_TICKS_PER_SLOT: u64 = 64;

/// The default tick rate that the cluster attempts to achieve (160 per second).
///
/// Note that the actual tick rate at any given time should be expected to drift.
pub const DEFAULT_TICKS_PER_SECOND: u64 = 160;

/// The expected duration of a slot (400 milliseconds).
// Actually calculation is supposed to be derived DEFAULT_TICKS_PER_SLOT / DEFAULT_TICKS_PER_SECOND
pub const DEFAULT_MS_PER_SLOT: u64 = 1_000 * DEFAULT_TICKS_PER_SLOT / DEFAULT_TICKS_PER_SECOND;

impl<'view> TrySysvarFromAccountView<'view> for Clock<'view> {
    #[inline(always)]
    fn try_sysvar_from_account_view(view: AccountView<'view>) -> Result<Clock<'view>> {
        if hint::unlikely(!address_eq(view.address(), &Clock::ID)) {
            return Err(ErrorCode::InvalidSysvarAccount.into());
        }

        // SAFETY: the account data is guaranteed to be the clock sysvar by the above.
        let clock = unsafe { &*(view.data_ptr() as *const layout::Clock) }.clone();

        Ok(Clock { view, clock })
    }
}

impl<'view> ToAccountView<'view> for Clock<'view> {
    #[inline(always)]
    fn to_account_view(&self) -> AccountView<'view> {
        self.view
    }
}

impl Deref for Clock<'_> {
    type Target = layout::Clock;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.clock
    }
}

impl<'view> AccountAddress<'view> for Clock<'view> {
    #[inline(always)]
    fn address(&self) -> &'view Address {
        &Self::ID
    }

    #[inline(always)]
    fn address_owned(&self) -> Address {
        Self::ID
    }
}

#[macro_export]
macro_rules! slot {
    () => {
        Clock::get()?.slot
    };
}

#[macro_export]
macro_rules! timestamp {
    () => {
        Clock::get()?.unix_timestamp
    };
}
