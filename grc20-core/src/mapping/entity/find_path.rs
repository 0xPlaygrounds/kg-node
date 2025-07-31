use uuid::Uuid;

use crate::{
    error::DatabaseError,
    mapping::query_utils::{MatchQuery, QueryBuilder, Subquery},
    system_ids,
};

// use crate::{
//     // entity::EntityFilter,
//     error::DatabaseError,
//     mapping::{
//         // order_by::FieldOrderBy,
//         query_utils::{
//             query_builder::{MatchQuery, QueryBuilder, Subquery},
//             VersionFilter,
//         },
//         AttributeFilter, PropFilter, Query,
//     },
//     system_ids::SCHEMA_TYPE,
// };

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Path {
    pub nodes_ids: Vec<Uuid>,
    pub relation_type_ids: Vec<Uuid>,
}

pub struct FindPathQuery {
    neo4j: neo4rs::Graph,
    id1: Uuid,
    id2: Uuid,
    limit: usize,
}

pub fn find_path(neo4j: &neo4rs::Graph, id1: Uuid, id2: Uuid) -> FindPathQuery {
    FindPathQuery::new(neo4j, id1, id2)
}

impl FindPathQuery {
    pub fn new(neo4j: &neo4rs::Graph, id1: Uuid, id2: Uuid) -> Self {
        Self {
            neo4j: neo4j.clone(),
            id1,
            id2,
            limit: 100, // Default limit
        }
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    fn subquery(&self) -> QueryBuilder {
        QueryBuilder::default()
            .subquery(MatchQuery::new(
                "p = allShortestPaths((e1:Entity {id: $id1}) -[:RELATION*1..10]-(e2:Entity {id: $id2}))",
            )
            .r#where(format!("NONE(n IN nodes(p) WHERE EXISTS((n)-[:RELATION]-(:Entity {{id: \"{}\"}})))", system_ids::SCHEMA_TYPE))) //makes sure to not use primitive types
            .limit(self.limit)
            .params("id1", self.id1.to_string())
            .params("id2", self.id2.to_string())
    }

    pub async fn send(self) -> Result<Vec<Path>, DatabaseError> {
        let query = self.subquery().r#return("p");

        if cfg!(debug_assertions) || cfg!(test) {
            println!(
                "entity::FindPathQuery::<T>:\n{}\nparams:{:?}",
                query.compile(),
                [self.id1, self.id2]
            );
        }

        let mut result = self.neo4j.execute(query.build()).await?;
        let mut all_relationship_data = Vec::new();

        // Process each row
        while let Some(row) = result.next().await? {
            let path: neo4rs::Path = row.get("p")?;
            tracing::info!("This is the info for Path: {:?}", path);

            let relationship_data = Path {
                nodes_ids: (path
                    .nodes()
                    .iter()
                    .filter_map(|rel| rel.get("id").ok())
                    .collect()),
                relation_type_ids: (path
                    .rels()
                    .iter()
                    .filter_map(|rel| rel.get("type").ok())
                    .collect()),
            };

            all_relationship_data.push(relationship_data);
        }

        Ok(all_relationship_data)
    }
}
