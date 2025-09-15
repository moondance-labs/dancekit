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
// along with Tanssi.  If not, see <http://www.gnu.org/licenses/>.

//! # XCM Executor Utils Pallet
//!
//! This is a utility pallet to help set the runtime parameters of XcmExecutor.
//! Currently it offers an intuitive, on-chain maanger to set trust policies on
//! incoming assets though `IsReserveFilter` and `IsTeleporterFilter`.

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;
pub mod weights;
pub use weights::WeightInfo;

#[cfg(any(test, feature = "runtime-benchmarks"))]
mod benchmarks;

pub mod filters;

pub mod migrations;

pub use pallet::*;

use {
    frame_support::{pallet_prelude::*, DefaultNoBound},
    frame_system::pallet_prelude::*,
    serde::{Deserialize, Serialize},
    staging_xcm::latest::{AssetId, Location},
};

#[frame_support::pallet]
pub mod pallet {
    use {super::*, alloc::vec::Vec, sp_runtime::BoundedVec};

    // Default trust policies for incoming assets
    #[derive(
        PartialEq,
        Eq,
        Clone,
        Encode,
        Decode,
        DecodeWithMemTracking,
        RuntimeDebug,
        TypeInfo,
        MaxEncodedLen,
        Deserialize,
        Serialize,
    )]
    pub enum DefaultTrustPolicy {
        // Allow all incoming assets
        All,
        // Only allow assets native of the origin
        AllNative,
        // Do not allow any assets
        Never,
    }

    #[derive(
        DebugNoBound,
        CloneNoBound,
        EqNoBound,
        PartialEqNoBound,
        Encode,
        Decode,
        DecodeWithMemTracking,
        TypeInfo,
        MaxEncodedLen,
        Deserialize,
        Serialize,
    )]
    #[serde(bound = "")]
    #[scale_info(skip_type_params(MaxAssets))]
    pub enum TrustPolicy<MaxAssets: Get<u32>> {
        DefaultTrustPolicy(DefaultTrustPolicy),
        AllowedAssets(BoundedVec<AssetId, MaxAssets>),
    }

    #[pallet::pallet]
    #[pallet::storage_version(migrations::STORAGE_VERSION)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        // Maximum number of allowed assets per origin on AllowedAssets policies
        type TrustPolicyMaxAssets: Get<u32>;

        type ReserveDefaultTrustPolicy: Get<DefaultTrustPolicy>;

        type SetReserveTrustOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        type TeleportDefaultTrustPolicy: Get<DefaultTrustPolicy>;

        type SetTeleportTrustOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::error]
    pub enum Error<T> {
        NotValidOrigin,
    }

    #[pallet::storage]
    #[pallet::getter(fn reserve_policy)]
    pub(super) type ReservePolicy<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        Location,
        TrustPolicy<T::TrustPolicyMaxAssets>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn teleport_policy)]
    pub(super) type TeleportPolicy<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        Location,
        TrustPolicy<T::TrustPolicyMaxAssets>,
        OptionQuery,
    >;

    #[pallet::genesis_config]
    #[derive(DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        pub reserve_policies: Vec<(Location, TrustPolicy<T::TrustPolicyMaxAssets>)>,
        pub teleport_policies: Vec<(Location, TrustPolicy<T::TrustPolicyMaxAssets>)>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            assert!(
                self.reserve_policies.len() < T::TrustPolicyMaxAssets::get() as usize,
                "Reserve policies should be less than FilterPolicyMaxAssets"
            );

            assert!(
                self.teleport_policies.len() < T::TrustPolicyMaxAssets::get() as usize,
                "Teleport policies should be less than FilterPolicyMaxAssets"
            );

            for (origin, policy) in self.reserve_policies.iter() {
                ReservePolicy::<T>::insert(origin, policy);
            }

            for (origin, policy) in self.teleport_policies.iter() {
                TeleportPolicy::<T>::insert(origin, policy);
            }
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ReservePolicySet { origin: Location },
        ReservePolicyRemoved { origin: Location },
        TeleportPolicySet { origin: Location },
        TeleportPolicyRemoved { origin: Location },
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::set_reserve_policy())]
        pub fn set_reserve_policy(
            origin: OriginFor<T>,
            origin_location: Location,
            policy: TrustPolicy<T::TrustPolicyMaxAssets>,
        ) -> DispatchResult {
            T::SetReserveTrustOrigin::ensure_origin(origin)?;

            ReservePolicy::<T>::insert(&origin_location, policy);

            Self::deposit_event(Event::ReservePolicySet {
                origin: origin_location,
            });

            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::remove_reserve_policy())]
        pub fn remove_reserve_policy(
            origin: OriginFor<T>,
            origin_location: Location,
        ) -> DispatchResult {
            T::SetReserveTrustOrigin::ensure_origin(origin)?;

            ReservePolicy::<T>::take(&origin_location).ok_or(Error::<T>::NotValidOrigin)?;

            Self::deposit_event(Event::ReservePolicyRemoved {
                origin: origin_location,
            });

            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::set_teleport_policy())]
        pub fn set_teleport_policy(
            origin: OriginFor<T>,
            origin_location: Location,
            policy: TrustPolicy<T::TrustPolicyMaxAssets>,
        ) -> DispatchResult {
            T::SetTeleportTrustOrigin::ensure_origin(origin)?;

            TeleportPolicy::<T>::insert(&origin_location, policy);

            Self::deposit_event(Event::TeleportPolicySet {
                origin: origin_location,
            });

            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::remove_teleport_policy())]
        pub fn remove_teleport_policy(
            origin: OriginFor<T>,
            origin_location: Location,
        ) -> DispatchResult {
            T::SetTeleportTrustOrigin::ensure_origin(origin)?;

            TeleportPolicy::<T>::take(&origin_location).ok_or(Error::<T>::NotValidOrigin)?;

            Self::deposit_event(Event::TeleportPolicyRemoved {
                origin: origin_location,
            });

            Ok(())
        }
    }
}
