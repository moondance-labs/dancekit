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

use frame_support::traits::ContainsPair;

use {
    super::*,
    crate::mock::{new_test_ext, AssetsFiltering, RuntimeOrigin, Test},
    staging_xcm::latest::{Fungibility::Fungible, MultiAsset},
};

#[test]
fn rule_can_be_set_and_removed() {
    new_test_ext().execute_with(|| {
        let origin_multilocation = MultiLocation::parent();
        let policy = Policy::DefaultPolicy(DefaultPolicy::Never);

        let _ =
            AssetsFiltering::set_rule(RuntimeOrigin::root(), origin_multilocation, policy.clone());

        assert_eq!(
            AssetsFiltering::origin_policy(origin_multilocation),
            Some(policy)
        );

        let _ = AssetsFiltering::remove_rule(RuntimeOrigin::root(), origin_multilocation);

        assert!(AssetsFiltering::origin_policy(origin_multilocation).is_none());
    });
}

#[test]
fn reserve_policy_all_allows_any() {
    new_test_ext().execute_with(|| {
        let parent_multilocation = MultiLocation::parent();
        let asset = MultiAsset {
            id: AssetId::Concrete(MultiLocation::here()),
            fun: Fungible(1_000),
        };

        let _ = AssetsFiltering::set_rule(
            RuntimeOrigin::root(),
            parent_multilocation,
            Policy::DefaultPolicy(DefaultPolicy::All),
        );

        assert!(reserves::AssetsFilteringReserve::<Test>::contains(
            &asset,
            &parent_multilocation
        ));
    });
}

#[test]
fn reserve_policy_all_native_allows_native_asset() {
    new_test_ext().execute_with(|| {
        let parent_multilocation = MultiLocation::parent();
        let parent_asset = MultiAsset {
            id: AssetId::Concrete(MultiLocation::parent()),
            fun: Fungible(1_000),
        };

        let _ = AssetsFiltering::set_rule(
            RuntimeOrigin::root(),
            parent_multilocation,
            Policy::DefaultPolicy(DefaultPolicy::AllNative),
        );

        assert!(reserves::AssetsFilteringReserve::<Test>::contains(
            &parent_asset,
            &parent_multilocation
        ));
    });
}

#[test]
fn reserve_policy_all_native_rejects_non_native_asset() {
    new_test_ext().execute_with(|| {
        let parent_multilocation = MultiLocation::parent();
        let grandparent_asset = MultiAsset {
            id: AssetId::Concrete(MultiLocation::grandparent()),
            fun: Fungible(1_000),
        };

        let _ = AssetsFiltering::set_rule(
            RuntimeOrigin::root(),
            parent_multilocation,
            Policy::DefaultPolicy(DefaultPolicy::AllNative),
        );

        assert_eq!(
            reserves::AssetsFilteringReserve::<Test>::contains(&grandparent_asset, &parent_multilocation),
            false
        );
    });
}

#[test]
fn reserve_policy_custom_allows_allowed_asset() {
    new_test_ext().execute_with(|| {
        let parent_multilocation = MultiLocation::parent();
        let grandparent_asset = MultiAsset {
            id: AssetId::Concrete(MultiLocation::grandparent()),
            fun: Fungible(1_000),
        };

        let _ = AssetsFiltering::set_rule(
            RuntimeOrigin::root(),
            parent_multilocation,
            Policy::AllowedAssets( BoundedVec::try_from(vec![grandparent_asset.id]).unwrap()),
        );

        assert!(
            reserves::AssetsFilteringReserve::<Test>::contains(&grandparent_asset, &parent_multilocation),
        );
    });
}

#[test]
fn reserve_policy_custom_reject_not_allowed_asset() {
    new_test_ext().execute_with(|| {
        let parent_multilocation = MultiLocation::parent();
        let parent_asset = MultiAsset {
            id: AssetId::Concrete(MultiLocation::parent()),
            fun: Fungible(1_000),
        };
        let grandparent_asset = MultiAsset {
            id: AssetId::Concrete(MultiLocation::grandparent()),
            fun: Fungible(1_000),
        };

        let _ = AssetsFiltering::set_rule(
            RuntimeOrigin::root(),
            parent_multilocation,
            Policy::AllowedAssets( BoundedVec::try_from(vec![grandparent_asset.id]).unwrap()),
        );

        assert_eq!(
            reserves::AssetsFilteringReserve::<Test>::contains(&parent_asset, &parent_multilocation),
            false
        );
    });
}
