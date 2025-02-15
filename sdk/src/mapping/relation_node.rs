use std::collections::HashMap;

use futures::TryStreamExt;
use neo4rs::BoltType;
use serde::Deserialize;

use crate::{error::DatabaseError, indexer_ids, models::BlockMetadata, pb, system_ids};

use super::{
    attributes, entity_node,
    query_utils::{PropFilter, Query, QueryPart, VersionFilter},
    triple, AttributeNode, Attributes, Triple, Value,
};

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct RelationNode {
    pub id: String,

    pub from: String,
    pub to: String,
    pub relation_type: String,
    pub index: AttributeNode,
}

impl RelationNode {
    pub fn new(
        id: impl Into<String>,
        from: impl Into<String>,
        to: impl Into<String>,
        relation_type: impl Into<String>,
        index: impl Into<Value>,
    ) -> Self {
        Self {
            id: id.into(),
            from: from.into(),
            to: to.into(),
            relation_type: relation_type.into(),
            index: AttributeNode::new(system_ids::RELATION_INDEX, index),
        }
    }

    /// Create a new TYPES relation
    pub fn new_types(
        id: impl Into<String>,
        from: impl Into<String>,
        to: impl Into<String>,
        index: impl Into<Value>,
    ) -> Self {
        Self::new(id, from, to, system_ids::TYPES_ATTRIBUTE, index)
    }

    pub fn insert(
        self,
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: impl Into<String>,
        space_version: i64,
    ) -> InsertOneQuery {
        InsertOneQuery::new(neo4j, block, space_id.into(), space_version, self)
    }

    pub fn get_attributes(
        &self,
        neo4j: &neo4rs::Graph,
        space_id: &str,
        space_version: Option<i64>,
    ) -> attributes::FindOneQuery {
        attributes::FindOneQuery::new(neo4j, self.id.clone(), space_id.to_owned(), space_version)
    }

    pub fn set_attribute(
        &self,
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: &str,
        space_version: i64,
        attribute: AttributeNode,
    ) -> triple::InsertOneQuery {
        triple::InsertOneQuery::new(
            neo4j,
            block,
            space_id.to_owned(),
            space_version,
            Triple {
                entity: self.id.clone(),
                attribute: attribute.id,
                value: attribute.value,
            },
        )
    }

    pub fn set_attributes(
        &self,
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: &str,
        space_version: i64,
        attributes: Attributes,
    ) -> attributes::InsertOneQuery<Attributes> {
        attributes::InsertOneQuery::new(
            neo4j,
            block,
            self.id.clone(),
            space_id.to_owned(),
            space_version,
            attributes,
        )
    }

    pub fn to(&self, neo4j: &neo4rs::Graph) -> entity_node::FindOneQuery {
        entity_node::find_one(neo4j, &self.to)
    }

    pub fn from(&self, neo4j: &neo4rs::Graph) -> entity_node::FindOneQuery {
        entity_node::find_one(neo4j, &self.from)
    }

    pub fn relation_type(&self, neo4j: &neo4rs::Graph) -> entity_node::FindOneQuery {
        entity_node::find_one(neo4j, &self.relation_type)
    }
}

impl From<pb::ipfs::Relation> for RelationNode {
    fn from(relation: pb::ipfs::Relation) -> Self {
        Self {
            id: relation.id,
            from: relation.from_entity,
            to: relation.to_entity,
            relation_type: relation.r#type,
            index: AttributeNode::new(system_ids::RELATION_INDEX, relation.index),
        }
    }
}

impl Into<BoltType> for RelationNode {
    fn into(self) -> BoltType {
        let mut triple_bolt_map = HashMap::new();
        triple_bolt_map.insert(neo4rs::BoltString { value: "id".into() }, self.id.into());
        triple_bolt_map.insert(
            neo4rs::BoltString {
                value: "from".into(),
            },
            self.from.into(),
        );
        triple_bolt_map.insert(neo4rs::BoltString { value: "to".into() }, self.to.into());
        triple_bolt_map.insert(
            neo4rs::BoltString {
                value: "relation_type".into(),
            },
            self.relation_type.into(),
        );
        triple_bolt_map.insert(
            neo4rs::BoltString {
                value: "index".into(),
            },
            self.index.into(),
        );

        BoltType::Map(neo4rs::BoltMap {
            value: triple_bolt_map,
        })
    }
}

pub fn delete_one(
    neo4j: &neo4rs::Graph,
    block: &BlockMetadata,
    relation_id: impl Into<String>,
    space_id: impl Into<String>,
    space_version: i64,
) -> DeleteOneQuery {
    DeleteOneQuery::new(
        neo4j.clone(),
        block.clone(),
        relation_id.into(),
        space_id.into(),
        space_version,
    )
}

pub fn delete_many(
    neo4j: &neo4rs::Graph,
    block: &BlockMetadata,
    space_id: impl Into<String>,
    space_version: i64,
) -> DeleteManyQuery {
    DeleteManyQuery::new(neo4j, block, space_id.into(), space_version)
}

pub fn insert_one(
    neo4j: &neo4rs::Graph,
    block: &BlockMetadata,
    space_id: impl Into<String>,
    space_version: i64,
    relation: RelationNode,
) -> InsertOneQuery {
    InsertOneQuery::new(
        neo4j,
        block,
        space_id.into(),
        space_version.into(),
        relation,
    )
}

pub fn insert_many(
    neo4j: &neo4rs::Graph,
    block: &BlockMetadata,
    space_id: impl Into<String>,
    space_version: i64,
) -> InsertManyQuery {
    InsertManyQuery::new(neo4j, block, space_id.into(), space_version)
}

pub fn find_one(
    neo4j: &neo4rs::Graph,
    relation_id: impl Into<String>,
    space_id: impl Into<String>,
    space_version: Option<i64>,
) -> FindOneQuery {
    FindOneQuery::new(neo4j, relation_id.into(), space_id.into(), space_version)
}

pub fn find_many(neo4j: &neo4rs::Graph) -> FindManyQuery {
    FindManyQuery::new(neo4j)
}

pub struct DeleteOneQuery {
    neo4j: neo4rs::Graph,
    block: BlockMetadata,
    relation_id: String,
    space_id: String,
    space_version: i64,
}

impl DeleteOneQuery {
    fn new(
        neo4j: neo4rs::Graph,
        block: BlockMetadata,
        relation_id: String,
        space_id: String,
        space_version: i64,
    ) -> Self {
        DeleteOneQuery {
            neo4j,
            block,
            relation_id,
            space_id,
            space_version,
        }
    }
}

impl Query<()> for DeleteOneQuery {
    async fn send(self) -> Result<(), DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
                MATCH (r {{id: $relation_id}})
                MATCH (r)-[r_to:`{TO_ENTITY}` {{space_id: $space_id}}]->()
                WHERE r_to.max_version IS NULL
                MATCH (r)-[r_from:`{FROM_ENTITY}` {{space_id: $space_id}}]->()
                WHERE r_from.max_version IS NULL
                MATCH (r)-[r_rt:`{RELATION_TYPE}` {{space_id: $space_id}}]->()
                WHERE r_rt.max_version IS NULL
                MATCH (r)-[r_index:ATTRIBUTE {{space_id: $space_id}}]->({{id: "{INDEX}"}})
                WHERE r_index.max_version IS NULL
                SET r_to.max_version = $space_version
                SET r_from.max_version = $space_version
                SET r_rt.max_version = $space_version
                SET r_index.max_version = $space_version
                SET r += {{
                    `{UPDATED_AT}`: datetime($block_timestamp),
                    `{UPDATED_AT_BLOCK}`: $block_number
                }}
            "#,
            FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
            TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
            RELATION_TYPE = system_ids::RELATION_TYPE_ATTRIBUTE,
            INDEX = system_ids::RELATION_INDEX,
            UPDATED_AT = indexer_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = indexer_ids::UPDATED_AT_BLOCK,
        );

        let query = neo4rs::query(QUERY)
            .param("relation_id", self.relation_id)
            .param("space_id", self.space_id)
            .param("space_version", self.space_version)
            .param("block_timestamp", self.block.timestamp.to_rfc3339())
            .param("block_number", self.block.block_number.to_string());

        Ok(self.neo4j.run(query).await?)
    }
}

pub struct DeleteManyQuery {
    neo4j: neo4rs::Graph,
    block: BlockMetadata,
    space_id: String,
    space_version: i64,
    relations: Vec<String>,
}

impl DeleteManyQuery {
    fn new(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: String,
        space_version: i64,
    ) -> Self {
        Self {
            neo4j: neo4j.clone(),
            block: block.clone(),
            space_id,
            space_version,
            relations: vec![],
        }
    }

    pub fn relation(mut self, relation_id: impl Into<String>) -> Self {
        self.relations.push(relation_id.into());
        self
    }

    pub fn relation_mut(&mut self, relation_id: impl Into<String>) {
        self.relations.push(relation_id.into());
    }

    pub fn relations(mut self, relation_ids: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.relations
            .extend(relation_ids.into_iter().map(Into::into));
        self
    }

    pub fn relations_mut(&mut self, relation_ids: impl IntoIterator<Item = impl Into<String>>) {
        self.relations
            .extend(relation_ids.into_iter().map(Into::into));
    }
}

impl Query<()> for DeleteManyQuery {
    async fn send(self) -> Result<(), DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
                UNWIND $relations as relation_id
                MATCH (r {{id: relation_id}})
                MATCH (r)-[r_to:`{TO_ENTITY}` {{space_id: $space_id}}]->()
                WHERE r_to.max_version IS NULL
                MATCH (r)-[r_from:`{FROM_ENTITY}` {{space_id: $space_id}}]->()
                WHERE r_from.max_version IS NULL
                MATCH (r)-[r_rt:`{RELATION_TYPE}` {{space_id: $space_id}}]->()
                WHERE r_rt.max_version IS NULL
                MATCH (r)-[r_index:ATTRIBUTE {{space_id: $space_id}}]->({{id: "{INDEX}"}})
                WHERE r_index.max_version IS NULL
                SET r_to.max_version = $space_version
                SET r_from.max_version = $space_version
                SET r_rt.max_version = $space_version
                SET r_index.max_version = $space_version
                SET r += {{
                    `{UPDATED_AT}`: datetime($block_timestamp),
                    `{UPDATED_AT_BLOCK}`: $block_number
                }}
            "#,
            FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
            TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
            RELATION_TYPE = system_ids::RELATION_TYPE_ATTRIBUTE,
            INDEX = system_ids::RELATION_INDEX,
            UPDATED_AT = indexer_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = indexer_ids::UPDATED_AT_BLOCK,
        );

        let query = neo4rs::query(QUERY)
            .param("space_id", self.space_id)
            .param("space_version", self.space_version)
            .param("relations", self.relations)
            .param("block_timestamp", self.block.timestamp.to_rfc3339())
            .param("block_number", self.block.block_number.to_string());

        Ok(self.neo4j.run(query).await?)
    }
}

pub struct InsertOneQuery {
    neo4j: neo4rs::Graph,
    block: BlockMetadata,
    space_id: String,
    space_version: i64,
    relation: RelationNode,
}

impl InsertOneQuery {
    fn new(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: String,
        space_version: i64,
        relation: RelationNode,
    ) -> Self {
        Self {
            neo4j: neo4j.clone(),
            block: block.clone(),
            space_id,
            space_version,
            relation,
        }
    }
}

impl Query<()> for InsertOneQuery {
    async fn send(self) -> Result<(), DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            MERGE (e {{id: $relation.id}})
            ON CREATE SET e += {{
                `{CREATED_AT}`: datetime($block_timestamp),
                `{CREATED_AT_BLOCK}`: $block_number
            }}
            SET e += {{
                `{UPDATED_AT}`: datetime($block_timestamp),
                `{UPDATED_AT_BLOCK}`: $block_number
            }}
            WITH e
            CALL (e) {{
                MATCH (e) -[r_from:`{FROM_ENTITY}` {{space_id: $space_id, max_version: null}}]-> ()
                MATCH (e) -[r_to:`{TO_ENTITY}` {{space_id: $space_id, max_version: null}}]-> ()
                MATCH (e) -[r_rt:`{RELATION_TYPE}` {{space_id: $space_id, max_version: null}}]-> ()
                MATCH (e) -[r_index:ATTRIBUTE {{space_id: $space_id, max_version: null}}]-> (index {{id: "{INDEX}"}})
                SET r_from.max_version = $space_version
                SET r_to.max_version = $space_version
                SET r_rt.max_version = $space_version
                SET r_index.max_version = $space_version
            }}
            MATCH (from {{id: $relation.from}})
            MATCH (to {{id: $relation.to}})
            MATCH (rt {{id: $relation.relation_type}})
            CREATE (e) -[:`{FROM_ENTITY}` {{space_id: $space_id, min_version: $space_version}}]-> (from)
            CREATE (e) -[:`{TO_ENTITY}` {{space_id: $space_id, min_version: $space_version}}]-> (to)
            CREATE (e) -[:`{RELATION_TYPE}` {{space_id: $space_id, min_version: $space_version}}]-> (rt)
            CREATE (e) -[:ATTRIBUTE {{space_id: $space_id, min_version: $space_version}}]-> (index {{id: "{INDEX}"}})
            SET index += $relation.index
            "#,
            CREATED_AT = indexer_ids::CREATED_AT_TIMESTAMP,
            CREATED_AT_BLOCK = indexer_ids::CREATED_AT_BLOCK,
            UPDATED_AT = indexer_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = indexer_ids::UPDATED_AT_BLOCK,
            FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
            TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
            RELATION_TYPE = system_ids::RELATION_TYPE_ATTRIBUTE,
            INDEX = system_ids::RELATION_INDEX,
        );

        let query = neo4rs::query(QUERY)
            .param("space_id", self.space_id)
            .param("space_version", self.space_version)
            .param("relation", self.relation)
            .param("block_number", self.block.block_number.to_string())
            .param("block_timestamp", self.block.timestamp.to_rfc3339());

        self.neo4j.run(query).await?;

        Ok(())
    }
}

pub struct InsertManyQuery {
    neo4j: neo4rs::Graph,
    block: BlockMetadata,
    space_id: String,
    space_version: i64,
    relations: Vec<RelationNode>,
}

impl InsertManyQuery {
    fn new(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: String,
        space_version: i64,
    ) -> Self {
        Self {
            neo4j: neo4j.clone(),
            block: block.clone(),
            space_id,
            space_version,
            relations: vec![],
        }
    }

    pub fn relation(mut self, relation: RelationNode) -> Self {
        self.relations.push(relation);
        self
    }

    pub fn relation_mut(&mut self, relation: RelationNode) {
        self.relations.push(relation);
    }

    pub fn relations(mut self, relations: impl IntoIterator<Item = RelationNode>) -> Self {
        self.relations.extend(relations);
        self
    }

    pub fn relations_mut(&mut self, relations: impl IntoIterator<Item = RelationNode>) {
        self.relations.extend(relations);
    }
}

impl Query<()> for InsertManyQuery {
    async fn send(self) -> Result<(), DatabaseError> {
        if self.relations.is_empty() {
            return Ok(());
        }

        const QUERY: &str = const_format::formatcp!(
            r#"
            UNWIND $relations as relation
            MERGE (e {{id: relation.id}})
            ON CREATE SET e += {{
                `{CREATED_AT}`: datetime($block_timestamp),
                `{CREATED_AT_BLOCK}`: $block_number
            }}
            SET e += {{
                `{UPDATED_AT}`: datetime($block_timestamp),
                `{UPDATED_AT_BLOCK}`: $block_number
            }}
            WITH e, relation
            CALL (e) {{
                MATCH (e) -[r_from:`{FROM_ENTITY}` {{space_id: $space_id}}]-> ()
                WHERE r_from.max_version IS NULL
                MATCH (e) -[r_to:`{TO_ENTITY}` {{space_id: $space_id}}]-> ()
                WHERE r_to.max_version IS NULL
                MATCH (e) -[r_rt:`{RELATION_TYPE}` {{space_id: $space_id}}]-> ()
                WHERE r_rt.max_version IS NULL
                MATCH (e) -[r_index:ATTRIBUTE {{space_id: $space_id}}]-> (index {{id: "{INDEX}"}})
                WHERE r_index.max_version IS NULL
                SET r_from.max_version = $space_version
                SET r_to.max_version = $space_version
                SET r_rt.max_version = $space_version
                SET r_index.max_version = $space_version
            }}
            MATCH (from {{id: relation.from}})
            MATCH (to {{id: relation.to}})
            MATCH (rt {{id: relation.relation_type}})
            CREATE (e) -[:`{FROM_ENTITY}` {{space_id: $space_id, min_version: $space_version}}]-> (from)
            CREATE (e) -[:`{TO_ENTITY}` {{space_id: $space_id, min_version: $space_version}}]-> (to)
            CREATE (e) -[:`{RELATION_TYPE}` {{space_id: $space_id, min_version: $space_version}}]-> (rt)
            CREATE (e) -[:ATTRIBUTE {{space_id: $space_id, min_version: $space_version}}]-> (index {{id: "{INDEX}"}})
            SET index += relation.index
            "#,
            CREATED_AT = indexer_ids::CREATED_AT_TIMESTAMP,
            CREATED_AT_BLOCK = indexer_ids::CREATED_AT_BLOCK,
            UPDATED_AT = indexer_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = indexer_ids::UPDATED_AT_BLOCK,
            FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
            TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
            RELATION_TYPE = system_ids::RELATION_TYPE_ATTRIBUTE,
            INDEX = system_ids::RELATION_INDEX,
        );

        let query = neo4rs::query(QUERY)
            .param("space_id", self.space_id)
            .param("space_version", self.space_version)
            .param("relations", self.relations)
            .param("block_number", self.block.block_number.to_string())
            .param("block_timestamp", self.block.timestamp.to_rfc3339());

        self.neo4j.run(query).await?;

        Ok(())
    }
}

pub struct FindOneQuery {
    neo4j: neo4rs::Graph,
    id: String,
    space_id: String,
    space_version: VersionFilter,
}

impl FindOneQuery {
    fn new(
        neo4j: &neo4rs::Graph,
        id: String,
        space_id: String,
        space_version: Option<i64>,
    ) -> Self {
        Self {
            neo4j: neo4j.clone(),
            id,
            space_id,
            space_version: VersionFilter::new(space_version),
        }
    }

    fn into_query_part(self) -> QueryPart {
        QueryPart::default()
            .match_clause("(e {id: $id})")
            .match_clause(format!(
                "(e) -[r_from:`{}` {{space_id: $space_id}}]-> (from)",
                system_ids::RELATION_FROM_ATTRIBUTE
            ))
            .match_clause(format!(
                "(e) -[r_to:`{}` {{space_id: $space_id}}]-> (to)",
                system_ids::RELATION_TO_ATTRIBUTE
            ))
            .match_clause(format!(
                "(e) -[r_rt:`{}` {{space_id: $space_id}}]-> (rt)",
                system_ids::RELATION_TYPE_ATTRIBUTE
            ))
            .match_clause(format!(
                r#"(e) -[r_index:ATTRIBUTE {{space_id: $space_id}}]-> (index {{id: "{}"}})"#,
                system_ids::RELATION_INDEX
            ))
            .merge(self.space_version.clone().into_query_part("r_from"))
            .merge(self.space_version.clone().into_query_part("r_to"))
            .merge(self.space_version.clone().into_query_part("r_rt"))
            .merge(self.space_version.into_query_part("r_index"))
            .return_clause("e{.id, from: from.id, to: to.id, relation_type: rt.id, index: index}")
            .order_by_clause("index.value")
            .params("id", self.id)
            .params("space_id", self.space_id)
    }
}

impl Query<Option<RelationNode>> for FindOneQuery {
    async fn send(self) -> Result<Option<RelationNode>, DatabaseError> {
        let neo4j = self.neo4j.clone();
        let query = self.into_query_part().build();

        Ok(neo4j
            .execute(query)
            .await?
            .next()
            .await?
            .map(|row| {
                Result::<_, DatabaseError>::Ok(row.to::<RelationNode>()?)
            })
            .transpose()?)
    }
}

pub struct FindManyQuery {
    neo4j: neo4rs::Graph,
    id: Option<PropFilter<String>>,
    space_id: Option<PropFilter<String>>,
    from_id: Option<PropFilter<String>>,
    to_id: Option<PropFilter<String>>,
    relation_type: Option<PropFilter<String>>,

    space_version: VersionFilter,
}

impl FindManyQuery {
    pub fn new(neo4j: &neo4rs::Graph) -> Self {
        Self {
            neo4j: neo4j.clone(),
            id: None,
            space_id: None,
            relation_type: None,
            from_id: None,
            to_id: None,
            space_version: VersionFilter::default(),
        }
    }

    pub fn space_id(mut self, space_id: PropFilter<String>) -> Self {
        self.space_id = Some(space_id);
        self
    }

    pub fn relation_type(mut self, relation_type: PropFilter<String>) -> Self {
        self.relation_type = Some(relation_type);
        self
    }

    pub fn from_id(mut self, from_id: PropFilter<String>) -> Self {
        self.from_id = Some(from_id);
        self
    }

    pub fn to_id(mut self, to_id: PropFilter<String>) -> Self {
        self.to_id = Some(to_id);
        self
    }

    fn into_query_part(self) -> QueryPart {
        let mut query_part = QueryPart::default()
            .match_clause("(e)")
            .match_clause(format!(
                "(e) -[r_rt:`{}`]-> (rt)",
                system_ids::RELATION_TYPE_ATTRIBUTE
            ))
            .match_clause(format!(
                "(e) -[r_from:`{}`]-> (from)",
                system_ids::RELATION_FROM_ATTRIBUTE
            ))
            .match_clause(format!(
                "(e) -[r_to:`{}`]-> (to)",
                system_ids::RELATION_TO_ATTRIBUTE
            ))
            .match_clause(format!(
                r#"(e) -[r_index:ATTRIBUTE]-> (index {{id: "{}"}})"#,
                system_ids::RELATION_INDEX
            ))
            .merge(self.space_version.clone().into_query_part("r_index"))
            .return_clause("e{.id, from: from.id, to: to.id, relation_type: rt.id, index: index}")
            .order_by_clause("index.value");

        if let Some(id_filter) = self.id {
            query_part.merge_mut(id_filter.into_query_part("e", "id"));
        }

        if let Some(from_id) = self.from_id {
            query_part = query_part
                .merge(from_id.into_query_part("from", "id"))
                .merge(self.space_version.clone().into_query_part("r_from"));
        }

        if let Some(to_id) = self.to_id {
            query_part = query_part
                .merge(to_id.into_query_part("to", "id"))
                .merge(self.space_version.clone().into_query_part("r_to"));
        }

        if let Some(relation_type) = self.relation_type {
            query_part = query_part
                .merge(relation_type.into_query_part("rt", "id"))
                .merge(self.space_version.clone().into_query_part("r_rt"));
        }

        if let Some(space_id) = self.space_id {
            query_part = query_part
                .merge(space_id.clone().into_query_part("r_from", "space_id"))
                .merge(space_id.clone().into_query_part("r_to", "space_id"))
                .merge(space_id.clone().into_query_part("r_rt", "space_id"))
                .merge(space_id.into_query_part("r_index", "space_id"));
        }

        query_part
    }
}

// TODO: (optimization) refactor into stream
impl Query<Vec<RelationNode>> for FindManyQuery {
    async fn send(self) -> Result<Vec<RelationNode>, DatabaseError> {
        let neo4j = self.neo4j.clone();
        let query = self.into_query_part().build();

        Ok(neo4j
            .execute(query)
            .await?
            .into_stream_as::<RelationNode>()
            .map_err(DatabaseError::from)
            .try_collect::<Vec<_>>()
            .await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use testcontainers::{
        core::{IntoContainerPort, WaitFor},
        runners::AsyncRunner,
        GenericImage, ImageExt,
    };

    const BOLT_PORT: u16 = 7687;
    const HTTP_PORT: u16 = 7474;

    #[tokio::test]
    async fn test_find_one() {
        // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
        let container = GenericImage::new("neo4j", "2025.01.0-community")
            .with_wait_for(WaitFor::Duration {
                length: std::time::Duration::from_secs(5),
            })
            .with_exposed_port(BOLT_PORT.tcp())
            .with_exposed_port(HTTP_PORT.tcp())
            .with_env_var("NEO4J_AUTH", "none")
            .start()
            .await
            .expect("Failed to start Neo 4J container");

        let port = container.get_host_port_ipv4(BOLT_PORT).await.unwrap();
        let host = container.get_host().await.unwrap().to_string();

        let neo4j = neo4rs::Graph::new(format!("neo4j://{host}:{port}"), "user", "password")
            .await
            .unwrap();

        neo4j.run(neo4rs::query(&format!(
            r#"
            CREATE (alice {{id: "alice"}})
            CREATE (bob {{id: "bob"}})
            CREATE (knows {{id: "knows"}})
            CREATE (r {{id: "abc"}})
            CREATE (r) -[:`{FROM_ENTITY}` {{space_id: "ROOT", min_version: 0}}]-> (alice)
            CREATE (r) -[:`{TO_ENTITY}` {{space_id: "ROOT", min_version: 0}}]-> (bob)
            CREATE (r) -[:`{RELATION_TYPE}` {{space_id: "ROOT", min_version: 0}}]-> (knows)
            CREATE (r) -[:ATTRIBUTE {{space_id: "ROOT", min_version: 0}}]-> (index {{id: "{INDEX}", value: "0", value_type: "TEXT"}})
            "#,
            FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
            TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
            RELATION_TYPE = system_ids::RELATION_TYPE_ATTRIBUTE,
            INDEX = system_ids::RELATION_INDEX,
        )))
            .await
            .expect("Failed to insert data");

        let relation_node = RelationNode::new("abc", "alice", "bob", "knows", "0");

        let found_relation = find_one(&neo4j, "abc", "ROOT", None)
            .send()
            .await
            .expect("Failed to find relation")
            .expect("Relation not found");

        assert_eq!(found_relation, relation_node);
    }

    #[tokio::test]
    async fn test_insert_find_one() {
        // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
        let container = GenericImage::new("neo4j", "2025.01.0-community")
            .with_wait_for(WaitFor::Duration {
                length: std::time::Duration::from_secs(5),
            })
            .with_exposed_port(BOLT_PORT.tcp())
            .with_exposed_port(HTTP_PORT.tcp())
            .with_env_var("NEO4J_AUTH", "none")
            .start()
            .await
            .expect("Failed to start Neo 4J container");

        let port = container.get_host_port_ipv4(BOLT_PORT).await.unwrap();
        let host = container.get_host().await.unwrap().to_string();

        let neo4j = neo4rs::Graph::new(format!("neo4j://{host}:{port}"), "user", "password")
            .await
            .unwrap();

        triple::insert_many(&neo4j, &BlockMetadata::default(), "space_id", 0)
            .triples(vec![
                Triple::new("alice", "name", "Alice"),
                Triple::new("bob", "name", "Bob"),
                Triple::new("knows", "name", "knows"),
            ])
            .send()
            .await
            .expect("Failed to insert triples");

        let relation_node = RelationNode::new("abc", "alice", "bob", "knows", "0");

        relation_node
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", 0)
            .send()
            .await
            .expect("Failed to insert relation");

        let found_relation = find_one(&neo4j, "abc", "ROOT", None)
            .send()
            .await
            .expect("Failed to find relation")
            .expect("Relation not found");

        assert_eq!(found_relation, relation_node);
    }

    #[tokio::test]
    async fn test_find_many() {
        // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
        let container = GenericImage::new("neo4j", "2025.01.0-community")
            .with_wait_for(WaitFor::Duration {
                length: std::time::Duration::from_secs(5),
            })
            .with_exposed_port(BOLT_PORT.tcp())
            .with_exposed_port(HTTP_PORT.tcp())
            .with_env_var("NEO4J_AUTH", "none")
            .start()
            .await
            .expect("Failed to start Neo 4J container");

        let port = container.get_host_port_ipv4(BOLT_PORT).await.unwrap();
        let host = container.get_host().await.unwrap().to_string();

        let neo4j = neo4rs::Graph::new(format!("neo4j://{host}:{port}"), "user", "password")
            .await
            .unwrap();

        neo4j.run(neo4rs::query(&format!(
            r#"
            CREATE (alice {{id: "alice"}})
            CREATE (bob {{id: "bob"}})
            CREATE (charlie {{id: "charlie"}})
            CREATE (knows {{id: "knows"}})
            CREATE (r1 {{id: "abc"}})
            CREATE (r1) -[:`{FROM_ENTITY}` {{space_id: "ROOT", min_version: 0}}]-> (alice)
            CREATE (r1) -[:`{TO_ENTITY}` {{space_id: "ROOT", min_version: 0}}]-> (bob)
            CREATE (r1) -[:`{RELATION_TYPE}` {{space_id: "ROOT", min_version: 0}}]-> (knows)
            CREATE (r1) -[:ATTRIBUTE {{space_id: "ROOT", min_version: 0}}]-> ({{id: "{INDEX}", value: "0", value_type: "TEXT"}})
            CREATE (r2 {{id: "dev"}})
            CREATE (r2) -[:`{FROM_ENTITY}` {{space_id: "ROOT", min_version: 0}}]-> (alice)
            CREATE (r2) -[:`{TO_ENTITY}` {{space_id: "ROOT", min_version: 0}}]-> (charlie)
            CREATE (r2) -[:`{RELATION_TYPE}` {{space_id: "ROOT", min_version: 0}}]-> (knows)
            CREATE (r2) -[:ATTRIBUTE {{space_id: "ROOT", min_version: 0}}]-> ({{id: "{INDEX}", value: "0", value_type: "TEXT"}})
            "#,
            FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
            TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
            RELATION_TYPE = system_ids::RELATION_TYPE_ATTRIBUTE,
            INDEX = system_ids::RELATION_INDEX,
        )))
            .await
            .expect("Failed to insert data");

        let relation_nodes = vec![
            RelationNode::new("abc", "alice", "bob", "knows", "0"),
            RelationNode::new("dev", "alice", "charlie", "knows", "0"),
        ];

        let found_relations = find_many(&neo4j)
            .relation_type(PropFilter::new().value("knows"))
            .from_id(PropFilter::new().value("alice"))
            .send()
            .await
            .expect("Failed to find relations");

        assert_eq!(found_relations, relation_nodes);
    }

    #[tokio::test]
    async fn test_insert_many() {
        // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
        let container = GenericImage::new("neo4j", "2025.01.0-community")
            .with_wait_for(WaitFor::Duration {
                length: std::time::Duration::from_secs(5),
            })
            .with_exposed_port(BOLT_PORT.tcp())
            .with_exposed_port(HTTP_PORT.tcp())
            .with_env_var("NEO4J_AUTH", "none")
            .start()
            .await
            .expect("Failed to start Neo 4J container");

        let port = container.get_host_port_ipv4(BOLT_PORT).await.unwrap();
        let host = container.get_host().await.unwrap().to_string();

        let neo4j = neo4rs::Graph::new(format!("neo4j://{host}:{port}"), "user", "password")
            .await
            .unwrap();

        triple::insert_many(&neo4j, &BlockMetadata::default(), "space_id", 0)
            .triples(vec![
                Triple::new("alice", "name", "Alice"),
                Triple::new("bob", "name", "Bob"),
                Triple::new("knows", "name", "knows"),
                Triple::new("charlie", "name", "Charlie"),
            ])
            .send()
            .await
            .expect("Failed to insert triples");

        let relation_nodes = vec![
            RelationNode::new("abc", "alice", "bob", "knows", "0"),
            RelationNode::new("dev", "alice", "charlie", "knows", "0"),
        ];

        insert_many(&neo4j, &BlockMetadata::default(), "ROOT", 0)
            .relations(relation_nodes.clone())
            .send()
            .await
            .expect("Failed to insert relations");

        let found_relations = find_many(&neo4j)
            .relation_type(PropFilter::new().value("knows"))
            .from_id(PropFilter::new().value("alice"))
            .send()
            .await
            .expect("Failed to find relations");

        assert_eq!(found_relations, relation_nodes);
    }
}
