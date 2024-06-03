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

use super::*;
use frame_support::migration::storage_key_iter;
use frame_support::traits::OnRuntimeUpgrade;
use sp_std::vec::Vec;
/// The in-code storage version.
pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

pub mod v1 {
    use frame_support::pallet_prelude::*;
    use sp_std::vec::Vec;
    use staging_xcm::latest::AssetId;
    use staging_xcm::v3::AssetId as OldAssetId;

    use crate::TrustPolicy;

    #[derive(
        DebugNoBound,
        CloneNoBound,
        EqNoBound,
        PartialEqNoBound,
        Encode,
        Decode,
        TypeInfo,
        MaxEncodedLen,
    )]
    #[scale_info(skip_type_params(MaxAssets))]
    pub enum OldTrustPolicy<MaxAssets: Get<u32>> {
        DefaultTrustPolicy(crate::DefaultTrustPolicy),
        AllowedAssets(BoundedVec<OldAssetId, MaxAssets>),
    }

    impl<MaxAssets: Get<u32>> Into<TrustPolicy<MaxAssets>> for OldTrustPolicy<MaxAssets> {
        fn into(self) -> TrustPolicy<MaxAssets> {
            match self {
                OldTrustPolicy::DefaultTrustPolicy(default_policy) => {
                    TrustPolicy::DefaultTrustPolicy(default_policy)
                }
                OldTrustPolicy::AllowedAssets(old_assets) => {
                    let new_assets: Vec<AssetId> = old_assets
                        .iter()
                        .filter_map(|old_asset| AssetId::try_from(old_asset.clone()).ok())
                        .collect();
                    TrustPolicy::AllowedAssets(
                        new_assets
                            .try_into()
                            .expect("we did not change the length so this should be convertible"),
                    )
                }
            }
        }
    }
}

pub struct MigrateToV1<T>(pub core::marker::PhantomData<T>);
use frame_support::StoragePrefixedMap;
impl<T: Config> OnRuntimeUpgrade for MigrateToV1<T> {
    fn on_runtime_upgrade() -> Weight {
        use frame_support::pallet_prelude::*;

        use staging_xcm::v3::MultiLocation as OldLocation;

        let in_code_version = Pallet::<T>::in_code_storage_version();
        let on_chain_version = Pallet::<T>::on_chain_storage_version();
        if on_chain_version == 0 && in_code_version == 1 {
            let pallet_prefix = ReservePolicy::<T>::pallet_prefix();
            let reserve_policy_storage_prefix = ReservePolicy::<T>::storage_prefix();
            let teleport_policy_storage_prefix = TeleportPolicy::<T>::storage_prefix();

            // Migrate reserve policy
            // Read all the data into memory.
            // https://crates.parity.io/frame_support/storage/migration/fn.storage_key_iter.html
            let reserve_policy_stored_data: Vec<_> =
                storage_key_iter::<
                    OldLocation,
                    v1::OldTrustPolicy<T::TrustPolicyMaxAssets>,
                    Blake2_128Concat,
                >(pallet_prefix, reserve_policy_storage_prefix)
                .drain()
                .collect();

            let teleport_policy_stored_data: Vec<_> =
                storage_key_iter::<
                    OldLocation,
                    v1::OldTrustPolicy<T::TrustPolicyMaxAssets>,
                    Blake2_128Concat,
                >(pallet_prefix, teleport_policy_storage_prefix)
                .drain()
                .collect();

            let migrated_count = reserve_policy_stored_data
                .len()
                .saturating_add(teleport_policy_stored_data.len());

            log::info!(target: LOG_TARGET, "Migrating {:?} elements", migrated_count);
            // Write to the new storage with removed and added fields
            for (old_location, old_policy) in reserve_policy_stored_data {
                if let Ok(new_location) = Location::try_from(old_location) {
                    let new_policy: TrustPolicy<T::TrustPolicyMaxAssets> = old_policy.into();
                    ReservePolicy::<T>::insert(new_location, new_policy);
                } else {
                    log::warn!(target: LOG_TARGET, "Location could not be converted safely to xcmV4")
                }
            }

            for (old_location, old_policy) in teleport_policy_stored_data {
                if let Ok(new_location) = Location::try_from(old_location) {
                    let new_policy: TrustPolicy<T::TrustPolicyMaxAssets> = old_policy.into();
                    TeleportPolicy::<T>::insert(new_location, new_policy);
                } else {
                    log::warn!(target: LOG_TARGET, "Location could not be converted safely to xcmV4")
                }
            }

            in_code_version.put::<Pallet<T>>();
            // One db read and one db write per element, plus the on-chain storage
            T::DbWeight::get()
                .reads(migrated_count as u64)
                .saturating_add(T::DbWeight::get().writes(migrated_count as u64 + 3u64))
        } else {
            log::info!(
                target: LOG_TARGET,
                "Migration did not execute. This probably should be removed"
            );
            T::DbWeight::get().reads(3)
        }
    }
}

impl<T: Config> pallet_migrations::Migration for MigrateToV1<T> {
    fn friendly_name(&self) -> &str {
        "TM_XcmExecutorUtilsMigrateToV1"
    }

    fn migrate(&self, _available_weight: Weight) -> Weight {
        Self::on_runtime_upgrade()
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade(&self) -> Result<Vec<u8>, sp_runtime::DispatchError> {
        <Self as OnRuntimeUpgrade>::pre_upgrade()
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(&self, state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
        <Self as OnRuntimeUpgrade>::post_upgrade(state)
    }
}
