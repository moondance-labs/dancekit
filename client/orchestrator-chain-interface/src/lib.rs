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
// along with Tanssi.  If not, see <http://www.gnu.org/licenses/>.

//! # Orchestrator chain interface client primitives
//!
//! This file contains the OrchestratorChainInterface trait which serves to generate
//! storage proofs to be provided to containerchains
//!
//! get_storage_by_key: retrieves a storage item from the Orchestrator interface at a given
//! Orchestrator parent
//!
//! prove_read: generates a storage proof of a given set of keys at a given Orchestrator parent

use {
    core::pin::Pin, dp_core::ParaId, futures::Stream, polkadot_overseer::Handle,
    sc_client_api::StorageProof, sp_api::ApiError, sp_state_machine::StorageValue, std::sync::Arc,
    parity_scale_codec::{Encode, Decode}
};
pub use {
    cumulus_primitives_core::relay_chain::Slot,
    dp_container_chain_genesis_data::ContainerChainGenesisData,
    dp_core::{BlockNumber, Hash as PHash, Header as PHeader},
};

#[derive(thiserror::Error, Debug)]
pub enum OrchestratorChainError {
    #[error("Blockchain returned an error: {0}")]
    BlockchainError(#[from] sp_blockchain::Error),

    #[error("State machine error occured: {0}")]
    StateMachineError(Box<dyn sp_state_machine::Error>),

    #[error(transparent)]
    Application(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),

    #[error("Unable to communicate with RPC worker: {0}")]
    WorkerCommunicationError(String),

    #[error("Unable to call RPC method '{0}': {1}")]
    RpcCallError(String, String),

    #[error("RPC Error: '{0}'")]
    JsonRpcError(#[from] jsonrpsee::core::ClientError),

    #[error("Scale codec deserialization error: {0}")]
    DeserializationError(#[from] parity_scale_codec::Error),

    #[error("API error: {0}")]
    ApiError(#[from] sp_api::ApiError),

    #[error("Unspecified error occured: {0}")]
    GenericError(String),
}

impl From<OrchestratorChainError> for ApiError {
    fn from(r: OrchestratorChainError) -> Self {
        sp_api::ApiError::Application(Box::new(r))
    }
}

impl From<OrchestratorChainError> for sp_blockchain::Error {
    fn from(r: OrchestratorChainError) -> Self {
        sp_blockchain::Error::Application(Box::new(r))
    }
}

impl<T: std::error::Error + Send + Sync + 'static> From<Box<T>> for OrchestratorChainError {
    fn from(r: Box<T>) -> Self {
        OrchestratorChainError::Application(r)
    }
}

impl From<Box<dyn sp_state_machine::Error>> for OrchestratorChainError {
    fn from(r: Box<dyn sp_state_machine::Error>) -> Self {
        OrchestratorChainError::StateMachineError(r)
    }
}

// TODO: proper errors
pub type OrchestratorChainResult<T> = Result<T, OrchestratorChainError>;

pub type DataPreserverProfileId = u64;

// Copy of Tanssi's pallet_data_preservers_runtime_api::Assignment
#[derive(Debug, Copy, Clone, PartialEq, Eq, Encode, Decode, serde::Serialize, serde::Deserialize)]
pub enum DataPreserverAssignment<ParaId> {
    /// Profile is not currently assigned.
    NotAssigned,
    /// Profile is activly assigned to this ParaId.
    Active(ParaId),
    /// Profile is assigned to this ParaId but is inactive for some reason.
    /// It may be causes by conditions defined in the assignement configuration,
    /// such as lacking payment.
    Inactive(ParaId),
}

/// Trait that provides all necessary methods for interaction between collator and orchestrator chain.
#[async_trait::async_trait]
pub trait OrchestratorChainInterface: Send + Sync {
    /// Fetch a storage item by key.
    async fn get_storage_by_key(
        &self,
        orchestrator_parent: PHash,
        key: &[u8],
    ) -> OrchestratorChainResult<Option<StorageValue>>;

    /// Get a handle to the overseer.
    fn overseer_handle(&self) -> OrchestratorChainResult<Handle>;

    /// Generate a storage read proof.
    async fn prove_read(
        &self,
        orchestrator_parent: PHash,
        relevant_keys: &Vec<Vec<u8>>,
    ) -> OrchestratorChainResult<StorageProof>;

    /// Get a stream of import block notifications.
    async fn import_notification_stream(
        &self,
    ) -> OrchestratorChainResult<Pin<Box<dyn Stream<Item = PHeader> + Send>>>;

    /// Get a stream of new best block notifications.
    async fn new_best_notification_stream(
        &self,
    ) -> OrchestratorChainResult<Pin<Box<dyn Stream<Item = PHeader> + Send>>>;

    /// Get a stream of finality notifications.
    async fn finality_notification_stream(
        &self,
    ) -> OrchestratorChainResult<Pin<Box<dyn Stream<Item = PHeader> + Send>>>;

    async fn genesis_data(
        &self,
        orchestrator_parent: PHash,
        para_id: ParaId,
    ) -> OrchestratorChainResult<Option<ContainerChainGenesisData>>;

    async fn boot_nodes(
        &self,
        orchestrator_parent: PHash,
        para_id: ParaId,
    ) -> OrchestratorChainResult<Vec<Vec<u8>>>;

    async fn latest_block_number(
        &self,
        orchestrator_parent: PHash,
        para_id: ParaId,
    ) -> OrchestratorChainResult<Option<BlockNumber>>;

    async fn best_block_hash(&self) -> OrchestratorChainResult<PHash>;

    async fn finalized_block_hash(&self) -> OrchestratorChainResult<PHash>;

    async fn data_preserver_active_assignment(
        &self,
        orchestrator_parent: PHash,
        profile_id: DataPreserverProfileId,
    ) -> OrchestratorChainResult<DataPreserverAssignment<ParaId>>;
}

#[async_trait::async_trait]
impl<T> OrchestratorChainInterface for Arc<T>
where
    T: OrchestratorChainInterface + ?Sized,
{
    fn overseer_handle(&self) -> OrchestratorChainResult<Handle> {
        (**self).overseer_handle()
    }

    async fn get_storage_by_key(
        &self,
        orchestrator_parent: PHash,
        key: &[u8],
    ) -> OrchestratorChainResult<Option<StorageValue>> {
        (**self).get_storage_by_key(orchestrator_parent, key).await
    }

    async fn prove_read(
        &self,
        orchestrator_parent: PHash,
        relevant_keys: &Vec<Vec<u8>>,
    ) -> OrchestratorChainResult<StorageProof> {
        (**self)
            .prove_read(orchestrator_parent, relevant_keys)
            .await
    }

    async fn import_notification_stream(
        &self,
    ) -> OrchestratorChainResult<Pin<Box<dyn Stream<Item = PHeader> + Send>>> {
        (**self).import_notification_stream().await
    }

    async fn new_best_notification_stream(
        &self,
    ) -> OrchestratorChainResult<Pin<Box<dyn Stream<Item = PHeader> + Send>>> {
        (**self).new_best_notification_stream().await
    }

    async fn finality_notification_stream(
        &self,
    ) -> OrchestratorChainResult<Pin<Box<dyn Stream<Item = PHeader> + Send>>> {
        (**self).finality_notification_stream().await
    }

    async fn genesis_data(
        &self,
        orchestrator_parent: PHash,
        para_id: ParaId,
    ) -> OrchestratorChainResult<Option<ContainerChainGenesisData>> {
        (**self).genesis_data(orchestrator_parent, para_id).await
    }

    async fn boot_nodes(
        &self,
        orchestrator_parent: PHash,
        para_id: ParaId,
    ) -> OrchestratorChainResult<Vec<Vec<u8>>> {
        (**self).boot_nodes(orchestrator_parent, para_id).await
    }

    async fn latest_block_number(
        &self,
        orchestrator_parent: PHash,
        para_id: ParaId,
    ) -> OrchestratorChainResult<Option<BlockNumber>> {
        (**self)
            .latest_block_number(orchestrator_parent, para_id)
            .await
    }

    async fn best_block_hash(&self) -> OrchestratorChainResult<PHash> {
        (**self).best_block_hash().await
    }

    async fn finalized_block_hash(&self) -> OrchestratorChainResult<PHash> {
        (**self).finalized_block_hash().await
    }

    async fn data_preserver_active_assignment(
        &self,
        orchestrator_parent: PHash,
        profile_id: DataPreserverProfileId,
    ) -> OrchestratorChainResult<DataPreserverAssignment<ParaId>> {
        (**self)
            .data_preserver_active_assignment(orchestrator_parent, profile_id)
            .await
    }
}
