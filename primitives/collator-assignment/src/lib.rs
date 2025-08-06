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

use {
    alloc::vec::Vec,
    cumulus_primitives_core::ParaId,
    parity_scale_codec::{Decode, Encode},
    scale_info::prelude::collections::{BTreeMap, BTreeSet},
    sp_runtime::Saturating,
};

#[derive(Clone, Encode, Decode, PartialEq, sp_core::RuntimeDebug, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct AssignedCollators<AccountId> {
    // This must be a Vec and not a BTreeSet because the order is important
    pub orchestrator_chain: Vec<AccountId>,
    // This is private to try to handle the edge case of empty vec here instead of in caller code
    container_chains: BTreeMap<ParaId, Vec<AccountId>>,
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
    /// Find the `ParaId` where collator `x` is assigned to. Returns `None` if not assigned to any.
    ///
    /// `orchestrator_chain_para_id` is used to simplify the return value: returning `Some` means
    /// the collator is assigned somewhere, but it could be a container chain or the orchestrator
    /// chain.
    pub fn para_id_of(&self, x: &AccountId, orchestrator_chain_para_id: ParaId) -> Option<ParaId> {
        if let Some(id) = self.container_para_id_of(x) {
            return Some(id);
        }

        if self.orchestrator_chain.contains(x) {
            return Some(orchestrator_chain_para_id);
        }

        None
    }

    /// Find the container `ParaId` where collator `x` is assigned to. Returns `None` if
    /// not assigned to any. If this returns `None`, the collator could still be assigned to the orchestrator chain.
    pub fn container_para_id_of(&self, x: &AccountId) -> Option<ParaId> {
        for (id, cs) in self.container_chains.iter() {
            if cs.contains(x) {
                return Some(*id);
            }
        }

        None
    }

    /// Map the collator type. Returns all collators in the same order as the input.
    // TODO: if we didn't need to support this method we could change all the `Vec<AccountId>` into
    // `BTreeSet<AccountId>`.
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

    /// Get collators assigned to container chain `para_id`. Handles the edge case of an empty list.
    /// If this returns Some, the Vec can be assumed to not be empty.
    pub fn get_container_chain(&self, para_id: &ParaId) -> Option<&Vec<AccountId>> {
        let x = self.container_chains.get(para_id);

        // Filter out empty assignment, return None instead
        match x {
            Some(x) if x.is_empty() => None,
            x => x,
        }
    }

    /// Remove all the container chains whose list of assigned collators is empty. That is logically
    /// equivalent to that para id not being in the map. Call this before serializing this type.
    pub fn cleanup_empty(&mut self) {
        self.container_chains.retain(|_, v| {
            // Keep the entries whose value is not an empty list
            !v.is_empty()
        });
    }

    /// Return container_chains map with all the chains that have at least 1 assigned collator.
    /// Ignores orchestrator chain.
    pub fn into_container_chains_with_collators(mut self) -> BTreeMap<ParaId, Vec<AccountId>> {
        self.cleanup_empty();

        self.container_chains
    }

    /// Merge `orchestrator_chain` into `container_chains` map as `orchestrator_para_id`, and return
    /// the resulting map. Empty chains will be removed, including orchestrator chain if its empty.
    pub fn into_single_map(
        mut self,
        orchestrator_para_id: ParaId,
    ) -> BTreeMap<ParaId, Vec<AccountId>> {
        self.container_chains.insert(
            orchestrator_para_id,
            core::mem::take(&mut self.orchestrator_chain),
        );

        self.cleanup_empty();

        self.container_chains
    }

    /// Create `Self` from a single map, removing `orchestrator_para_id`.
    /// This calls `Self::cleanup_empty` internally, so the resulting assignment will only include
    /// chains with collators.
    pub fn from_single_map(
        mut container_chains: BTreeMap<ParaId, Vec<AccountId>>,
        orchestrator_para_id: &ParaId,
    ) -> Self {
        let orchestrator_chain = container_chains
            .remove(orchestrator_para_id)
            .unwrap_or_default();

        let mut x = Self {
            orchestrator_chain,
            container_chains,
        };

        x.cleanup_empty();

        x
    }

    /// Return the total number of collators assigned to all chains, orchestrator + containers
    pub fn count_collators(&self) -> usize {
        let mut num_collators: usize = 0;
        num_collators.saturating_accrue(self.orchestrator_chain.len());
        for collators in self.container_chains.values() {
            num_collators.saturating_accrue(collators.len());
        }

        num_collators
    }

    /// Iterate over all the non-empty container chains.
    pub fn container_chains_iter(&self) -> impl Iterator<Item = (&ParaId, &Vec<AccountId>)> {
        self.container_chains.iter().filter_map(
            |(k, v)| {
                if v.is_empty() {
                    None
                } else {
                    Some((k, v))
                }
            },
        )
    }

    /// Convenience method to get a new `BTreeMap` of all the non-empty container chains.
    /// Prefer using some other method to avoid creating this temporary map:
    /// * `container_chains_iter` if you just need to iterate
    /// * `get_container_chain` to query 1 chain
    /// * `insert_container_chain` / `remove_container_chain` to add new chains
    /// * `into_single_map` / `from_single_map` if you prefer to work on a raw `BTreeMap`
    pub fn container_chains(&self) -> BTreeMap<&ParaId, &Vec<AccountId>> {
        self.container_chains_iter().collect()
    }

    /// Return all the container chain para ids with at least 1 collator assigned
    pub fn container_para_ids(&self) -> BTreeSet<ParaId> {
        self.container_chains_iter().map(|(k, _v)| *k).collect()
    }

    /// If `v` is not empty, insert into `self.container_chains`
    pub fn insert_container_chain(&mut self, k: ParaId, v: Vec<AccountId>) {
        if !v.is_empty() {
            self.container_chains.insert(k, v);
        }
    }

    /// Remove container chain
    pub fn remove_container_chain(&mut self, k: &ParaId) -> Option<Vec<AccountId>> {
        self.container_chains.remove(k)
    }
}

impl<AccountId> AssignedCollators<AccountId>
where
    AccountId: Ord,
{
    /// Return all collators assigned to some chain. Includes orchestartor chain.
    pub fn into_collators(mut self) -> BTreeSet<AccountId> {
        let mut collators = BTreeSet::new();
        collators.extend(core::mem::take(&mut self.orchestrator_chain));
        collators.extend(
            self.into_container_chains_with_collators()
                .into_values()
                .flatten(),
        );

        collators
    }

    /// Invert the map relation and return a map of collator to para id.
    /// Useful for testing and for checking the assignment of multiple collators at once.
    pub fn invert_map(mut self, orchestrator_para_id: ParaId) -> BTreeMap<AccountId, ParaId> {
        let mut x = BTreeMap::new();

        for collator in core::mem::take(&mut self.orchestrator_chain) {
            x.insert(collator, orchestrator_para_id);
        }

        for (para_id, collators) in self.container_chains {
            for collator in collators {
                x.insert(collator, para_id);
            }
        }

        x
    }
}
