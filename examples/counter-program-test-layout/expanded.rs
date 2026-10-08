#![feature(prelude_import)]
#![no_std]
extern crate core;
#[prelude_import]
use core::prelude::rust_2021::*;
use hayabusa::prelude::*;
/// The program ID.
pub const ID: &'static ::hayabusa_common::address::Address = &Address::new(
    ::solana_address::Address::from_str_const(
        "HPoDm7Kf63B6TpFKV7S8YSd7sGde6sVdztiDBEVkfuxz",
    ),
);
/// Returns the program ID.
pub const fn id() -> &'static ::hayabusa_common::address::Address {
    ID
}
pub mod instruction {
    use super::*;
    #[borsh(crate = "::hayabusa::prelude::borsh")]
    #[discriminator(namespace = "instruction")]
    pub struct InitializeCounterIx {
        pub num: u64,
    }
    #[automatically_derived]
    impl ::hayabusa::prelude::Discriminator for InitializeCounterIx {
        const DISCRIMINATOR: &[u8] = &[
            23u8, 153u8, 241u8, 67u8, 150u8, 141u8, 56u8, 133u8,
        ];
    }
    #[automatically_derived]
    impl ::hayabusa::prelude::borsh::de::BorshDeserialize for InitializeCounterIx {
        fn deserialize_reader<__R: ::hayabusa::prelude::borsh::io::Read>(
            reader: &mut __R,
        ) -> ::core::result::Result<Self, ::hayabusa::prelude::borsh::io::Error> {
            Ok(Self {
                num: ::hayabusa::prelude::borsh::BorshDeserialize::deserialize_reader(
                    reader,
                )?,
            })
        }
    }
    #[automatically_derived]
    impl ::hayabusa::prelude::borsh::ser::BorshSerialize for InitializeCounterIx {
        fn serialize<__W: ::hayabusa::prelude::borsh::io::Write>(
            &self,
            writer: &mut __W,
        ) -> ::core::result::Result<(), ::hayabusa::prelude::borsh::io::Error> {
            ::hayabusa::prelude::borsh::BorshSerialize::serialize(&self.num, writer)?;
            Ok(())
        }
    }
    #[borsh(crate = "::hayabusa::prelude::borsh")]
    #[discriminator(namespace = "instruction")]
    pub struct IncrementCounterIx {}
    #[automatically_derived]
    impl ::hayabusa::prelude::Discriminator for IncrementCounterIx {
        const DISCRIMINATOR: &[u8] = &[133u8, 59u8, 94u8, 72u8, 5u8, 232u8, 164u8, 85u8];
    }
    #[automatically_derived]
    impl ::hayabusa::prelude::borsh::de::BorshDeserialize for IncrementCounterIx {
        fn deserialize_reader<__R: ::hayabusa::prelude::borsh::io::Read>(
            reader: &mut __R,
        ) -> ::core::result::Result<Self, ::hayabusa::prelude::borsh::io::Error> {
            Ok(Self {})
        }
    }
    #[automatically_derived]
    impl ::hayabusa::prelude::borsh::ser::BorshSerialize for IncrementCounterIx {
        fn serialize<__W: ::hayabusa::prelude::borsh::io::Write>(
            &self,
            writer: &mut __W,
        ) -> ::core::result::Result<(), ::hayabusa::prelude::borsh::io::Error> {
            Ok(())
        }
    }
}
#[allow(unexpected_cfgs)]
mod counter_program {
    use super::*;
    use super::instruction::*;
    extern crate alloc;
    /// Program entrypoint.
    #[no_mangle]
    pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
        ::hayabusa::entrypoint::process_entrypoint::<
            { ::hayabusa::entrypoint::MAX_TX_ACCOUNTS },
        >(input, dispatcher)
    }
    fn dispatcher(
        program_id: &Address,
        views: &[AccountView],
        ix_data: &[u8],
    ) -> Result<()> {
        if hint::unlikely(!address_eq(program_id, &crate::ID)) {
            return Err(ProgramError::IncorrectProgramId);
        }
        const DISC_LEN: usize = 8;
        if hint::unlikely(ix_data.len() < DISC_LEN) {
            return Err(ProgramError::InvalidInstructionData);
        }
        let (disc, rest) = {
            let ptr = ix_data.as_ptr();
            (&ix_data[..DISC_LEN], &ix_data[DISC_LEN..])
        };
        match disc {
            <InitializeCounterIx as Discriminator>::DISCRIMINATOR => {
                let mut accounts_iter = AccountIter::new(views);
                let ix_data = <InitializeCounterIx as ::hayabusa::prelude::borsh::BorshDeserialize>::try_from_slice(
                        rest,
                    )
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                let mut bumps = <<InitializeCounter as Bumps>::Bumps as Default>::default();
                let mut accounts = <InitializeCounter as ParseAccounts<
                    '_,
                    <InitializeCounter as Bumps>::Bumps,
                >>::parse_accounts(&mut accounts_iter, rest, &mut bumps)?;
                let ctx = Ctx::new(
                    program_id,
                    &mut accounts,
                    accounts_iter.remaining(),
                    bumps,
                );
                initialize_counter(ctx, ix_data.num).map_err(Into::into)
            }
            <IncrementCounterIx as Discriminator>::DISCRIMINATOR => {
                let mut accounts_iter = AccountIter::new(views);
                let ix_data = <IncrementCounterIx as ::hayabusa::prelude::borsh::BorshDeserialize>::try_from_slice(
                        rest,
                    )
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                let mut bumps = <<IncrementCounter as Bumps>::Bumps as Default>::default();
                let mut accounts = <IncrementCounter as ParseAccounts<
                    '_,
                    <IncrementCounter as Bumps>::Bumps,
                >>::parse_accounts(&mut accounts_iter, rest, &mut bumps)?;
                let ctx = Ctx::new(
                    program_id,
                    &mut accounts,
                    accounts_iter.remaining(),
                    bumps,
                );
                increment_counter(ctx).map_err(Into::into)
            }
            _ => Err(ErrorCode::UnknownInstruction.into()),
        }
    }
    pub fn initialize_counter(ctx: Ctx<InitializeCounter>, num: u64) -> Result<()> {
        let mut counter = ctx.accounts.counter.cast_mut()?;
        counter.authority = ctx.accounts.signer.address_owned();
        counter.count = num;
        counter.bump = ctx.bumps.counter;
        Ok(())
    }
    pub fn increment_counter(ctx: Ctx<IncrementCounter>) -> Result<()> {
        let mut counter = ctx.accounts.counter.cast_mut()?;
        counter.count += 1;
        Ok(())
    }
}
pub struct InitializeCounter<'view> {
    pub signer: Mut<Signer<'view>>,
    #[account(payer = signer, seeds = [Counter::SEED, signer.address().as_ref()], bump)]
    pub counter: Init<Pda<Account<'view, Counter>>>,
    pub system_program: Program<'view, System>,
}
#[automatically_derived]
pub struct InitializeCounterBumps {
    pub counter: u8,
}
#[automatically_derived]
impl ::core::default::Default for InitializeCounterBumps {
    #[inline]
    fn default() -> InitializeCounterBumps {
        InitializeCounterBumps {
            counter: ::core::default::Default::default(),
        }
    }
}
#[automatically_derived]
impl Bumps for InitializeCounter<'_> {
    type Bumps = InitializeCounterBumps;
}
#[automatically_derived]
impl<'view> ParseAccounts<'view, InitializeCounterBumps> for InitializeCounter<'view> {
    const NUM_ACCOUNTS: usize = 3usize;
    fn parse_accounts(
        views: &mut AccountIter<'view>,
        data: &[u8],
        bumps: &mut InitializeCounterBumps,
    ) -> Result<Self> {
        views.has_remaining(Self::NUM_ACCOUNTS)?;
        let signer_view = unsafe { views.next_unchecked() };
        let counter_view = unsafe { views.next_unchecked() };
        let system_program_view = unsafe { views.next_unchecked() };
        let signer: Mut<Signer<'view>> = <Mut<
            Signer<'view>,
        > as ParseAccount<'_>>::parse(signer_view, &mut NoMeta)?;
        let system_program: Program<'view, System> = <Program<
            'view,
            System,
        > as ParseAccount<'_>>::parse(system_program_view, &mut NoMeta)?;
        let mut __seeds_buffer = [
            ::hayabusa_common::cpi::Seed::from(Counter::SEED),
            ::hayabusa_common::cpi::Seed::from(signer.address().as_ref()),
            ::hayabusa_common::cpi::Seed::from(::hayabusa::prelude::Seed::from(&[][..])),
        ];
        let __growable_signer = ::hayabusa::prelude::GrowableSigner::try_new(
            &mut __seeds_buffer,
            2usize,
        )?;
        let counter: Init<Pda<Account<'view, Counter>>> = <Init<
            Pda<Account<'view, Counter>>,
        > as ParseAccount<
            '_,
        >>::parse(
            counter_view,
            &mut ::hayabusa::prelude::__AccountPdaInitMeta::new(
                &signer,
                ::hayabusa::prelude::SignerBumpness::without(
                    __growable_signer,
                    Some(&mut bumps.counter),
                ),
            ),
        )?;
        Ok(Self {
            signer,
            counter,
            system_program,
        })
    }
}
pub struct IncrementCounter<'view> {
    pub authority: Signer<'view>,
    #[account(
        has_one = authority,
        seeds = [Counter::SEED,
        authority.address().as_ref(),
        ],
        bump = counter.bump,
    )]
    pub counter: Mut<Pda<Account<'view, Counter>>>,
}
#[automatically_derived]
impl Bumps for IncrementCounter<'_> {
    type Bumps = ();
}
#[automatically_derived]
impl<'view> ParseAccounts<'view, ()> for IncrementCounter<'view> {
    const NUM_ACCOUNTS: usize = 2usize;
    fn parse_accounts(
        views: &mut AccountIter<'view>,
        data: &[u8],
        bumps: &mut (),
    ) -> Result<Self> {
        views.has_remaining(Self::NUM_ACCOUNTS)?;
        let authority_view = unsafe { views.next_unchecked() };
        let counter_view = unsafe { views.next_unchecked() };
        let authority: Signer<'view> = <Signer<
            'view,
        > as ParseAccount<'_>>::parse(authority_view, &mut NoMeta)?;
        let counter: Mut<Pda<Account<'view, Counter>>> = <Mut<
            Pda<Account<'view, Counter>>,
        > as ParseAccount<
            '_,
        >>::parse(
            counter_view,
            &mut ::hayabusa::prelude::__PdaByteSliceMeta::new(
                &[
                    Counter::SEED,
                    authority.address().as_ref(),
                    &[Counter::cast(counter_view)?.bump],
                ],
                None,
            ),
        )?;
        let __cast_counter = unsafe { counter.cast_untracked() };
        if ::hayabusa::prelude::hint::unlikely(
            authority.address() != &__cast_counter.authority,
        ) {
            return Err(::hayabusa::prelude::ErrorCode::InvalidAccount.into());
        }
        Ok(Self { authority, counter })
    }
}
#[repr(C)]
#[bytemuck(crate = "::hayabusa::prelude::bytemuck")]
pub struct Counter {
    pub count: u64,
    pub authority: Address,
    pub bump: u8,
    _padding: [u8; 7],
}
const _: () = {
    if !(::core::mem::size_of::<Counter>()
        == (::core::mem::size_of::<u64>() + ::core::mem::size_of::<Address>()
            + ::core::mem::size_of::<u8>() + ::core::mem::size_of::<[u8; 7]>()))
    {
        ::core::panicking::panic("derive(Pod) was applied to a type with padding")
    }
};
const _: fn() = || {
    #[allow(clippy::missing_const_for_fn)]
    #[doc(hidden)]
    fn check() {
        fn assert_impl<T: ::hayabusa::prelude::bytemuck::Pod>() {}
        assert_impl::<u64>();
    }
};
const _: fn() = || {
    #[allow(clippy::missing_const_for_fn)]
    #[doc(hidden)]
    fn check() {
        fn assert_impl<T: ::hayabusa::prelude::bytemuck::Pod>() {}
        assert_impl::<Address>();
    }
};
const _: fn() = || {
    #[allow(clippy::missing_const_for_fn)]
    #[doc(hidden)]
    fn check() {
        fn assert_impl<T: ::hayabusa::prelude::bytemuck::Pod>() {}
        assert_impl::<u8>();
    }
};
const _: fn() = || {
    #[allow(clippy::missing_const_for_fn)]
    #[doc(hidden)]
    fn check() {
        fn assert_impl<T: ::hayabusa::prelude::bytemuck::Pod>() {}
        assert_impl::<[u8; 7]>();
    }
};
unsafe impl ::hayabusa::prelude::bytemuck::Pod for Counter {}
const _: fn() = || {
    #[allow(clippy::missing_const_for_fn)]
    #[doc(hidden)]
    fn check() {
        fn assert_impl<T: ::hayabusa::prelude::bytemuck::Zeroable>() {}
        assert_impl::<u64>();
    }
};
const _: fn() = || {
    #[allow(clippy::missing_const_for_fn)]
    #[doc(hidden)]
    fn check() {
        fn assert_impl<T: ::hayabusa::prelude::bytemuck::Zeroable>() {}
        assert_impl::<Address>();
    }
};
const _: fn() = || {
    #[allow(clippy::missing_const_for_fn)]
    #[doc(hidden)]
    fn check() {
        fn assert_impl<T: ::hayabusa::prelude::bytemuck::Zeroable>() {}
        assert_impl::<u8>();
    }
};
const _: fn() = || {
    #[allow(clippy::missing_const_for_fn)]
    #[doc(hidden)]
    fn check() {
        fn assert_impl<T: ::hayabusa::prelude::bytemuck::Zeroable>() {}
        assert_impl::<[u8; 7]>();
    }
};
unsafe impl ::hayabusa::prelude::bytemuck::Zeroable for Counter {}
#[automatically_derived]
impl ::hayabusa::prelude::Discriminator for Counter {
    const DISCRIMINATOR: &[u8] = &[255u8, 176u8, 4u8, 245u8, 188u8, 253u8, 124u8, 25u8];
}
#[automatically_derived]
impl ::core::marker::Copy for Counter {}
#[automatically_derived]
#[doc(hidden)]
unsafe impl ::core::clone::TrivialClone for Counter {}
#[automatically_derived]
impl ::core::clone::Clone for Counter {
    #[inline]
    fn clone(&self) -> Counter {
        let _: ::core::clone::AssertParamIsClone<u64>;
        let _: ::core::clone::AssertParamIsClone<Address>;
        let _: ::core::clone::AssertParamIsClone<u8>;
        let _: ::core::clone::AssertParamIsClone<[u8; 7]>;
        *self
    }
}
impl Counter {
    pub const SEED: &[u8] = "counter".as_bytes();
}
#[automatically_derived]
impl Owner for Counter {
    const OWNER: &'static Address = crate::ID;
}
#[automatically_derived]
unsafe impl ::hayabusa::prelude::__AccountDiscriminatorMode for Counter {
    type Mode = __WithDiscriminator;
}
#[automatically_derived]
unsafe impl __AccountMarker for Counter {}
#[automatically_derived]
unsafe impl ::hayabusa::prelude::Ownership for Counter {
    type OwnerType = ::hayabusa::prelude::__Owner;
    #[inline(always)]
    fn check_ownership(view: AccountView<'_>) -> Result<()> {
        Self::check_owner(view)
    }
}
#[automatically_derived]
impl<'view> ::hayabusa::prelude::AccountInit<'view> for Counter {
    type Meta<'a> = ::hayabusa::prelude::__AccountInitMeta<'view, 'a> where 'view: 'a;
    type PdaMeta<'a, 'b, 'c> = ::hayabusa::prelude::__AccountPdaInitMeta<
        'view,
        'a,
        'b,
        'c,
    >
    where
        'view: 'c,
        'c: 'b,
        'b: 'a;
    #[inline(never)]
    fn init<'a>(view: AccountView<'view>, meta: &Self::Meta<'a>) -> Result<()>
    where
        'view: 'a,
    {
        let _: ::hayabusa::prelude::Signer<'view> = <::hayabusa::prelude::Signer as ::hayabusa::prelude::ParseAccount<
            'view,
        >>::parse(view, &mut ::hayabusa::prelude::NoMeta)?;
        ::hayabusa::prelude::create_or_allocate_account(
            view,
            meta.payer.to_account_view(),
            crate::ID,
            Self::SPACE,
        )?;
        let mut borrow = view.try_borrow_mut()?;
        let disc_bytes = &mut borrow[..Self::DISCRIMINATOR.len()];
        disc_bytes.copy_from_slice(Self::DISCRIMINATOR);
        Ok(())
    }
    #[inline(never)]
    fn init_pda<'a, 'b, 'c>(
        view: AccountView<'view>,
        meta: &mut Self::PdaMeta<'a, 'b, 'c>,
    ) -> Result<()>
    where
        'view: 'c,
        'c: 'b,
        'b: 'a,
    {
        match &mut meta.signer {
            ::hayabusa::prelude::SignerBumpness::With(signer) => {
                ::hayabusa::prelude::create_or_allocate_pda(
                    meta.payer.to_account_view(),
                    view,
                    crate::ID,
                    &[*signer],
                    Self::SPACE,
                )?;
            }
            ::hayabusa::prelude::SignerBumpness::Without(signer, bump_slot) => {
                let (_, bump) = ::hayabusa::prelude::try_find_program_address(
                    signer.as_slice_of_slices(),
                    crate::ID,
                )?;
                let slot: &'c mut u8 = bump_slot
                    .take()
                    .ok_or(ErrorCode::MetaAlreadyConsumed)?;
                *slot = bump;
                let bump_ref: &'c u8 = slot;
                signer.push(Seed::from(core::slice::from_ref(bump_ref)))?;
                ::hayabusa::prelude::create_or_allocate_pda(
                    meta.payer.to_account_view(),
                    view,
                    crate::ID,
                    &[signer.as_signer()],
                    Self::SPACE,
                )?;
            }
        }
        let mut borrow = view.try_borrow_mut()?;
        let disc_bytes = &mut borrow[..Self::DISCRIMINATOR.len()];
        disc_bytes.copy_from_slice(Self::DISCRIMINATOR);
        Ok(())
    }
}
