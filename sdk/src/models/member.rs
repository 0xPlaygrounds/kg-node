use crate::{
    error::DatabaseError,
    ids, indexer_ids,
    mapping::{self, query_utils::Query, relation, Relation},
};

use super::BlockMetadata;

/// Space editor relation.
#[derive(Clone)]
pub struct SpaceMember;

impl SpaceMember {
    pub fn generate_id(member_id: &str, space_id: &str) -> String {
        ids::create_id_from_unique_string(format!("MEMBER:{space_id}:{member_id}"))
    }

    pub fn new(member_id: &str, space_id: &str) -> Relation<Self> {
        Relation::new(
            Self::generate_id(member_id, space_id),
            member_id,
            space_id,
            indexer_ids::MEMBER_RELATION,
            "0",
            Self,
        )
    }

    /// Delete a relation between an member and a space.
    pub async fn remove(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        member_id: &str,
        space_id: &str,
    ) -> Result<(), DatabaseError> {
        relation::delete_one(
            neo4j,
            block,
            Self::generate_id(member_id, space_id),
            indexer_ids::INDEXER_SPACE_ID,
            "0",
        )
        .send()
        .await
    }
}

impl mapping::IntoAttributes for SpaceMember {
    fn into_attributes(self) -> Result<mapping::Attributes, mapping::TriplesConversionError> {
        Ok(mapping::Attributes::default())
    }
}

impl mapping::FromAttributes for SpaceMember {
    fn from_attributes(
        _attributes: mapping::Attributes,
    ) -> Result<Self, mapping::TriplesConversionError> {
        Ok(Self)
    }
}
