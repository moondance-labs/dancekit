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
    crate::{Call, Config, DefaultTrustPolicy, MultiLocation, Pallet, TrustPolicy},
    frame_benchmarking::{impl_benchmark_test_suite, v2::*},
    frame_system::RawOrigin,
    sp_std::vec,
    staging_xcm::v3::Junctions::Here,
};

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn set_reserve_policy() -> Result<(), BenchmarkError> {
        #[extrinsic_call]
        _(
            RawOrigin::Root,
            MultiLocation {
                parents: 1,
                interior: Here,
            },
            TrustPolicy::DefaultTrustPolicy(DefaultTrustPolicy::Never),
        );

        // assert!(
        Ok(())
    }

    #[benchmark]
    fn remove_reserve_policy() -> Result<(), BenchmarkError> {
        let _ = Pallet::<T>::set_reserve_policy(
            RawOrigin::Root.into(),
            MultiLocation {
                parents: 1,
                interior: Here,
            },
            TrustPolicy::DefaultTrustPolicy(DefaultTrustPolicy::Never),
        );

        #[extrinsic_call]
        _(
            RawOrigin::Root,
            MultiLocation {
                parents: 1,
                interior: Here,
            },
        );
        assert!(Pallet::<T>::reserve_policy(MultiLocation {
            parents: 1,
            interior: Here,
        })
        .is_none());

        Ok(())
    }

    #[benchmark]
    fn set_teleport_policy() -> Result<(), BenchmarkError> {
        #[extrinsic_call]
        _(
            RawOrigin::Root,
            MultiLocation {
                parents: 1,
                interior: Here,
            },
            TrustPolicy::DefaultTrustPolicy(DefaultTrustPolicy::Never),
        );

        // assert!(
        Ok(())
    }

    #[benchmark]
    fn remove_teleport_policy() -> Result<(), BenchmarkError> {
        let _ = Pallet::<T>::set_teleport_policy(
            RawOrigin::Root.into(),
            MultiLocation {
                parents: 1,
                interior: Here,
            },
            TrustPolicy::DefaultTrustPolicy(DefaultTrustPolicy::Never),
        );

        #[extrinsic_call]
        _(
            RawOrigin::Root,
            MultiLocation {
                parents: 1,
                interior: Here,
            },
        );
        assert!(Pallet::<T>::teleport_policy(MultiLocation {
            parents: 1,
            interior: Here,
        })
        .is_none());

        Ok(())
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
