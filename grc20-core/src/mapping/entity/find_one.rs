use super::models::Entity;
use crate::error::DatabaseError;
use crate::mapping::query_utils::{QueryBuilder, Subquery};
use crate::mapping::value::models::Value;
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub(super) struct EntityQueryResult {
    pub(super) entity_id: Uuid,
    pub(super) types: Vec<String>,
    pub(super) spaces: Vec<Uuid>,
    pub(super) properties: Vec<HashMap<Uuid, String>>,
}

impl EntityQueryResult {
    pub fn into_entity(self) -> Entity {
        Entity {
            id: self.entity_id,
            types: self
                .types
                .into_iter()
                .filter_map(|id| Uuid::parse_str(&id).ok())
                .collect(),
            values: self
                .spaces
                .into_iter()
                .zip(self.properties)
                .map(|(space_id, props)| {
                    (
                        space_id,
                        props
                            .into_iter()
                            .map(|(key, value)| {
                                (
                                    key,
                                    Value {
                                        property: key,
                                        value,
                                        options: None,
                                    },
                                )
                            })
                            .collect(),
                    )
                })
                .collect(),
        }
    }
}

#[derive(Clone)]
pub struct FindOneQuery {
    pub neo4j: neo4rs::Graph,
    pub entity_id: Uuid,
}

impl FindOneQuery {
    pub fn new(neo4j: &neo4rs::Graph, entity_id: Uuid) -> Self {
        Self {
            neo4j: neo4j.clone(),
            entity_id,
        }
    }

    pub async fn send(self) -> Result<Option<Entity>, DatabaseError> {
        let query = QueryBuilder::default()
            .subqueries(vec![
                "MATCH (e:Entity {id: $entity_id})",
                "OPTIONAL MATCH (e)-[p:PROPERTIES]->(props:Properties)",
                "WHERE size(keys(props)) > 0",
                "WITH e, collect(p.space_id) AS spaces, collect(apoc.map.removeKey(properties(props), 'embedding')) AS props",
                "RETURN {entity_id: e.id, spaces: spaces, properties: props, types: labels(e)}",
            ])
            .params("entity_id", self.entity_id.to_string());

        if cfg!(debug_assertions) || cfg!(test) {
            println!(
                "entity_node::FindOneQuery:\n{}\nparams:{:?}",
                query.compile(),
                query.params
            );
        };

        let mut result = self.neo4j.execute(query.build()).await?;

        if let Some(row) = result.next().await? {
            let query_result: EntityQueryResult = row.to()?;

            Ok(Some(query_result.into_entity()))
        } else {
            Ok(None)
        }
    }
}

pub fn find_one(neo4j: &neo4rs::Graph, entity_id: Uuid) -> FindOneQuery {
    FindOneQuery::new(neo4j, entity_id)
}
