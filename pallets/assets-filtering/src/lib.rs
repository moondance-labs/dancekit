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

pub mod reserves;

pub use pallet::*;

use {
    frame_support::pallet_prelude::*,
    frame_system::pallet_prelude::*,
    staging_xcm::latest::{AssetId, MultiLocation},
};

#[frame_support::pallet(dev_mode)]
pub mod pallet {
    use super::*;
    use sp_runtime::BoundedVec;
    use sp_std::vec::Vec;

    // Default filtering policy for all assets
    #[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum DefaultPolicy {
        // All assets (DANGEROUS!)
        All,
        // Only native assets
        AllNative,
        // Do not allow any assets
        Never,
    }

    #[derive(Clone, Eq, PartialEq, Encode, Decode, Debug, TypeInfo, MaxEncodedLen)]
    pub enum Policy {
        DefaultPolicy(DefaultPolicy),
        AllowedAssets(BoundedVec<AssetId, ConstU32<100>>),
    }

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        // Maximum number of allowed assets per origin
        type MaxAssets: Get<u32>;

        // Default policy
        type DefaultPolicy: Get<DefaultPolicy>;

        type SetReservesOrigin: EnsureOrigin<Self::RuntimeOrigin>;
        type SetTeleportsOrigin: EnsureOrigin<Self::RuntimeOrigin>;
    }

    #[pallet::error]
    pub enum Error<T> {
        NotValidOrigin,
    }

    #[pallet::storage]
    #[pallet::getter(fn origin_policy)]
    pub(super) type OriginPolicy<T: Config> =
        StorageMap<_, Blake2_128Concat, MultiLocation, Policy, OptionQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub reserve_policies: Vec<(MultiLocation, Policy)>,
        pub _config: PhantomData<T>,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                reserve_policies: Default::default(),
                _config: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            assert!(
                self.reserve_policies.len() < T::MaxAssets::get() as usize,
                "Valid origins should be less than the maximum"
            );

            for (origin, policy) in self.reserve_policies {
                OriginPolicy::<T>::insert(origin, policy);
            }
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        OriginPolicySet { origin: MultiLocation },
        OriginPolicyRemoved { origin: MultiLocation },
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(0)]
        pub fn set_rule(
            origin: OriginFor<T>,
            origin_multilocation: MultiLocation,
            policy: Policy,
        ) -> DispatchResult {
            ensure_root(origin)?;

            OriginPolicy::<T>::insert(origin_multilocation, policy);

            Self::deposit_event(Event::OriginPolicySet {
                origin: origin_multilocation,
            });

            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(0)]
        pub fn remove_rule(
            origin: OriginFor<T>,
            origin_multilocation: MultiLocation,
        ) -> DispatchResult {
            ensure_root(origin)?;

            OriginPolicy::<T>::take(origin_multilocation).ok_or(Error::<T>::NotValidOrigin)?;

            Self::deposit_event(Event::OriginPolicyRemoved {
                origin: origin_multilocation,
            });

            Ok(())
        }
    }
}
