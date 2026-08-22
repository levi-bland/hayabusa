#![feature(prelude_import)]
#![allow(dead_code, unexpected_cfgs)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use hayabusa::prelude::*;
/// The const program ID.
pub const ID: ::solana_address::Address = ::solana_address::Address::from_str_const(
    "HPoDm7Kf63B6TpFKV7S8YSd7sGde6sVdztiDBEVkfuxz",
);
/// Returns `true` if given address is the ID.
pub fn check_id(id: &::solana_address::Address) -> bool {
    id == &ID
}
/// Returns the ID.
pub const fn id() -> ::solana_address::Address {
    { ID }
}
mod instruction {
    use super::*;
    #[discriminator(namespace = "instruction")]
    #[repr(C)]
    pub struct UpdateCounterIx {
        pub amount: u64,
    }
    impl Discriminator for UpdateCounterIx {
        const DISCRIMINATOR: &'static [u8] = &[
            14u8, 180u8, 32u8, 168u8, 86u8, 13u8, 182u8, 42u8,
        ];
    }
    impl<'ix> DecodeIx<'ix> for UpdateCounterIx {
        #[inline(always)]
        fn decode(bytes: &'ix [u8]) -> Result<Self> {
            if bytes.len() != core::mem::size_of::<u64>() {
                return Err(ProgramError::InvalidInstructionData);
            }
            let mut __off: usize = 0usize;
            let amount: u64 = unsafe {
                core::ptr::read_unaligned(bytes.as_ptr().add(__off) as *const u64)
            };
            __off += core::mem::size_of::<u64>();
            Ok(Self { amount: amount })
        }
    }
    #[discriminator(namespace = "instruction")]
    #[repr(C)]
    pub struct InitializeCounterIx {}
    impl Discriminator for InitializeCounterIx {
        const DISCRIMINATOR: &'static [u8] = &[
            23u8, 153u8, 241u8, 67u8, 150u8, 141u8, 56u8, 133u8,
        ];
    }
    impl<'ix> DecodeIx<'ix> for InitializeCounterIx {
        #[inline(always)]
        fn decode(bytes: &'ix [u8]) -> Result<Self> {
            if bytes.len() != 0usize {
                return Err(ProgramError::InvalidInstructionData);
            }
            let mut __off: usize = 0usize;
            Ok(Self {})
        }
    }
}
pub struct UpdateCounter<'ix> {
    pub counter: Mut<ZcAccount<'ix, CounterAccount>>,
    #[meta(address = &counter.try_deserialize()?.authority)]
    pub user: CheckedAddress<Signer<'ix>>,
    #[meta(num = data.amount)]
    pub num_test_account: NumTestAccount,
}
impl<'ix> FromAccountViews<'ix> for UpdateCounter<'ix> {
    type IxData = crate::instruction::UpdateCounterIx;
    #[inline(always)]
    fn try_from_account_views(
        account_views: &mut AccountIter<'ix>,
        data: &Self::IxData,
    ) -> Result<Self> {
        let _ = data;
        let counter = <Mut<
            ZcAccount<'ix, CounterAccount>,
        > as FromAccountView<
            'ix,
        >>::try_from_account_view(account_views.next()?, NoMeta)?;
        let user = <CheckedAddress<
            Signer<'ix>,
        > as FromAccountView<
            'ix,
        >>::try_from_account_view(
            account_views.next()?,
            CheckedAddressMeta {
                address: &counter.try_deserialize()?.authority,
            },
        )?;
        let num_test_account = <NumTestAccount as FromAccountView<
            'ix,
        >>::try_from_account_view(
            account_views.next()?,
            NumTestAccountMeta {
                num: data.amount,
            },
        )?;
        Ok(Self {
            counter,
            user,
            num_test_account,
        })
    }
}
pub struct InitializeCounter<'ix> {
    pub user: Mut<Signer<'ix>>,
    pub counter: Mut<ZcAccount<'ix, CounterAccount>>,
    pub system_program: Program<'ix, System>,
}
impl<'ix> FromAccountViews<'ix> for InitializeCounter<'ix> {
    type IxData = crate::instruction::InitializeCounterIx;
    #[inline(always)]
    fn try_from_account_views(
        account_views: &mut AccountIter<'ix>,
        data: &Self::IxData,
    ) -> Result<Self> {
        let _ = data;
        let user = <Mut<
            Signer<'ix>,
        > as FromAccountView<
            'ix,
        >>::try_from_account_view(account_views.next()?, NoMeta)?;
        let counter = <Mut<
            ZcAccount<'ix, CounterAccount>,
        > as FromAccountView<
            'ix,
        >>::try_from_account_view(account_views.next()?, NoMeta)?;
        let system_program = <Program<
            'ix,
            System,
        > as FromAccountView<
            'ix,
        >>::try_from_account_view(account_views.next()?, NoMeta)?;
        Ok(Self {
            user,
            counter,
            system_program,
        })
    }
}
#[repr(C)]
pub struct CounterAccount {
    pub count: u64,
    pub authority: Address,
}
impl hayabusa::idl::IdlBuild for CounterAccount {
    fn create_type() -> Option<hayabusa::idl::IdlTypeDef> {
        Some(hayabusa::idl::IdlTypeDef {
            name: "CounterAccount".to_string(),
            docs: ::alloc::vec::Vec::new(),
            serialization: hayabusa::idl::IdlSerialization::Bytemuck,
            repr: ::hayabusa::idl::IdlRepr::C(hayabusa::idl::IdlReprModifier {
                packed: false,
                align: None,
            }),
            ty: ::hayabusa::idl::IdlTypeDefTy::Struct {
                fields: Some(
                    ::hayabusa::idl::IdlDefinedFields::Named(
                        <[_]>::into_vec(
                            ::alloc::boxed::box_new([
                                ::hayabusa::idl::IdlField {
                                    name: "count".to_string(),
                                    docs: ::alloc::vec::Vec::new(),
                                    ty: ::hayabusa::idl::IdlType::U64,
                                },
                                ::hayabusa::idl::IdlField {
                                    name: "authority".to_string(),
                                    docs: ::alloc::vec::Vec::new(),
                                    ty: ::hayabusa::idl::IdlType::Pubkey,
                                },
                            ]),
                        ),
                    ),
                ),
            },
        })
    }
    fn insert_types(
        types: &mut std::collections::BTreeMap<String, hayabusa::idl::IdlTypeDef>,
    ) {
        if let Some(ty) = Self::create_type() {
            types.insert(Self::get_full_path(), ty);
        }
        <Address as hayabusa::idl::IdlBuild>::insert_types(types);
        <u64 as hayabusa::idl::IdlBuild>::insert_types(types);
    }
}
const _: () = {
    if !(::core::mem::size_of::<CounterAccount>()
        == (::core::mem::size_of::<u64>() + ::core::mem::size_of::<Address>()))
    {
        {
            ::std::rt::begin_panic("derive(Pod) was applied to a type with padding");
        }
    }
};
const _: fn() = || {
    #[allow(clippy::missing_const_for_fn)]
    #[doc(hidden)]
    fn check() {
        fn assert_impl<T: ::bytemuck::Pod>() {}
        assert_impl::<u64>();
    }
};
const _: fn() = || {
    #[allow(clippy::missing_const_for_fn)]
    #[doc(hidden)]
    fn check() {
        fn assert_impl<T: ::bytemuck::Pod>() {}
        assert_impl::<Address>();
    }
};
unsafe impl ::bytemuck::Pod for CounterAccount {}
const _: fn() = || {
    #[allow(clippy::missing_const_for_fn)]
    #[doc(hidden)]
    fn check() {
        fn assert_impl<T: ::bytemuck::Zeroable>() {}
        assert_impl::<u64>();
    }
};
const _: fn() = || {
    #[allow(clippy::missing_const_for_fn)]
    #[doc(hidden)]
    fn check() {
        fn assert_impl<T: ::bytemuck::Zeroable>() {}
        assert_impl::<Address>();
    }
};
unsafe impl ::bytemuck::Zeroable for CounterAccount {}
impl Discriminator for CounterAccount {
    const DISCRIMINATOR: &'static [u8] = &[
        164u8, 8u8, 153u8, 71u8, 8u8, 44u8, 93u8, 22u8,
    ];
}
impl Len for CounterAccount {}
impl Deserialize for CounterAccount {}
impl DeserializeMut for CounterAccount {}
impl Zc for CounterAccount {}
impl ZcDeserialize for CounterAccount {}
impl ZcDeserializeMut for CounterAccount {}
impl ZcInitialize for CounterAccount {}
#[automatically_derived]
impl ::core::marker::Copy for CounterAccount {}
#[automatically_derived]
impl ::core::clone::Clone for CounterAccount {
    #[inline]
    fn clone(&self) -> CounterAccount {
        let _: ::core::clone::AssertParamIsClone<u64>;
        let _: ::core::clone::AssertParamIsClone<Address>;
        *self
    }
}
impl OwnerProgram for CounterAccount {
    const OWNER: Address = crate::ID;
    fn owner() -> Address {
        Self::OWNER
    }
}
pub struct NumTestAccount {
    pub num: u64,
}
unsafe impl<'ix> FromAccountView<'ix> for NumTestAccount {
    type Meta<'a> = NumTestAccountMeta where 'ix: 'a;
    fn try_from_account_view<'a>(
        _: &'ix AccountView,
        meta: Self::Meta<'a>,
    ) -> Result<Self>
    where
        'ix: 'a,
    {
        Ok(Self { num: meta.num })
    }
}
pub struct NumTestAccountMeta {
    pub num: u64,
}
