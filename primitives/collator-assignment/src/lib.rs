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

use {
    cumulus_primitives_core::ParaId,
    parity_scale_codec::{Decode, Encode},
    scale_info::prelude::collections::BTreeMap,
    sp_std::vec::Vec,
};

#[derive(Clone, Encode, Decode, PartialEq, sp_core::RuntimeDebug, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct AssignedCollators<AccountId> {
    pub orchestrator_chain: Vec<AccountId>,
    pub container_chains: BTreeMap<ParaId, Vec<AccountId>>,
}

// Manual default impl that does not require AccountId: Default
impl<AccountId> Default for AssignedCollators<AccountId> {
    fn default() -> Self {
        Self {
            orchestrator_chain: Default::default(),
            container_chains: Default::default(),
        }
    }
}

impl<AccountId> AssignedCollators<AccountId>
where
    AccountId: PartialEq,
{
    pub fn para_id_of(&self, x: &AccountId, orchestrator_chain_para_id: ParaId) -> Option<ParaId> {
        for (id, cs) in self.container_chains.iter() {
            if cs.contains(x) {
                return Some(*id);
            }
        }

        if self.orchestrator_chain.contains(x) {
            return Some(orchestrator_chain_para_id);
        }

        None
    }

    pub fn map<T, F>(&self, mut f: F) -> AssignedCollators<T>
    where
        F: FnMut(&AccountId) -> T,
    {
        let mut a = AssignedCollators {
            orchestrator_chain: self.orchestrator_chain.iter().map(&mut f).collect(),
            ..Default::default()
        };

        for (para_id, collators) in self.container_chains.iter() {
            let a_collators = collators.iter().map(&mut f).collect();
            a.container_chains.insert(*para_id, a_collators);
        }

        a
    }
}
