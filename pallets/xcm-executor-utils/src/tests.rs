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

use {
    super::*,
    crate::{
        filters::{IsReserveFilter, IsTeleportFilter},
        mock::mock_never::{new_test_ext, RuntimeOrigin, TestNever, XcmExecutorUtils},
    },
    frame_support::traits::ContainsPair,
    staging_xcm::latest::{Asset, Fungibility::Fungible},
};

#[test]
fn reserve_policy_can_be_set_and_removed() {
    new_test_ext().execute_with(|| {
        let origin_location = Location::parent();
        let trust_policy = TrustPolicy::DefaultTrustPolicy(DefaultTrustPolicy::Never);

        let _ = XcmExecutorUtils::set_reserve_policy(
            RuntimeOrigin::root(),
            origin_location.clone(),
            trust_policy.clone(),
        );

        assert_eq!(
            XcmExecutorUtils::reserve_policy(origin_location.clone()),
            Some(trust_policy)
        );

        let _ = XcmExecutorUtils::remove_reserve_policy(RuntimeOrigin::root(), origin_location.clone());

        assert!(XcmExecutorUtils::reserve_policy(origin_location).is_none());
    });
}

#[test]
fn teleport_policy_can_be_set_and_removed() {
    new_test_ext().execute_with(|| {
        let origin_location = Location::parent();
        let trust_policy = TrustPolicy::DefaultTrustPolicy(DefaultTrustPolicy::Never);

        let _ = XcmExecutorUtils::set_teleport_policy(
            RuntimeOrigin::root(),
            origin_location.clone(),
            trust_policy.clone(),
        );

        assert_eq!(
            XcmExecutorUtils::teleport_policy(origin_location.clone()),
            Some(trust_policy)
        );

        let _ = XcmExecutorUtils::remove_teleport_policy(RuntimeOrigin::root(), origin_location.clone());

        assert!(XcmExecutorUtils::teleport_policy(origin_location).is_none());
    });
}

#[test]
fn reserve_policy_is_applied() {
    new_test_ext().execute_with(|| {
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
        let _ = XcmExecutorUtils::set_reserve_policy(
            RuntimeOrigin::root(),
            parent_location.clone(),
            TrustPolicy::AllowedAssets(BoundedVec::try_from(vec![grandparent_asset.clone().id]).unwrap()),
        );

        // Should allow grandparent_asset
        assert!(filters::IsReserveFilter::<TestNever>::contains(
            &grandparent_asset,
            &parent_location
        ));

        // Should reject parent_asset
        assert_eq!(
            IsReserveFilter::<TestNever>::contains(&parent_asset, &parent_location),
            false
        );
    });
}

#[test]
fn teleport_policy_is_applied() {
    new_test_ext().execute_with(|| {
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
        let _ = XcmExecutorUtils::set_teleport_policy(
            RuntimeOrigin::root(),
            parent_location.clone(),
            TrustPolicy::AllowedAssets(BoundedVec::try_from(vec![grandparent_asset.clone().id]).unwrap()),
        );

        // Should allow grandparent_asset
        assert!(IsTeleportFilter::<TestNever>::contains(
            &grandparent_asset,
            &parent_location
        ),);

        // Should reject parent_asset
        assert_eq!(
            IsTeleportFilter::<TestNever>::contains(&parent_asset, &parent_location),
            false
        );
    });
}
