use serde::{Deserialize, Serialize};
use web3_utils::checksum_address;

use crate::{ids, mapping::Entity, system_ids};


#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub struct GeoAccount {
    pub address: String,
}

impl GeoAccount {
    pub fn new(address: String) -> Entity<Self> {
        let checksummed_address = checksum_address(&address, None);
        Entity::new(
            &ids::create_id_from_unique_string(&checksummed_address),
            system_ids::INDEXER_SPACE_ID,
            Self {
                address: checksummed_address,
            }
        )
        .with_type(system_ids::GEO_ACCOUNT)
    }

    pub fn new_id(address: &str) -> String {
        ids::create_id_from_unique_string(&checksum_address(address, None))
    }
}