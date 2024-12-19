use serde::{Deserialize, Serialize};

use crate::{ids, mapping::{Query, Relation}, system_ids};

/// Space editor relation.
#[derive(Deserialize, Serialize)]
pub struct SpaceEditor;

impl SpaceEditor {
    pub fn new(
        editor_id: &str,
        space_id: &str,
    ) -> Relation<Self> {
        Relation::new(
            &ids::create_geo_id(),
            system_ids::INDEXER_SPACE_ID,
            editor_id,
            space_id,
            Self,
        )
        .with_type(system_ids::EDITOR_RELATION)
    }

    /// Returns a query to delete a relation between an editor and a space.
    pub fn remove_query(
        editor_id: &str,
        space_id: &str,
    ) -> Query<()> {
        const QUERY: &str = const_format::formatcp!(
            r#"
                MATCH ({{id: $from}})<-[:`{FROM_ENTITY}`]-(r:`{EDITOR_RELATION}`)-[:`{TO_ENTITY}`]->({{id: $to}})
                DETACH DELETE r
            "#,
            FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
            TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
            EDITOR_RELATION = system_ids::EDITOR_RELATION,
        );

        Query::new(QUERY)
            .param("from", editor_id)
            .param("to", space_id)
    }
}