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

use {
    crate::{Config, DefaultTrustPolicy, TrustPolicy},
    frame_support::{pallet_prelude::*, traits::ContainsPair},
    staging_xcm::latest::{Asset, Junction::Parachain, Junctions::Here, Location},
};

fn apply_policy<T: Config>(
    asset: &Asset,
    origin: &Location,
    maybe_origin_policy: Option<TrustPolicy<T::TrustPolicyMaxAssets>>,
    default_policy: DefaultTrustPolicy,
) -> bool {
    if let Some(origin_policy) = maybe_origin_policy {
        match origin_policy {
            TrustPolicy::AllowedAssets(allowed_assets) => allowed_assets.contains(&asset.id),
            TrustPolicy::DefaultTrustPolicy(origin_default_policy) => match origin_default_policy {
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
impl<T> ContainsPair<Asset, Location> for IsReserveFilter<T>
where
    T: Config,
{
    fn contains(asset: &Asset, origin: &Location) -> bool {
        let maybe_origin_policy = crate::Pallet::<T>::reserve_policy(origin);
        let default_policy = <T as crate::Config>::ReserveDefaultTrustPolicy::get();

        apply_policy::<T>(asset, origin, maybe_origin_policy, default_policy)
    }
}

pub struct IsTeleportFilter<T>(PhantomData<T>);
impl<T> ContainsPair<Asset, Location> for IsTeleportFilter<T>
where
    T: Config,
{
    fn contains(asset: &Asset, origin: &Location) -> bool {
        let maybe_origin_policy = crate::Pallet::<T>::teleport_policy(origin);
        let default_policy = <T as crate::Config>::TeleportDefaultTrustPolicy::get();

        apply_policy::<T>(asset, origin, maybe_origin_policy, default_policy)
    }
}

// TODO: this should probably move to somewhere in the polkadot-sdk repo
pub struct NativeAssetReserve;
impl ContainsPair<Asset, Location> for NativeAssetReserve {
    fn contains(asset: &Asset, origin: &Location) -> bool {
        let reserve = if asset.id.0.parents == 0
            && !matches!(asset.id.0.first_interior(), Some(Parachain(_)))
        {
            Some(Location::here())
        } else {
            asset.id.0.chain_part()
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
    fn chain_part(&self) -> Option<Location>;
    /// Returns "non-chain" location part.
    fn non_chain_part(&self) -> Option<Location>;
}

impl Parse for Location {
    fn chain_part(&self) -> Option<Location> {
        match (self.parents, self.first_interior()) {
            // sibling parachain
            (1, Some(Parachain(id))) => Some(Location::new(1, [Parachain(*id)])),
            // parent
            (1, _) => Some(Location::parent()),
            // children parachain
            (0, Some(Parachain(id))) => Some(Location::new(0, [Parachain(*id)])),
            _ => None,
        }
    }

    fn non_chain_part(&self) -> Option<Location> {
        let junctions = self.interior();
        while matches!(junctions.first(), Some(Parachain(_))) {
            let _ = junctions.clone().take_first();
        }

        if junctions.clone() != Here {
            Some(Location::new(0, junctions.clone()))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use {
        super::*,
        crate::mock::{mock_all::TestAll, mock_all_native::TestAllNative, mock_never::TestNever},
        staging_xcm::latest::{AssetId, Fungibility::Fungible},
    };

    #[test]
    fn default_policy_all_allows_any() {
        let parent_location = Location::parent();
        let grandparent_asset = Asset {
            id: AssetId(Location::new(2, [])),
            fun: Fungible(1_000),
        };

        assert!(apply_policy::<TestAll>(
            &grandparent_asset,
            &parent_location,
            None,
            <TestAll as Config>::ReserveDefaultTrustPolicy::get(),
        ));
    }

    #[test]
    fn default_policy_all_native_allows_native() {
        let parent_location = Location::parent();
        let parent_asset = Asset {
            id: AssetId(Location::parent()),
            fun: Fungible(1_000),
        };

        assert!(apply_policy::<TestAllNative>(
            &parent_asset,
            &parent_location,
            None,
            <TestAllNative as Config>::ReserveDefaultTrustPolicy::get(),
        ));
    }

    #[test]
    fn default_policy_all_native_rejects_non_native() {
        let parent_location = Location::parent();
        let grandparent_asset = Asset {
            id: AssetId(Location::new(2, [])),
            fun: Fungible(1_000),
        };

        assert_eq!(
            apply_policy::<TestAllNative>(
                &grandparent_asset,
                &parent_location,
                None,
                <TestAllNative as Config>::ReserveDefaultTrustPolicy::get(),
            ),
            false
        );
    }

    #[test]
    fn default_policy_never_rejects_any() {
        let parent_location = Location::parent();
        let parent_asset = Asset {
            id: AssetId(Location::parent()),
            fun: Fungible(1_000),
        };

        assert_eq!(
            apply_policy::<TestNever>(
                &parent_asset,
                &parent_location,
                None,
                <TestNever as Config>::ReserveDefaultTrustPolicy::get(),
            ),
            false
        );
    }

    #[test]
    fn policy_all_allows_any() {
        let default_policy = DefaultTrustPolicy::Never;

        let parent_location = Location::parent();
        let grandparent_asset = Asset {
            id: AssetId(Location::new(2, [])),
            fun: Fungible(1_000),
        };

        let origin_policy = Some(TrustPolicy::DefaultTrustPolicy(DefaultTrustPolicy::All));

        assert!(apply_policy::<TestNever>(
            &grandparent_asset,
            &parent_location,
            origin_policy,
            default_policy
        ));
    }

    #[test]
    fn policy_all_native_allows_native_asset() {
        let default_policy = DefaultTrustPolicy::Never;

        let parent_location = Location::parent();
        let parent_asset = Asset {
            id: AssetId(Location::parent()),
            fun: Fungible(1_000),
        };

        let origin_policy = Some(TrustPolicy::DefaultTrustPolicy(
            DefaultTrustPolicy::AllNative,
        ));

        assert!(apply_policy::<TestNever>(
            &parent_asset,
            &parent_location,
            origin_policy,
            default_policy
        ));
    }

    #[test]
    fn policy_all_native_rejects_non_native_asset() {
        let default_policy = DefaultTrustPolicy::Never;

        let parent_location = Location::parent();
        let grandparent_asset = Asset {
            id: AssetId(Location::new(2, [])),
            fun: Fungible(1_000),
        };

        let origin_policy = Some(TrustPolicy::DefaultTrustPolicy(
            DefaultTrustPolicy::AllNative,
        ));

        assert_eq!(
            apply_policy::<TestNever>(
                &grandparent_asset,
                &parent_location,
                origin_policy,
                default_policy
            ),
            false
        );
    }

    #[test]
    fn policy_custom_allows_allowed_asset() {
        let default_policy = DefaultTrustPolicy::Never;

        let parent_location = Location::parent();
        let grandparent_asset = Asset {
            id: AssetId(Location::new(2, [])),
            fun: Fungible(1_000),
        };

        // Only allow grandparent_asset
        let origin_policy = Some(TrustPolicy::AllowedAssets(
            BoundedVec::try_from(vec![grandparent_asset.clone().id]).unwrap(),
        ));

        assert!(apply_policy::<TestNever>(
            &grandparent_asset,
            &parent_location,
            origin_policy,
            default_policy
        ));
    }

    #[test]
    fn policy_custom_reject_not_allowed_asset() {
        let default_policy = DefaultTrustPolicy::Never;

        let parent_location = Location::parent();
        let parent_asset = Asset {
            id: AssetId(Location::parent()),
            fun: Fungible(1_000),
        };
        let grandparent_asset = Asset {
            id: AssetId(Location::new(2, [])),
            fun: Fungible(1_000),
        };

        // Only allow grandparent_asset
        let origin_policy = Some(TrustPolicy::AllowedAssets(
            BoundedVec::try_from(vec![grandparent_asset.id]).unwrap(),
        ));

        // parent_asset should be rejected
        assert_eq!(
            apply_policy::<TestNever>(
                &parent_asset,
                &parent_location,
                origin_policy,
                default_policy
            ),
            false
        );
    }
}
