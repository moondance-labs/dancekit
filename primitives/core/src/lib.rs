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
extern crate alloc;

pub use cumulus_primitives_core::ParaId;

use {
    sp_core::{Decode, Encode},
    sp_runtime::{
        generic,
        traits::{BlakeTwo256, IdentifyAccount, Verify},
        MultiAddress, MultiSignature, OpaqueExtrinsic,
    },
};

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// An index to a block.
pub type BlockNumber = u32;

/// The address format for describing accounts.
pub type Address = MultiAddress<AccountId, ()>;

/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, OpaqueExtrinsic>;

/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;

/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;

// Copied from polkadot-sdk because its fields are not public
// polkadot-sdk/polkadot/runtime/common/src/paras_registrar/mod.rs
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default)]
pub struct ParaInfo<Account, Balance> {
    /// The account that has placed a deposit for registering this para.
    pub manager: Account,
    /// The amount reserved by the `manager` account for the registration.
    pub deposit: Balance,
    /// Whether the para registration should be locked from being controlled by the manager.
    /// None means the lock had not been explicitly set, and should be treated as false.
    pub locked: Option<bool>,
}

/// A declarations of storage keys where an external observer can find some interesting data.
pub mod well_known_keys {
    use {
        alloc::vec::Vec, cumulus_primitives_core::ParaId, sp_core::Encode, sp_io::hashing::twox_64,
    };

    // They key to retrieve the para heads
    pub const PARAS_HEADS_INDEX: &[u8] =
        &hex_literal::hex!["cd710b30bd2eab0352ddcc26417aa1941b3c252fcb29d88eff4f3de5de4476c3"];

    // Retrieves the full key representing the para->heads and the paraId
    pub fn para_id_head(para_id: ParaId) -> Vec<u8> {
        para_id.using_encoded(|para_id: &[u8]| {
            PARAS_HEADS_INDEX
                .iter()
                .chain(twox_64(para_id).iter())
                .chain(para_id.iter())
                .cloned()
                .collect()
        })
    }

    /// authorityAssignment pallet prefix
    pub const AUTHORITY_ASSIGNMENT_PREFIX: &[u8] =
        &hex_literal::hex!["ebe78423c7e3ed25234f80d54547285a170f16afec7d161bc6acec3964492a0c"];

    /// tanssiAuthorityAssignment instead of authorityAssignment for solochain
    pub const SOLOCHAIN_AUTHORITY_ASSIGNMENT_PREFIX: &[u8] =
        &hex_literal::hex!["7a201242ca61564279dc11734e3f8772170f16afec7d161bc6acec3964492a0c"];

    pub fn authority_assignment_for_session(
        session_index: u32,
        custom_prefix: Option<&[u8]>,
    ) -> Vec<u8> {
        session_index.using_encoded(|index| {
            let prefix = custom_prefix.unwrap_or(AUTHORITY_ASSIGNMENT_PREFIX);

            prefix
                .iter()
                .chain(twox_64(index).iter())
                .chain(index.iter())
                .copied()
                .collect()
        })
    }

    pub const SESSION_INDEX: &[u8] =
        &hex_literal::hex!["cec5070d609dd3497f72bde07fc96ba072763800a36a99fdfc7c10f6415f6ee6"];

    // Retrieves the full key for registrar->paras given the paraId
    pub fn registrar_para_info(para_id: ParaId) -> Vec<u8> {
        para_id.using_encoded(|para_id| {
            REGISTRAR_PARAS_INDEX
                .iter()
                .chain(twox_64(para_id).iter())
                .chain(para_id.iter())
                .copied()
                .collect()
        })
    }

    pub const REGISTRAR_PARAS_INDEX: &[u8] =
        &hex_literal::hex!["3fba98689ebed1138735e0e7a5a790abcd710b30bd2eab0352ddcc26417aa194"];
}
