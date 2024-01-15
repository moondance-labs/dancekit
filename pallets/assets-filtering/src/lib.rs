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

pub use pallet::*;

use {
    frame_support::pallet_prelude::*, frame_system::pallet_prelude::*,
    staging_xcm::v3::MultiLocation,
};

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use sp_runtime::BoundedVec;
    use sp_std::vec::Vec;

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        // Maximum number of allowed origins
        type MaxOrigins: Get<u32>;
    }

    #[pallet::error]
    pub enum Error<T> {
        OriginAlreadyAdded,
        NotValidOrigin,
        TooManyOrigins,
    }

    #[pallet::storage]
    #[pallet::getter(fn valid_origins)]
    pub(super) type ValidOrigins<T: Config> =
        StorageValue<_, BoundedVec<MultiLocation, T::MaxOrigins>, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub valid_origins: Vec<MultiLocation>,
        pub _config: PhantomData<T>,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                valid_origins: Default::default(),
                _config: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            assert!(
                self.valid_origins.len() < T::MaxOrigins::get() as usize,
                "Valid origins should be less than the maximum"
            );

            <ValidOrigins<T>>::put(BoundedVec::try_from(self.valid_origins.clone()).unwrap())
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        OriginAdded { origin: MultiLocation },
        OriginRemoved { origin: MultiLocation },
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(0)]
        pub fn add_valid_origin(
            origin: OriginFor<T>,
            multilocation: MultiLocation,
        ) -> DispatchResult {
            ensure_root(origin)?;

            ValidOrigins::<T>::try_mutate(|valid_origins| -> DispatchResult {
                if valid_origins.contains(&multilocation) {
                    Err(Error::<T>::OriginAlreadyAdded)?;
                }

                valid_origins
                    .try_push(multilocation.clone())
                    .map_err(|_| Error::<T>::TooManyOrigins)?;
                Ok(())
            })?;

            Self::deposit_event(Event::OriginAdded {
                origin: multilocation,
            });

            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(0)]
        pub fn remove_valid_origin(
            origin: OriginFor<T>,
            multilocation: MultiLocation,
        ) -> DispatchResult {
            ensure_root(origin)?;

            ValidOrigins::<T>::try_mutate(|valid_origins| -> DispatchResult {
                let pos = valid_origins
                    .iter()
                    .position(|x| x == &multilocation)
                    .ok_or(Error::<T>::NotValidOrigin)?;
                valid_origins.remove(pos);
                Ok(())
            })?;

            Self::deposit_event(Event::OriginRemoved {
                origin: multilocation,
            });

            Ok(())
        }
    }
}
