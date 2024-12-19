use serde::{Deserialize, Serialize};

use crate::{error::DatabaseError, ids, mapping::Relation, system_ids};

/// Space editor relation.
#[derive(Deserialize, Serialize)]
pub struct SpaceMember;

impl SpaceMember {
    pub fn new(member_id: &str, space_id: &str) -> Relation<Self> {
        Relation::new(
            &ids::create_geo_id(),
            system_ids::INDEXER_SPACE_ID,
            member_id,
            space_id,
            Self,
        )
        .with_type(system_ids::MEMBER_RELATION)
    }

    /// Returns a query to delete a relation between an member and a space.
    pub async fn remove(
        neo4j: &neo4rs::Graph,
        member_id: &str,
        space_id: &str,
    ) -> Result<(), DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
                MATCH ({{id: $from}})<-[:`{FROM_ENTITY}`]-(r:`{MEMBER_RELATION}`)-[:`{TO_ENTITY}`]->({{id: $to}})
                DETACH DELETE r
            "#,
            FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
            TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
            MEMBER_RELATION = system_ids::MEMBER_RELATION,
        );

        let query = neo4rs::query(QUERY)
            .param("from", member_id)
            .param("to", space_id);

        Ok(neo4j.run(query).await?)
    }
}
