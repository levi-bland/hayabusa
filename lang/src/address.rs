// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct Address(solana_address::Address);

impl From<solana_address::Address> for Address {
    #[inline(always)]
    fn from(addr: solana_address::Address) -> Address {
        Address(addr)
    }
}

impl From<[u8; 32]> for Address {
    #[inline(always)]
    fn from(arr: [u8; 32]) -> Address {
        Address(solana_address::Address::from(arr))
    }
}

impl Address {
    pub const fn new(addr: solana_address::Address) -> Address {
        Address(addr)
    }
}

unsafe impl bytemuck::Pod for Address {}
unsafe impl bytemuck::Zeroable for Address {}

impl core::ops::Deref for Address {
    type Target = solana_address::Address;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for Address {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[macro_export]
macro_rules! declare_id {
    ($address:literal) => {
        /// The program ID.
        pub const ID: $crate::prelude::Address = $crate::prelude::address!($address);

        /// Returns the program ID.
        pub const fn id() -> $crate::prelude::Address {
            ID
        }
    };
}

#[macro_export]
macro_rules! address {
    ($bs58str:literal) => {
        Address::new($crate::prelude::solana_address::address!($bs58str))
    };
}

#[inline(always)]
pub fn address_eq(a: &Address, b: &Address) -> bool {
    solana_address::address_eq(&a.0, &b.0)
}
