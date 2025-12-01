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

#![cfg_attr(not(feature = "std"), no_std)]

use {
    frame_support::traits::ContainsPair,
    staging_xcm::latest::{Asset, Junction::Parachain, Location},
};

// TODO: this should probably move to somewhere in the polkadot-sdk repo
pub struct NativeAssetReserve;
impl ContainsPair<Asset, Location> for NativeAssetReserve {
    fn contains(asset: &Asset, origin: &Location) -> bool {
        log::trace!(target: "xcm::contains", "NativeAssetReserve asset: {:?}, origin: {:?}", asset, origin);
        let reserve = if asset.id.0.parents == 0
            && !matches!(asset.id.0.first_interior(), Some(Parachain(_)))
        {
            Some(Location::here())
        } else {
            chain_part(&asset.id.0)
        };

        if let Some(ref reserve) = reserve {
            if reserve == origin {
                return true;
            }
        }
        false
    }
}

/// Returns the "chain" location part. It could be parent, sibling
/// parachain, or child parachain.
pub fn chain_part(this: &Location) -> Option<Location> {
    match (this.parents, this.first_interior()) {
        // sibling parachain
        (1, Some(Parachain(id))) => Some(Location::new(1, [Parachain(*id)])),
        // parent
        (1, _) => Some(Location::parent()),
        // children parachain
        (0, Some(Parachain(id))) => Some(Location::new(0, [Parachain(*id)])),
        _ => None,
    }
}
