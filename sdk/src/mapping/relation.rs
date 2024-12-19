use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{models::BlockMetadata, neo4j_utils::serde_value_to_bolt, system_ids};

use super::{attributes::Attributes, query::Query};

#[derive(Debug, Deserialize, PartialEq)]
pub struct Relation<T> {
    pub id: String,
    pub types: Vec<String>,
    pub from: String,
    pub to: String,
    #[serde(flatten)]
    pub attributes: Attributes<T>,
}

impl<T> Relation<T> {
    pub fn new(id: &str, space_id: &str, from: &str, to: &str, data: T) -> Self {
        Self {
            id: id.to_string(),
            from: from.to_string(),
            to: to.to_string(),
            types: vec![system_ids::RELATION_TYPE.to_string()],
            attributes: Attributes {
                id: id.to_string(),
                space_id: space_id.to_string(),
                attributes: data,
            },
        }
    }

    pub fn id(&self) -> &str {
        &self.attributes.id
    }

    pub fn space_id(&self) -> &str {
        &self.attributes.space_id
    }

    pub fn attributes(&self) -> &T {
        &self.attributes.attributes
    }

    pub fn attributes_mut(&mut self) -> &mut T {
        &mut self.attributes.attributes
    }

    pub fn with_type(mut self, type_id: &str) -> Self {
        self.types.push(type_id.to_string());
        self
    }

    /// Returns a query to delete the current relation
    pub fn delete_query(id: &str) -> Query<()> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            MATCH (r {{id: $id}})
            DETACH DELETE r
            "#,
        );

        Query::new(QUERY).param("id", id)
    }
}

impl<T> Relation<T>
where
    T: Serialize,
{
    /// Returns a query to upsert the current relation
    pub fn upsert_query(&self, block: &BlockMetadata) -> Result<Query<()>, serde_json::Error> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            MATCH (from {{id: $from_id}})
            MATCH (to {{id: $to_id}})
            MERGE (from)<-[:`{FROM_ENTITY}`]-(r {{id: $id, space_id: $space_id}})-[:`{TO_ENTITY}`]->(to)
            ON CREATE SET r += {{
                `{CREATED_AT}`: datetime($created_at),
                `{CREATED_AT_BLOCK}`: $created_at_block
            }}
            SET r:$($labels)
            SET r += {{
                `{UPDATED_AT}`: datetime($updated_at),
                `{UPDATED_AT_BLOCK}`: $updated_at_block
            }}
            SET r += $data
            "#,
            FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
            TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
            CREATED_AT = system_ids::CREATED_AT_TIMESTAMP,
            CREATED_AT_BLOCK = system_ids::CREATED_AT_BLOCK,
            UPDATED_AT = system_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = system_ids::UPDATED_AT_BLOCK,
        );

        let bolt_data = match serde_value_to_bolt(serde_json::to_value(self.attributes())?) {
            neo4rs::BoltType::Map(map) => neo4rs::BoltType::Map(map),
            _ => neo4rs::BoltType::Map(Default::default()),
        };

        let query = Query::new(QUERY)
            .param("id", self.id())
            .param("space_id", self.space_id())
            .param("from_id", self.from.clone())
            .param("to_id", self.to.clone())
            .param("space_id", self.space_id())
            .param("created_at", block.timestamp.to_rfc3339())
            .param("created_at_block", block.block_number.to_string())
            .param("updated_at", block.timestamp.to_rfc3339())
            .param("updated_at_block", block.block_number.to_string())
            .param("labels", self.types.clone())
            .param("data", bolt_data);

        Ok(query)
    }
}

impl Relation<HashMap<String, neo4rs::BoltType>> {
    pub fn with_attribute<T>(mut self, key: String, value: T) -> Self
    where
        T: Into<neo4rs::BoltType>,
    {
        self.attributes_mut().insert(key, value.into());
        self
    }
}
