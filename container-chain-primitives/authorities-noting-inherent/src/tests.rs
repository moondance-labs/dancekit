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
    crate::ContainerChainAuthoritiesInherentData,
    async_trait::async_trait,
    cumulus_primitives_core::{
        relay_chain::{
            vstaging::CoreState, BlockId, CoreIndex, HeadData, OccupiedCoreAssumption,
            SessionIndex, ValidationCodeHash, ValidatorId,
        },
        InboundDownwardMessage, InboundHrmpMessage, ParaId, PersistedValidationData,
    },
    cumulus_relay_chain_interface::{PHash, PHeader, RelayChainInterface, RelayChainResult},
    dc_orchestrator_chain_interface::{
        BlockNumber, ContainerChainGenesisData, DataPreserverAssignment, DataPreserverProfileId,
        OrchestratorChainInterface, OrchestratorChainResult,
    },
    dp_core::{well_known_keys, Header as OrchestratorHeader},
    futures::Stream,
    nimbus_primitives::NimbusId,
    polkadot_overseer::Handle,
    polkadot_primitives::vstaging::CommittedCandidateReceiptV2 as CommittedCandidateReceipt,
    sc_client_api::{HeaderBackend, StorageKey, StorageProvider},
    sp_inherents::{InherentData, InherentDataProvider},
    sp_state_machine::{prove_read, StorageValue},
    sp_version::RuntimeVersion,
    std::{
        collections::{BTreeMap, VecDeque},
        pin::Pin,
        sync::Arc,
    },
    substrate_test_runtime_client::{
        ClientExt, DefaultTestClientBuilderExt, TestClient, TestClientBuilder, TestClientBuilderExt,
    },
};

#[derive(Clone)]
struct DummyOrchestratorChainInterface {
    orchestrator_client: Arc<TestClient>,
}

#[derive(Clone)]
struct DummyRelayChainInterface {
    relay_client: Arc<TestClient>,
}

impl DummyOrchestratorChainInterface {
    fn new(session: u32) -> Self {
        let builder = TestClientBuilder::new().add_extra_storage(
            well_known_keys::SESSION_INDEX.to_vec(),
            session.encode().to_vec(),
        );

        Self {
            orchestrator_client: Arc::new(builder.build()),
        }
    }
}

impl DummyRelayChainInterface {
    fn new(orchestrator_para_id: ParaId, header: OrchestratorHeader) -> Self {
        Self::new_with_head_data(
            orchestrator_para_id,
            HeadData(header.encode()).encode().to_vec(),
        )
    }

    fn new_with_head_data(orchestrator_para_id: ParaId, head_data: Vec<u8>) -> Self {
        let builder = TestClientBuilder::new().add_extra_storage(
            well_known_keys::para_id_head(orchestrator_para_id).to_vec(),
            head_data,
        );

        Self {
            relay_client: Arc::new(builder.build()),
        }
    }
}

#[async_trait]
impl OrchestratorChainInterface for DummyOrchestratorChainInterface {
    fn overseer_handle(&self) -> OrchestratorChainResult<Handle> {
        unimplemented!("Not needed for test")
    }

    async fn get_storage_by_key(
        &self,
        hash: PHash,
        key: &[u8],
    ) -> OrchestratorChainResult<Option<StorageValue>> {
        self.orchestrator_client
            .storage(hash, &StorageKey(key.to_vec()))
            .map(|a| a.map(|b| b.0))
            .map_err(|e| e.into())
    }

    async fn prove_read(
        &self,
        hash: PHash,
        keys: &Vec<Vec<u8>>,
    ) -> OrchestratorChainResult<sc_client_api::StorageProof> {
        self.orchestrator_client
            .state_at(hash)
            .map(|state| prove_read(state, keys))
            .unwrap()
            .map_err(|e| e.into())
    }

    async fn import_notification_stream(
        &self,
    ) -> OrchestratorChainResult<Pin<Box<dyn Stream<Item = PHeader> + Send>>> {
        unimplemented!("Not needed for test")
    }

    async fn new_best_notification_stream(
        &self,
    ) -> OrchestratorChainResult<Pin<Box<dyn Stream<Item = PHeader> + Send>>> {
        unimplemented!("Not needed for test")
    }

    async fn finality_notification_stream(
        &self,
    ) -> OrchestratorChainResult<Pin<Box<dyn Stream<Item = PHeader> + Send>>> {
        unimplemented!("Not needed for test")
    }

    async fn genesis_data(
        &self,
        _orchestrator_parent: PHash,
        _para_id: ParaId,
    ) -> OrchestratorChainResult<Option<ContainerChainGenesisData>> {
        unimplemented!("Not needed for test")
    }

    async fn boot_nodes(
        &self,
        _orchestrator_parent: PHash,
        _para_id: ParaId,
    ) -> OrchestratorChainResult<Vec<Vec<u8>>> {
        unimplemented!("Not needed for test")
    }

    async fn latest_block_number(
        &self,
        _orchestrator_parent: PHash,
        _para_id: ParaId,
    ) -> OrchestratorChainResult<Option<BlockNumber>> {
        unimplemented!("Not needed for test")
    }

    async fn best_block_hash(&self) -> OrchestratorChainResult<PHash> {
        unimplemented!("Not needed for test")
    }

    async fn finalized_block_hash(&self) -> OrchestratorChainResult<PHash> {
        unimplemented!("Not needed for test")
    }

    async fn data_preserver_active_assignment(
        &self,
        _orchestrator_parent: PHash,
        _profile_id: DataPreserverProfileId,
    ) -> OrchestratorChainResult<DataPreserverAssignment<ParaId>> {
        unimplemented!("Not needed for test")
    }

    async fn check_para_id_assignment(
        &self,
        _orchestrator_parent: PHash,
        _authority: NimbusId,
    ) -> OrchestratorChainResult<Option<ParaId>> {
        unimplemented!("Not needed for test")
    }

    async fn check_para_id_assignment_next_session(
        &self,
        _orchestrator_parent: PHash,
        _authority: NimbusId,
    ) -> OrchestratorChainResult<Option<ParaId>> {
        unimplemented!("Not needed for test")
    }
}

#[async_trait]
impl RelayChainInterface for DummyRelayChainInterface {
    fn overseer_handle(&self) -> RelayChainResult<Handle> {
        unimplemented!("Not needed for test")
    }

    async fn validators(&self, _: PHash) -> RelayChainResult<Vec<ValidatorId>> {
        unimplemented!("Not needed for test")
    }

    async fn best_block_hash(&self) -> RelayChainResult<PHash> {
        unimplemented!("Not needed for test")
    }

    async fn finalized_block_hash(&self) -> RelayChainResult<PHash> {
        unimplemented!("Not needed for test")
    }

    async fn retrieve_dmq_contents(
        &self,
        _: ParaId,
        _: PHash,
    ) -> RelayChainResult<Vec<InboundDownwardMessage>> {
        unimplemented!("Not needed for test")
    }

    async fn retrieve_all_inbound_hrmp_channel_contents(
        &self,
        _: ParaId,
        _: PHash,
    ) -> RelayChainResult<BTreeMap<ParaId, Vec<InboundHrmpMessage>>> {
        unimplemented!("Not needed for test")
    }

    async fn persisted_validation_data(
        &self,
        _: PHash,
        _: ParaId,
        _: OccupiedCoreAssumption,
    ) -> RelayChainResult<Option<PersistedValidationData>> {
        unimplemented!("Not needed for test")
    }

    async fn candidate_pending_availability(
        &self,
        _: PHash,
        _: ParaId,
    ) -> RelayChainResult<Option<CommittedCandidateReceipt>> {
        unimplemented!("Not needed for test")
    }

    async fn session_index_for_child(&self, _: PHash) -> RelayChainResult<SessionIndex> {
        unimplemented!("Not needed for test")
    }

    async fn import_notification_stream(
        &self,
    ) -> RelayChainResult<Pin<Box<dyn Stream<Item = PHeader> + Send>>> {
        unimplemented!("Not needed for test")
    }

    async fn finality_notification_stream(
        &self,
    ) -> RelayChainResult<Pin<Box<dyn Stream<Item = PHeader> + Send>>> {
        unimplemented!("Not needed for test")
    }

    async fn is_major_syncing(&self) -> RelayChainResult<bool> {
        unimplemented!("Not needed for test")
    }

    async fn wait_for_block(&self, _hash: PHash) -> RelayChainResult<()> {
        unimplemented!("Not needed for test")
    }

    async fn new_best_notification_stream(
        &self,
    ) -> RelayChainResult<Pin<Box<dyn Stream<Item = PHeader> + Send>>> {
        unimplemented!("Not needed for test")
    }

    async fn get_storage_by_key(
        &self,
        hash: PHash,
        key: &[u8],
    ) -> RelayChainResult<Option<StorageValue>> {
        Ok(self
            .relay_client
            .storage(hash, &StorageKey(key.to_vec()))
            .map(|a| a.map(|b| b.0))
            .unwrap())
    }

    async fn prove_read(
        &self,
        hash: PHash,
        keys: &Vec<Vec<u8>>,
    ) -> RelayChainResult<sc_client_api::StorageProof> {
        Ok(self
            .relay_client
            .state_at(hash)
            .map(|state| prove_read(state, keys))
            .unwrap()
            .unwrap())
    }

    async fn header(&self, block_id: BlockId) -> RelayChainResult<Option<PHeader>> {
        let hash = match block_id {
            BlockId::Hash(hash) => hash,
            BlockId::Number(num) => {
                if let Some(hash) = self.relay_client.hash(num.into())? {
                    hash
                } else {
                    return Ok(None);
                }
            }
        };
        let header = self.relay_client.header(hash)?;

        // this returns a substrate client header, we should convert
        let relay_header = header.map(|header| PHeader {
            parent_hash: header.parent_hash,
            number: header.number.try_into().unwrap(),
            state_root: header.state_root,
            extrinsics_root: header.extrinsics_root,
            digest: header.digest,
        });
        Ok(relay_header)
    }
    async fn validation_code_hash(
        &self,
        _relay_parent: PHash,
        _para_id: ParaId,
        _occupied_core_assumption: OccupiedCoreAssumption,
    ) -> RelayChainResult<Option<ValidationCodeHash>> {
        Ok(None)
    }

    async fn candidates_pending_availability(
        &self,
        _: PHash,
        _: ParaId,
    ) -> RelayChainResult<Vec<CommittedCandidateReceipt>> {
        unimplemented!("Not needed for test")
    }

    async fn availability_cores(
        &self,
        _relay_parent: PHash,
    ) -> RelayChainResult<Vec<CoreState<PHash, BlockNumber>>> {
        unimplemented!("Not needed for test");
    }

    async fn version(&self, _: PHash) -> RelayChainResult<RuntimeVersion> {
        unimplemented!("Not needed for test")
    }

    async fn claim_queue(
        &self,
        _: PHash,
    ) -> RelayChainResult<BTreeMap<CoreIndex, VecDeque<ParaId>>> {
        unimplemented!("Not needed for test");
    }

    async fn call_runtime_api(
        &self,
        _method_name: &'static str,
        _hash: PHash,
        _payload: &[u8],
    ) -> RelayChainResult<Vec<u8>> {
        unimplemented!("Not needed for test")
    }

    async fn scheduling_lookahead(&self, _relay_parent: PHash) -> RelayChainResult<u32> {
        unimplemented!("Not needed for test")
    }
}

#[tokio::test]
async fn test_orchestrator_inherent_insertion() {
    let orch_session = 1u32;
    let orch_para_id = 1000u32;
    let orchestrator_chain_interface = Arc::new(DummyOrchestratorChainInterface::new(orch_session));
    let orchestrator_genesis_hash = orchestrator_chain_interface
        .orchestrator_client
        .genesis_hash();

    let header = orchestrator_chain_interface
        .orchestrator_client
        .header(orchestrator_genesis_hash)
        .unwrap()
        .unwrap();

    // The substrate example header is not the same as the tanssi one in the block num parameter
    let orchestrator_header = OrchestratorHeader {
        parent_hash: header.parent_hash,
        number: header.number.try_into().unwrap(),
        state_root: header.state_root,
        extrinsics_root: header.extrinsics_root,
        digest: header.digest,
    };
    let relay_chain_interface = Arc::new(DummyRelayChainInterface::new(
        orch_para_id.into(),
        orchestrator_header.clone(),
    ));
    let relay_genesis_hash = relay_chain_interface.relay_client.genesis_hash();
    let relay_header = relay_chain_interface
        .relay_client
        .header(relay_genesis_hash)
        .unwrap()
        .unwrap();

    // get latest header info
    let latest_header_info =
        ContainerChainAuthoritiesInherentData::get_latest_orchestrator_head_info(
            relay_header.hash(),
            &relay_chain_interface,
            orch_para_id.into(),
        )
        .await;

    // assert creation went well
    assert_eq!(latest_header_info, Some(orchestrator_header));

    let created = ContainerChainAuthoritiesInherentData::create_at(
        relay_header.hash(),
        &relay_chain_interface,
        &orchestrator_chain_interface,
        orch_para_id.into(),
    )
    .await;

    // assert creation went well
    assert!(created.is_some());

    // Assert we can put inherent data
    let mut inherent_data = InherentData::new();
    assert!(created
        .clone()
        .unwrap()
        .provide_inherent_data(&mut inherent_data)
        .await
        .is_ok());
    assert_eq!(
        inherent_data.get_data(&crate::INHERENT_IDENTIFIER).unwrap(),
        created
    );
}

#[tokio::test]
async fn test_header_not_present_error() {
    let orch_session = 1u32;
    let orch_para_id = 1000u32;
    let orchestrator_chain_interface = Arc::new(DummyOrchestratorChainInterface::new(orch_session));
    let orchestrator_genesis_hash = orchestrator_chain_interface
        .orchestrator_client
        .genesis_hash();

    let header = orchestrator_chain_interface
        .orchestrator_client
        .header(orchestrator_genesis_hash)
        .unwrap()
        .unwrap();

    // The substrate example header is not the same as the tanssi one in the block num parameter
    let orchestrator_header = OrchestratorHeader {
        parent_hash: header.parent_hash,
        number: header.number.try_into().unwrap(),
        state_root: header.state_root,
        extrinsics_root: header.extrinsics_root,
        digest: header.digest,
    };
    let relay_chain_interface = Arc::new(DummyRelayChainInterface::new(
        orch_para_id.into(),
        orchestrator_header.clone(),
    ));
    let relay_genesis_hash = relay_chain_interface.relay_client.genesis_hash();
    let relay_header = relay_chain_interface
        .relay_client
        .header(relay_genesis_hash)
        .unwrap()
        .unwrap();

    // get latest header info, but for another paraId
    let latest_header_info =
        ContainerChainAuthoritiesInherentData::get_latest_orchestrator_head_info(
            relay_header.hash(),
            &relay_chain_interface,
            (orch_para_id + 1).into(),
        )
        .await;

    // assert creation went well
    assert_eq!(latest_header_info, None);

    let created = ContainerChainAuthoritiesInherentData::create_at(
        relay_header.hash(),
        &relay_chain_interface,
        &orchestrator_chain_interface,
        (orch_para_id + 1).into(),
    )
    .await;

    assert_eq!(created, None);
}

#[tokio::test]
async fn test_head_data_not_decodable_error() {
    let orch_session = 1u32;
    let orch_para_id = 1000u32;
    let orchestrator_chain_interface = Arc::new(DummyOrchestratorChainInterface::new(orch_session));
    // Put a non decodable HeadData
    let relay_chain_interface = Arc::new(DummyRelayChainInterface::new_with_head_data(
        orch_para_id.into(),
        vec![10u8],
    ));

    let relay_genesis_hash = relay_chain_interface.relay_client.genesis_hash();
    let relay_header = relay_chain_interface
        .relay_client
        .header(relay_genesis_hash)
        .unwrap()
        .unwrap();

    // get latest header info, but cannot since head data does not decode
    let latest_header_info =
        ContainerChainAuthoritiesInherentData::get_latest_orchestrator_head_info(
            relay_header.hash(),
            &relay_chain_interface,
            (orch_para_id).into(),
        )
        .await;

    assert_eq!(latest_header_info, None);

    let created = ContainerChainAuthoritiesInherentData::create_at(
        relay_header.hash(),
        &relay_chain_interface,
        &orchestrator_chain_interface,
        (orch_para_id).into(),
    )
    .await;

    assert_eq!(created, None);
}

#[tokio::test]
async fn test_header_not_decodable() {
    let orch_session = 1u32;
    let orch_para_id = 1000u32;
    let orchestrator_chain_interface = Arc::new(DummyOrchestratorChainInterface::new(orch_session));
    // Put a decodable HeadData, but a non-decodable header
    let relay_chain_interface = Arc::new(DummyRelayChainInterface::new_with_head_data(
        orch_para_id.into(),
        HeadData(vec![1u8]).encode(),
    ));

    let relay_genesis_hash = relay_chain_interface.relay_client.genesis_hash();
    let relay_header = relay_chain_interface
        .relay_client
        .header(relay_genesis_hash)
        .unwrap()
        .unwrap();

    // get latest header info, but cannot since header does not decode
    let latest_header_info =
        ContainerChainAuthoritiesInherentData::get_latest_orchestrator_head_info(
            relay_header.hash(),
            &relay_chain_interface,
            (orch_para_id).into(),
        )
        .await;

    assert_eq!(latest_header_info, None);

    let created = ContainerChainAuthoritiesInherentData::create_at(
        relay_header.hash(),
        &relay_chain_interface,
        &orchestrator_chain_interface,
        (orch_para_id).into(),
    )
    .await;

    assert_eq!(created, None);
}
