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

use {
    crate::{Config, DefaultTrustPolicy, TrustPolicy},
    frame_support::{pallet_prelude::*, traits::ContainsPair},
    staging_xcm::v3::{
        AssetId,
        Junction::Parachain,
        Junctions::{Here, X1},
        MultiAsset, MultiLocation,
    },
};

fn apply_policy<T: Config>(
    asset: &MultiAsset,
    origin: &MultiLocation,
    maybe_origin_policy: Option<TrustPolicy<T::TrustPolicyMaxAssets>>,
    default_policy: DefaultTrustPolicy,
) -> bool {
    if let Some(origin_policy) = maybe_origin_policy {
        match origin_policy {
            TrustPolicy::AllowedAssets(allowed_assets) => allowed_assets.contains(&asset.id),
            TrustPolicy::DefaultTrustPolicy(origin_default_policy) => match origin_default_policy
            {
                DefaultTrustPolicy::All => true,
                DefaultTrustPolicy::AllNative => NativeAssetReserve::contains(asset, origin),
                DefaultTrustPolicy::Never => false,
            },
        }
    } else {
        match default_policy {
            DefaultTrustPolicy::All => true,
            DefaultTrustPolicy::AllNative => NativeAssetReserve::contains(asset, origin),
            DefaultTrustPolicy::Never => false,
        }
    }
}

pub struct IsReserveFilter<T>(PhantomData<T>);
impl<T> ContainsPair<MultiAsset, MultiLocation> for IsReserveFilter<T>
where
    T: Config,
{
    fn contains(asset: &MultiAsset, origin: &MultiLocation) -> bool {
        let maybe_origin_policy = crate::Pallet::<T>::reserve_policy(origin);
        let default_policy = <T as crate::Config>::ReserveDefaultTrustPolicy::get();

        apply_policy::<T>(asset, origin, maybe_origin_policy, default_policy)
    }
}

pub struct IsTeleportFilter<T>(PhantomData<T>);
impl<T> ContainsPair<MultiAsset, MultiLocation> for IsTeleportFilter<T>
where
    T: Config,
{
    fn contains(asset: &MultiAsset, origin: &MultiLocation) -> bool {
        let maybe_origin_policy = crate::Pallet::<T>::teleport_policy(origin);
        let default_policy = <T as crate::Config>::TeleportDefaultTrustPolicy::get();

        apply_policy::<T>(asset, origin, maybe_origin_policy, default_policy)
    }
}

// TODO: this should probably move to somewhere in the polkadot-sdk repo
pub struct NativeAssetReserve;
impl ContainsPair<MultiAsset, MultiLocation> for NativeAssetReserve {
    fn contains(asset: &MultiAsset, origin: &MultiLocation) -> bool {
        let reserve = if let AssetId::Concrete(location) = &asset.id {
            if location.parents == 0 && !matches!(location.first_interior(), Some(Parachain(_))) {
                Some(MultiLocation::here())
            } else {
                location.chain_part()
            }
        } else {
            None
        };

        if let Some(ref reserve) = reserve {
            if reserve == origin {
                return true;
            }
        }
        false
    }
}

pub trait Parse {
    /// Returns the "chain" location part. It could be parent, sibling
    /// parachain, or child parachain.
    fn chain_part(&self) -> Option<MultiLocation>;
    /// Returns "non-chain" location part.
    fn non_chain_part(&self) -> Option<MultiLocation>;
}

impl Parse for MultiLocation {
    fn chain_part(&self) -> Option<MultiLocation> {
        match (self.parents, self.first_interior()) {
            // sibling parachain
            (1, Some(Parachain(id))) => Some(MultiLocation::new(1, X1(Parachain(*id)))),
            // parent
            (1, _) => Some(MultiLocation::parent()),
            // children parachain
            (0, Some(Parachain(id))) => Some(MultiLocation::new(0, X1(Parachain(*id)))),
            _ => None,
        }
    }

    fn non_chain_part(&self) -> Option<MultiLocation> {
        let mut junctions = *self.interior();
        while matches!(junctions.first(), Some(Parachain(_))) {
            let _ = junctions.take_first();
        }

        if junctions != Here {
            Some(MultiLocation::new(0, junctions))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use {super::*, crate::mock::Test, staging_xcm::latest::Fungibility::Fungible};

    #[test]
    fn policy_all_allows_any() {
        let default_policy = DefaultTrustPolicy::Never;

        let parent_multilocation = MultiLocation::parent();
        let grandparent_asset = MultiAsset {
            id: AssetId::Concrete(MultiLocation::grandparent()),
            fun: Fungible(1_000),
        };

        let origin_policy = Some(TrustPolicy::DefaultTrustPolicy(DefaultTrustPolicy::All));

        assert!(apply_policy::<Test>(
            &grandparent_asset,
            &parent_multilocation,
            origin_policy,
            default_policy
        ));
    }

    #[test]
    fn policy_all_native_allows_native_asset() {
        let default_policy = DefaultTrustPolicy::Never;

        let parent_multilocation = MultiLocation::parent();
        let parent_asset = MultiAsset {
            id: AssetId::Concrete(MultiLocation::parent()),
            fun: Fungible(1_000),
        };

        let origin_policy = Some(TrustPolicy::DefaultTrustPolicy(
            DefaultTrustPolicy::AllNative,
        ));

        assert!(apply_policy::<Test>(
            &parent_asset,
            &parent_multilocation,
            origin_policy,
            default_policy
        ));
    }

    #[test]
    fn policy_all_native_rejects_non_native_asset() {
        let default_policy = DefaultTrustPolicy::Never;

        let parent_multilocation = MultiLocation::parent();
        let grandparent_asset = MultiAsset {
            id: AssetId::Concrete(MultiLocation::grandparent()),
            fun: Fungible(1_000),
        };

        let origin_policy = Some(TrustPolicy::DefaultTrustPolicy(
            DefaultTrustPolicy::AllNative,
        ));

        assert_eq!(
            apply_policy::<Test>(
                &grandparent_asset,
                &parent_multilocation,
                origin_policy,
                default_policy
            ),
            false
        );
    }

    #[test]
    fn policy_custom_allows_allowed_asset() {
        let default_policy = DefaultTrustPolicy::Never;

        let parent_multilocation = MultiLocation::parent();
        let grandparent_asset = MultiAsset {
            id: AssetId::Concrete(MultiLocation::grandparent()),
            fun: Fungible(1_000),
        };

        // Only allow grandparent_asset
        let origin_policy = Some(TrustPolicy::AllowedAssets(
            BoundedVec::try_from(vec![grandparent_asset.id]).unwrap(),
        ));

        assert!(apply_policy::<Test>(
            &grandparent_asset,
            &parent_multilocation,
            origin_policy,
            default_policy
        ));
    }

    #[test]
    fn policy_custom_reject_not_allowed_asset() {
        let default_policy = DefaultTrustPolicy::Never;

        let parent_multilocation = MultiLocation::parent();
        let parent_asset = MultiAsset {
            id: AssetId::Concrete(MultiLocation::parent()),
            fun: Fungible(1_000),
        };
        let grandparent_asset = MultiAsset {
            id: AssetId::Concrete(MultiLocation::grandparent()),
            fun: Fungible(1_000),
        };

        // Only allow grandparent_asset
        let origin_policy = Some(TrustPolicy::AllowedAssets(
            BoundedVec::try_from(vec![grandparent_asset.id]).unwrap(),
        ));

        // parent_asset should be rejected
        assert_eq!(
            apply_policy::<Test>(
                &parent_asset,
                &parent_multilocation,
                origin_policy,
                default_policy
            ),
            false
        );
    }
}
