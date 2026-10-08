// Copyright (c) 2026, Levi Bland <levi.bland@icloud.com>
// SPDX-License-Identifier: Apache-2.0

pub use solana_address;

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

    pub const fn new_from_array(arr: [u8; 32]) -> Address {
        Address(solana_address::Address::new_from_array(arr))
    }

    pub const fn to_solana_address(&self) -> solana_address::Address {
        self.0
    }
}

unsafe impl bytemuck::Pod for Address {}
unsafe impl bytemuck::Zeroable for Address {}

const _: () = assert!(size_of::<Address>() == size_of::<[u8; 32]>());

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

impl core::convert::AsRef<[u8]> for Address {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl core::convert::AsMut<[u8]> for Address {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut [u8] {
        self.0.as_mut()
    }
}

#[macro_export]
macro_rules! declare_id {
    ($address:literal) => {
        /// The program ID.
        pub const ID: &'static $crate::address::Address = &$crate::address!($address);

        /// Returns the program ID.
        pub const fn id() -> &'static $crate::address::Address {
            ID
        }
    };
}

#[macro_export]
macro_rules! address {
    ($bs58str:literal) => {
        Address::new($crate::address::solana_address::address!($bs58str))
    };
}

#[inline(always)]
pub fn address_eq(a: &Address, b: &Address) -> bool {
    solana_address::address_eq(&a.0, &b.0)
}
