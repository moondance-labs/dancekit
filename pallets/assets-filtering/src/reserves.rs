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
    crate::{Config, DefaultPolicy, Policy},
    frame_support::{pallet_prelude::*, traits::ContainsPair},
    staging_xcm::v3::{
        AssetId,
        Junction::Parachain,
        Junctions::{Here, X1},
        MultiAsset, MultiLocation,
    },
};

fn default_policy_matcher(
    asset: &MultiAsset,
    origin: &MultiLocation,
    policy: DefaultPolicy,
) -> bool {
    match policy {
        DefaultPolicy::All => true,
        DefaultPolicy::AllNative => NativeAssetReserve::contains(asset, origin),
        DefaultPolicy::Never => false,
    }
}

pub struct AssetsFilteringReserve<T>(PhantomData<T>);
impl<T> ContainsPair<MultiAsset, MultiLocation> for AssetsFilteringReserve<T>
where
    T: Config,
{
    fn contains(asset: &MultiAsset, origin: &MultiLocation) -> bool {
        if let Some(policy) = crate::Pallet::<T>::origin_policy(origin) {
            match policy {
                Policy::AllowedAssets(assets) => assets.contains(&asset.id),
                Policy::DefaultPolicy(origin_default_policy) => {
                    default_policy_matcher(asset, origin, origin_default_policy)
                }
            }
        } else {
            let default_policy = <T as crate::Config>::DefaultPolicy::get();
            default_policy_matcher(asset, origin, default_policy)
        }
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
