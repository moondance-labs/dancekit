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

#![cfg(feature = "runtime-benchmarks")]

//! Benchmarking
use {
    crate::{Call, Config, Pallet, MultiLocation},
    frame_benchmarking::benchmarks,
    frame_system::RawOrigin,
    staging_xcm::v3::Junctions::Here,
};

benchmarks! {
    add_valid_origin {}: add_valid_origin(RawOrigin::Root, MultiLocation {
        parents: 1,
        interior: Here,
    })

    // remove_valid_origin {
    //     T::AssetsFiltering::add_valid_origin(RawOrigin::Root, MultiLocation {
    //         parents: 1,
    //         interior: Here,
    //     });
    // }: remove_valid_origin(RawOrigin::Root, MultiLocation {
    //     parents: 1,
    //     interior: Here,
    // })

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::new_test_ext(),
        crate::mock::Test
    );
}
