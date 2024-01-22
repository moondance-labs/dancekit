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
    crate::mock::{new_test_ext, RuntimeOrigin, XcmExecutorUtils},
    staging_xcm::latest::{Fungibility::Fungible, MultiAsset},
};

#[test]
fn reserve_policy_can_be_set_and_removed() {
    new_test_ext().execute_with(|| {
        let origin_multilocation = MultiLocation::parent();
        let filter_policy = FilterPolicy::DefaultFilterPolicy(DefaultFilterPolicy::Never);

        let _ = XcmExecutorUtils::set_reserve_policy(
            RuntimeOrigin::root(),
            origin_multilocation,
            filter_policy.clone(),
        );

        assert_eq!(
            XcmExecutorUtils::reserve_policy(origin_multilocation),
            Some(filter_policy)
        );

        let _ =
            XcmExecutorUtils::remove_reserve_policy(RuntimeOrigin::root(), origin_multilocation);

        assert!(XcmExecutorUtils::reserve_policy(origin_multilocation).is_none());
    });
}

#[test]
fn teleport_policy_can_be_set_and_removed() {
    new_test_ext().execute_with(|| {
        let origin_multilocation = MultiLocation::parent();
        let filter_policy = FilterPolicy::DefaultFilterPolicy(DefaultFilterPolicy::Never);

        let _ = XcmExecutorUtils::set_teleport_policy(
            RuntimeOrigin::root(),
            origin_multilocation,
            filter_policy.clone(),
        );

        assert_eq!(
            XcmExecutorUtils::teleport_policy(origin_multilocation),
            Some(filter_policy)
        );

        let _ =
            XcmExecutorUtils::remove_teleport_policy(RuntimeOrigin::root(), origin_multilocation);

        assert!(XcmExecutorUtils::teleport_policy(origin_multilocation).is_none());
    });
}
