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
    crate::{ContainerChainGenesisData, ContainerChainGenesisDataItem, Properties, TokenMetadata},
    frame_support::BoundedVec,
    serde_json,
};

#[test]
fn test_serde_bounded_vec() {
    let data = ContainerChainGenesisData {
        storage: BoundedVec::try_from(vec![ContainerChainGenesisDataItem {
            key: vec![],
            value: vec![],
        }])
        .unwrap(),
        name: BoundedVec::try_from(vec![1, 2, 3, 4]).unwrap(),
        id: BoundedVec::try_from(vec![5, 6, 7, 8]).unwrap(),
        fork_id: Some(BoundedVec::try_from(vec![9, 10]).unwrap()),
        extensions: BoundedVec::try_from(vec![11, 12]).unwrap(),
        properties: Properties {
            token_metadata: TokenMetadata {
                token_symbol: BoundedVec::truncate_from(b"TEST".to_vec()),
                ss58_format: 12345,
                token_decimals: 12345,
            },
            is_ethereum: false,
        },
    };

    let serialized = serde_json::to_string(&data).unwrap();
    let expected = r#"{"storage":[{"key":"0x","value":"0x"}],"name":"0x01020304","id":"0x05060708","fork_id":[9,10],"extensions":"0x0b0c","properties":{"token_metadata":{"token_symbol":[84,69,83,84],"ss58_format":12345,"token_decimals":12345},"is_ethereum":false}}"#;
    assert_eq!(serialized, expected);

    let deserialized: ContainerChainGenesisData = serde_json::from_str(&serialized).unwrap();
    assert_eq!(data, deserialized);
}
