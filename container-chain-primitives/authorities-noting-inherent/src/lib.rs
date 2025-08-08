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

//! # Authorities Noting Inherent Primitives
//!
//! This crate defines the client-side primitives that should be taken into account when building
//! the authorities-noting pallet inherent.
//!
//! Runtime-side methods should be implemented in `ccp-authorities-noting-inherent-core` crate instead.
//!
//! In particular, this crate contains:
//! - The client side trait implementations to introduce the inherent
//! - The mock version that gets used both in test files and manual seal
//! - The sproof builder that generates a fake proof that mimics the relay chain sproof
//! - Tests
#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

#[cfg(feature = "std")]
mod client_side;
#[cfg(feature = "std")]
pub use client_side::*;
#[cfg(feature = "std")]
mod mock;
#[cfg(test)]
mod tests;
#[cfg(feature = "std")]
pub use mock::*;

pub use ccp_authorities_noting_inherent_core::ContainerChainAuthoritiesInherentData;
pub use ccp_authorities_noting_inherent_core::INHERENT_IDENTIFIER;
