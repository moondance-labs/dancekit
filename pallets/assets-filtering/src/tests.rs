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
    crate::mock::{new_test_ext, AssetsFiltering, RuntimeOrigin},
    staging_xcm::v3::Junctions::Here,
};

#[test]
fn origin_can_be_added_and_removed() {
    new_test_ext().execute_with(|| {
        let origin_multi_location = MultiLocation {
            parents: 1,
            interior: Here,
        };

        let _ = AssetsFiltering::add_valid_origin(RuntimeOrigin::root(), origin_multi_location);

        assert_eq!(
            AssetsFiltering::valid_origins().to_vec(),
            [origin_multi_location]
        );

        let _ = AssetsFiltering::remove_valid_origin(RuntimeOrigin::root(), origin_multi_location);

        assert_eq!(AssetsFiltering::valid_origins().to_vec(), []);
    });
}
