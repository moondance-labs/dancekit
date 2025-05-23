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

//! Data structures used to store a ContainerChain ChainSpec in the registrar pallet

#![cfg_attr(not(feature = "std"), no_std)]

use {
    frame_support::BoundedVec,
    frame_support::{
        traits::ConstU32, CloneNoBound, DebugNoBound, DefaultNoBound, EqNoBound, PartialEqNoBound,
    },
    parity_scale_codec::{Decode, DecodeWithMemTracking, Encode},
    serde::{Deserializer, Serializer},
    sp_core::bytes,
    sp_std::vec::Vec,
};

#[cfg(test)]
mod tests;

#[cfg(feature = "json")]
pub mod json;

// TODO: improve serialization of storage field
// Currently it looks like this:
/*
"storage": [
    {
      "key": "0x0d715f2646c8f85767b5d2764bb2782604a74d81251e398fd8a0a4d55023bb3f"
      "value": "0xd1070000"
    },
    {
      "key": "0x0d715f2646c8f85767b5d2764bb278264e7b9012096b41c4eb3aaf947f6ea429"
      "value": "0x0000"
    }
]
 */
// Ideally it would be:
/*
"storage": {
    "0x0d715f2646c8f85767b5d2764bb2782604a74d81251e398fd8a0a4d55023bb3f": "0xd1070000",
    "0x0d715f2646c8f85767b5d2764bb278264e7b9012096b41c4eb3aaf947f6ea429": "0x0000"
}
 */
// This is just so it looks nicer on polkadot.js, the functionality is the same
// The original approach of using `storage: BTreeMap<Vec<u8>, Vec<u8>>` looks very bad
// in polkadot.js, because `Vec<u8>` is serialized as `[12, 51, 124]` instead of hex.
// That's why we use `serde(with = "sp_core::bytes")` everywhere, to convert it to hex.
#[derive(
    DebugNoBound,
    CloneNoBound,
    EqNoBound,
    DefaultNoBound,
    PartialEqNoBound,
    Encode,
    Decode,
    DecodeWithMemTracking,
    scale_info::TypeInfo,
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(bound = "")]
pub struct ContainerChainGenesisData {
    // Assume 5MB max total size / 8 bytes per item = 655360 items
    pub storage: BoundedVec<ContainerChainGenesisDataItem, ConstU32<655360>>,
    #[serde(
        serialize_with = "serialize_bounded_vec_as_hex",
        deserialize_with = "deserialize_bounded_vec_as_hex"
    )]
    pub name: BoundedVec<u8, ConstU32<1024>>,
    #[serde(
        serialize_with = "serialize_bounded_vec_as_hex",
        deserialize_with = "deserialize_bounded_vec_as_hex"
    )]
    pub id: BoundedVec<u8, ConstU32<1024>>,
    pub fork_id: Option<BoundedVec<u8, ConstU32<1024>>>,
    #[serde(
        serialize_with = "serialize_bounded_vec_as_hex",
        deserialize_with = "deserialize_bounded_vec_as_hex"
    )]
    pub extensions: BoundedVec<u8, ConstU32<1024>>,
    pub properties: Properties,
}

fn serialize_bounded_vec_as_hex<S, const N: u32>(
    bv: &BoundedVec<u8, ConstU32<N>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    bytes::serialize(&bv.clone().into_inner(), serializer)
}

fn deserialize_bounded_vec_as_hex<'de, D, const N: u32>(
    deserializer: D,
) -> Result<BoundedVec<u8, ConstU32<N>>, D::Error>
where
    D: Deserializer<'de>,
{
    let vec: Vec<u8> = bytes::deserialize(deserializer)?;
    BoundedVec::try_from(vec.clone())
        .map_err(|_| serde::de::Error::custom("Failed to convert Vec to BoundedVec"))
}

#[derive(
    DebugNoBound,
    CloneNoBound,
    EqNoBound,
    DefaultNoBound,
    PartialEqNoBound,
    Encode,
    Decode,
    DecodeWithMemTracking,
    scale_info::TypeInfo,
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(bound = "")]
pub struct Properties {
    pub token_metadata: TokenMetadata,
    pub is_ethereum: bool,
}

#[derive(
    DebugNoBound,
    CloneNoBound,
    EqNoBound,
    PartialEqNoBound,
    Encode,
    Decode,
    DecodeWithMemTracking,
    scale_info::TypeInfo,
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(bound = "")]
pub struct TokenMetadata {
    pub token_symbol: BoundedVec<u8, sp_core::ConstU32<255>>,
    pub ss58_format: u32,
    pub token_decimals: u32,
}

impl Default for TokenMetadata {
    fn default() -> Self {
        // Default values from polkadot.js
        Self {
            token_symbol: BoundedVec::truncate_from(b"UNIT".to_vec()),
            ss58_format: 42,
            token_decimals: 12,
        }
    }
}

#[derive(
    Debug,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Encode,
    Decode,
    DecodeWithMemTracking,
    scale_info::TypeInfo,
    serde::Deserialize,
    serde::Serialize,
)]
pub struct ContainerChainGenesisDataItem {
    #[serde(with = "sp_core::bytes")]
    pub key: Vec<u8>,
    #[serde(with = "sp_core::bytes")]
    pub value: Vec<u8>,
}

impl From<(Vec<u8>, Vec<u8>)> for ContainerChainGenesisDataItem {
    fn from(x: (Vec<u8>, Vec<u8>)) -> Self {
        Self {
            key: x.0,
            value: x.1,
        }
    }
}

impl From<ContainerChainGenesisDataItem> for (Vec<u8>, Vec<u8>) {
    fn from(x: ContainerChainGenesisDataItem) -> Self {
        (x.key, x.value)
    }
}
