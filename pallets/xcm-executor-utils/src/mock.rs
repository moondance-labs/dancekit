// Copyright (C) Moondance Labs Ltd.
// This file is part of Tanssi.

// Tanssi is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Tanssi is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Tanssi.  If not, see <http://www.gnu.org/licenses/>

use {
    crate::{self as pallet_xcm_executor_utils, DefaultTrustPolicy},
    frame_support::{
        construct_runtime, parameter_types,
        traits::{ConstU16, ConstU64},
    },
    frame_system::{self as system, EnsureRoot},
    sp_core::H256,
    sp_runtime::{
        traits::{BlakeTwo256, IdentityLookup},
        BuildStorage,
    },
};

pub mod mock_never {
    use super::*;

    type Block = frame_system::mocking::MockBlock<TestNever>;

    // Configure a mock runtime to test the pallet.
    construct_runtime!(
        pub enum TestNever
        {
            System: frame_system,
            XcmExecutorUtils: pallet_xcm_executor_utils,
        }
    );

    impl system::Config for TestNever {
        type BaseCallFilter = frame_support::traits::Everything;
        type BlockWeights = ();
        type BlockLength = ();
        type DbWeight = ();
        type RuntimeOrigin = RuntimeOrigin;
        type RuntimeCall = RuntimeCall;
        type Nonce = u64;
        type Block = Block;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type RuntimeEvent = RuntimeEvent;
        type BlockHashCount = ConstU64<250>;
        type Version = ();
        type PalletInfo = PalletInfo;
        type AccountData = ();
        type OnNewAccount = ();
        type OnKilledAccount = ();
        type SystemWeightInfo = ();
        type SS58Prefix = ConstU16<42>;
        type OnSetCode = ();
        type MaxConsumers = frame_support::traits::ConstU32<16>;
        type RuntimeTask = ();
        type SingleBlockMigrations = ();
        type MultiBlockMigrator = ();
        type PreInherents = ();
        type PostInherents = ();
        type PostTransactions = ();
        type ExtensionsWeightInfo = ();
    }

    parameter_types! {
        pub const MaxAssetsMock: u32 = 100u32;
        pub const DefaultPolicyNever: DefaultTrustPolicy = DefaultTrustPolicy::Never;
    }

    impl pallet_xcm_executor_utils::Config for TestNever {
        type RuntimeEvent = RuntimeEvent;
        type SetReserveTrustOrigin = EnsureRoot<u64>;
        type SetTeleportTrustOrigin = EnsureRoot<u64>;
        type ReserveDefaultTrustPolicy = DefaultPolicyNever;
        type TeleportDefaultTrustPolicy = DefaultPolicyNever;
        type TrustPolicyMaxAssets = MaxAssetsMock;
        type WeightInfo = ();
    }

    // Build genesis storage according to the mock runtime.
    pub fn new_test_ext() -> sp_io::TestExternalities {
        system::GenesisConfig::<TestNever>::default()
            .build_storage()
            .unwrap()
            .into()
    }
}

pub mod mock_all {
    use super::*;

    type Block = frame_system::mocking::MockBlock<TestAll>;

    // Configure a mock runtime to test the pallet.
    construct_runtime!(
        pub enum TestAll
        {
            System: frame_system,
            XcmExecutorUtils: pallet_xcm_executor_utils,
        }
    );

    impl system::Config for TestAll {
        type BaseCallFilter = frame_support::traits::Everything;
        type BlockWeights = ();
        type BlockLength = ();
        type DbWeight = ();
        type RuntimeOrigin = RuntimeOrigin;
        type RuntimeCall = RuntimeCall;
        type Nonce = u64;
        type Block = Block;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type RuntimeEvent = RuntimeEvent;
        type BlockHashCount = ConstU64<250>;
        type Version = ();
        type PalletInfo = PalletInfo;
        type AccountData = ();
        type OnNewAccount = ();
        type OnKilledAccount = ();
        type SystemWeightInfo = ();
        type SS58Prefix = ConstU16<42>;
        type OnSetCode = ();
        type MaxConsumers = frame_support::traits::ConstU32<16>;
        type RuntimeTask = ();
        type SingleBlockMigrations = ();
        type MultiBlockMigrator = ();
        type PreInherents = ();
        type PostInherents = ();
        type PostTransactions = ();
        type ExtensionsWeightInfo = ();
    }

    parameter_types! {
        pub const MaxAssetsMock: u32 = 100u32;
        pub const DefaultPolicyAll: DefaultTrustPolicy = DefaultTrustPolicy::All;
    }

    impl pallet_xcm_executor_utils::Config for TestAll {
        type RuntimeEvent = RuntimeEvent;
        type SetReserveTrustOrigin = EnsureRoot<u64>;
        type SetTeleportTrustOrigin = EnsureRoot<u64>;
        type ReserveDefaultTrustPolicy = DefaultPolicyAll;
        type TeleportDefaultTrustPolicy = DefaultPolicyAll;
        type TrustPolicyMaxAssets = MaxAssetsMock;
        type WeightInfo = ();
    }
}

pub mod mock_all_native {
    use super::*;

    type Block = frame_system::mocking::MockBlock<TestAllNative>;

    // Configure a mock runtime to test the pallet.
    construct_runtime!(
        pub enum TestAllNative
        {
            System: frame_system,
            XcmExecutorUtils: pallet_xcm_executor_utils,
        }
    );

    impl system::Config for TestAllNative {
        type BaseCallFilter = frame_support::traits::Everything;
        type BlockWeights = ();
        type BlockLength = ();
        type DbWeight = ();
        type RuntimeOrigin = RuntimeOrigin;
        type RuntimeCall = RuntimeCall;
        type Nonce = u64;
        type Block = Block;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type RuntimeEvent = RuntimeEvent;
        type BlockHashCount = ConstU64<250>;
        type Version = ();
        type PalletInfo = PalletInfo;
        type AccountData = ();
        type OnNewAccount = ();
        type OnKilledAccount = ();
        type SystemWeightInfo = ();
        type SS58Prefix = ConstU16<42>;
        type OnSetCode = ();
        type MaxConsumers = frame_support::traits::ConstU32<16>;
        type RuntimeTask = ();
        type SingleBlockMigrations = ();
        type MultiBlockMigrator = ();
        type PreInherents = ();
        type PostInherents = ();
        type PostTransactions = ();
        type ExtensionsWeightInfo = ();
    }

    parameter_types! {
        pub const MaxAssetsMock: u32 = 100u32;
        pub const DefaultPolicyAllNative: DefaultTrustPolicy = DefaultTrustPolicy::AllNative;
    }

    impl pallet_xcm_executor_utils::Config for TestAllNative {
        type RuntimeEvent = RuntimeEvent;
        type SetReserveTrustOrigin = EnsureRoot<u64>;
        type SetTeleportTrustOrigin = EnsureRoot<u64>;
        type ReserveDefaultTrustPolicy = DefaultPolicyAllNative;
        type TeleportDefaultTrustPolicy = DefaultPolicyAllNative;
        type TrustPolicyMaxAssets = MaxAssetsMock;
        type WeightInfo = ();
    }
}
