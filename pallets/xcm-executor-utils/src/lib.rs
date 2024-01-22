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

//! # Authorities Noting Pallet
//!
//! This pallet notes the authorities assigned to this container-chain in an orchestrator chain
//!
//! First the pallet receives a storage proof of the header of the orchestrator chain
//! Once the storage proof is verified against the relay, the storage root of the orchestrator
//! chain is retrieved from the header
//!  
//! A second storage proof is verified against the storage root of the orchestrator chain. From
//! this the collator-assignation is read, and the authorities assigned to these container-chain
//! are retrieved and stored

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(any(test, feature = "runtime-benchmarks"))]
mod benchmarks;

pub mod filters;

pub use pallet::*;

use {
    frame_support::pallet_prelude::*,
    frame_system::pallet_prelude::*,
    serde::{Deserialize, Serialize},
    staging_xcm::latest::{AssetId, MultiLocation},
};

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use sp_runtime::BoundedVec;
    use sp_std::vec::Vec;

    // Default filtering policies for incoming assets
    #[derive(
        PartialEq,
        Eq,
        Clone,
        Encode,
        Decode,
        RuntimeDebug,
        TypeInfo,
        MaxEncodedLen,
        Serialize,
        Deserialize,
    )]
    pub enum DefaultFilterPolicy {
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
        TypeInfo,
        MaxEncodedLen,
        Serialize,
        Deserialize,
    )]
    #[scale_info(skip_type_params(MaxAssets))]
    #[serde(bound = "")]
    pub enum FilterPolicy<MaxAssets: Get<u32>> {
        DefaultFilterPolicy(DefaultFilterPolicy),
        AllowedAssets(BoundedVec<AssetId, MaxAssets>),
    }

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        // Maximum number of allowed assets per origin on AllowedAssets policies
        type FilterPolicyMaxAssets: Get<u32>;

        type ReserveDefaultFilterPolicy: Get<DefaultFilterPolicy>;

        type SetReserveFilterOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        type TeleportDefaultFilterPolicy: Get<DefaultFilterPolicy>;

        type SetTeleportFilterOrigin: EnsureOrigin<Self::RuntimeOrigin>;
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
        MultiLocation,
        FilterPolicy<T::FilterPolicyMaxAssets>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn teleport_policy)]
    pub(super) type TeleportPolicy<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        MultiLocation,
        FilterPolicy<T::FilterPolicyMaxAssets>,
        OptionQuery,
    >;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub reserve_policies: Vec<(MultiLocation, FilterPolicy<T::FilterPolicyMaxAssets>)>,
        pub teleport_policies: Vec<(MultiLocation, FilterPolicy<T::FilterPolicyMaxAssets>)>,
        pub _config: PhantomData<T>,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                reserve_policies: Default::default(),
                teleport_policies: Default::default(),
                _config: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            assert!(
                self.reserve_policies.len() < T::FilterPolicyMaxAssets::get() as usize,
                "Reserve policies should be less than FilterPolicyMaxAssets"
            );

            assert!(
                self.teleport_policies.len() < T::FilterPolicyMaxAssets::get() as usize,
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
        ReservePolicySet { origin: MultiLocation },
        ReservePolicyRemoved { origin: MultiLocation },
        TeleportPolicySet { origin: MultiLocation },
        TeleportPolicyRemoved { origin: MultiLocation },
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(0)]
        pub fn set_reserve_policy(
            origin: OriginFor<T>,
            origin_multilocation: MultiLocation,
            policy: FilterPolicy<T::FilterPolicyMaxAssets>,
        ) -> DispatchResult {
            T::SetReserveFilterOrigin::ensure_origin(origin)?;

            ReservePolicy::<T>::insert(origin_multilocation, policy);

            Self::deposit_event(Event::ReservePolicySet {
                origin: origin_multilocation,
            });

            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(0)]
        pub fn remove_reserve_policy(
            origin: OriginFor<T>,
            origin_multilocation: MultiLocation,
        ) -> DispatchResult {
            T::SetReserveFilterOrigin::ensure_origin(origin)?;

            ReservePolicy::<T>::take(origin_multilocation).ok_or(Error::<T>::NotValidOrigin)?;

            Self::deposit_event(Event::ReservePolicyRemoved {
                origin: origin_multilocation,
            });

            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(0)]
        pub fn set_teleport_policy(
            origin: OriginFor<T>,
            origin_multilocation: MultiLocation,
            policy: FilterPolicy<T::FilterPolicyMaxAssets>,
        ) -> DispatchResult {
            T::SetTeleportFilterOrigin::ensure_origin(origin)?;

            TeleportPolicy::<T>::insert(origin_multilocation, policy);

            Self::deposit_event(Event::TeleportPolicySet {
                origin: origin_multilocation,
            });

            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(0)]
        pub fn remove_teleport_policy(
            origin: OriginFor<T>,
            origin_multilocation: MultiLocation,
        ) -> DispatchResult {
            T::SetTeleportFilterOrigin::ensure_origin(origin)?;

            TeleportPolicy::<T>::take(origin_multilocation).ok_or(Error::<T>::NotValidOrigin)?;

            Self::deposit_event(Event::TeleportPolicyRemoved {
                origin: origin_multilocation,
            });

            Ok(())
        }
    }
}
