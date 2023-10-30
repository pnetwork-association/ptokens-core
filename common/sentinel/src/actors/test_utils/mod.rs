#![cfg(test)]
use std::{fs::read_to_string, str::FromStr};

use common_eth::{EthLog, EthSubmissionMaterial};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};

use crate::{
    actors::{Actor, ActorType, Actors},
    NetworkId,
};

pub fn get_sample_actors_propagated_sub_mat() -> EthSubmissionMaterial {
    EthSubmissionMaterial::from_str(
        &read_to_string("src/actors/test_utils/polygon-block-48520980-with-actors-propagated-event.json").unwrap(),
    )
    .unwrap()
}

pub fn get_sample_actors_propagated_sub_mat_2() -> EthSubmissionMaterial {
    EthSubmissionMaterial::from_str(
        &read_to_string("src/actors/test_utils/polygon-block-49103205-with-actors-propagated-event.json").unwrap(),
    )
    .unwrap()
}

pub fn get_sample_actors_propagated_log() -> EthLog {
    get_sample_actors_propagated_sub_mat().receipts[0].logs[0].clone()
}

pub fn get_sample_actors() -> Actors {
    // NOTE: See here:
    // https://polygonscan.com/tx/0xf577503260b8f1c6608d3e50c93895833f783509ae059f1bd0e6f0922720fa67#eventlog
    let epoch = U256::from(26);
    let network_id = NetworkId::try_from("Polygon").unwrap();
    let tx_hash = EthHash::from_str("0xf577503260b8f1c6608d3e50c93895833f783509ae059f1bd0e6f0922720fa67").unwrap();
    let governance_contract = EthAddress::from_str("0x186d7656ca8e16d6e04b2a87b196d473f3566f54").unwrap();
    let actors = vec![
        Actor::new(
            ActorType::from_str("guardian").unwrap(),
            EthAddress::from_str("0x0ef13b2668dbe1b3edfe9ffb7cbc398363b50f79").unwrap(),
        ),
        Actor::new(
            ActorType::from_str("guardian").unwrap(),
            EthAddress::from_str("0xdb30d31ce9a22f36a44993b1079ad2d201e11788").unwrap(),
        ),
        Actor::new(
            ActorType::from_str("guardian").unwrap(),
            EthAddress::from_str("0x20fa4d3b5124caa8bcd8b88c5e9293ddfa439efb").unwrap(),
        ),
        Actor::new(
            ActorType::from_str("sentinel").unwrap(),
            EthAddress::from_str("0xe06c8959f4c10fcaa9a7ff0d4c4acdda2610da22").unwrap(),
        ),
        Actor::new(
            ActorType::from_str("sentinel").unwrap(),
            EthAddress::from_str("0x988e8c89cca8f54f144d270bcfb02c4584f005e6").unwrap(),
        ),
        Actor::new(
            ActorType::from_str("sentinel").unwrap(),
            EthAddress::from_str("0x73659a0f105905121edbf44fb476b97c785688ec").unwrap(),
        ),
    ];
    Actors::new(epoch, tx_hash, actors, network_id, governance_contract)
}
